use std::{sync::Arc, time::Duration};

use eyre::{eyre, Result};
use tokio::sync::{mpsc, Mutex};
use tracing::{error, info, Instrument};
use uuid::Uuid;

use crate::{
    config::{
        BackupOperation, BackupStatus, Backups, Configuration, Context, ExecuteCommandOperation,
        FailedStatus, FinishingStatus, HostConfiguration, Hosts, ShareSnapshotMethod,
        DEFAULT_CHANNEL_BUFFER_SIZE,
    },
    server::client::Client,
    utils::lock_redis::{LockOperation, PoolLockOperation, PoolLockRedis},
    Share, SnapshotMethod,
};

use super::{save::BackupSave, save_state::BackupState};

/// Represents the execution phase of the backup state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
enum BackupPhase {
    /// Execute the backup (auth, init, commands, file sync, backup creation)
    Backup,
    /// Recover in-memory progression stats from on-disk journal/log after a crash,
    /// then jump to the wrapped phase.
    Recover(Box<BackupPhase>),
    /// Compact the manifest files
    Compact,
    /// Count chunk references
    CountReferences,
    /// Add reference count to the pool
    AddToPool,
    /// Backup process completed successfully
    Finished,
}

impl BackupPhase {
    /// Resolves the starting phase and target completion status from an existing backup entry.
    ///
    /// This is the **single source of truth** for resuming a backup. It handles:
    /// - `None` → brand-new backup, start from scratch, optimistically target `Completed`
    /// - `InProgress` → worker crashed mid-transfer, skip re-download, jump directly to
    ///   `Compact` and mark result as `Aborted` (partial data, but properly closed)
    /// - `Finishing(x)` → interrupted during finalization of a successful backup, resume
    ///   from the interrupted phase and keep targeting `Completed`
    /// - `Aborting(x)` → interrupted during finalization of an already-aborted backup,
    ///   resume from the interrupted phase, preserve `Aborted` target
    /// - `Failed(x)` → previous attempt hard-failed at a phase, retry from that phase
    /// - Terminal states (`Completed`, `Aborted`, `Removing`) → nothing to do
    fn from_existing(existing_status: Option<BackupStatus>) -> (Self, BackupStatus) {
        match existing_status {
            // Brand-new backup — no recovery needed
            None => (Self::Backup, BackupStatus::Completed),

            // Worker crashed during chunk transfer — skip re-download, close as Aborted.
            // Recover first to rebuild stats from the journal.
            Some(BackupStatus::InProgress) => (
                Self::Recover(Box::new(Self::Compact)),
                BackupStatus::Aborted,
            ),

            // Interrupted while finalizing a successful backup.
            // Recover first to rebuild stats from journal/log before resuming.
            Some(BackupStatus::Finishing(FinishingStatus::ToCompact)) => (
                Self::Recover(Box::new(Self::Compact)),
                BackupStatus::Completed,
            ),
            Some(BackupStatus::Finishing(FinishingStatus::ToCountRef)) => (
                Self::Recover(Box::new(Self::CountReferences)),
                BackupStatus::Completed,
            ),
            Some(BackupStatus::Finishing(FinishingStatus::ToAddInPool)) => (
                Self::Recover(Box::new(Self::AddToPool)),
                BackupStatus::Completed,
            ),

            // Interrupted while finalizing an already-aborted backup.
            Some(BackupStatus::Aborting(FinishingStatus::ToCompact)) => (
                Self::Recover(Box::new(Self::Compact)),
                BackupStatus::Aborted,
            ),
            Some(BackupStatus::Aborting(FinishingStatus::ToCountRef)) => (
                Self::Recover(Box::new(Self::CountReferences)),
                BackupStatus::Aborted,
            ),
            Some(BackupStatus::Aborting(FinishingStatus::ToAddInPool)) => (
                Self::Recover(Box::new(Self::AddToPool)),
                BackupStatus::Aborted,
            ),

            // Hard failure at a specific phase — retry it (with recovery first).
            Some(BackupStatus::Failed(FailedStatus::Compact)) => (
                Self::Recover(Box::new(Self::Compact)),
                BackupStatus::Completed,
            ),
            Some(BackupStatus::Failed(FailedStatus::RefCount)) => (
                Self::Recover(Box::new(Self::CountReferences)),
                BackupStatus::Completed,
            ),
            Some(BackupStatus::Failed(FailedStatus::InPool)) => (
                Self::Recover(Box::new(Self::AddToPool)),
                BackupStatus::Completed,
            ),

            // Terminal states — nothing to do
            Some(BackupStatus::Completed) => (Self::Finished, BackupStatus::Completed),
            Some(BackupStatus::Aborted) => (Self::Finished, BackupStatus::Aborted),
            Some(BackupStatus::Removing(_)) => (Self::Finished, BackupStatus::Aborted),
        }
    }
}

