use napi::{
  bindgen_prelude::{BigInt, Result},
  threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
  JsFunction,
};
use std::path::PathBuf;
use tokio::{spawn, sync::mpsc};
use woodstock::{
  config::{Configuration, GlobalConfiguration, DEFAULT_CHANNEL_BUFFER_SIZE},
  server::pool::{
    pool_cleaner_machine::PoolCleanerMachine,
    pool_cleaner_state::{
      CleanerExecutionState as InnerCleanerExecutionState,
      CleanerProgression as InnerCleanerProgression, CleanerState,
      ErrorState as InnerCleanerErrorState,
    },
  },
  EventPoolCleanedInformation, EventSource,
};

use crate::{events::JsEventSource, server::AbortHandle};

// --- JS-friendly Enums ---
#[napi(string_enum)]
pub enum JsCleanerExecutionState {
  Waiting,
  Initialization,
  ApplyingRefcnt,
  Cleaning,
  Completed,
}

impl From<InnerCleanerExecutionState> for JsCleanerExecutionState {
  fn from(state: InnerCleanerExecutionState) -> Self {
    match state {
      InnerCleanerExecutionState::Waiting => Self::Waiting,
      InnerCleanerExecutionState::Initialization => Self::Initialization,
      InnerCleanerExecutionState::ApplyingRefcnt => Self::ApplyingRefcnt,
      InnerCleanerExecutionState::Cleaning => Self::Cleaning,
      InnerCleanerExecutionState::Completed => Self::Completed,
    }
  }
}

#[napi(string_enum)]
pub enum JsCleanerErrorState {
  InitializationError,
  ApplyingRefcntError,
  CleaningError,
  Unknown,
}

impl From<&InnerCleanerErrorState> for JsCleanerErrorState {
  fn from(state: &InnerCleanerErrorState) -> Self {
    match state {
      InnerCleanerErrorState::InitializationError(_) => Self::InitializationError,
      InnerCleanerErrorState::ApplyingRefcntError(_) => Self::ApplyingRefcntError,
      InnerCleanerErrorState::CleaningError(_) => Self::CleaningError,
      InnerCleanerErrorState::Unknown(_) => Self::Unknown,
    }
  }
}

#[napi(object)]
pub struct JsCleanerProgression {
  pub progress_max: u32,
  pub progress_current: u32,
  pub file_size: BigInt,
  pub compressed_file_size: BigInt,
}

impl From<&InnerCleanerProgression> for JsCleanerProgression {
  fn from(prog: &InnerCleanerProgression) -> Self {
    Self {
      progress_max: prog.progress_max as u32,
      progress_current: prog.progress_current as u32,
      file_size: BigInt::from(prog.file_size),
      compressed_file_size: BigInt::from(prog.compressed_file_size),
    }
  }
}

#[napi(object)]
#[derive(Clone)]
pub struct JsEventPoolCleanedInformation {
  pub size: BigInt,
  pub count: BigInt,
}

impl From<EventPoolCleanedInformation> for JsEventPoolCleanedInformation {
  fn from(event: EventPoolCleanedInformation) -> Self {
    Self {
      size: BigInt::from(event.size),
      count: BigInt::from(event.count),
    }
  }
}

#[napi(object)]
pub struct JsCleanerStatusUpdate {
  pub execution_state: JsCleanerExecutionState,
  pub error_state: Option<JsCleanerErrorState>,
  pub error_message: Option<String>,
  pub progression: JsCleanerProgression,
}

impl JsCleanerStatusUpdate {
  /// Creates a new `JsCleanerStatusUpdate` from a `CleanerState`.
  ///
  /// # Arguments
  /// * `state` - The current state of the pool cleaner.
  fn from_state(state: &CleanerState) -> Self {
    let error_message = state.error_state.as_ref().map(|es| match es {
      InnerCleanerErrorState::InitializationError(s) => s.clone(),
      InnerCleanerErrorState::ApplyingRefcntError(s) => s.clone(),
      InnerCleanerErrorState::CleaningError(s) => s.clone(),
      InnerCleanerErrorState::Unknown(s) => s.clone(),
    });
    Self {
      execution_state: state.execution_state.clone().into(),
      error_state: state.error_state.as_ref().map(JsCleanerErrorState::from),
      error_message,
      progression: JsCleanerProgression::from(&state.progression),
    }
  }
}

