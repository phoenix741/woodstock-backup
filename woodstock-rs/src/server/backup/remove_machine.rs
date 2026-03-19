use eyre::{eyre, Result};
use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc, Mutex};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    config::{Backups, Configuration, Context},
    utils::lock_redis::{LockOperation, PoolLockOperation, PoolLockRedis},
};

use super::{remove::BackupRemove, remove_state::RemoveState};

/// Represents the execution phase of the removal state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RemovalPhase {
    /// Finalize removal publication in the shared pool index
    FinalizePoolRemoval,
    /// Cleanup host-side removal bookkeeping
    CleanupHostRemovalState,
    /// Remove backup files
    RemoveBackup,
    /// Removal process completed
    Finished,
}

pub struct RemoveBackupMachine {
    /// The configuration for the backup removal machine.
    config: Arc<Configuration>,

    /// The client responsible for backup removal operations.
    client: BackupRemove,
    /// The state of the backup removal progression.
    progression_state: Arc<Mutex<RemoveState>>,
    /// An optional channel for sending state updates.
    state_tx: Option<mpsc::Sender<RemoveState>>,

    /// The hostname being removed (stored for resume detection)
    hostname: String,

    /// The backup UUID v7 identifier (stored for resume detection)
    backup_id: Uuid,

    /// Backups service used for marker path resolution.
    backups: Arc<Backups>,
}

impl RemoveBackupMachine {
    /// Creates a new instance of `RemoveBackupMachine`.
    ///
    /// # Arguments
    /// * `hostname` - The hostname of the backup to remove.
    /// * `backup_id` - The UUID v7 identifier of the backup to remove.
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
        backup_id: Uuid,
        ctxt: &Context,
        state_tx: Option<mpsc::Sender<RemoveState>>,

        config: Arc<Configuration>,
        backups: Arc<Backups>,
    ) -> Self {
        let client = BackupRemove::new(hostname, backup_id, ctxt, config.clone(), backups.clone());
        let progression_state = RemoveState::default();

        Self {
            config,
            client,
            progression_state: Arc::new(Mutex::new(progression_state)),
            state_tx,

            hostname: hostname.to_string(),
            backup_id,
            backups,
        }
    }

    fn removal_marker_path(&self) -> std::path::PathBuf {
        self.backups
            .get_pool_v3_removal_marker_path(&self.hostname, self.backup_id)
    }

    async fn persist_removal_marker(&self) -> Result<()> {
        tokio::fs::write(self.removal_marker_path(), b"removing\n").await?;
        Ok(())
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

    /// Finalizes the Pool V3 removal publication for the current backup.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the reference count is added successfully.
    /// * `Err(eyre::Report)` if an error occurs while adding the reference count.
    ///
    /// # Errors
    ///
    /// Returns an error if the reference count cannot be added to the pool.
    pub async fn finalize_pool_removal(&self) -> Result<()> {
        self.send_progres().await;

        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state.start_finalize_pool_removal();
        }
        self.send_progres().await;

        let result = self.client.finalize_pool_removal().await;

        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state
                .process_finalize_pool_removal_result(result)
                .inspect_err(|_| {
                    if let Some(tx) = &self.state_tx {
                        let _ = tx.try_send(progression_state.clone());
                    }
                })?;
        }

        self.send_progres().await;
        Ok(())
    }

    /// Cleans up host-side removal bookkeeping.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the removal process.
    ///
    /// # Errors
    ///
    /// Returns an error if the reference count removal fails.
    async fn cleanup_host_removal_state(&mut self) -> Result<()> {
        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state.start_cleanup_host_removal_state();
        }

        self.send_progres().await;

        let result = self.client.cleanup_host_removal_state().await;
        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state
                .process_cleanup_host_removal_state_result(result)
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

    /// Executes the pool removal finalization phase.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs.
    async fn execute_finalize_pool_removal_phase(&mut self) -> Result<()> {
        info!(
            "Executing pool removal finalization phase for removal {}/{}",
            self.hostname, self.backup_id
        );

        self.finalize_pool_removal().await?;

        Ok(())
    }

    /// Executes the host removal state cleanup phase.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs.
    async fn execute_cleanup_host_removal_phase(&mut self) -> Result<()> {
        info!(
            "Executing host removal state cleanup phase for removal {}/{}",
            self.hostname, self.backup_id
        );

        self.cleanup_host_removal_state().await?;

        Ok(())
    }

    /// Executes the remove backup phase.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs.
    async fn execute_remove_backup_phase(&mut self) -> Result<()> {
        info!(
            "Executing remove backup phase for removal {}/{}",
            self.hostname, self.backup_id
        );

        self.remove_backup().await?;

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
    pub async fn execute(&mut self) -> Result<RemoveState> {
        let pool_directory = &self.config.path.pool_path;
        let redis_url = self.config.redis_url();
        let _lock = PoolLockRedis::new_with_path(
            &redis_url,
            pool_directory,
            LockOperation::Pool(PoolLockOperation::RemoveBackup),
        )
        .await?
        .try_lock_shared_wait(Duration::from_secs(60))
        .await?
        .ok_or_else(|| eyre!("Timed out waiting for shared pool lock during backup removal"))?;

        self.send_progres().await;

        self.persist_removal_marker().await?;

        let mut current_phase = RemovalPhase::FinalizePoolRemoval;

        info!(
            "Removal state machine starting for {}/{}: starting_phase={:?}",
            self.hostname, self.backup_id, current_phase
        );

        // State machine main loop
        loop {
            match current_phase {
                RemovalPhase::FinalizePoolRemoval => {
                    match self.execute_finalize_pool_removal_phase().await {
                        Ok(()) => {
                            current_phase = RemovalPhase::CleanupHostRemovalState;
                        }
                        Err(err) => {
                            error!("Error during pool removal finalization phase: {err}");
                            return Err(err);
                        }
                    }
                }

                RemovalPhase::CleanupHostRemovalState => {
                    match self.execute_cleanup_host_removal_phase().await {
                        Ok(()) => {
                            current_phase = RemovalPhase::RemoveBackup;
                        }
                        Err(err) => {
                            error!("Error during host removal state cleanup phase: {err}");
                            return Err(err);
                        }
                    }
                }

                RemovalPhase::RemoveBackup => match self.execute_remove_backup_phase().await {
                    Ok(()) => {
                        current_phase = RemovalPhase::Finished;
                    }
                    Err(err) => {
                        error!("Error during remove backup phase: {err}");
                        return Err(err);
                    }
                },

                RemovalPhase::Finished => {
                    info!(
                        "Removal state machine completed for {}/{}",
                        self.hostname, self.backup_id
                    );
                    break;
                }
            }
        }

        // Mark the removal as completed
        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state.complete();
        }
        self.send_progres().await;

        let state = {
            let progression_state = self.progression_state.lock().await;
            progression_state.clone()
        };

        Ok(state)
    }
}