pub struct SaveBackupMachine<Clt: Client> {
    /// The configuration of the application, containing paths and other settings.
    config: Arc<Configuration>,

    /// The client used for backup operations.
    client: BackupSave<Clt>,

    /// The configuration of the host being backed up.
    host_configuration: HostConfiguration,

    /// The current progression state of the backup.
    progression_state: Arc<Mutex<BackupState>>,

    /// An optional sender for transmitting backup state updates.
    state_tx: Option<mpsc::Sender<BackupState>>,

    /// The hostname being backed up (stored for resume detection)
    hostname: String,

    /// The backup UUID (stored for resume detection)
    backup_id: Uuid,

    /// Reference to backups for loading existing backup status
    backups: Arc<Backups>,
}

impl<Clt: Client> SaveBackupMachine<Clt> {
    /// Determines the appropriate status to save based on the current context.
    ///
    /// If the backup is already aborted, returns `Aborting(stage)` to track
    /// where the aborted backup stopped. Otherwise returns `Finishing(stage)`.
    fn get_status_for_stage(current_status: &BackupStatus, stage: FinishingStatus) -> BackupStatus {
        if current_status.is_aborted() {
            BackupStatus::Aborting(stage)
        } else {
            BackupStatus::Finishing(stage)
        }
    }

    /// Creates a new instance of `SaveBackupMachine`.
    ///
    /// # Arguments
    /// * `client` - The client used for saving backups.
    /// * `hostname` - The hostname of the backup to save.
    /// * `backup_id` - The UUID v7 identifier for this backup.
    /// * `number` - The sequential display number for this backup. For a resume, pass the
    ///   existing number read from `backup.yml`; for a new backup, pass the next allocated number.
    /// * `previous_id` - The UUID of the preceding completed backup, used to clone its manifest
    ///   for incremental deduplication. `None` triggers a full backup.
    /// * `ctxt` - The context containing the event source.
    /// * `config` - The configuration for the backup system.
    /// * `state_tx` - An optional channel for sending state updates.
    ///
    /// # Returns
    ///
    /// * `Ok(SaveBackupMachine)` - A new instance of `SaveBackupMachine`.
    /// * `Err(eyre::Report)` if an error occurs during initialization.
    ///
    /// # Errors
    /// This function returns an error if the initialization fails.
    pub async fn new(
        client: Clt,
        hostname: &str,
        backup_id: Uuid,
        number: usize,
        previous_id: Option<Uuid>,
        ctxt: &Context,
        state_tx: Option<mpsc::Sender<BackupState>>,
        config: Arc<Configuration>,
        backups: Arc<Backups>,
        hosts: Arc<Hosts>,
    ) -> Result<Self> {
        let existing_backup = backups.get_backup(hostname, backup_id).await;

        // Resolve the actual number: for a resume, trust what is stored in backup.yml.
        let number = existing_backup.as_ref().map(|b| b.number).unwrap_or(number);

        // For a resume the previous backup is already cloned — keep previous_id as-is.
        // For a fresh backup, use the caller-supplied previous_id (already computed by the
        // scheduler) so that init_backup_directory can clone the right manifest without
        // relying on backup.yml being updated first.
        if let Some(ref backup) = existing_backup {
            if backup.status.is_resumable() {
                info!(
                    "Resuming backup {hostname}#{} from status {:?}",
                    backup.number, backup.status
                );
            }
        }

        let client = BackupSave::new(
            client,
            hostname,
            backup_id,
            number,
            previous_id,
            ctxt,
            config.clone(),
            backups.clone(),
        );

        let host_configuration = hosts.get_host(hostname).await?;
        let progression_state = BackupState::from_configuration(&host_configuration);

        Ok(Self {
            config: config.clone(),

            client,
            host_configuration,

            progression_state: Arc::new(Mutex::new(progression_state)),
            state_tx,

            hostname: hostname.to_string(),
            backup_id,
            backups,
        })
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

    /// Executes the authentication process.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the authentication succeeds.
    /// * `Err(eyre::Report)` if an error occurs during authentication.
    ///
    /// # Errors
    ///
    /// Returns an error if the authentication fails.
    async fn execute_authentication(&self) -> Result<()> {
        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state.start_authentication();
        }
        self.send_progres().await;

        let auth_result = self
            .client
            .authenticate(&self.host_configuration.password)
            .await;

        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state
                .process_authentification_result(auth_result)
                .inspect_err(|_| {
                    if let Some(tx) = &self.state_tx {
                        let _ = tx.try_send(progression_state.clone());
                    }
                })?;
        }

