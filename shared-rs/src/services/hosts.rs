use napi::{Error, Result};
use woodstock::config::{Context, Hosts};

use crate::{config::context::JsBackupContext, models::JsHostConfiguration};

#[napi(js_name = "CoreHostsService")]
pub struct JsHostsService {
  hosts: Hosts,
}

#[napi]
impl JsHostsService {
  #[napi(constructor)]
  #[must_use]
  pub fn new(context: &JsBackupContext) -> Self {
    let context: Context = context.into();

    Self {
      hosts: Hosts::new(&context),
    }
  }

  #[napi]
  pub async fn list(&self) -> Result<Vec<String>> {
    self
      .hosts
      .list_hosts()
      .await
      .map_err(|e| Error::from_reason(e.to_string()))
  }

  #[napi]
  pub async fn get(&self, name: String) -> Result<JsHostConfiguration> {
    self
      .hosts
      .get_host(&name)
      .await
      .map(std::convert::Into::into)
      .map_err(|e| Error::from_reason(e.to_string()))
  }
}
