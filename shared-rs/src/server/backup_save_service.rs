use log::error;
use napi::{
  bindgen_prelude::BigInt,
  threadsafe_function::{ErrorStrategy, ThreadsafeFunction},
  Error, JsFunction, Result,
};
use woodstock::{
  config::{Context as WoodstockContext, GlobalConfiguration, DEFAULT_CHANNEL_BUFFER_SIZE},
  server::{
    backup::{
      save_machine::SaveBackupMachine,
      save_state::{
        BackupExecutionState, BackupMachineCommandResult as WoodstockBackupMachineCommandResult,
        BackupState, ErrorState, ExecuteCommandExecutionState, ExecuteCommandState,
        ShareExecutionState, ShareState,
      },
    },
    client::grpc::BackupGrpcClient,
    progression::FileListProgression,
  },
  utils::thread::spawn_with_context_id,
};

use crate::{config::context::JsBackupContext, log::LogContext, models::JsExecuteCommandOperation};

use super::{AbortHandle, JsBackupProgression};

// --- Enums N-API pour les états ---

#[napi(string_enum)]
pub enum JsBackupExecutionState {
  Waiting,
  Authenticate,
  Initialization,
  PreCommands,
  DownloadFileList,
  DownloadChunks,
  PostCommands,
  Compact,
  CountReferences,
  AddReferencesToPool,
  Completed,
}

impl From<BackupExecutionState> for JsBackupExecutionState {
  fn from(state: BackupExecutionState) -> Self {
    match state {
      BackupExecutionState::Waiting => Self::Waiting,
      BackupExecutionState::Authenticate => Self::Authenticate,
      BackupExecutionState::Initialization => Self::Initialization,
      BackupExecutionState::PreCommands(_) => Self::PreCommands,
      BackupExecutionState::DownloadFileList(_) => Self::DownloadFileList,
      BackupExecutionState::DownloadChunks(_) => Self::DownloadChunks,
      BackupExecutionState::PostCommands(_) => Self::PostCommands,
      BackupExecutionState::Compact(_) => Self::Compact,
      BackupExecutionState::CountReferences => Self::CountReferences,
      BackupExecutionState::AddReferencesToPool => Self::AddReferencesToPool,
      BackupExecutionState::Completed => Self::Completed,
    }
  }
}

#[napi(string_enum)]
pub enum JsShareExecutionState {
  Waiting,
  FileList,
  InProgress,
  Success,
  Failed,
}

impl From<ShareExecutionState> for JsShareExecutionState {
  fn from(state: ShareExecutionState) -> Self {
    match state {
      ShareExecutionState::Waiting => Self::Waiting,
      ShareExecutionState::FileList => Self::FileList,
      ShareExecutionState::InProgress => Self::InProgress,
      ShareExecutionState::Success => Self::Success,
      ShareExecutionState::Failed(_) => Self::Failed,
    }
  }
}

#[napi(string_enum)]
pub enum JsExecuteCommandExecutionState {
  Waiting,
  InProgress,
  Success,
  Failed,
}

impl From<ExecuteCommandExecutionState> for JsExecuteCommandExecutionState {
  fn from(state: ExecuteCommandExecutionState) -> Self {
    match state {
      ExecuteCommandExecutionState::Waiting => JsExecuteCommandExecutionState::Waiting,
      ExecuteCommandExecutionState::InProgress => JsExecuteCommandExecutionState::InProgress,
      ExecuteCommandExecutionState::Success(_) => JsExecuteCommandExecutionState::Success,
      ExecuteCommandExecutionState::Failed(_) => JsExecuteCommandExecutionState::Failed,
    }
  }
}

#[napi(string_enum)]
pub enum JsErrorState {
  AuthenticationError,
  InitializationError,
  CommandExecutionError,
  BackupError,
  CompactError,
  CountReferencesError,
  AddReferencesToPoolError,
  Unknown,
}

impl From<ErrorState> for JsErrorState {
  fn from(state: ErrorState) -> Self {
    match state {
      ErrorState::AuthenticationError(_) => Self::AuthenticationError,
      ErrorState::InitializationError(_) => Self::InitializationError,
      ErrorState::CommandExecutionError(_) => Self::CommandExecutionError,
      ErrorState::BackupError(_) => Self::BackupError,
      ErrorState::CompactError(_) => Self::CompactError,
      ErrorState::CountReferencesError(_) => Self::CountReferencesError,
      ErrorState::AddReferencesToPoolError(_) => Self::AddReferencesToPoolError,
      ErrorState::Unknown(_) => Self::Unknown,
    }
  }
}

// --- N-API Structures for backup machine state ---

