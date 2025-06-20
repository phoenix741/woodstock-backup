use napi::bindgen_prelude::Reference;
use woodstock::config::Context;

use crate::log::LogContext;

#[napi(object)]
pub struct ContextInput {
  pub username: Option<String>,
  pub log_context: Reference<LogContext>,
}

#[napi(js_name = "BackupContext")]
#[derive(Clone)]
pub struct JsBackupContext {
  /// The backup context from the core Woodstock library.
  context: Context,
  log_context: LogContext,
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

impl From<&JsBackupContext> for LogContext {
  fn from(context: &JsBackupContext) -> Self {
    context.log_context.clone()
  }
}

#[napi]
#[must_use]
pub fn generate_context(context: ContextInput) -> JsBackupContext {
  JsBackupContext {
    context: Context {
      source: woodstock::EventSource::Woodstock,
      username: context.username,
    },
    log_context: LogContext::new_with_id(context.log_context.get_id()),
  }
}
