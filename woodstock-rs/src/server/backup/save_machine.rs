use std::sync::Arc;

use eyre::Result;
use log::error;
use tokio::sync::{mpsc, Mutex};

use crate::{
    config::{
        BackupOperation, BackupStatus, Configuration, Context, ExecuteCommandOperation,
        HostConfiguration, Hosts, DEFAULT_CHANNEL_BUFFER_SIZE,
    },
    server::client::Client,
    utils::{lock::PoolLock, thread::spawn_with_context},
    Share,
};

use super::{save::BackupSave, save_state::BackupState};

pub struct SaveBackupMachine<Clt: Client> {
    /// The configuration of the application, containing paths and other settings.
    config: Configuration,

    /// The client used for backup operations.
    client: BackupSave<Clt>,

    /// The configuration of the host being backed up.
    host_configuration: HostConfiguration,

    /// The current progression state of the backup.
    progression_state: Arc<Mutex<BackupState>>,

    /// An optional sender for transmitting backup state updates.
    state_tx: Option<mpsc::Sender<BackupState>>,
}

impl<Clt: Client> SaveBackupMachine<Clt> {
    /// Creates a new instance of `SaveBackupMachine`.
    ///
    /// # Arguments
    /// * `client` - The client used for saving backups.
    /// * `hostname` - The hostname of the backup to save.
    /// * `backup_number` - The backup number to save.
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
        backup_number: usize,
        ctxt: &Context,
        config: &Configuration,
        state_tx: Option<mpsc::Sender<BackupState>>,
    ) -> Result<Self> {
        let client = BackupSave::new(client, hostname, backup_number, ctxt, config);

        let hosts = Hosts::new(config);
        let host_configuration = hosts.get_host(hostname).await?;
        let progression_state = BackupState::from_configuration(&host_configuration);

        Ok(Self {
            config: config.clone(),

            client,
            host_configuration,

            progression_state: Arc::new(Mutex::new(progression_state)),
            state_tx,
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

        let file_list_task = spawn_with_context(async move {
            while let Some(progress) = file_rx.recv().await {
                let mut progression_state = progression_state.lock().await;
                progression_state.process_synchronize_file_list_progress(&share_path, &progress);
                if let Some(tx) = &state_tx_clone {
                    if let Err(e) = tx.send(progression_state.clone()).await {
                        error!("Failed to send state update during file list: {}", e);
                    }
                }
            }
        });

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

        let backup_task = spawn_with_context(async move {
            while let Some(progress) = backup_rx.recv().await {
                let mut progression_state = progression_state_clone.lock().await;
                progression_state.process_backup_progress(&share_path_clone, &progress);
                if let Some(tx) = &state_tx_clone {
                    if let Err(e) = tx.send(progression_state.clone()).await {
                        error!("Failed to send state update during file list: {}", e);
                    }
                }
            }
        });

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
    pub async fn execute(&mut self) -> Result<()> {
        let pool_directory = &self.config.path.pool_path;
        let _lock = PoolLock::new_with_name(&pool_directory, "save_backup")
            .lock_shared()
            .await?;

        let mut status = BackupStatus::Completed;
        let pre_commands = &self.host_configuration.operations.pre_commands;
        let post_commands = &self.host_configuration.operations.post_commands;
        let operation = &self.host_configuration.operations.operation;

        // Créer un canal si aucun n'a été fourni
        self.send_progres().await;

        // In case of error, stop here!
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

        // In case of error, stop here!
        self.init_backup_directory(&shares).await?;

        // Now, in case of error, we must compact and refcount
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

        if let Err(err) = self.compact_backup(operation.as_ref()).await {
            error!("Error compacting backup: {err}");
            status = BackupStatus::Failed;
        }

        if let Err(err) = self.count_references().await {
            error!("Error counting references: {err}");
            status = BackupStatus::Failed;
        }

        if let Err(err) = self.add_refcnt_to_pool().await {
            error!("Error adding reference count to pool: {err}");
            status = BackupStatus::Failed;
        }

        self.client.save_backup(status).await?;

        Ok(())
    }
}
