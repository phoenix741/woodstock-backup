use log::info;
use napi::{Error, Result};
use woodstock::{
  config::{GlobalConfiguration, DEFAULT_PORT},
  server::resolve::SocketAddrResolver,
  utils::thread::spawn_with_context,
};

use super::AbortHandle;

#[napi(object)]
pub struct JsSocketAddrInformation {
  pub hostname: String,
  pub port: u16,
  pub version: String,
  pub addresses: Vec<String>,
  pub is_online: bool,
}

impl From<woodstock::server::resolve::SocketAddrInformation> for JsSocketAddrInformation {
  fn from(info: woodstock::server::resolve::SocketAddrInformation) -> Self {
    Self {
      hostname: info.hostname,
      port: info.port,
      version: info.version,
      addresses: info.addresses.iter().map(ToString::to_string).collect(),
      is_online: info.is_online,
    }
  }
}

#[must_use]
#[napi]
pub fn resolve_dns(hostname: String) -> Vec<String> {
  woodstock::server::resolve::resolve_dns(&hostname)
    .iter()
    .map(ToString::to_string)
    .collect()
}

#[napi(js_name = "CoreClientResolver")]
/// Resolver for client socket addresses, used to manage network service discovery.
pub struct CoreClientResolver {
  /// The underlying socket address resolver.
  resolver: SocketAddrResolver,
}

#[napi]
impl CoreClientResolver {
  #[napi(constructor)]
  /// Create a new `CoreClientResolver`.
  ///
  /// # Errors
  /// Returns an error if the socket address resolver cannot be created.
  pub fn new() -> Result<Self> {
    let resolver = SocketAddrResolver::new(&GlobalConfiguration)
      .map_err(|_| Error::from_reason("Can't create socket address resolver".to_string()))?;

    Ok(Self { resolver })
  }

  #[napi]
  /// Start listening for network service discovery events.
  ///
  /// # Errors
  /// Returns an error if the listener cannot be started.
  pub fn listen(&self) -> Result<AbortHandle> {
    let resolver = self.resolver.clone();

    let handle = spawn_with_context(async move {
      let _ = resolver.listen().await;
    });

    Ok(AbortHandle::new(handle))
  }

  #[napi]
  /// Resolve a hostname to a list of addresses asynchronously.
  ///
  /// # Errors
  /// Returns an error if the resolution fails.
  pub async fn resolve(&self, hostname: String, default_port: Option<u16>) -> Result<Vec<String>> {
    let default_port = default_port.unwrap_or(DEFAULT_PORT);
    let resolver = self.resolver.clone();

    info!("Try resolving {hostname} with default port {default_port}");

    let addresses = resolver
      .resolve(&hostname, default_port)
      .await
      .map(|addresses| {
        info!("Resolved {hostname} to {addresses:?}");
        addresses
          .iter()
          .map(|addr| addr.to_string())
          .collect::<Vec<_>>()
      })
      .map_err(|e| Error::from_reason(format!("Can't resolve {hostname}: {e}").to_string()))?;

    Ok(addresses)
  }

  #[napi]
  /// Register a network service for discovery.
  ///
  /// # Panics
  /// Panics if an address cannot be parsed.
  ///
  /// # Errors
  /// Returns an error if the service cannot be registered.
  pub async fn register_service(&self, information: JsSocketAddrInformation) -> Result<()> {
    let resolver = self.resolver.clone();
    let information = woodstock::server::resolve::SocketAddrInformation {
      refresh_date: chrono::Utc::now().timestamp(),
      hostname: information.hostname,
      port: information.port,
      version: information.version,
      addresses: information
        .addresses
        .iter()
        .map(|addr| addr.parse().unwrap())
        .collect(),
      is_online: information.is_online,
      source: woodstock::server::resolve::SocketAddrInformationSource::DIRECT,
    };

    resolver
      .register_service(&information)
      .await
      .map_err(|_| Error::from_reason("Can't register service".to_string()))?;

    Ok(())
  }

  #[napi]
  /// Get information about a registered service by hostname.
  ///
  /// # Errors
  /// Returns an error if the information cannot be retrieved.
  pub async fn get_informations(
    &self,
    hostname: String,
  ) -> Result<Option<JsSocketAddrInformation>> {
    let resolver = self.resolver.clone();
    let informations = resolver
      .get_informations(&hostname)
      .await
      .map_err(|err| Error::from_reason(err.to_string()))?;

    Ok(informations.map(Into::into))
  }

  #[napi]
  /// Update the online status of a registered service by hostname.
  ///
  /// # Errors
  /// Returns an error if the status cannot be updated.
  pub async fn update_online_status(&self, hostname: String, is_online: bool) -> Result<()> {
    let resolver = self.resolver.clone();
    resolver
      .update_online_status(&hostname, is_online)
      .await
      .map_err(|err| Error::from_reason(err.to_string()))?;

    Ok(())
  }
}
