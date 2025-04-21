use log::debug;
use napi::{Error, Result};
use woodstock::{config::GlobalConfiguration, server::tools::ping};

#[napi]
/// Pings a gRPC server at the specified IP and hostname.
///
/// This function attempts to contact a gRPC server using the provided IP address and hostname,
/// returning true if the server responds successfully.
///
/// # Arguments
/// * `ip` - The IP address of the server to ping.
/// * `hostname` - The hostname of the server to ping.
///
/// # Errors
/// Returns an error if the ping operation fails or if the server does not respond.
pub async fn grpc_ping(ip: String, hostname: String) -> Result<bool> {
  ping(ip, hostname, &GlobalConfiguration)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))
    .inspect(|&ping| {
      debug!("Ping result: {:?}", ping);
    })
}
