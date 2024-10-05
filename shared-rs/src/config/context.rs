use std::path::PathBuf;

use log::Level;
use woodstock::config::{Configuration, ConfigurationPath, Context, OptionalConfigurationPath};

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
  pub events_path: Option<String>,
  pub log_level: Option<String>,
  pub cache_size: Option<u32>,
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
      source: woodstock::EventSource::Woodstock,
      username: None,
      config: Configuration {
        path: ConfigurationPath::new(
          PathBuf::from(&context.backup_path),
          OptionalConfigurationPath {
            certificates_path: context.certificates_path.map(|p| PathBuf::from(&p)),
            config_path: context.config_path.map(|p| PathBuf::from(&p)),
            hosts_path: context.hosts_path.map(|p| PathBuf::from(&p)),
            logs_path: context.logs_path.map(|p| PathBuf::from(&p)),
            pool_path: context.pool_path.map(|p| PathBuf::from(&p)),
            jobs_path: context.jobs_path.map(|p| PathBuf::from(&p)),
            events_path: context.events_path.map(|p| PathBuf::from(&p)),
          },
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
        cache_size: context
          .cache_size
          .and_then(|f| usize::try_from(f).ok())
          .unwrap_or(1),
      },
    },
  }
}
