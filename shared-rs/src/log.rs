use log::{info, LevelFilter};
use log::{Level, Metadata, Record};

use napi::threadsafe_function::{ErrorStrategy, ThreadsafeFunction};
use napi::{Error, JsFunction, Result};
use woodstock::config::Context;

use crate::config::context::{JsBackupContext, LogLevel};

#[napi(object)]
pub struct JsBackupLog {
  pub level: LogLevel,
  pub context: String,
  pub message: String,
}

#[napi(object)]
pub struct JsBackupLogMessage {
  pub progress: Option<JsBackupLog>,
  pub error: Option<String>,
  pub complete: bool,
}

struct JavascriptLog {
  tsfn: ThreadsafeFunction<JsBackupLogMessage, ErrorStrategy::Fatal>,
}

impl JavascriptLog {
  pub fn new(tsfn: ThreadsafeFunction<JsBackupLogMessage, ErrorStrategy::Fatal>) -> Self {
    Self { tsfn }
  }
}

impl log::Log for JavascriptLog {
  fn enabled(&self, metadata: &Metadata) -> bool {
    metadata.level() <= Level::Debug
  }

  fn log(&self, record: &Record) {
    if self.enabled(record.metadata()) {
      self.tsfn.call(
        JsBackupLogMessage {
          progress: Some(JsBackupLog {
            level: record.level().into(),
            context: record.target().to_string(),
            message: record.args().to_string(),
          }),
          error: None,
          complete: false,
        },
        napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
      );
    }
  }

  fn flush(&self) {}
}

#[napi]
pub fn init_log(
  context: &JsBackupContext,
  #[napi(ts_arg_type = "(result: JsBackupLogMessage) => void")] callback: JsFunction,
) -> Result<()> {
  let context: Context = context.into();
  let log_level: LevelFilter = context.config.log_level.to_level_filter();
  let tsfn = callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

  log::set_boxed_logger(Box::new(JavascriptLog::new(tsfn)))
    .map(|()| log::set_max_level(log_level))
    .map_err(|e| Error::from_reason(e.to_string()))?;

  info!("Logging initialized with {log_level}");

  Ok(())
}
