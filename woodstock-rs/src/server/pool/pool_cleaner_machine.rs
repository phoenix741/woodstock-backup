use std::{path::PathBuf, sync::Arc, time::SystemTime};

use eyre::Result;
use log::error;
use tokio::sync::{mpsc, Mutex};

use crate::{
    config::Configuration, pool::apply_pending_refcnt_operations, utils::lock::PoolLock,
    EventPoolCleanedInformation, EventSource,
};

use super::{
    pool_cleaner::PoolCleaner, pool_cleaner::PoolProgression, pool_cleaner_state::CleanerState,
};

pub struct PoolCleanerMachine {
    /// The configuration for the pool cleaner.
    config: Configuration,
    /// The `PoolCleaner` instance responsible for performing cleaning operations on the storage pool.
    cleaner: PoolCleaner,
    /// Optional target path specifying where the cleaning operation should be applied.
    target: Option<PathBuf>,
    /// The event source used to provide context or events for the cleaning process.
    source: EventSource,
    /// Shared state of the cleaning process, protected by a mutex for safe concurrent access.
    state: Arc<Mutex<CleanerState>>,
    /// Optional channel for sending state updates to observers or other components.
    state_tx: Option<mpsc::Sender<CleanerState>>,
}

impl PoolCleanerMachine {
    /// Creates a new instance of `PoolCleanerMachine`.
    ///
    /// # Arguments
    /// * `config` - The configuration for the pool cleaner.
    /// * `target` - An optional target path for cleaning.
    /// * `source` - The event source.
    /// * `state_tx` - An optional channel for sending state updates.
    ///
    /// # Returns
    ///
    /// A new instance of `PoolCleanerMachine`.
    #[must_use]
    pub fn new(
        config: &Configuration,
        target: Option<PathBuf>,
        source: EventSource,
        state_tx: Option<mpsc::Sender<CleanerState>>,
    ) -> Self {
        let cleaner = PoolCleaner::new(config);
        let state = CleanerState::new();

        Self {
            config: config.clone(),
            cleaner,
            target,
            source,
            state: Arc::new(Mutex::new(state)),
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
            let state = self.state.lock().await;

            if let Err(e) = state_tx.send(state.clone()).await {
                error!("Failed to send state update: {}", e);
            }
        }
    }

    /// Applies pending refcnt operations.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operations are applied successfully.
    /// * `Err(eyre::Report)` if an error occurs during the operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the refcnt operations cannot be applied.
    async fn apply_refcnt_operations(&self) -> Result<()> {
        {
            let mut state = self.state.lock().await;
            state.start_applying_refcnt();
        }
        self.send_state().await;

        let current_time = SystemTime::now();
        let result = apply_pending_refcnt_operations(&self.config, &current_time).await;

        {
            let mut state = self.state.lock().await;
            state.process_applying_refcnt_result(result)?;
        }
        self.send_state().await;
        Ok(())
    }

    /// Initializes the cleaning process.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the initialization succeeds.
    /// * `Err(eyre::Report)` if an error occurs during initialization.
    ///
    /// # Errors
    ///
    /// Returns an error if the initialization fails.
    async fn init_cleaning(&self) -> Result<()> {
        {
            let mut state = self.state.lock().await;
            state.start_initialization();
        }
        self.send_state().await;

        let result = self.cleaner.clean_unused_max().await;

        {
            let mut state = self.state.lock().await;
            state.process_initialization_result(result)?;
        }
        self.send_state().await;
        Ok(())
    }

    /// Executes the cleaning process.
    ///
    /// # Returns
    ///
    /// * `Ok(EventPoolCleanedInformation)` if the cleaning succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the cleaning process.
    ///
    /// # Errors
    ///
    /// Returns an error if the cleaning fails.
    async fn execute_cleaning(&self) -> Result<EventPoolCleanedInformation> {
        {
            let mut state = self.state.lock().await;
            state.start_cleaning();
        }
        self.send_state().await;

        // Create a channel to receive progress updates
        let (progress_tx, mut progress_rx) = mpsc::channel::<PoolProgression>(10);

        // Create a task to process progress updates
        let state_clone = self.state.clone();
        let state_tx_clone = self.state_tx.clone();

        let progress_task = tokio::spawn(async move {
            while let Some(progress) = progress_rx.recv().await {
                let mut state = state_clone.lock().await;
                state.process_cleaning_progress(
                    progress.progress_current,
                    progress.file_size,
                    progress.compressed_file_size,
                );

                // Send the updated state
                if let Some(tx) = &state_tx_clone {
                    if let Err(e) = tx.send(state.clone()).await {
                        error!("Failed to send state update during cleaning: {}", e);
                    }
                }
            }
        });

        // Execute the cleaning
        let result = self
            .cleaner
            .clean_unused_pool(self.target.clone(), self.source, Some(progress_tx))
            .await;

        // Wait for the progress task to complete
        if let Err(e) = progress_task.await {
            error!("Error in cleaning progression task: {}", e);
        }

        let result = {
            let mut state = self.state.lock().await;
            state.process_cleaning_result(result)
        };
        self.send_state().await;
        result
    }

    /// Executes the entire cleaning process, including initialization and cleaning.
    ///
    /// # Returns
    ///
    /// * `Ok(EventPoolCleanedInformation)` if the process succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the process.
    ///
    /// # Errors
    ///
    /// Returns an error if any step of the process fails.
    pub async fn execute(&self) -> Result<EventPoolCleanedInformation> {
        // Apply pending refcnt operations
        self.apply_refcnt_operations().await?;

        let _lock = PoolLock::new_with_name(&self.config.path.pool_path, "execute_cleaning")
            .lock_exclusive()
            .await?;

        // Initialize
        self.init_cleaning().await?;

        // Execute the cleaning
        let result = self.execute_cleaning().await?;

        Ok(result)
    }

    /// Retrieves the current state of the pool cleaner.
    ///
    /// # Returns
    ///
    /// The current `CleanerState`.
    pub async fn get_state(&self) -> CleanerState {
        self.state.lock().await.clone()
    }
}
