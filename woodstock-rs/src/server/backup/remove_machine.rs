use eyre::Result;
use log::error;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

use crate::{
    config::{Configuration, Context},
    utils::lock::PoolLock,
};

use super::{remove::BackupRemove, remove_state::RemoveState};

pub struct RemoveBackupMachine {
    /// The configuration for the backup removal machine.
    config: Configuration,

    /// The client responsible for backup removal operations.
    client: BackupRemove,
    /// The state of the backup removal progression.
    progression_state: Arc<Mutex<RemoveState>>,
    /// An optional channel for sending state updates.
    state_tx: Option<mpsc::Sender<RemoveState>>,
}

impl RemoveBackupMachine {
    /// Creates a new instance of `RemoveBackupMachine`.
    ///
    /// # Arguments
    /// * `hostname` - The hostname of the backup to remove.
    /// * `backup_number` - The backup number to remove.
    /// * `ctxt` - The context containing the event source.
    /// * `config` - The configuration for the backup system.
    /// * `state_tx` - An optional channel for sending state updates.
    ///
    /// # Returns
    ///
    /// A new instance of `RemoveBackupMachine`.
    #[must_use]
    pub fn new(
        hostname: &str,
        backup_number: usize,
        ctxt: &Context,
        config: &Configuration,
        state_tx: Option<mpsc::Sender<RemoveState>>,
    ) -> Self {
        let client = BackupRemove::new(hostname, backup_number, ctxt, config);
        let progression_state = RemoveState::default();

        Self {
            config: config.clone(),
            client,
            progression_state: Arc::new(Mutex::new(progression_state)),
            state_tx,
        }
    }

    /// Sends the current progression state to the state channel.
    ///
    /// # Errors
    ///
    /// Logs an error if sending the state update fails.
    async fn send_progres(&self) {
        if let Some(state_tx) = &self.state_tx {
            let state = self.progression_state.lock().await;

            if let Err(e) = state_tx.send(state.clone()).await {
                error!("Failed to send state update: {}", e);
            }
        }
    }

    /// Adds a reference count to the pool for removal for the current backup.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the reference count is added successfully.
    /// * `Err(eyre::Report)` if an error occurs while adding the reference count.
    ///
    /// # Errors
    ///
    /// Returns an error if the reference count cannot be added to the pool.
    pub async fn add_refcnt_to_pool(&self) -> Result<()> {
        self.send_progres().await;

        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state.start_add_references_to_pool();
        }
        self.send_progres().await;

        let result = self.client.add_refcnt_to_pool().await;

        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state
                .process_add_references_to_pool_result(result)
                .inspect_err(|_| {
                    if let Some(tx) = &self.state_tx {
                        let _ = tx.try_send(progression_state.clone());
                    }
                })?;
        }

        self.send_progres().await;
        Ok(())
    }

    /// Removes reference counts for the host.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the removal process.
    ///
    /// # Errors
    ///
    /// Returns an error if the reference count removal fails.
    async fn remove_refcnt(&mut self) -> Result<()> {
        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state.start_refcnt_removal();
        }

        self.send_progres().await;

        let result = self.client.remove_refcnt_of_host().await;
        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state
                .process_refcnt_removal_result(result)
                .inspect_err(|_| {
                    if let Some(tx) = &self.state_tx {
                        let _ = tx.try_send(progression_state.clone());
                    }
                })?;
        }
        self.send_progres().await;

        Ok(())
    }

    /// Removes the backup.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the removal process.
    ///
    /// # Errors
    ///
    /// Returns an error if the backup removal fails.
    async fn remove_backup(&mut self) -> Result<()> {
        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state.start_backup_removal();
        }
        self.send_progres().await;

        let result = self.client.remove_backup().await;
        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state
                .process_backup_removal_result(result)
                .inspect_err(|_| {
                    if let Some(tx) = &self.state_tx {
                        let _ = tx.try_send(progression_state.clone());
                    }
                })?;
        }
        self.send_progres().await;

        Ok(())
    }

    /// Executes the backup removal process.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the removal process.
    ///
    /// # Errors
    ///
    /// Returns an error if any step of the removal process fails.
    pub async fn execute(&mut self) -> Result<()> {
        let pool_directory = &self.config.path.pool_path;
        let _lock = PoolLock::new_with_name(&pool_directory, "remove")
            .lock_shared()
            .await?;

        self.send_progres().await;

        self.add_refcnt_to_pool().await?;

        self.remove_refcnt().await?;

        self.remove_backup().await?;

        // Mark the removal as completed
        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state.complete();
        }
        self.send_progres().await;

        Ok(())
    }
}
