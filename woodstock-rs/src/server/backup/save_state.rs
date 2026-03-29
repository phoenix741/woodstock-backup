use std::collections::HashMap;

use chrono::Local;
use eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{
    config::{ShareSnapshotMethod, ExecuteCommandOperation, HostConfiguration},
    server::progression::{BackupProgression, FileListProgression},
    ExecuteCommandReply,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ErrorState {
    AuthenticationError(String),
    InitializationError(String),
    CommandExecutionError(String),
    BackupError(String),
    CompactError(String),
    CountReferencesError(String),
    AddReferencesToPoolError(String),
    Unknown(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BackupExecutionState {
    Waiting,
    Skipped,
    Authenticate,
    Initialization,
    PreCommands(ExecuteCommandOperation),
    DownloadFileList(String),
    DownloadChunks(String),
    PostCommands(ExecuteCommandOperation),
    Compact(String),
    CountReferences,
    AddReferencesToPool,
    Completed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackupMachineCommandResult {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl From<ExecuteCommandReply> for BackupMachineCommandResult {
    fn from(reply: ExecuteCommandReply) -> Self {
        Self {
            code: reply.code,
            stdout: reply.stdout,
            stderr: reply.stderr,
        }
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub enum ExecuteCommandExecutionState {
    #[default]
    Waiting,
    InProgress,
    Success(BackupMachineCommandResult),
    Failed(BackupMachineCommandResult),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecuteCommandState {
    pub command: ExecuteCommandOperation,
    pub execution_state: ExecuteCommandExecutionState,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub enum ShareExecutionState {
    #[default]
    Waiting,
    FileList,
    InProgress,
    Success,
    Failed(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShareState {
    pub share: String,
    pub file_list_progression: FileListProgression,
    pub backup_progression: BackupProgression,
    pub execution_state: ShareExecutionState,
    pub snapshot_method: ShareSnapshotMethod,
    pub snapshot_failure_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackupState {
    pub execution_state: BackupExecutionState,
    pub error_state: Option<ErrorState>,
    pub global_progression: BackupProgression,
    /// Keyed by command string to ensure JSON serialization compatibility.
    pub pre_command_states: HashMap<String, ExecuteCommandState>,
    pub share_states: HashMap<String, ShareState>,
    /// Keyed by command string to ensure JSON serialization compatibility.
    pub post_command_states: HashMap<String, ExecuteCommandState>,
}

/// Creates a map of command states from a list of commands.
///
/// # Arguments
/// * `commands` - An optional list of commands to initialize states for.
///
/// # Returns
/// A `HashMap` where each command is associated with its initial state.
///
/// # Errors
/// This function does not return errors.
///
/// # Panics
/// This function does not panic.
fn create_command_states(
    commands: Option<&[ExecuteCommandOperation]>,
) -> HashMap<String, ExecuteCommandState> {
    let mut states = HashMap::new();
    if let Some(cmds) = commands {
        for cmd in cmds {
            states.insert(
                cmd.command.clone(),
                ExecuteCommandState {
                    command: cmd.clone(),
                    execution_state: ExecuteCommandExecutionState::Waiting,
                },
            );
        }
    }
    states
}

/// Creates a map of share states from a list of share names.
///
/// # Arguments
/// * `shares` - A slice of share names to initialize states for.
///
/// # Returns
/// A `HashMap` where each share name is associated with its initial state.
///
/// # Errors
/// This function does not return errors.
///
/// # Panics
/// This function does not panic.
fn create_share_states(shares: &[String]) -> HashMap<String, ShareState> {
    let mut states = HashMap::new();
    for share in shares {
        states.insert(
            share.clone(),
            ShareState {
                share: share.clone(),
                file_list_progression: FileListProgression::default(),
                backup_progression: BackupProgression::default(),
                execution_state: ShareExecutionState::Waiting,
                snapshot_method: ShareSnapshotMethod::None,
                snapshot_failure_reason: None,
            },
        );
    }
    states
}

impl BackupState {
    /// Creates a new `BackupState` from the given host configuration.
    ///
    /// # Arguments
    /// * `configuration` - The host configuration to initialize the state.
    ///
    /// # Returns
    ///
    /// A new instance of `BackupState`.
    #[must_use]
    pub fn from_configuration(configuration: &HostConfiguration) -> Self {
        // Helper function to create command states from operations

        let pre_command_states =
            create_command_states(configuration.operations.pre_commands.as_deref());
        let post_command_states =
            create_command_states(configuration.operations.post_commands.as_deref());
        let share_states = match &configuration.operations.operation {
            Some(op) => {
                let share_strings: Vec<String> =
                    op.shares.iter().map(|share| share.name.clone()).collect();
                create_share_states(&share_strings)
            }
            None => create_share_states(&[]),
        };

        Self {
            execution_state: BackupExecutionState::Waiting,
            error_state: None,
            global_progression: BackupProgression::default(),
            pre_command_states,
            share_states,
            post_command_states,
        }
    }

    pub fn calculate_global_progression(&self) -> BackupProgression {
        let mut global_progression = BackupProgression::default();
        global_progression.start_date = self.global_progression.start_date;
        global_progression.start_transfer_date = self.global_progression.start_transfer_date;
        global_progression.end_transfer_date = self.global_progression.end_transfer_date;

        // Calculate global progression based on individual states
        for share_state in self.share_states.values() {
            global_progression.progress_max += share_state.file_list_progression.file_size;
            global_progression.progress_current += share_state.backup_progression.progress_current;

            global_progression.error_count += share_state.backup_progression.error_count;

            global_progression.file_count += share_state.backup_progression.file_count;
            global_progression.new_file_count += share_state.backup_progression.new_file_count;
            global_progression.modified_file_count +=
                share_state.backup_progression.modified_file_count;
            global_progression.removed_file_count +=
                share_state.backup_progression.removed_file_count;

            global_progression.file_size += share_state.backup_progression.file_size;
            global_progression.new_file_size += share_state.backup_progression.new_file_size;
            global_progression.modified_file_size +=
                share_state.backup_progression.modified_file_size;

            global_progression.compressed_file_size +=
                share_state.backup_progression.compressed_file_size;
            global_progression.new_compressed_file_size +=
                share_state.backup_progression.new_compressed_file_size;
            global_progression.modified_compressed_file_size +=
                share_state.backup_progression.modified_compressed_file_size;

            global_progression.removed_file_count +=
                share_state.backup_progression.removed_file_count;
        }

        global_progression
    }

    /// Starts the authentication process by updating the execution state.
    pub fn start_authentication(&mut self) {
        self.execution_state = BackupExecutionState::Authenticate;
    }

    /// Processes the result of the authentication process.
    ///
    /// # Arguments
    /// * `result` - The result of the authentication operation.
    ///
    /// # Returns
    /// * `Ok(())` if the authentication was successful.
    /// * `Err(eyre::Report)` if an error occurred during authentication.
    ///
    /// # Errors
    /// This function returns an error if the authentication operation fails.
    pub fn process_authentification_result(&mut self, result: Result<()>) -> Result<()> {
        match result {
            Ok(()) => {
                info!("Authentication successful");
                self.execution_state = BackupExecutionState::Initialization;

                Ok(())
            }
            Err(err) => {
                error!("Error authenticating: {}", err);
                self.error_state = Some(ErrorState::AuthenticationError(err.to_string()));

                Err(err)
            }
        }
    }

    /// Processes the result of the backup directory initialization.
    ///
    /// # Arguments
    /// * `result` - The result of the initialization operation.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the initialization was successful.
    /// * `Err(eyre::Report)` if an error occurred during initialization.
    ///
    /// # Errors
    /// This function returns an error if the initialization operation fails.
    pub fn process_init_backup_directory_result(&mut self, result: Result<()>) -> Result<()> {
        match result {
            Ok(()) => {
                info!("Initialization successful");

                Ok(())
            }
            Err(err) => {
                error!("Error initializing backup directory: {}", err);
                self.error_state = Some(ErrorState::InitializationError(err.to_string()));

                Err(err)
            }
        }
    }

    /// Starts the execution of a pre-command.
    ///
    /// # Arguments
    /// * `command` - The pre-command to execute.
    pub fn start_pre_command(&mut self, command: &ExecuteCommandOperation) {
        self.execution_state = BackupExecutionState::PreCommands(command.clone());
        Self::set_command_in_progress(command, &mut self.pre_command_states);
    }

    /// Starts the execution of a post-command.
    ///
    /// # Arguments
    /// * `command` - The post-command to execute.
    pub fn start_post_command(&mut self, command: &ExecuteCommandOperation) {
        self.execution_state = BackupExecutionState::PostCommands(command.clone());
        Self::set_command_in_progress(command, &mut self.post_command_states);
    }

    /// Sets the state of a command to `InProgress`.
    ///
    /// # Arguments
    /// * `command` - The command whose state is to be updated.
    /// * `command_states` - A mutable reference to the map of command states.
    ///
    /// # Panics
    /// This function does not panic.
    fn set_command_in_progress(
        command: &ExecuteCommandOperation,
        command_states: &mut HashMap<String, ExecuteCommandState>,
    ) {
        if let Some(state) = command_states.get_mut(&command.command) {
            state.execution_state = ExecuteCommandExecutionState::InProgress;
        }
    }

    /// Processes the result of a pre-command execution.
    ///
    /// # Arguments
    /// * `command` - The pre-command that was executed.
    /// * `result` - The result of the pre-command execution.
    ///
    /// # Returns
    /// * `Ok(())` if the pre-command execution was successful.
    /// * `Err(eyre::Report)` if an error occurred during execution.
    ///
    /// # Errors
    /// This function returns an error if the pre-command execution fails.
    pub fn process_pre_command_result(
        &mut self,
        command: &ExecuteCommandOperation,
        result: Result<ExecuteCommandReply>,
    ) -> Result<()> {
        Self::process_command_result_internal(command, result, &mut self.pre_command_states)
            .inspect_err(|err| {
                self.error_state = Some(ErrorState::CommandExecutionError(err.to_string()));
            })
    }

    /// Processes the result of a post-command execution.
    ///
    /// # Arguments
    /// * `command` - The post-command that was executed.
    /// * `result` - The result of the post-command execution.
    ///
    /// # Returns
    /// * `Ok(())` if the post-command execution was successful.
    /// * `Err(eyre::Report)` if an error occurred during execution.
    ///
    /// # Errors
    /// This function returns an error if the post-command execution fails.
    pub fn process_post_command_result(
        &mut self,
        command: &ExecuteCommandOperation,
        result: Result<ExecuteCommandReply>,
    ) -> Result<()> {
        Self::process_command_result_internal(command, result, &mut self.post_command_states)
            .inspect_err(|err| {
                self.error_state = Some(ErrorState::CommandExecutionError(err.to_string()));
            })
    }

    /// Processes the result of a command execution and updates the command state accordingly.
    ///
    /// This function is responsible for handling the result of a command execution, updating the
    /// corresponding command state in the provided `command_states` map. It logs the outcome and
    /// returns an error if the command failed or if the execution itself resulted in an error.
    ///
    /// # Arguments
    ///
    /// * `command` - The command operation that was executed.
    /// * `result` - The result of the command execution, containing either a successful reply or an error.
    /// * `command_states` - A mutable reference to the map of command states to update.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the command was successful and the state was updated.
    /// * `Err(eyre::Report)` if the command failed or if an error occurred during execution.
    ///
    /// # Errors
    ///
    /// Returns an error if the command execution fails or if the command returns a non-zero exit code.
    fn process_command_result_internal(
        command: &ExecuteCommandOperation,
        result: Result<ExecuteCommandReply>,
        command_states: &mut HashMap<String, ExecuteCommandState>,
    ) -> Result<()> {
        match result {
            Ok(reply) => {
                info!(
                    "Command {} executed with code {}",
                    command.command, reply.code
                );
                if !reply.stdout.is_empty() {
                    info!("{}", reply.stdout);
                }
                if !reply.stderr.is_empty() {
                    error!("{}", reply.stderr);
                }

                let code = reply.code;
                if let Some(state) = command_states.get_mut(&command.command) {
                    if code != 0 {
                        state.execution_state = ExecuteCommandExecutionState::Failed(reply.into());
                        return Err(eyre::eyre!("Command {command} failed with code {code}"));
                    }

                    state.execution_state = ExecuteCommandExecutionState::Success(reply.into());
                }

                Ok(())
            }
            Err(err) => {
                error!("Error executing command: {}", err);
                Err(err)
            }
        }
    }

    /// Starts the synchronization of the file list for a given share.
    ///
    /// # Arguments
    /// * `share` - The name of the share to synchronize.
    pub fn start_synchronize_file_list(&mut self, share: &str) {
        self.execution_state = BackupExecutionState::DownloadFileList(share.to_string());
        if let Some(share_state) = self.share_states.get_mut(share) {
            share_state.execution_state = ShareExecutionState::FileList;
        }
    }

    /// Processes the progress of the file list synchronization.
    ///
    /// # Arguments
    /// * `share` - The name of the share being synchronized.
    /// * `progression` - The current progression of the file list synchronization.
    pub fn process_synchronize_file_list_progress(
        &mut self,
        share: &str,
        progression: &FileListProgression,
    ) {
        if let Some(share_state) = self.share_states.get_mut(share) {
            share_state.file_list_progression = progression.clone();
        }
        self.global_progression = self.calculate_global_progression();
    }

    /// Processes the result of the synchronization of the file list for a given share.
    ///
    /// # Arguments
    /// * `share` - The name of the share being synchronized.
    /// * `result` - The result of the synchronization operation.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the synchronization was successful.
    /// * `Err(eyre::Report)` if an error occurred during synchronization.
    ///
    /// # Errors
    /// This function returns an error if the synchronization fails.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn process_synchronize_file_list_result(
        &mut self,
        share: &str,
        result: Result<()>,
    ) -> Result<()> {
        match result {
            Ok(()) => {
                info!("File list synchronization for share '{share}' successful");
                let system_time = Local::now();
                if let Some(share_state) = self.share_states.get_mut(share) {
                    share_state.execution_state = ShareExecutionState::InProgress;
                    share_state.backup_progression.start_transfer_date = Some(system_time);
                }
                self.global_progression.start_transfer_date = Some(system_time);
                Ok(())
            }
            Err(err) => {
                error!("Error synchronizing file list for share '{share}': {err}");
                self.error_state = Some(ErrorState::BackupError(err.to_string()));
                if let Some(share_state) = self.share_states.get_mut(share) {
                    share_state.execution_state = ShareExecutionState::Failed(err.to_string());
                }
                Err(err)
            }
        }
    }

    /// Starts the backup process for a given share.
    ///
    /// # Arguments
    /// * `share` - The name of the share to back up.
    pub fn start_backup(&mut self, share: &str) {
        self.execution_state = BackupExecutionState::DownloadChunks(share.to_string());
        if let Some(share_state) = self.share_states.get_mut(share) {
            share_state.backup_progression.start_transfer_date = Some(Local::now());
            share_state.execution_state = ShareExecutionState::InProgress;
        }
    }

    /// Processes the progress of the backup operation.
    ///
    /// # Arguments
    /// * `share` - The name of the share being backed up.
    /// * `progression` - The current progression of the backup operation.
    pub fn process_backup_progress(&mut self, share: &str, progression: &BackupProgression) {
        if let Some(share_state) = self.share_states.get_mut(share) {
            share_state.backup_progression = *progression;
        }
        self.global_progression = self.calculate_global_progression();
    }

    /// Processes the result of the backup operation.
    ///
    /// # Arguments
    /// * `share` - The name of the share being backed up.
    /// * `result` - The result of the backup operation.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the backup was successful.
    /// * `Err(eyre::Report)` if an error occurred during the backup.
    ///
    /// # Errors
    /// This function returns an error if the backup fails.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn process_backup_result(&mut self, share: &str, result: Result<()>) -> Result<()> {
        match result {
            Ok(()) => {
                info!("Backup for {share} successful");
                if let Some(share_state) = self.share_states.get_mut(share) {
                    share_state.execution_state = ShareExecutionState::Success;
                }

                Ok(())
            }
            Err(err) => {
                error!("Error during backup: {}", err);
                self.error_state = Some(ErrorState::BackupError(err.to_string()));
                if let Some(share_state) = self.share_states.get_mut(share) {
                    share_state.execution_state = ShareExecutionState::Failed(err.to_string());
                }

                Err(err)
            }
        }
    }

    /// Starts the compact operation for a given share.
    ///
    /// # Arguments
    /// * `share` - The name of the share to compact.
    pub fn start_compact(&mut self, share: &str) {
        self.execution_state = BackupExecutionState::Compact(share.to_string());
    }

    /// Processes the result of the compact operation.
    ///
    /// # Arguments
    /// * `share` - The name of the share being compacted.
    /// * `result` - The result of the compact operation.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the compact operation was successful.
    /// * `Err(eyre::Report)` if an error occurred during the compact operation.
    ///
    /// # Errors
    /// This function returns an error if the compact operation fails.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn process_compact_result(&mut self, share: &str, result: Result<()>) -> Result<()> {
        match result {
            Ok(()) => {
                info!("Compact operation for share '{share}' successful");
                Ok(())
            }
            Err(err) => {
                error!("Error during compact operation for share '{share}': {err}");
                self.error_state = Some(ErrorState::CompactError(err.to_string()));
                Err(err)
            }
        }
    }

    /// Starts the count references operation.
    pub fn start_count_references(&mut self) {
        self.execution_state = BackupExecutionState::CountReferences;
    }

    /// Processes the result of the count references operation.
    ///
    /// # Arguments
    /// * `result` - The result of the count references operation.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the count references operation was successful.
    /// * `Err(eyre::Report)` if an error occurred during the operation.
    ///
    /// # Errors
    /// This function returns an error if the count references operation fails.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn process_count_references_result(&mut self, result: Result<()>) -> Result<()> {
        match result {
            Ok(()) => {
                info!("Count references operation successful");
                Ok(())
            }
            Err(err) => {
                error!("Error during count references operation: {}", err);
                self.error_state = Some(ErrorState::CountReferencesError(err.to_string()));
                Err(err)
            }
        }
    }

    /// Starts the add references to pool operation.
    pub fn start_add_references_to_pool(&mut self) {
        self.execution_state = BackupExecutionState::AddReferencesToPool;
    }

    /// Processes the result of the add references to pool operation.
    ///     
    /// # Arguments
    /// * `result` - The result of the add references to pool operation.
    ///
    /// # Returns
    /// * `Ok(())` if the add references to pool operation was successful.
    /// * `Err(eyre::Report)` if an error occurred during the operation.
    ///
    /// # Errors
    /// This function returns an error if the add references to pool operation fails.
    pub fn process_add_references_to_pool_result(&mut self, result: Result<()>) -> Result<()> {
        match result {
            Ok(()) => {
                info!("Add references to pool operation successful");
                self.execution_state = BackupExecutionState::Completed;
                Ok(())
            }
            Err(err) => {
                error!("Error during add references to pool operation: {}", err);
                self.error_state = Some(ErrorState::AddReferencesToPoolError(err.to_string()));
                Err(err)
            }
        }
    }

    /// Records snapshot method and optional failure for a share.
    pub fn set_share_snapshot_result(
        &mut self,
        share: &str,
        method: ShareSnapshotMethod,
        failure_reason: Option<String>,
    ) {
        if let Some(ref reason) = failure_reason {
            tracing::warn!(share = share, reason = %reason, "Snapshot failed for share");
        }
        if let Some(share_state) = self.share_states.get_mut(share) {
            share_state.snapshot_method = method;
            share_state.snapshot_failure_reason = failure_reason;
        }
    }
}
