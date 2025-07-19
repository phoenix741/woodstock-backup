use napi::{
  threadsafe_function::{ErrorStrategy, ThreadsafeFunction},
  Error, JsFunction, Result,
};
use tokio::spawn;
use woodstock::{
  config::{Context as WoodstockContext, GlobalConfiguration, DEFAULT_CHANNEL_BUFFER_SIZE},
  server::{
    backup::{
      restore_machine::RestoreBackupMachine,
      restore_state::{ErrorState, RestoreExecutionState, RestoreState},
    },
    client::grpc::BackupGrpcClient,
  },
};

use crate::{config::context::JsBackupContext, server::abort_handle::AbortHandle};

use super::JsBackupProgression;

// --- Enums N-API ---
#[napi(string_enum)]
pub enum JsRestoreExecutionState {
  Waiting,
  Authentication,
  Preparation,
  Restoring,
  Completed,
}

impl From<RestoreExecutionState> for JsRestoreExecutionState {
  fn from(state: RestoreExecutionState) -> Self {
    match state {
      RestoreExecutionState::Waiting => Self::Waiting,
      RestoreExecutionState::Authentication => Self::Authentication,
      RestoreExecutionState::Preparation(_) => Self::Preparation,
      RestoreExecutionState::Restoring(_) => Self::Restoring,
      RestoreExecutionState::Completed => Self::Completed,
    }
  }
}

#[napi(string_enum)]
pub enum JsRestoreErrorState {
  AuthenticationError,
  PreparationError,
  RestoreError,
  Unknown,
}

impl From<ErrorState> for JsRestoreErrorState {
  fn from(state: ErrorState) -> Self {
    match state {
      ErrorState::AuthenticationError(_) => Self::AuthenticationError,
      ErrorState::PreparationError(_) => Self::PreparationError,
      ErrorState::RestoreError(_) => Self::RestoreError,
      ErrorState::Unknown(_) => Self::Unknown,
    }
  }
}

#[napi(object)]
pub struct JsRestoreState {
  pub execution_state: JsRestoreExecutionState,
  pub global_progression: JsBackupProgression,
  pub error_state: Option<JsRestoreErrorState>,
  pub error_message: Option<String>,
}

impl From<RestoreState> for JsRestoreState {
  fn from(state: RestoreState) -> Self {
    let error_message = match &state.error_state {
      Some(ErrorState::AuthenticationError(msg)) => Some(msg.clone()),
      Some(ErrorState::PreparationError(msg)) => Some(msg.clone()),
      Some(ErrorState::RestoreError(msg)) => Some(msg.clone()),
      _ => None,
    };

    Self {
      execution_state: state.execution_state.into(),
      global_progression: (&state.global_progression).into(),
      error_state: state.error_state.map(|e| e.into()),
      error_message,
    }
  }
}

#[napi(object)]
pub struct JsRestoreCallbackMessage {
  pub state: Option<JsRestoreState>,
  pub error: Option<String>,
  pub complete: bool,
}

#[napi(object)]
pub struct JsShareSelection {
  /// The share to restore.
  pub share: String,
  /// The list of files to restore.
  pub selection: Vec<String>,
}

#[napi(js_name = "JsBackupRestoreService")]
/// Represents a backup and restore service for a specific host and backup number.
///
/// This struct manages the context and state required to perform backup and restore operations
/// for a given host and backup number, including the network address and Woodstock context.
///
/// # Fields
/// * `hostname` - The hostname of the machine to backup or restore.
/// * `ip` - The IP address of the machine.
/// * `backup_number` - The backup number associated with this service.
/// * `context` - The Woodstock context for configuration and state management.
pub struct JsBackupRestoreService {
  /// The hostname of the machine to backup or restore.
  hostname: String,
  /// The IP address of the machine.
  ip: String,
  /// The backup number associated with this service.
  backup_number: usize,
  /// The Woodstock context for configuration and state management.
  context: WoodstockContext,
}

#[napi]
impl JsBackupRestoreService {
  /// Creates a new `JsBackupRestoreService` instance.
  ///
  /// # Arguments
  /// * `hostname` - The hostname of the machine to backup or restore.
  /// * `ip` - The IP address of the machine.
  /// * `backup_number` - The backup number associated with this service.
  /// * `context` - The Woodstock backup context.
  ///
  /// # Errors
  /// Returns an error if the backup number cannot be converted to `usize`.
  #[napi(factory)]
  pub fn create_service(
    hostname: String,
    ip: String,
    backup_number: u32,
    context: &JsBackupContext,
  ) -> Result<Self> {
    let context: WoodstockContext = context.into();
    let backup_number_usize = usize::try_from(backup_number)
      .map_err(|_| Error::from_reason("Backup number is too large".to_string()))?;

    Ok(Self {
      hostname,
      ip,
      backup_number: backup_number_usize,
      context,
    })
  }

