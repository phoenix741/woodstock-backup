use napi::{Error, Result};
use woodstock::config::{GlobalConfiguration, Hosts};

use crate::models::JsHostConfiguration;

#[napi(js_name = "CoreHostsService")]
/// Provides host management services for the Woodstock backup system.
///
/// This struct manages the hosts configuration and allows listing and retrieving host configurations.
///
/// # Fields
/// * `hosts` - The hosts configuration and state manager.
pub struct JsHostsService {
  /// The hosts configuration and state manager.
  hosts: Hosts,
}

impl Default for JsHostsService {
  fn default() -> Self {
    Self::new()
  }
}

#[napi]
impl JsHostsService {
  #[napi(constructor)]
  #[must_use]
  pub fn new() -> Self {
    Self {
      hosts: Hosts::new(&GlobalConfiguration),
    }
  }

  #[napi]
  /// Lists all hostnames managed by the backup system.
  ///
  /// # Errors
  /// Returns an error if the host list cannot be retrieved.
  pub async fn list(&self) -> Result<Vec<String>> {
    self
      .hosts
      .list_hosts()
      .await
      .map_err(|e| Error::from_reason(e.to_string()))
  }

  #[napi]
  /// Retrieves the configuration for the specified host.
  ///
  /// # Arguments
  /// * `name` - The name of the host to retrieve.
  ///
  /// # Errors
  /// Returns an error if the host configuration cannot be retrieved.
  pub async fn get(&self, name: String) -> Result<JsHostConfiguration> {
    self
      .hosts
      .get_host(&name)
      .await
      .map(std::convert::Into::into)
      .map_err(|e| Error::from_reason(e.to_string()))
  }
}
