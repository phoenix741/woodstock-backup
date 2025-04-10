use woodstock::config::{Configuration, Context};

#[napi(object)]
pub struct ContextInput {
  pub username: Option<String>,
}

#[napi(js_name = "BackupContext")]
#[derive(Clone)]
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
      username: context.username,
      config: Configuration::default(),
    },
  }
}
