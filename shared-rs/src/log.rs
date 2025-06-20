use lazy_static::lazy_static;
use log::{info, LevelFilter};
use log::{Level, Metadata, Record};
use napi::bindgen_prelude::{FromNapiValue, Promise};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use woodstock::utils::thread::{get_current_context_id, CURRENT_CONTEXT_ID};

use napi::threadsafe_function::{ErrorStrategy, ThreadsafeFunction};
use napi::{Env, Error, JsFunction, JsUnknown, Result};
use woodstock::config::GlobalConfiguration;

use crate::config::configuration::JsLogLevel;

// Unique context identifier for each task
static NEXT_CONTEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[napi(object)]
#[derive(Clone)]
pub struct JsBackupLog {
  pub level: JsLogLevel,
  pub context: String,
  pub message: String,
}

#[napi(object)]
#[derive(Clone)]
pub struct JsBackupLogMessage {
  pub progress: Option<JsBackupLog>,
  pub error: Option<String>,
  pub complete: bool,
}

lazy_static! {
  /// Initialize the logger with a default level.
  static ref DEFAULT_LOGGER: Mutex<JavascriptLog> = Mutex::new(JavascriptLog::default());
}

pub fn get_default_logger() -> &'static Mutex<JavascriptLog> {
  &DEFAULT_LOGGER
}

// Structure for representing a task context
#[napi]
#[derive(Clone)]
pub struct LogContext {
  id: usize,
}

#[napi]
impl LogContext {
  #[napi(constructor)]
  pub fn new() -> Self {
    let id = NEXT_CONTEXT_ID.fetch_add(1, Ordering::SeqCst);
    Self { id }
  }

  pub fn new_with_id(id: usize) -> Self {
    Self { id }
  }

  #[napi]
  pub fn get_id(&self) -> usize {
    self.id
  }

  #[napi]
  pub fn to_string(&self) -> String {
    format!("LogContext({})", self.id)
  }
}

pub struct JavascriptLog {
  // Map associating a context ID with its threadsafe function
  tsfn_map: HashMap<usize, ThreadsafeFunction<JsBackupLogMessage, ErrorStrategy::Fatal>>,
  default_tsfn: Option<ThreadsafeFunction<JsBackupLogMessage, ErrorStrategy::Fatal>>,
}

impl Default for JavascriptLog {
  fn default() -> Self {
    Self {
      tsfn_map: HashMap::new(),
      default_tsfn: None,
    }
  }
}

impl JavascriptLog {
  // Add a new threadsafe function to the logger for a specific context
  pub fn add_tsfn(
    &mut self,
    context_id: usize,
    tsfn: ThreadsafeFunction<JsBackupLogMessage, ErrorStrategy::Fatal>,
  ) {
    self.tsfn_map.entry(context_id).or_insert_with(|| tsfn);
  }

  // Remove a threadsafe function for a specific context
  pub fn remove_tsfn(&mut self, context_id: usize) {
    self.tsfn_map.remove(&context_id);
  }

  // Get the last added threadsafe function for a context
  fn get_tsfn_for_context(
    &self,
    context_id: usize,
  ) -> Option<&ThreadsafeFunction<JsBackupLogMessage, ErrorStrategy::Fatal>> {
    self.tsfn_map.get(&context_id)
  }

  fn set_default_tsfn(
    &mut self,
    tsfn: ThreadsafeFunction<JsBackupLogMessage, ErrorStrategy::Fatal>,
  ) {
    self.default_tsfn = Some(tsfn);
  }

  fn get_default_tsfn(
    &self,
  ) -> Option<&ThreadsafeFunction<JsBackupLogMessage, ErrorStrategy::Fatal>> {
    self.default_tsfn.as_ref()
  }
}

impl log::Log for JavascriptLog {
  fn enabled(&self, metadata: &Metadata) -> bool {
    metadata.level() <= Level::Debug
  }