        self.send_progres().await;
        Ok(())
    }

    /// Initializes the backup directory for the specified shares.
    ///
    /// # Arguments
    /// * `shares` - The list of shares to initialize.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the initialization succeeds.
    /// * `Err(eyre::Report)` if an error occurs during initialization.
    ///
    /// # Errors
    ///
    /// Returns an error if the initialization fails.
    async fn init_backup_directory(&self, shares: &[&str]) -> Result<()> {
        self.send_progres().await;

        let result = self.client.init_backup_directory(shares).await;
        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state
                .process_init_backup_directory_result(result)
                .inspect_err(|_| {
                    if let Some(tx) = &self.state_tx {
                        let _ = tx.try_send(progression_state.clone());
                    }
                })?;
        }

        self.send_progres().await;
        Ok(())
    }

    /// Executes the pre-backup command.
    ///
    /// # Arguments
    /// * `command` - The command to execute.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the command execution succeeds.
    /// * `Err(eyre::Report)` if an error occurs during command execution.
    ///
    /// # Errors
    ///
    /// Returns an error if the command execution fails.
    async fn execute_pre_command(&self, command: &ExecuteCommandOperation) -> Result<()> {
        self.send_progres().await;

        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state.start_pre_command(command);
        }
        self.send_progres().await;

        let result = self.client.execute_command(&command.command).await;

        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state
                .process_pre_command_result(command, result)
                .inspect_err(|_| {
                    if let Some(tx) = &self.state_tx {
                        let _ = tx.try_send(progression_state.clone());
                    }
                })?;
        }

        self.send_progres().await;
        Ok(())
    }

    /// Executes the pre-backup commands.
    ///
    /// # Arguments
    /// * `commands` - The list of commands to execute.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the command execution succeeds.
    /// * `Err(eyre::Report)` if an error occurs during command execution.
    ///
    /// # Errors
    ///
    /// Returns an error if the command execution fails.
    async fn execute_pre_commands(
        &self,
        commands: Option<&[ExecuteCommandOperation]>,
    ) -> Result<()> {
        if let Some(commands) = commands {
            for command in commands {
                self.execute_pre_command(command).await?;
            }
        }
        Ok(())
    }

    /// Executes the post-backup command.
    ///
    /// # Arguments
    /// * `command` - The command to execute.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the command execution succeeds.
    /// * `Err(eyre::Report)` if an error occurs during command execution.
    ///
    /// # Errors
    ///
    /// Returns an error if the command execution fails.
    async fn execute_post_command(&self, command: &ExecuteCommandOperation) -> Result<()> {
        self.send_progres().await;

        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state.start_post_command(command);
        }
        self.send_progres().await;

        let result = self.client.execute_command(&command.command).await;
        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state
                .process_post_command_result(command, result)
                .inspect_err(|_| {
                    if let Some(tx) = &self.state_tx {
                        let _ = tx.try_send(progression_state.clone());
                    }
                })?;
        }

        self.send_progres().await;
        Ok(())
    }

    /// Executes the post-backup commands.
    ///
    /// # Arguments
    /// * `commands` - The list of commands to execute.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the command execution succeeds.
    /// * `Err(eyre::Report)` if an error occurs during command execution.
    ///
    /// # Errors
    ///
    /// Returns an error if the command execution fails.
    async fn execute_post_commands(
        &self,
        commands: &Option<Vec<ExecuteCommandOperation>>,
    ) -> Result<()> {
        if let Some(ref commands) = commands {
            for command in commands {
                self.execute_post_command(command).await?;
            }
        }
        Ok(())
    }

    /// Synchronizes the file list for the specified share.
    ///
    /// # Arguments
    /// * `share` - The share to synchronize.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the synchronization succeeds.
    /// * `Err(eyre::Report)` if an error occurs during synchronization.
    ///
    /// # Errors
    ///
    /// Returns an error if the synchronization fails.
    async fn synchronize_file_list_for_share(&self, share: &Share) -> Result<()> {
        self.send_progres().await;

        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state.start_synchronize_file_list(&share.share_path);
        }
        self.send_progres().await;

        // Créer un canal pour recevoir les mises à jour de progression de la liste de fichiers
        let (file_tx, mut file_rx) = mpsc::channel(DEFAULT_CHANNEL_BUFFER_SIZE);

        // Spawn une tâche pour traiter les mises à jour de FileListProgression
        let share_path = share.share_path.clone();
        let state_tx_clone = self.state_tx.clone();
        let progression_state = self.progression_state.clone();

        let file_list_task = tokio::spawn(
            async move {
                while let Some(progress) = file_rx.recv().await {
                    let mut progression_state = progression_state.lock().await;
                    progression_state
                        .process_synchronize_file_list_progress(&share_path, &progress);
                    if let Some(tx) = &state_tx_clone {
                        if let Err(e) = tx.send(progression_state.clone()).await {
                            error!("Failed to send state update during file list: {}", e);
                        }
                    }
                }
            }
            .in_current_span(),
        );

        // Appeler synchronize_file_list avec le canal de progression
        let result = self
            .client
            .synchronize_file_list(share, Some(file_tx))
            .await;

        // Attendre que la tâche de mise à jour termine
        if let Err(e) = file_list_task.await {
            error!("Error in file list progression task: {}", e);
        }

        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state
                .process_synchronize_file_list_result(&share.share_path, result)
                .inspect_err(|_| {
                    if let Some(tx) = &self.state_tx {
                        let _ = tx.try_send(progression_state.clone());
                    }
                })?;
        }

        // Read and propagate snapshot result from agent
        if let Some(sr) = self.client.get_snapshot_result(&share.share_path).await {
            let method = SnapshotMethod::try_from(sr.method)
                .map(|m| match m {
                    SnapshotMethod::Btrfs => ShareSnapshotMethod::Btrfs,
                    SnapshotMethod::Vss => ShareSnapshotMethod::Vss,
                    SnapshotMethod::None => ShareSnapshotMethod::None,
                })
                .unwrap_or(ShareSnapshotMethod::None);
            let mut progression_state = self.progression_state.lock().await;
            progression_state.set_share_snapshot_result(
                &share.share_path,
                method,
                sr.failure_reason,
            );
        }

        self.send_progres().await;
        Ok(())
    }

    /// Synchronizes the file list for the specified operation.
    ///
    /// # Arguments
    /// * `operation` - The operation to synchronize.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the synchronization succeeds.
    /// * `Err(eyre::Report)` if an error occurs during synchronization.
    ///
    /// # Errors
    ///
    /// Returns an error if the synchronization fails.
    async fn synchronize_file_list(&self, operation: Option<&BackupOperation>) -> Result<()> {
        if let Some(ref operation) = operation {
            let includes = operation.includes.clone().unwrap_or_default();
            let excludes = operation.excludes.clone().unwrap_or_default();

            for share in &operation.shares {
                let mut share_includes = share.includes.clone().unwrap_or_default();
                let mut share_excludes = share.excludes.clone().unwrap_or_default();

                share_includes.extend(includes.clone());
                share_excludes.extend(excludes.clone());

                let share = Share {
                    includes: share_includes,
                    excludes: share_excludes,
                    share_path: share.name.clone(),
                };

                self.synchronize_file_list_for_share(&share).await?;
            }
        }
        Ok(())
    }

    /// Creates a backup for the specified share.
    ///
    /// # Arguments
    /// * `share_path` - The path of the share to back up.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the backup succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the backup process.
    ///
    /// # Errors
    ///
    /// Returns an error if the backup process fails.
    async fn create_backup_for_share(&self, share_path: &str) -> Result<()> {
        self.send_progres().await;

        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state.start_backup(share_path);
        }
        self.send_progres().await;

        // Créer un canal pour recevoir les mises à jour de progression de la sauvegarde
        let (backup_tx, mut backup_rx) = mpsc::channel(DEFAULT_CHANNEL_BUFFER_SIZE);

        // Spawn une tâche pour traiter les mises à jour de BackupProgression
        let progression_state_clone = self.progression_state.clone();
        let share_path_clone = share_path.to_string();
        let state_tx_clone = self.state_tx.clone();

        let backup_task = tokio::spawn(
            async move {
                while let Some(progress) = backup_rx.recv().await {
                    let mut progression_state = progression_state_clone.lock().await;
                    progression_state.process_backup_progress(&share_path_clone, &progress);
                    if let Some(tx) = &state_tx_clone {
                        if let Err(e) = tx.send(progression_state.clone()).await {
                            error!("Failed to send state update during file list: {}", e);
                        }
                    }
                }
            }
            .in_current_span(),
        );

        // Appeler create_backup avec le canal de progression
        let result = self.client.create_backup(share_path, Some(backup_tx)).await;

        // Attendre que la tâche de mise à jour termine
        if let Err(e) = backup_task.await {
            error!("Error in backup progression task: {}", e);
        }

        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state
                .process_backup_result(share_path, result)
                .inspect_err(|_| {
                    if let Some(tx) = &self.state_tx {
                        let _ = tx.try_send(progression_state.clone());
                    }
                })?;
        }

        self.send_progres().await;
        Ok(())
    }

    /// Creates a backup for the specified operation.
    ///
    /// # Arguments
    /// * `operation` - The operation to back up.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the backup succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the backup process.
    ///
    /// # Errors
    ///
    /// Returns an error if the backup process fails.
    async fn create_backup(&self, operation: &Option<BackupOperation>) -> Result<()> {
        if let Some(ref operation) = operation {
            for share in &operation.shares {
                self.create_backup_for_share(&share.name).await?;
            }
        }
        Ok(())
    }

    /// Recovers in-memory progression stats from on-disk journal/log for a single share.
    async fn recover_progression_for_share(&self, share: &str) -> Result<()> {
        self.client.recover_progression(share).await
    }

    /// Recovers in-memory progression stats for all shares in the operation.
    ///
    /// # Errors
    ///
    /// Returns an error if recovering a share fails.
    async fn recover_progression(&self, operation: Option<&BackupOperation>) -> Result<()> {
        if let Some(ref operation) = operation {
            for share in &operation.shares {
                self.recover_progression_for_share(&share.name).await?;
            }
        }
        Ok(())
    }

    /// Compacts the backup for the specified share.
    ///
    /// # Arguments
    /// * `share` - The share to compact.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the compaction succeeds.
    /// * `Err(eyre::Report)` if an error occurs during compaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the compaction fails.
    async fn compact_backup_for_share(&self, share: &str) -> Result<()> {
        self.send_progres().await;

        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state.start_compact(share);
        }
        self.send_progres().await;

        let result = self.client.compact(share).await;
        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state
                .process_compact_result(share, result)
                .inspect_err(|_| {
                    if let Some(tx) = &self.state_tx {
                        let _ = tx.try_send(progression_state.clone());
                    }
                })?;
        }

        self.send_progres().await;
        Ok(())
    }

    /// Compacts the backup for the specified operation.
    ///
    /// # Arguments
    /// * `operation` - The operation to compact.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the compaction succeeds.
    /// * `Err(eyre::Report)` if an error occurs during compaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the compaction fails.
    async fn compact_backup(&self, operation: Option<&BackupOperation>) -> Result<()> {
        if let Some(ref operation) = operation {
            for share in &operation.shares {
                self.compact_backup_for_share(&share.name).await?;
            }
        }
        Ok(())
    }

    /// Counts the references in the backup.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the reference count succeeds.
    /// * `Err(eyre::Report)` if an error occurs during reference counting.
    ///
    /// # Errors
    ///
    /// Returns an error if the reference counting fails.
    async fn count_references(&self) -> Result<()> {
        self.send_progres().await;

        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state.start_count_references();
        }
        self.send_progres().await;

        let result = self.client.count_references().await;
        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state
                .process_count_references_result(result)
                .inspect_err(|_| {
                    if let Some(tx) = &self.state_tx {
                        let _ = tx.try_send(progression_state.clone());
                    }
                })?;
        }

        self.send_progres().await;
        Ok(())
    }

    /// Add a reference count to the pool for the current backup.
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

    /// Executes the backup phase (authentication, init, commands, file sync, backup creation).
    ///
    /// # Returns
    ///
    /// * `Ok(BackupStatus::Completed)` if backup phase succeeds without errors.
    /// * `Ok(BackupStatus::Aborted)` if backup phase completes but with errors.
    /// * `Err(eyre::Report)` if a critical error occurs (auth/init failures).
    async fn execute_backup_phase(&mut self) -> Result<BackupStatus> {
        info!(
            "Executing backup phase for {}/{}",
            self.hostname, self.backup_id
        );

        let pre_commands = &self.host_configuration.operations.pre_commands;
        let post_commands = &self.host_configuration.operations.post_commands;
        let operation = &self.host_configuration.operations.operation;

        // Critical operations - fail fast
        self.execute_authentication().await?;

        let shares = self
            .host_configuration
            .operations
            .operation
            .as_ref()
            .map(|op| {
                op.shares
                    .iter()
                    .map(|share| share.name.as_str())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        self.init_backup_directory(&shares).await?;

        // Non-critical operations - mark as aborted but continue to compact/refcount
        let mut status = BackupStatus::Completed;

        if self
            .execute_pre_commands(pre_commands.as_deref())
            .await
            .is_err()
        {
            status = BackupStatus::Aborted;
        }

        if !status.is_aborted()
            && self
                .synchronize_file_list(operation.as_ref())
                .await
                .is_err()
        {
            status = BackupStatus::Aborted;
        }

        if !status.is_aborted() && self.create_backup(operation).await.is_err() {
            status = BackupStatus::Aborted;
        }

        if !status.is_aborted() && self.execute_post_commands(post_commands).await.is_err() {
            status = BackupStatus::Aborted;
        }

        if let Err(err) = self.client.close().await {
            error!("Error closing the connection: {}", err);
            status = BackupStatus::Aborted;
        }

        Ok(status)
    }

    /// Executes the recover phase: rebuilds in-memory progression stats from on-disk
    /// journal/log before resuming a previously interrupted backup.
    ///
    /// A failure here is non-fatal — the backup data is intact; only the statistics
    /// would be missing. The error is logged and the next phase proceeds regardless.
    async fn execute_recover_phase(&mut self, next_phase: BackupPhase) -> BackupPhase {
        info!(
            "Executing recover phase for {}/{}",
            self.hostname, self.backup_id
        );

        let operation = self.host_configuration.operations.operation.clone();
        if let Err(err) = self.recover_progression(operation.as_ref()).await {
            error!(
                "Error during recover phase for {}/{}: {err} — stats will be zero",
                self.hostname, self.backup_id
            );
        }

        next_phase
    }

    /// Executes the compact phase.
    ///
    /// # Arguments
    /// * `current_status` - Current backup status (to preserve Aborted state)
    ///
    /// # Returns
    ///
    /// * `Ok(())` if compaction succeeds.
    /// * `Err(eyre::Report)` if compaction fails.
    async fn execute_compact_phase(&mut self, current_status: &BackupStatus) -> Result<()> {
        info!(
            "Executing compact phase for {}/{}",
            self.hostname, self.backup_id
        );

        let status = Self::get_status_for_stage(current_status, FinishingStatus::ToCompact);
        self.client.save_backup(status).await?;

        let operation = &self.host_configuration.operations.operation;
        self.compact_backup(operation.as_ref()).await?;

        Ok(())
    }

    /// Executes the count references phase.
    ///
    /// # Arguments
    /// * `current_status` - Current backup status (to preserve Aborted state)
    ///
    /// # Returns
    ///
    /// * `Ok(())` if counting references succeeds.
    /// * `Err(eyre::Report)` if counting references fails.
    async fn execute_count_references_phase(
        &mut self,
        current_status: &BackupStatus,
    ) -> Result<()> {
        info!(
            "Executing count references phase for {}/{}",
            self.hostname, self.backup_id
        );

        let status = Self::get_status_for_stage(current_status, FinishingStatus::ToCountRef);
        self.client.save_backup(status).await?;

        self.count_references().await?;

        Ok(())
    }

    /// Executes the add to pool phase.
    ///
    /// # Arguments
    /// * `current_status` - Current backup status (to preserve Aborted state)
    ///
    /// # Returns
    ///
    /// * `Ok(())` if adding to pool succeeds.
    /// * `Err(eyre::Report)` if adding to pool fails.
    async fn execute_add_to_pool_phase(&mut self, current_status: &BackupStatus) -> Result<()> {
        info!(
            "Executing add to pool phase for {}/{}",
            self.hostname, self.backup_id
        );

        let status = Self::get_status_for_stage(current_status, FinishingStatus::ToAddInPool);
        self.client.save_backup(status).await?;

        self.add_refcnt_to_pool().await?;

        Ok(())
    }

    /// Exécute le processus de sauvegarde avec notification des états via un canal mpsc
    ///
    /// @param `state_rx` Récepteur optionnel pour recevoir les états de progression
    /// @return Le résultat de l'opération de sauvegarde
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the backup process succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the backup process.
    ///
    /// # Errors
    ///
    /// Returns an error if any step of the backup process fails.
    pub async fn execute(&mut self) -> Result<BackupState> {
        let pool_directory = &self.config.path.pool_path;
        let redis_url = self.config.redis_url();
        let _lock = PoolLockRedis::new_with_path(
            &redis_url,
            pool_directory,
            LockOperation::Pool(PoolLockOperation::SaveBackup),
        )
        .await?
        .try_lock_shared_wait(Duration::from_secs(60))
        .await?
        .ok_or_else(|| {
            eyre!("Timed out waiting for shared pool lock during backup finalization")
        })?;

        self.send_progres().await;

        // Determine initial state and starting phase
        let existing_backup = self
            .backups
            .get_backup(&self.hostname, self.backup_id)
            .await;

        let existing_status = existing_backup.map(|b| b.status.clone());
        let (mut current_phase, mut current_status) =
            BackupPhase::from_existing(existing_status.clone());

        info!(
            "Backup state machine starting for {}/{}: existing_status={:?}, starting_phase={:?}, target_status={:?}",
            self.hostname, self.backup_id, existing_status, current_phase, current_status
        );

        // State machine main loop
        loop {
            match current_phase {
                BackupPhase::Recover(next_phase) => {
                    current_phase = self.execute_recover_phase(*next_phase).await;
                }

                BackupPhase::Backup => match self.execute_backup_phase().await {
                    Ok(status) => {
                        current_status = status;
                        current_phase = BackupPhase::Compact;
                    }
                    Err(err) => {
                        // Even on a critical backup phase failure (e.g. host unreachable,
                        // auth error, mid-stream disconnect), we must still run
                        // compact/count_references/add_refcnt_to_pool to properly close the
                        // backup. These phases are purely local and do not require the
                        // remote agent to be reachable.
                        error!(
                            "Critical error during backup phase for {}/{}: {err} — \
                             proceeding to finalize and abort",
                            self.hostname, self.backup_id
                        );
                        current_status = BackupStatus::Aborted;
                        current_phase = BackupPhase::Compact;
                    }
                },

                BackupPhase::Compact => match self.execute_compact_phase(&current_status).await {
                    Ok(()) => {
                        current_phase = BackupPhase::CountReferences;
                    }
                    Err(err) => {
                        error!("Error during compact phase: {err}");
                        let failed_status = if current_status.is_aborted() {
                            BackupStatus::Aborted
                        } else {
                            BackupStatus::Failed(FailedStatus::Compact)
                        };
                        self.client.save_backup(failed_status).await?;
                        break;
                    }
                },

                BackupPhase::CountReferences => {
                    match self.execute_count_references_phase(&current_status).await {
                        Ok(()) => {
                            current_phase = BackupPhase::AddToPool;
                        }
                        Err(err) => {
                            error!("Error during count references phase: {err}");
                            let failed_status = if current_status.is_aborted() {
                                BackupStatus::Aborted
                            } else {
                                BackupStatus::Failed(FailedStatus::RefCount)
                            };
                            self.client.save_backup(failed_status).await?;
                            break;
                        }
                    }
                }

                BackupPhase::AddToPool => {
                    match self.execute_add_to_pool_phase(&current_status).await {
                        Ok(()) => {
                            current_phase = BackupPhase::Finished;
                        }
                        Err(err) => {
                            error!("Error during add to pool phase: {err}");
                            let failed_status = if current_status.is_aborted() {
                                BackupStatus::Aborted
                            } else {
                                BackupStatus::Failed(FailedStatus::InPool)
                            };
                            self.client.save_backup(failed_status).await?;
                            break;
                        }
                    }
                }

                BackupPhase::Finished => {
                    info!(
                        "Backup state machine completed for {}/{} with status {:?}",
                        self.hostname, self.backup_id, current_status
                    );
                    self.client.save_backup(current_status).await?;
                    break;
                }
            }
        }

        let state = self.progression_state.lock().await.clone();
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{FailedStatus, FinishingStatus, RemovingStatus};

    // ── from_existing ────────────────────────────────────────────────────────

    #[test]
    fn test_new_backup() {
        assert_eq!(
            BackupPhase::from_existing(None),
            (BackupPhase::Backup, BackupStatus::Completed)
        );
    }

    #[test]
    fn test_crashed_mid_transfer_recovers_then_compact_as_aborted() {
        // InProgress with an existing entry means worker crashed during chunk transfer.
        // Must NOT re-download. Must recover stats then close as Aborted.
        assert_eq!(
            BackupPhase::from_existing(Some(BackupStatus::InProgress)),
            (
                BackupPhase::Recover(Box::new(BackupPhase::Compact)),
                BackupStatus::Aborted
            )
        );
    }

    #[test]
    fn test_finishing_stages_recover_then_resume_towards_completed() {
        assert_eq!(
            BackupPhase::from_existing(Some(BackupStatus::Finishing(FinishingStatus::ToCompact))),
            (
                BackupPhase::Recover(Box::new(BackupPhase::Compact)),
                BackupStatus::Completed
            )
        );
        assert_eq!(
            BackupPhase::from_existing(Some(BackupStatus::Finishing(FinishingStatus::ToCountRef))),
            (
                BackupPhase::Recover(Box::new(BackupPhase::CountReferences)),
                BackupStatus::Completed
            )
        );
        assert_eq!(
            BackupPhase::from_existing(Some(BackupStatus::Finishing(FinishingStatus::ToAddInPool))),
            (
                BackupPhase::Recover(Box::new(BackupPhase::AddToPool)),
                BackupStatus::Completed
            )
        );
    }

    #[test]
    fn test_aborting_stages_recover_then_resume_towards_aborted() {
        assert_eq!(
            BackupPhase::from_existing(Some(BackupStatus::Aborting(FinishingStatus::ToCompact))),
            (
                BackupPhase::Recover(Box::new(BackupPhase::Compact)),
                BackupStatus::Aborted
            )
        );
        assert_eq!(
            BackupPhase::from_existing(Some(BackupStatus::Aborting(FinishingStatus::ToCountRef))),
            (
                BackupPhase::Recover(Box::new(BackupPhase::CountReferences)),
                BackupStatus::Aborted
            )
        );
        assert_eq!(
            BackupPhase::from_existing(Some(BackupStatus::Aborting(FinishingStatus::ToAddInPool))),
            (
                BackupPhase::Recover(Box::new(BackupPhase::AddToPool)),
                BackupStatus::Aborted
            )
        );
    }

    #[test]
    fn test_failed_stages_recover_then_retry_from_failed_phase() {
        assert_eq!(
            BackupPhase::from_existing(Some(BackupStatus::Failed(FailedStatus::Compact))),
            (
                BackupPhase::Recover(Box::new(BackupPhase::Compact)),
                BackupStatus::Completed
            )
        );
        assert_eq!(
            BackupPhase::from_existing(Some(BackupStatus::Failed(FailedStatus::RefCount))),
            (
                BackupPhase::Recover(Box::new(BackupPhase::CountReferences)),
                BackupStatus::Completed
            )
        );
        assert_eq!(
            BackupPhase::from_existing(Some(BackupStatus::Failed(FailedStatus::InPool))),
            (
                BackupPhase::Recover(Box::new(BackupPhase::AddToPool)),
                BackupStatus::Completed
            )
        );
    }

    #[test]
    fn test_terminal_states_go_to_finished() {
        assert_eq!(
            BackupPhase::from_existing(Some(BackupStatus::Completed)),
            (BackupPhase::Finished, BackupStatus::Completed)
        );
        assert_eq!(
            BackupPhase::from_existing(Some(BackupStatus::Aborted)),
            (BackupPhase::Finished, BackupStatus::Aborted)
        );
    }

    #[test]
    fn test_removing_states_go_to_finished() {
        assert_eq!(
            BackupPhase::from_existing(Some(BackupStatus::Removing(
                RemovingStatus::ToRemoveInPool
            ))),
            (BackupPhase::Finished, BackupStatus::Aborted)
        );
        assert_eq!(
            BackupPhase::from_existing(Some(BackupStatus::Removing(
                RemovingStatus::RemoveFromHost
            ))),
            (BackupPhase::Finished, BackupStatus::Aborted)
        );
    }

    // ── get_status_for_stage ─────────────────────────────────────────────────

    #[test]
    fn test_get_status_for_stage_normal_backup() {
        let current = BackupStatus::Completed;
        assert_eq!(
            SaveBackupMachine::<crate::server::client::grpc::BackupGrpcClient>::get_status_for_stage(
                &current,
                FinishingStatus::ToCompact
            ),
            BackupStatus::Finishing(FinishingStatus::ToCompact)
        );
    }

    #[test]
    fn test_get_status_for_stage_aborted_backup() {
        let current = BackupStatus::Aborted;
        assert_eq!(
            SaveBackupMachine::<crate::server::client::grpc::BackupGrpcClient>::get_status_for_stage(
                &current,
                FinishingStatus::ToCountRef
            ),
            BackupStatus::Aborting(FinishingStatus::ToCountRef)
        );
    }

    #[test]
    fn test_get_status_for_stage_preserves_aborting() {
        let current = BackupStatus::Aborting(FinishingStatus::ToCompact);
        assert_eq!(
            SaveBackupMachine::<crate::server::client::grpc::BackupGrpcClient>::get_status_for_stage(
                &current,
                FinishingStatus::ToAddInPool
            ),
            BackupStatus::Aborting(FinishingStatus::ToAddInPool)
        );
    }
}
