use napi::{
  bindgen_prelude::Result,
  threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
  JsFunction,
};
use tokio::sync::mpsc;
use woodstock::{
  config::{Configuration, GlobalConfiguration, DEFAULT_CHANNEL_BUFFER_SIZE},
  server::pool::{
    fsck_machine::FsckMachine,
    fsck_state::{
      ChunkProgression as InnerChunkProgression, ErrorState as InnerFsckErrorState,
      FsckExecutionState, FsckState, RefcntProgression as InnerRefcntProgression,
      UnusedProgression as InnerUnusedProgression,
    },
  },
  EventSource,
};

use crate::{events::JsEventSource, server::AbortHandle};

// --- JS-friendly Enums ---
#[napi(string_enum)]
pub enum JsFsckExecutionState {
  Waiting,
  Initialization,
  ApplyingRefcnt,
  VerifyRefcnt,
  VerifyUnused,
  VerifyChunk,
  Completed,
}

impl From<FsckExecutionState> for JsFsckExecutionState {
  fn from(state: FsckExecutionState) -> Self {
    match state {
      FsckExecutionState::Waiting => Self::Waiting,
      FsckExecutionState::Initialization => Self::Initialization,
      FsckExecutionState::ApplyingRefcnt => Self::ApplyingRefcnt,
      FsckExecutionState::VerifyRefcnt => Self::VerifyRefcnt,
      FsckExecutionState::VerifyUnused => Self::VerifyUnused,
      FsckExecutionState::VerifyChunk => Self::VerifyChunk,
      FsckExecutionState::Completed => Self::Completed,
    }
  }
}

#[napi(string_enum)]
pub enum JsFsckErrorState {
  InitializationError,
  ApplyingRefcntError,
  VerifyRefcntError,
  VerifyUnusedError,
  VerifyChunkError,
  Unknown,
}

impl From<&InnerFsckErrorState> for JsFsckErrorState {
  fn from(state: &InnerFsckErrorState) -> Self {
    match state {
      InnerFsckErrorState::InitializationError(_) => Self::InitializationError,
      InnerFsckErrorState::ApplyingRefcntError(_) => Self::ApplyingRefcntError,
      InnerFsckErrorState::VerifyRefcntError(_) => Self::VerifyRefcntError,
      InnerFsckErrorState::VerifyUnusedError(_) => Self::VerifyUnusedError,
      InnerFsckErrorState::VerifyChunkError(_) => Self::VerifyChunkError,
      InnerFsckErrorState::Unknown(_) => Self::Unknown,
    }
  }
}

#[napi(object)]
#[derive(Clone)]
pub struct JsRefcntProgression {
  pub progress_max: u32,
  pub progress_current: u32,
  pub error_count: u32,
  pub total_count: u32,
}

impl From<&InnerRefcntProgression> for JsRefcntProgression {
  fn from(prog: &InnerRefcntProgression) -> Self {
    Self {
      progress_max: prog.progress_max as u32,
      progress_current: prog.progress_current as u32,
      error_count: prog.error_count as u32,
      total_count: prog.total_count as u32,
    }
  }
}

#[napi(object)]
#[derive(Clone)]
pub struct JsUnusedProgression {
  pub progress_max: u32,
  pub progress_current: u32,
  pub in_nothing: u32,
  pub in_refcnt: u32,
  pub in_unused: u32,
  pub missing: u32,
}

impl From<&InnerUnusedProgression> for JsUnusedProgression {
  fn from(prog: &InnerUnusedProgression) -> Self {
    Self {
      progress_max: prog.progress_max as u32,
      progress_current: prog.progress_current as u32,
      in_nothing: prog.in_nothing as u32,
      in_refcnt: prog.in_refcnt as u32,
      in_unused: prog.in_unused as u32,
      missing: prog.missing as u32,
    }
  }
}

#[napi(object)]
#[derive(Clone)]
pub struct JsChunkProgression {
  pub progress_max: u32,
  pub progress_current: u32,
  pub error_count: u32,
  pub total_count: u32,
}

impl From<&InnerChunkProgression> for JsChunkProgression {
  fn from(prog: &InnerChunkProgression) -> Self {
    Self {
      progress_max: prog.progress_max as u32,
      progress_current: prog.progress_current as u32,
      error_count: prog.error_count as u32,
      total_count: prog.total_count as u32,
    }
  }
}

#[napi(object)]
#[derive(Clone)]
pub struct JsFsckStatusUpdate {
  pub execution_state: JsFsckExecutionState,
  pub error_state: Option<JsFsckErrorState>,
  pub error_message: Option<String>,
  pub refcnt_progression: JsRefcntProgression,
  pub unused_progression: JsUnusedProgression,
  pub chunk_progression: JsChunkProgression,
  pub dry_run: bool,
}

