use log::error;
use napi::{
  threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
  Error, JsFunction, Result, Status,
};
use woodstock::{
  config::{Context as WoodstockContext, GlobalConfiguration, DEFAULT_CHANNEL_BUFFER_SIZE},
  server::backup::{
    remove_machine::RemoveBackupMachine,
    remove_state::{ErrorState as WoodstockErrorState, RemoveExecutionState, RemoveState},
  },
};

use crate::{
  config::context::JsBackupContext,
  log::{LogBackupContext, LOG_CONTEXT},
  server::abort_handle::AbortHandle,
};

#[napi(string_enum)]
pub enum JsRemoveExecutionState {
  Waiting,
  AddReferencesToPool,
  RemovingRefCnt,
  RemovingBackup,
  Completed,
}

impl From<RemoveExecutionState> for JsRemoveExecutionState {
  fn from(state: RemoveExecutionState) -> Self {
    match state {
      RemoveExecutionState::Waiting => JsRemoveExecutionState::Waiting,
      RemoveExecutionState::RemovingRefcnt => JsRemoveExecutionState::RemovingRefCnt,
      RemoveExecutionState::RemovingBackup => JsRemoveExecutionState::RemovingBackup,
      RemoveExecutionState::AddReferencesToPool => JsRemoveExecutionState::AddReferencesToPool,
      RemoveExecutionState::Completed => JsRemoveExecutionState::Completed,
    }
  }
}

#[napi(string_enum)]
pub enum JsRemoveErrorState {
  AddReferencesToPoolError,
  RefcntRemovalError,
  BackupRemovalError,
  Unknown,
}

impl From<WoodstockErrorState> for JsRemoveErrorState {
  fn from(state: WoodstockErrorState) -> Self {
    match state {
      WoodstockErrorState::RefcntRemovalError(_) => JsRemoveErrorState::RefcntRemovalError,
      WoodstockErrorState::BackupRemovalError(_) => JsRemoveErrorState::BackupRemovalError,
      WoodstockErrorState::AddReferencesToPoolError(_) => {
        JsRemoveErrorState::AddReferencesToPoolError
      }
      WoodstockErrorState::Unknown(_) => JsRemoveErrorState::Unknown,
    }
  }
}

#[napi(object)]
pub struct JsRemoveState {
  pub execution_state: JsRemoveExecutionState,
  pub error_state: Option<JsRemoveErrorState>,
  pub error_message: Option<String>,
}

impl From<RemoveState> for JsRemoveState {
  fn from(state: RemoveState) -> Self {
    let error_message = match &state.error_state {
      Some(WoodstockErrorState::RefcntRemovalError(e)) => Some(e.clone()),
      Some(WoodstockErrorState::BackupRemovalError(e)) => Some(e.clone()),
      Some(WoodstockErrorState::AddReferencesToPoolError(e)) => Some(e.clone()),
      _ => None,
    };

    Self {
      execution_state: state.execution_state.into(),
      error_state: state.error_state.as_ref().map(|e| e.clone().into()),
      error_message,
    }
  }
}

#[napi(object)]
/// Callback message for remove operations, sent to JavaScript.
pub struct JsRemoveCallbackMessage {
  /// Current remove state, if available.
  pub state: Option<JsRemoveState>,
  /// Error message, if any.
  pub error: Option<String>,
  /// Whether the operation is complete.
  pub complete: bool,
}

#[napi]
/// Service for removing a backup, exposed to JavaScript.
pub struct JsBackupRemoveService {
  /// Hostname of the backup to remove.
  hostname: String,
  /// Backup number to remove.
  backup_number: usize,
  /// Woodstock context for the backup operation.
  woodstock_context: WoodstockContext,
}

#[napi]
impl JsBackupRemoveService {
  /// Create a new backup remove service instance.
  ///
  /// # Errors
  /// Returns an error if the backup number cannot be converted or the context is invalid.
  #[napi(factory)]
  pub fn create_service(
    hostname: String,
    backup_number: u32,
    context: &JsBackupContext,
  ) -> Result<Self> {
    let backup_number_usize = usize::try_from(backup_number)
      .map_err(|_| Error::from_reason("Backup number is too large".to_string()))?;
    let woodstock_context = WoodstockContext::from(context);

    Ok(Self {
      hostname,
      backup_number: backup_number_usize,
      woodstock_context,
    })
  }

  #[napi]
  /// Execute the backup removal asynchronously and send progress to the callback.
  ///
  /// # Errors
  /// Returns an error if the callback cannot be created or the task cannot be spawned.
  pub fn execute(
    &self,
    #[napi(ts_arg_type = "(result: JsRemoveCallbackMessage) => void")] callback: JsFunction,
  ) -> Result<AbortHandle> {
    let tsfn: ThreadsafeFunction<JsRemoveCallbackMessage, ErrorStrategy::Fatal> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

    let log_hostname = self.hostname.clone();
    let log_backup_number = self.backup_number as u32;

    let hostname_clone = self.hostname.clone();
    let backup_number_clone = self.backup_number;
    let woodstock_context_clone = self.woodstock_context.clone();
    let config_for_machine = &GlobalConfiguration;

    let handle = tokio::spawn(async move {
      LOG_CONTEXT
        .scope(
          LogBackupContext {
            hostname: log_hostname,
            backup_number: log_backup_number,
          },
          async {
            let (tx_state, mut rx_state) =
              tokio::sync::mpsc::channel::<RemoveState>(DEFAULT_CHANNEL_BUFFER_SIZE);

            let mut machine = RemoveBackupMachine::new(
              &hostname_clone,
              backup_number_clone,
              &woodstock_context_clone,
              config_for_machine,
              Some(tx_state),
            );

            let tsfn_clone_for_state_update = tsfn.clone();
            let state_update_task = tokio::spawn(async move {
              while let Some(state) = rx_state.recv().await {
                let js_state: JsRemoveState = state.into();
                let call_result = tsfn_clone_for_state_update.call(
                  JsRemoveCallbackMessage {
                    state: Some(js_state),
                    error: None,
                    complete: false,
                  },
                  ThreadsafeFunctionCallMode::NonBlocking,
                );
                if call_result != Status::Ok {
                  error!("Failed to send state update to JS: {:?}", call_result);
                }
              }
            });

            let result = machine.execute().await;

            drop(machine);
            let _ = state_update_task.await;

            match result {
              Ok(()) => {
                let call_result = tsfn.call(
                  JsRemoveCallbackMessage {
                    state: None,
                    error: None,
                    complete: true,
                  },
                  ThreadsafeFunctionCallMode::Blocking,
                );
                if call_result != Status::Ok {
                  error!("Failed to call completion callback: {:?}", call_result);
                }
              }
              Err(e) => {
                let call_result = tsfn.call(
                  JsRemoveCallbackMessage {
                    state: None,
                    error: Some(e.to_string()),
                    complete: true,
                  },
                  ThreadsafeFunctionCallMode::Blocking,
                );
                if call_result != Status::Ok {
                  error!("Failed to call error callback: {:?}", call_result);
                }
              }
            }
          },
        )
        .await;
    });

    Ok(AbortHandle::new(handle))
  }
}
