use napi::{Error, Result};
use woodstock::config::{Configuration, Hosts};

use crate::models::JsHostConfiguration;

#[napi(js_name = "CoreHostsService")]
pub struct JsHostsService {
  hosts: Hosts,
}

#[napi]
impl JsHostsService {
  #[napi(constructor)]
  #[must_use]
  pub fn new() -> Self {
    let config = Configuration::default();

    Self {
      hosts: Hosts::new(&config),
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
