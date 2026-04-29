use std::{net::SocketAddr, sync::Arc};

use super::{client::grpc::BackupGrpcClient, client::Client};
use crate::config::Configuration;
use tracing::{debug, error};

/// Pings a server to check its availability.
///
/// # Arguments
/// * `addr` - The IP address of the server.
/// * `hostname` - The hostname of the server.
/// * `config` - The configuration for the gRPC client.
///
/// # Returns
///
/// * `Ok(true)` if the server is reachable.
/// * `Ok(false)` if the server is not reachable.
/// * `Err(eyre::Report)` if an error occurs during the ping process.
///
/// # Errors
///
/// Returns an error if the gRPC client cannot be created or if the ping operation fails unexpectedly.
pub async fn ping(addr: &SocketAddr, hostname: &str, config: Arc<Configuration>) -> bool {
    let grpc_client = BackupGrpcClient::new(hostname, addr, config).await;
    match grpc_client {
        Ok(grpc_client) => {
            let ping = grpc_client.ping().await;
            match ping {
                Ok(ping) => ping,
                Err(e) => {
                    debug!("Error pinging grpc client: {:?}", e);
                    false
                }
            }
        }
        Err(e) => {
            error!("Error creating grpc client: {:?}", e);
            false
        }
    }
}
