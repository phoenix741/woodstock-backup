use std::sync::Arc;

use eyre::Result;
use tokio::sync::mpsc;
use tracing::{error, Instrument};

use crate::{
    config::{Backups, Configuration, Hosts, DEFAULT_CHANNEL_BUFFER_SIZE},
    server::pool::convert_state::ConvertState,
    utils::lock_redis::PoolLockRedis,
    EventSource,
};

use super::convert::PoolConvert;

pub struct HashConverterMachine {
    /// Configuration of the application, containing paths and other settings.
    config: Arc<Configuration>,
    /// The `PoolConvert` instance responsible for performing the hash conversion operations on the storage pool.
    pool_converter: PoolConvert,
    /// The new configuration to be applied during the hash conversion process.
    new_config: Arc<Configuration>,
    /// The event source used to provide context or events for the hash conversion process.
    source: EventSource,
    /// Shared state of the conversion process, protected by a mutex for safe concurrent access.
    convert_state: Arc<tokio::sync::Mutex<ConvertState>>,
    /// Optional channel for sending state updates to observers or other components.
    state_tx: Option<mpsc::Sender<ConvertState>>,
}

impl HashConverterMachine {
    /// Creates a new instance of `HashConverterMachine`.
    ///
    /// # Arguments
    /// * `config` - The current configuration.
    /// * `new_config` - The new configuration for the conversion.
    /// * `source` - The event source.
    /// * `state_tx` - An optional channel for sending state updates.
    ///
    /// # Returns
    ///
    /// A new instance of `HashConverterMachine`.
    #[must_use]
    pub fn new(
        source: EventSource,
        state_tx: Option<mpsc::Sender<ConvertState>>,
        config: Arc<Configuration>,
        hosts: Arc<Hosts>,
        backups: Arc<Backups>,
        new_config: Arc<Configuration>,
    ) -> Self {
        let pool_converter = PoolConvert::new(config.clone(), hosts, backups);
        let convert_state = ConvertState::default();

        Self {
            config: config.clone(),
            pool_converter,
            new_config: new_config.clone(),
            source,
            convert_state: Arc::new(tokio::sync::Mutex::new(convert_state)),
            state_tx,
        }
    }

    /// Sends the current state to the state channel.
    ///
    /// # Errors
    ///
    /// Logs an error if sending the state update fails.
    async fn send_state(&self) {
        if let Some(state_tx) = &self.state_tx {
            let state = self.convert_state.lock().await;

            if let Err(e) = state_tx.send(state.clone()).await {
                error!("Failed to send state update: {}", e);
            }
        }
    }

    /// Initializes the hash conversion process.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the initialization succeeds.
    /// * `Err(eyre::Report)` if an error occurs during initialization.
    ///
    /// # Errors
    ///
    /// Returns an error if the initialization fails.
    async fn initialize(&self) -> Result<()> {
        {
            let mut convert_state = self.convert_state.lock().await;
            convert_state.start_initialization();
        }
        self.send_state().await;

        let max_result = self.pool_converter.get_max().await;

        {
            let mut convert_state = self.convert_state.lock().await;
            convert_state
                .process_initialization_result(max_result)
                .inspect_err(|_err| {
                    if let Some(tx) = &self.state_tx {
                        let _ = tx.try_send(convert_state.clone());
                    }
                })?;
        }
        self.send_state().await;

        Ok(())
    }

    /// Performs the hash conversion process on the storage pool.
    ///
    /// This method initiates the conversion process, manages progress updates,
    /// and updates the internal state accordingly. It handles the conversion
    /// workflow, including sending state updates to observers if a channel is provided.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the conversion process completes successfully.
    /// * `Err(eyre::Report)` if an error occurs during the conversion process.
    ///
    /// # Errors
    ///
    /// Returns an error if any step of the conversion process fails, such as
    /// initialization, progress update handling, or the conversion itself.
    async fn convert(&self) -> Result<()> {
        {
            let mut convert_state = self.convert_state.lock().await;
            convert_state.start_conversion();
        }
        self.send_state().await;

        // Create a channel to receive progress updates
        let (progress_tx, mut progress_rx) = mpsc::channel(DEFAULT_CHANNEL_BUFFER_SIZE);

        // Spawn a task to process progress updates
        let convert_state_clone = self.convert_state.clone();
        let state_tx_clone = self.state_tx.clone();

        let progress_task = tokio::spawn(
            async move {
                while let Some(progress) = progress_rx.recv().await {
                    let mut convert_state = convert_state_clone.lock().await;
                    convert_state.process_conversion_progress(progress);
                    if let Some(tx) = &state_tx_clone {
                        if let Err(e) = tx.send(convert_state.clone()).await {
                            error!("Failed to send state update during conversion: {}", e);
                        }
                    }
                }
            }
            .in_current_span(),
        );

        // Execute the conversion with the progress channel
        let result = self
            .pool_converter
            .convert_backup_dir(self.new_config.clone(), self.source, Some(progress_tx))
            .await;

        // Wait for the progress update task to complete
        if let Err(e) = progress_task.await {
            error!("Error in conversion progression task: {}", e);
        }

        {
            let mut convert_state = self.convert_state.lock().await;
            convert_state
                .process_conversion_result(result)
                .inspect_err(|_err| {
                    if let Some(tx) = &self.state_tx {
                        let _ = tx.try_send(convert_state.clone());
                    }
                })?;
        }
        self.send_state().await;

        Ok(())
    }

    /// Executes the hash conversion process.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the conversion succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the conversion.
    ///
    /// # Errors
    ///
    /// Returns an error if the conversion fails.
    pub async fn execute(&self) -> Result<()> {
        let redis_url = self.config.redis_url();
        let _lock = PoolLockRedis::new_with_path(
            &redis_url,
            &self.config.path.pool_path,
            "execute_hash_conversion",
        )
        .await?
        .lock_exclusive()
        .await?;

        self.send_state().await;

        // Initialization - calculate the pool size
        self.initialize().await?;

        // Update the max in the state
        self.send_state().await;

        // Conversion
        self.convert().await?;

        self.send_state().await;

        Ok(())
    }
}
