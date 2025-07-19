use lazy_static::lazy_static;
use log::{info, LevelFilter};
use log::{Level, Metadata, Record};
use std::sync::Mutex;

use napi::threadsafe_function::{ErrorStrategy, ThreadsafeFunction};
use napi::{Error, JsFunction, Result};
use woodstock::config::GlobalConfiguration;

use crate::config::configuration::JsLogLevel;

// Unique context identifier for each task
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

pub struct JavascriptLog {
  // Map associating a context ID with its threadsafe function
  default_tsfn: Option<ThreadsafeFunction<JsBackupLogMessage, ErrorStrategy::Fatal>>,
}

impl Default for JavascriptLog {
  fn default() -> Self {
    Self { default_tsfn: None }
  }
}

impl JavascriptLog {
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
      if let Some(tsfn) = self.get_default_tsfn() {
        tsfn.call(
          message.clone(), // Clone required since we send to multiple handlers
          napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
        );
      }
    }
  }

  fn flush(&self) {}
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