  /// Returns the hostname associated with this service.
  #[napi(getter)]
  pub fn hostname(&self) -> String {
    self.hostname.clone()
  }

  /// Returns the backup number associated with this service as a `u32`.
  ///
  /// # Panics
  /// Panics if the backup number cannot be converted to `u32`.
  #[napi(getter)]
  pub fn backup_number(&self) -> u32 {
    u32::try_from(self.backup_number).unwrap()
  }

  /// Executes the restore operation for the specified share and selection.
  ///
  /// This function starts the restore process for the given share, destination directory, and file selection.
  /// It will invoke the provided callback with progress and completion updates.
  ///
  /// # Arguments
  /// * `share` - The name of the share to restore.
  /// * `destination_directory` - The directory where the restored files will be placed.
  /// * `selection` - The list of files or directories to restore.
  /// * `callback` - A JavaScript callback function to receive progress and completion updates.
  ///
  /// # Errors
  /// Returns an error if the restore process cannot be started, if the gRPC client or restore machine cannot be created, or if the callback cannot be invoked.
  #[napi]
  pub fn execute(
    &self,
    destination_directory: String,
    share_selection: Vec<JsShareSelection>,
    #[napi(ts_arg_type = "(result: JsRestoreCallbackMessage) => void")] callback: JsFunction,
  ) -> Result<AbortHandle> {
    let tsfn: ThreadsafeFunction<JsRestoreCallbackMessage, ErrorStrategy::Fatal> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

    let hostname = self.hostname.clone();
    let backup_number_usize = self.backup_number;
    let woodstock_context = self.context.clone();
    let ip = self.ip.clone();

    let tsfn_clone_for_state_update = tsfn.clone();

    let (tx_state, mut rx_state) =
      tokio::sync::mpsc::channel::<RestoreState>(DEFAULT_CHANNEL_BUFFER_SIZE);

    let state_update_task = tokio::spawn(async move {
      while let Some(state) = rx_state.recv().await {
        let js_state: JsRestoreState = state.into();
        tsfn_clone_for_state_update.call(
          JsRestoreCallbackMessage {
            state: Some(js_state),
            error: None,
            complete: false,
          },
          napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
        );
      }
    });

    let handle = spawn(async move {
      let Ok(grpc_client) = BackupGrpcClient::new(&hostname, &ip, &GlobalConfiguration).await
      else {
        tsfn.call(
          JsRestoreCallbackMessage {
            state: None,
            error: Some("Failed to create gRPC client".to_string()),
            complete: true,
          },
          napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
        );
        return;
      };

      let Ok(mut machine) = RestoreBackupMachine::new(
        grpc_client,
        &hostname,
        backup_number_usize,
        &woodstock_context,
        &GlobalConfiguration,
        Some(tx_state),
      )
      .await
      .map_err(|e| Error::from_reason(format!("Can't create RestoreBackupMachine: {}", e))) else {
        tsfn.call(
          JsRestoreCallbackMessage {
            state: None,
            error: Some("Failed to create RestoreBackupMachine".to_string()),
            complete: true,
          },
          napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
        );
        return;
      };

      // Convert JsShareSelection to ShareSelection<&str, &str>
      let share_selection_native = share_selection
        .iter()
        .map(
          |sel| woodstock::server::backup::restore_machine::ShareSelection {
            share: sel.share.as_str(),
            selection: sel.selection.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
          },
        )
        .collect::<Vec<_>>();

      let result = machine
        .execute(&destination_directory, &share_selection_native)
        .await;

      drop(machine);

      let _ = state_update_task.await;

      match result {
        Ok(()) => {
          tsfn.call(
            JsRestoreCallbackMessage {
              state: None,
              error: None,
              complete: true,
            },
            napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
          );
        }
        Err(e) => {
          tsfn.call(
            JsRestoreCallbackMessage {
              state: None,
              error: Some(e.to_string()),
              complete: true,
            },
            napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
          );
        }
      }
    });

    Ok(AbortHandle::new(handle))
  }
}