#[napi(object)]
pub struct JsBackupMachineCommandResult {
  pub code: i32,
  pub stdout: String,
  pub stderr: String,
}

impl From<WoodstockBackupMachineCommandResult> for JsBackupMachineCommandResult {
  fn from(result: WoodstockBackupMachineCommandResult) -> Self {
    Self {
      code: result.code,
      stdout: result.stdout,
      stderr: result.stderr,
    }
  }
}

#[napi(object)]
pub struct JsExecuteCommandState {
  pub command: JsExecuteCommandOperation,
  pub execution_state: JsExecuteCommandExecutionState,
}

impl From<ExecuteCommandState> for JsExecuteCommandState {
  fn from(state: ExecuteCommandState) -> Self {
    Self {
      command: state.command.into(),
      execution_state: state.execution_state.into(),
    }
  }
}

#[napi(object)]
pub struct JsFileListProgression {
  pub file_size: BigInt,
  pub new_file_size: BigInt,
  pub modified_file_size: BigInt,
  pub new_file_count: u32,
  pub modified_file_count: u32,
  pub removed_file_count: u32,
}

impl From<FileListProgression> for JsFileListProgression {
  fn from(progression: FileListProgression) -> Self {
    Self {
      file_size: BigInt::from(progression.file_size),
      new_file_size: BigInt::from(progression.new_file_size),
      modified_file_size: BigInt::from(progression.modified_file_size),
      new_file_count: progression.new_file_count as u32,
      modified_file_count: progression.modified_file_count as u32,
      removed_file_count: progression.removed_file_count as u32,
    }
  }
}

#[napi(object)]
pub struct JsShareState {
  pub share: String,
  pub file_list_progression: JsFileListProgression,
  pub backup_progression: JsBackupProgression,
  pub execution_state: JsShareExecutionState,
}

impl From<ShareState> for JsShareState {
  fn from(state: ShareState) -> Self {
    Self {
      share: state.share,
      file_list_progression: state.file_list_progression.into(),
      backup_progression: (&state.backup_progression).into(),
      execution_state: state.execution_state.into(),
    }
  }
}

#[napi(object)]
pub struct JsBackupState {
  pub execution_state: JsBackupExecutionState,
  pub error_state: Option<JsErrorState>,
  pub error_message: Option<String>,
  pub progression: JsBackupProgression,
  pub pre_command_states: Vec<JsExecuteCommandState>,
  pub share_states: Vec<JsShareState>,
  pub post_command_states: Vec<JsExecuteCommandState>,
}

impl From<BackupState> for JsBackupState {
  fn from(state: BackupState) -> Self {
    let error_message = match &state.error_state {
      Some(ErrorState::AuthenticationError(e)) => Some(e.clone()),
      Some(ErrorState::InitializationError(e)) => Some(e.clone()),
      Some(ErrorState::CommandExecutionError(e)) => Some(e.clone()),
      Some(ErrorState::BackupError(e)) => Some(e.clone()),
      Some(ErrorState::CompactError(e)) => Some(e.clone()),
      Some(ErrorState::CountReferencesError(e)) => Some(e.clone()),
      Some(ErrorState::AddReferencesToPoolError(e)) => Some(e.clone()),
      _ => None,
    };

    Self {
      execution_state: state.execution_state.into(),
      error_state: state.error_state.map(|e| e.into()),
      error_message,
      progression: (&state.global_progression).into(),
      pre_command_states: state
        .pre_command_states
        .into_values()
        .map(Into::<JsExecuteCommandState>::into)
        .collect(),
      share_states: state
        .share_states
        .into_values()
        .map(Into::<JsShareState>::into)
        .collect(),
      post_command_states: state
        .post_command_states
        .into_values()
        .map(Into::<JsExecuteCommandState>::into)
        .collect(),
    }
  }
}

#[napi(object)]
pub struct JsBackupSaveMessage {
  pub progress: Option<JsBackupState>,
  pub error: Option<String>,
  pub complete: bool,
}

#[napi(js_name = "JsBackupSaveService")]
/// Represents a backup save service for a specific host and backup number.
///
/// This struct manages the context and state required to perform backup save operations
/// for a given host and backup number, including the network address and Woodstock context.
///
/// # Fields
/// * `ip` - The IP address of the machine.
/// * `hostname` - The hostname of the machine to backup.
/// * `backup_number` - The backup number associated with this service.
/// * `context` - The Woodstock context for configuration and state management.
pub struct JsBackupSaveService {
  /// The IP address of the machine.
  ip: String,
  /// The hostname of the machine to backup.
  hostname: String,
  /// The backup number associated with this service.
  backup_number: usize,
  /// The Woodstock context for configuration and state management.
  context: WoodstockContext,
  /// The log context used for restore logging
  log_context: LogContext,
}

