use napi::{Error, Result};
use woodstock::server::resolve::SocketAddrResolver;

use super::AbortHandle;

#[napi(object)]
pub struct JsSocketAddrInformation {
  pub hostname: String,
  pub port: u16,
  pub version: String,
  pub addresses: Vec<String>,
}

impl From<woodstock::server::resolve::SocketAddrInformation> for JsSocketAddrInformation {
  fn from(info: woodstock::server::resolve::SocketAddrInformation) -> Self {
    Self {
      hostname: info.hostname,
      port: info.port,
      version: info.version,
      addresses: info.addresses.iter().map(|addr| addr.to_string()).collect(),
    }
  }
}

#[napi]
pub async fn resolve_dns(hostname: String) -> Option<Vec<String>> {
  woodstock::server::resolve::resolve_dns(&hostname)
    .await
    .map(|addresses| addresses.iter().map(|addr| addr.to_string()).collect())
}

#[napi(js_name = "CoreClientResolver")]
pub struct CoreClientResolver {
  resolver: SocketAddrResolver,
}

#[napi]
impl CoreClientResolver {
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let resolver = SocketAddrResolver::new()
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
  pub async fn resolve(&self, hostname: String) -> Option<Vec<String>> {
    let resolver = self.resolver.clone();

    resolver
      .resolve(&hostname)
      .await
      .map(|addresses| addresses.iter().map(|addr| addr.to_string()).collect())
  }

  #[napi]
  pub async fn get_informations(&self, hostname: String) -> Option<JsSocketAddrInformation> {
    let resolver = self.resolver.clone();
    let informations = resolver.get_informations(&hostname).await;

    informations.map(|info| info.into())
  }
}
