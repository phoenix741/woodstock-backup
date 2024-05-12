use std::path::PathBuf;

use log::Level;
use woodstock::config::{Configuration, ConfigurationPath, Context};

#[napi]
pub enum LogLevel {
  Error = 1,
  Warn,
  Info,
  Debug,
  Trace,
}

impl From<Level> for LogLevel {
  fn from(level: Level) -> Self {
    match level {
      Level::Error => LogLevel::Error,
      Level::Warn => LogLevel::Warn,
      Level::Info => LogLevel::Info,
      Level::Debug => LogLevel::Debug,
      Level::Trace => LogLevel::Trace,
    }
  }
}

#[napi(object)]
pub struct ContextInput {
  pub backup_path: String,
  pub certificates_path: Option<String>,
  pub config_path: Option<String>,
  pub hosts_path: Option<String>,
  pub logs_path: Option<String>,
  pub pool_path: Option<String>,
  pub jobs_path: Option<String>,
  pub log_level: Option<String>,
}

#[napi(js_name = "BackupContext")]
pub struct JsBackupContext {
  context: Context,
}

impl From<JsBackupContext> for Context {
  fn from(context: JsBackupContext) -> Self {
    context.context
  }
}

impl From<&JsBackupContext> for Context {
  fn from(context: &JsBackupContext) -> Self {
    context.context.clone()
  }
}

#[napi]
#[must_use]
pub fn generate_context(context: ContextInput) -> JsBackupContext {
  JsBackupContext {
    context: Context {
      config: Configuration {
        path: ConfigurationPath::new(
          PathBuf::from(&context.backup_path),
          context.certificates_path.map(|p| PathBuf::from(&p)),
          context.config_path.map(|p| PathBuf::from(&p)),
          context.hosts_path.map(|p| PathBuf::from(&p)),
          context.logs_path.map(|p| PathBuf::from(&p)),
          context.pool_path.map(|p| PathBuf::from(&p)),
          context.jobs_path.map(|p| PathBuf::from(&p)),
        ),
        log_level: match context
          .log_level
          .unwrap_or_default()
          .to_lowercase()
          .as_str()
        {
          "error" => Level::Error,
          "warn" => Level::Warn,
          "info" => Level::Info,
          "debug" => Level::Debug,
          "trace" => Level::Trace,
          _ => Level::Info,
        },
      },
    },
  }
}