#[napi]
impl JsBackupSaveService {
  #[napi(factory)]
  /// Creates a new `JsBackupSaveService` instance.
  ///
  /// # Arguments
  /// * `hostname` - The hostname of the machine to backup.
  /// * `ip` - The IP address of the machine.
  /// * `backup_number` - The backup number associated with this service.
  /// * `context` - The Woodstock backup context.
  ///
  /// # Errors
  /// Returns an error if the backup number cannot be converted to `usize`.
  pub fn create_service(
    hostname: String,
    ip: String,
    backup_number: u32,
    context: &JsBackupContext,
  ) -> Result<Self> {
    let log_context: LogContext = context.into();
    let context: WoodstockContext = context.into();
    let backup_number_usize = usize::try_from(backup_number)
      .map_err(|_| Error::from_reason("Backup number is too large".to_string()))?;

    Ok(Self {
      ip,
      hostname,
      backup_number: backup_number_usize,
      context,
      log_context,
    })
  }

  #[napi(getter)]
  pub fn hostname(&self) -> String {
    self.hostname.clone()
  }

  #[napi(getter)]
  pub fn ip(&self) -> String {
    self.ip.clone()
  }

  #[napi(getter)]
  pub fn backup_number(&self) -> u32 {
    u32::try_from(self.backup_number).unwrap_or(0) // Should not fail if created with u32
  }

  #[napi]
  /// Executes the backup save operation and invokes the callback with progress and completion updates.
  ///
  /// This function starts the backup save process and will call the provided JavaScript callback
  /// with progress and completion updates as the operation proceeds.
  ///
  /// # Arguments
  /// * `callback` - A JavaScript callback function to receive progress and completion updates.
  ///
  /// # Errors
  /// Returns an error if the backup save process cannot be started or if the callback cannot be invoked.
  pub fn execute(
    &self,
    #[napi(ts_arg_type = "(result: JsBackupSaveMessage) => void")] callback: JsFunction,
  ) -> Result<AbortHandle> {
    let tsfn: ThreadsafeFunction<JsBackupSaveMessage, ErrorStrategy::Fatal> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

    let hostname = self.hostname.clone();
    let ip = self.ip.clone();
    let backup_number_usize = self.backup_number;
    let context = self.context.clone();
    let config = &GlobalConfiguration;

    let tsfn_clone = tsfn.clone();
    let (state_tx, mut state_rx) =
      tokio::sync::mpsc::channel::<BackupState>(DEFAULT_CHANNEL_BUFFER_SIZE);

    let progress_listener = tokio::spawn(async move {
      while let Some(state) = state_rx.recv().await {
        let js_state: JsBackupState = state.into();
        tsfn_clone.call(
          JsBackupSaveMessage {
            progress: Some(js_state),
            error: None,
            complete: false,
          },
          napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
        );
      }
    });

    let handle = spawn_with_context_id(self.log_context.get_id(), async move {
      let Ok(grpc_client) = BackupGrpcClient::new(&hostname, &ip, config).await else {
        tsfn.call(
          JsBackupSaveMessage {
            progress: None,
            error: Some("Can't create gRPC client".to_string()),
            complete: false,
          },
          napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
        );
        return;
      };

      let Ok(mut machine) = SaveBackupMachine::new(
        grpc_client,
        &hostname,
        backup_number_usize,
        &context,
        config,
        Some(state_tx),
      )
      .await
      else {
        tsfn.call(
          JsBackupSaveMessage {
            progress: None,
            error: Some("Failed to create SaveBackupMachine".to_string()),
            complete: false,
          },
          napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
        );
        return;
      };

      let result = machine.execute().await;

      drop(machine);

      if let Err(e) = progress_listener.await {
        error!("Backup status listener task panicked: {:?}", e);
        tsfn.call(
          JsBackupSaveMessage {
            progress: None,
            error: Some(format!("Backup status listener task failed: {:?}", e)),
            complete: false,
          },
          napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
        );
      }

      match result {
        Ok(()) => {
          tsfn.call(
            JsBackupSaveMessage {
              progress: None, // Final state already sent by listener if it was the last message
              error: None,
              complete: true,
            },
            napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
          );
        }
        Err(e) => {
          tsfn.call(
            JsBackupSaveMessage {
              progress: None,
              error: Some(format!("Machine execution failed: {}", e)),
              complete: false, // Or some failed status if applicable
            },
            napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
          );
        }
      }
    });

    Ok(AbortHandle::new(handle))
  }
}