impl JsFsckStatusUpdate {
  /// Creates a new `JsFsckStatusUpdate` from a `FsckState`.
  ///
  /// # Arguments
  /// * `state` - The current state of the pool fsck operation.
  fn from_state(state: &FsckState) -> Self {
    let error_message = state.error_state.as_ref().map(|es| match es {
      InnerFsckErrorState::InitializationError(s) => s.clone(),
      InnerFsckErrorState::ApplyingRefcntError(s) => s.clone(),
      InnerFsckErrorState::VerifyRefcntError(s) => s.clone(),
      InnerFsckErrorState::VerifyUnusedError(s) => s.clone(),
      InnerFsckErrorState::VerifyChunkError(s) => s.clone(),
      InnerFsckErrorState::Unknown(s) => s.clone(),
    });
    Self {
      execution_state: state.execution_state.clone().into(),
      error_state: state.error_state.as_ref().map(JsFsckErrorState::from),
      error_message,
      refcnt_progression: JsRefcntProgression::from(&state.refcnt_progression),
      unused_progression: JsUnusedProgression::from(&state.unused_progression),
      chunk_progression: JsChunkProgression::from(&state.chunk_progression),
      dry_run: state.dry_run,
    }
  }
}

#[napi(object)]
pub struct PoolFsckMessage {
  pub progress: Option<JsFsckStatusUpdate>,
  pub error: Option<String>,
  pub complete: Option<JsFsckStatusUpdate>,
}

#[napi(js_name = "CorePoolFsckService")]
/// Provides pool fsck (filesystem check) services for the Woodstock backup system.
///
/// This struct manages the configuration and state required to perform pool fsck operations.
///
/// # Fields
/// * `config` - The configuration used for pool fsck operations.
pub struct JsPoolFsckService {
  /// The configuration used for pool fsck operations.
  config: Configuration,
}

impl Default for JsPoolFsckService {
  fn default() -> Self {
    Self::new()
  }
}

#[napi]
impl JsPoolFsckService {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      config: GlobalConfiguration.clone(),
    }
  }

  #[napi]
  /// Executes the pool fsck (filesystem check) operation and invokes the callback with progress and completion updates.
  ///
  /// This function starts the pool fsck process and will call the provided JavaScript callback
  /// with progress and completion updates as the operation proceeds.
  ///
  /// # Arguments
  /// * `dry_run` - If true, performs a dry run without making changes.
  /// * `verify_chunks` - If true, verifies chunk integrity during the check.
  /// * `source` - The event source for the fsck operation.
  /// * `callback` - A JavaScript callback function to receive progress and completion updates.
  ///
  /// # Errors
  /// Returns an error if the pool fsck process cannot be started or if the callback cannot be invoked.
  pub fn execute_fsck(
    &self,
    dry_run: bool,
    verify_chunks: bool,
    source: JsEventSource,
    #[napi(ts_arg_type = "(message: PoolFsckMessage) => void")] callback: JsFunction,
  ) -> Result<AbortHandle> {
    let tsfn: ThreadsafeFunction<PoolFsckMessage, ErrorStrategy::Fatal> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

    let config = self.config.clone();
    let event_source: EventSource = source.into();

    let (state_tx, mut state_rx) = mpsc::channel::<FsckState>(DEFAULT_CHANNEL_BUFFER_SIZE);

    // Task to listen for state changes and send them as 'progress'
    let tsfn_clone_listener = tsfn.clone();
    let listener_handle = tokio::spawn(async move {
      while let Some(state) = state_rx.recv().await {
        tsfn_clone_listener.call(
          PoolFsckMessage {
            progress: Some(JsFsckStatusUpdate::from_state(&state)),
            error: None,
            complete: None,
          },
          ThreadsafeFunctionCallMode::NonBlocking,
        );
      }
    });

    // Main task to run the FsckMachine
    let tsfn_clone_task = tsfn.clone();
    let handle = tokio::spawn(async move {
      let machine = FsckMachine::new(
        &config,
        event_source,
        dry_run,
        verify_chunks,
        false,
        Some(state_tx),
      );
      let machine_result = machine.execute().await;
      let final_state = machine.get_state().await;

      drop(machine);

      if let Err(join_err) = listener_handle.await {
        eprintln!("Fsck status listener task panicked: {:?}", join_err);
        if machine_result.is_ok() {
          tsfn_clone_task.call(
            PoolFsckMessage {
              progress: None,
              error: Some(format!("Fsck status listener task failed: {:?}", join_err)),
              complete: None,
            },
            ThreadsafeFunctionCallMode::Blocking,
          );
          return;
        }
      }

      match machine_result {
        Ok(()) => {
          tsfn_clone_task.call(
            PoolFsckMessage {
              progress: None,
              error: None,
              complete: Some(JsFsckStatusUpdate::from_state(&final_state)),
            },
            ThreadsafeFunctionCallMode::Blocking,
          );
        }
        Err(e) => {
          tsfn_clone_task.call(
            PoolFsckMessage {
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