  fn log(&self, record: &Record) {
    if self.enabled(record.metadata()) {
      // Try to get the current context ID
      let context_id = get_current_context_id();

      let message = JsBackupLogMessage {
        progress: Some(JsBackupLog {
          level: record.level().into(),
          context: record.target().to_string(),
          message: record.args().to_string(),
        }),
        error: None,
        complete: false,
      };

      // If we have a specific context, send only to that context
      if context_id > 0 {
        if let Some(tsfn) = self.get_tsfn_for_context(context_id) {
          tsfn.call(
            message,
            napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
          );
        }
      } else if let Some(tsfn) = self.get_default_tsfn() {
        tsfn.call(
          message.clone(), // Clone required since we send to multiple handlers
          napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
        );
      }
    }
  }

  fn flush(&self) {}
}

pub struct DropLogger {
  cancel_drop: bool,
  context_id: usize,
}

impl DropLogger {
  pub fn new(context_id: usize) -> Self {
    Self {
      cancel_drop: false,
      context_id,
    }
  }
}

impl Drop for DropLogger {
  fn drop(&mut self) {
    if !self.cancel_drop {
      DEFAULT_LOGGER.lock().unwrap().remove_tsfn(self.context_id);
    }
  }
}

// Proxy struct to implement log::Log for the mutex-protected logger.
struct LoggerProxy;

impl log::Log for LoggerProxy {
  fn enabled(&self, metadata: &log::Metadata) -> bool {
    DEFAULT_LOGGER.lock().unwrap().enabled(metadata)
  }

  fn log(&self, record: &log::Record) {
    DEFAULT_LOGGER.lock().unwrap().log(record)
  }

  fn flush(&self) {
    DEFAULT_LOGGER.lock().unwrap().flush()
  }
}

#[napi]
/// Initialize the logger and forward log messages to JavaScript.
///
/// # Errors
/// Returns an error if the logger cannot be set or if the callback cannot be created.
pub fn init_log(
  #[napi(ts_arg_type = "(result: JsBackupLogMessage) => void")] callback: JsFunction,
) -> Result<()> {
  let log_level: LevelFilter = GlobalConfiguration.log_level.to_level_filter();
  let tsfn = callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

  // Use a mutex lock to get mutable access to DEFAULT_LOGGER.
  DEFAULT_LOGGER.lock().unwrap().set_default_tsfn(tsfn);

  // Use a reference to DEFAULT_LOGGER because Log is implemented for &T, not for the value itself.
  log::set_boxed_logger(Box::new(LoggerProxy))
    .map(|()| log::set_max_level(log_level))
    .map_err(|e| Error::from_reason(e.to_string()))?;

  info!("Logging initialized with {log_level}");

  Ok(())
}

// Modified use_rust_logger function to use context
#[napi(
  ts_generic_types = "T extends object",
  ts_args_type = "context: LogContext, callback: () => T | Promise<T>, cb_shared: (result: JsBackupLogMessage) => void",
  ts_return_type = "T | Promise<T>"
)]
pub fn use_rust_logger<T: Fn() -> Result<JsUnknown>>(
  env: Env,
  context: &LogContext,
  callback: T,
  cb_shared: JsFunction,
) -> Result<JsUnknown> {
  let context_id = context.id;

  {
    let log_tsfn: ThreadsafeFunction<JsBackupLogMessage, ErrorStrategy::Fatal> =
      cb_shared.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

    // Store the context ID with the function
    let mut logger = get_default_logger().lock().unwrap();
    logger.add_tsfn(context_id, log_tsfn);
  }

  let logger_drop = DropLogger::new(context_id);

  // Execute the callback in the appropriate context
  let value = CURRENT_CONTEXT_ID.sync_scope(context_id, || callback())?;

  if value.is_promise()? {
    let promise = Promise::<serde_json::Value>::from_unknown(value)?;

    let mut logger_drop = logger_drop;
    logger_drop.cancel_drop = true;

    let context_id_clone = context_id;

    env
      .execute_tokio_future(
        async move {
          let _logger_drop = DropLogger::new(context_id_clone);

          // Execute the promise in the appropriate context
          let result = CURRENT_CONTEXT_ID
            .scope(context_id_clone, async move {
              let s = promise.await;
              if let Err(e) = s {
                log::error!("Error in promise: {:?}", e);
              }

              Ok(())
            })
            .await;

          result
        },
        |env, _| env.get_undefined(),
      )
      .map(|v| v.into_unknown())
  } else {
    Ok(value)
  }
}
