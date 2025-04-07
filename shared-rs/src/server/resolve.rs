use napi::{Error, Result};
use woodstock::{
  config::{Context, DEFAULT_PORT},
  server::resolve::SocketAddrResolver,
};

use crate::config::context::JsBackupContext;

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
      addresses: info.addresses.iter().map(|addr| addr.to_string()).collect(),
      is_online: info.is_online,
    }
  }
}

#[napi]
pub fn resolve_dns(hostname: String) -> Vec<String> {
  woodstock::server::resolve::resolve_dns(&hostname)
    .iter()
    .map(|addr| addr.to_string())
    .collect()
}

#[napi(js_name = "CoreClientResolver")]
pub struct CoreClientResolver {
  resolver: SocketAddrResolver,
}

#[napi]
impl CoreClientResolver {
  #[napi(constructor)]
  pub fn new(ctxt: &JsBackupContext) -> Result<Self> {
    let context: Context = ctxt.into();

    let resolver = SocketAddrResolver::new(&context)
      .map_err(|_| Error::from_reason("Can't create socket address resolver".to_string()))?;

    Ok(Self { resolver })
  }

  #[napi]
  pub fn listen(&self) -> Result<AbortHandle> {
    let resolver = self.resolver.clone();

    let handle = tokio::spawn(async move {
      let _ = resolver.listen().await;
    });

    Ok(AbortHandle::new(handle))
  }

  #[napi]
  pub async fn resolve(&self, hostname: String, default_port: Option<u16>) -> Result<Vec<String>> {
    let default_port = default_port.unwrap_or(DEFAULT_PORT);
    let resolver = self.resolver.clone();

    let addresses = resolver
      .resolve(&hostname, default_port)
      .await
      .map(|addresses| {
        addresses
          .iter()
          .map(|addr| addr.to_string())
          .collect::<Vec<_>>()
      })
      .map_err(|e| Error::from_reason(format!("Can't resolve {hostname}: {e}").to_string()))?;

    Ok(addresses)
  }

  #[napi]
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
  pub async fn get_informations(
    &self,
    hostname: String,
  ) -> Result<Option<JsSocketAddrInformation>> {
    let resolver = self.resolver.clone();
    let informations = resolver
      .get_informations(&hostname)
      .await
      .map_err(|err| Error::from_reason(err.to_string()))?;

    Ok(informations.map(|info| info.into()))
  }

  #[napi]
  pub async fn update_online_status(&self, hostname: String, is_online: bool) -> Result<()> {
    let resolver = self.resolver.clone();
    resolver
      .update_online_status(&hostname, is_online)
      .await
      .map_err(|err| Error::from_reason(err.to_string()))?;

    Ok(())
  }
}
