use eyre::{eyre, Result};
use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc, Mutex};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    config::{Backups, Configuration, Context, RemovingStatus},
    utils::lock_redis::{LockOperation, PoolLockOperation, PoolLockRedis},
};

use super::{remove::BackupRemove, remove_state::RemoveState};

/// Represents the execution phase of the removal state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RemovalPhase {
    /// Add refcnt to pool for removal
    AddRefcntToPool,
    /// Remove refcnt from host
    RemoveRefcnt,
    /// Remove backup files
    RemoveBackup,
    /// Removal process completed
    Finished,
}

impl RemovalPhase {
    /// Determines the next phase to execute based on the current backup status.
    fn from_status(status: &crate::config::BackupStatus) -> Self {
        match status {
            crate::config::BackupStatus::Removing(RemovingStatus::ToRemoveInPool) => {
                Self::AddRefcntToPool
            }
            crate::config::BackupStatus::Removing(RemovingStatus::RemoveFromHost) => {
                Self::RemoveRefcnt
            }
            crate::config::BackupStatus::Removing(RemovingStatus::ToRemove) => Self::RemoveBackup,
            // Default: start from beginning
            _ => Self::AddRefcntToPool,
        }
    }
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

    /// Reference to backups for loading existing backup status
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
            backups: backups.clone(),
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

    /// Executes the add refcnt to pool phase.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs.
    async fn execute_add_refcnt_to_pool_phase(&mut self) -> Result<()> {
        info!(
            "Executing add refcnt to pool phase for removal {}/{}",
            self.hostname, self.backup_id
        );

        self.client
            .save_backup(crate::config::BackupStatus::Removing(
                RemovingStatus::ToRemoveInPool,
            ))
            .await?;

        self.add_refcnt_to_pool().await?;

        Ok(())
    }

    /// Executes the remove refcnt phase.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs.
    async fn execute_remove_refcnt_phase(&mut self) -> Result<()> {
        info!(
            "Executing remove refcnt phase for removal {}/{}",
            self.hostname, self.backup_id
        );

        self.client
            .save_backup(crate::config::BackupStatus::Removing(
                RemovingStatus::RemoveFromHost,
            ))
            .await?;

        self.remove_refcnt().await?;

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

        self.client
            .save_backup(crate::config::BackupStatus::Removing(
                RemovingStatus::ToRemove,
            ))
            .await?;

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

        // Determine initial state and starting phase
        let existing_backup = self
            .backups
            .get_backup(&self.hostname, self.backup_id)
            .await;

        let initial_status = existing_backup
            .as_ref()
            .map(|b| b.status.clone())
            .unwrap_or(crate::config::BackupStatus::Completed);

        let mut current_phase = RemovalPhase::from_status(&initial_status);

        info!(
            "Removal state machine starting for {}/{}: initial_status={:?}, starting_phase={:?}",
            self.hostname, self.backup_id, initial_status, current_phase
        );

        // State machine main loop
        loop {
            match current_phase {
                RemovalPhase::AddRefcntToPool => {
                    match self.execute_add_refcnt_to_pool_phase().await {
                        Ok(()) => {
                            current_phase = RemovalPhase::RemoveRefcnt;
                        }
                        Err(err) => {
                            error!("Error during add refcnt to pool phase: {err}");
                            return Err(err);
                        }
                    }
                }

                RemovalPhase::RemoveRefcnt => match self.execute_remove_refcnt_phase().await {
                    Ok(()) => {
                        current_phase = RemovalPhase::RemoveBackup;
                    }
                    Err(err) => {
                        error!("Error during remove refcnt phase: {err}");
                        return Err(err);
                    }
                },

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
