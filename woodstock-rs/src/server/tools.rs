use super::{client::Client, grpc_client::BackupGrpcClient};
use crate::config::Context;
use eyre::Result;
use log::{debug, error};

pub async fn ping(ip: String, hostname: String, context: &Context) -> Result<bool> {
    let grpc_client = BackupGrpcClient::new(&hostname, &ip, context).await;
    match grpc_client {
        Ok(grpc_client) => {
            let ping = grpc_client.ping().await;
            match ping {
                Ok(ping) => Ok(ping),
                Err(e) => {
                    debug!("Error pinging grpc client: {:?}", e);
                    Ok(false)
                }
            }
        }
        Err(e) => {
            error!("Error creating grpc client: {:?}", e);
            Ok(false)
        }
    }
}