#[napi(object)]
pub struct PoolCleanerMessage {
  pub progress: Option<JsCleanerStatusUpdate>,
  pub error: Option<String>,
  pub complete: Option<JsEventPoolCleanedInformation>,
}

#[napi(js_name = "CorePoolCleanerService")]
/// Provides pool cleaning services for the Woodstock backup system.
///
/// This struct manages the configuration and state required to perform pool cleaning operations.
///
/// # Fields
/// * `config` - The configuration used for pool cleaning operations.
pub struct JsPoolCleanerService {
  /// The configuration used for pool cleaning operations.
  config: Configuration,
}

#[napi]
impl JsPoolCleanerService {
  #[napi(factory)]
  /// Creates a new `JsPoolCleanerService` instance.
  ///
  /// # Arguments
  /// * `context` - The Woodstock backup context.
  ///
  /// # Errors
  /// Returns an error if the backup number cannot be converted to `usize`.
  pub fn create_service() -> Result<Self> {
    Ok(Self {
      config: GlobalConfiguration.clone(),
    })
  }

  #[napi]
  /// Cleans the backup pool and invokes the callback with progress and completion updates.
  ///
  /// This function starts the pool cleaning process and will call the provided JavaScript callback
  /// with progress and completion updates as the operation proceeds.
  ///
  /// # Arguments
  /// * `target` - The optional target directory to clean.
  /// * `source` - The event source for the cleaning operation.
  /// * `callback` - A JavaScript callback function to receive progress and completion updates.
  ///
  /// # Errors
  /// Returns an error if the pool cleaning process cannot be started or if the callback cannot be invoked.
  pub fn clean_pool(
    &self,
    target: Option<String>,
    source: JsEventSource,
    #[napi(ts_arg_type = "(message: PoolCleanerMessage) => void")] callback: JsFunction,
  ) -> Result<AbortHandle> {
    let tsfn: ThreadsafeFunction<PoolCleanerMessage, ErrorStrategy::Fatal> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

    let config = self.config.clone();
    let target_path: Option<PathBuf> = target.map(PathBuf::from);
    let event_source: EventSource = source.into();

    let (state_tx, mut state_rx) = mpsc::channel::<CleanerState>(DEFAULT_CHANNEL_BUFFER_SIZE);

    let tsfn_clone_listener = tsfn.clone();
    let listener_handle = tokio::spawn(async move {
      while let Some(state) = state_rx.recv().await {
        tsfn_clone_listener.call(
          PoolCleanerMessage {
            progress: Some(JsCleanerStatusUpdate::from_state(&state)),
            error: None,
            complete: None,
          },
          ThreadsafeFunctionCallMode::NonBlocking,
        );
      }
    });

    let tsfn_clone_task = tsfn.clone();
    let handle = spawn(async move {
      let machine = PoolCleanerMachine::new(&config, target_path, event_source, Some(state_tx));
      let machine_result = machine.execute().await;

      drop(machine);

      if let Err(join_err) = listener_handle.await {
        eprintln!("Cleaner status listener task panicked: {:?}", join_err);
        if machine_result.is_ok() {
          tsfn_clone_task.call(
            PoolCleanerMessage {
              progress: None,
              error: Some(format!(
                "Cleaner status listener task failed: {:?}",
                join_err
              )),
              complete: None,
            },
            ThreadsafeFunctionCallMode::Blocking,
          );
          return;
        }
      }

      match machine_result {
        Ok(info) => {
          tsfn_clone_task.call(
            PoolCleanerMessage {
              progress: None,
              error: None,
              complete: Some(info.into()),
            },
            ThreadsafeFunctionCallMode::Blocking,
          );
        }
        Err(e) => {
          tsfn_clone_task.call(
            PoolCleanerMessage {
              progress: None,
              error: Some(e.to_string()),
              complete: None,
            },
            ThreadsafeFunctionCallMode::Blocking,
          );
        }
      }
    });

    Ok(AbortHandle::new(handle))
  }
}
