use log::debug;
use napi::{Error, Result};
use woodstock::{config::GlobalConfiguration, server::tools::ping};

#[napi]
pub async fn grpc_ping(ip: String, hostname: String) -> Result<bool> {
  ping(ip, hostname, &GlobalConfiguration)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))
    .inspect(|&ping| {
      debug!("Ping result: {:?}", ping);
    })
}
