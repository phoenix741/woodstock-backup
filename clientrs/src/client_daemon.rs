//! This file contains the main entry point for the Woodstock Backup client application.
//! It defines the `WoodstockClient` struct and implements the `WoodstockClientService` trait.
//! The `WoodstockClient` struct is responsible for handling client requests and managing the client's state.
//! The `WoodstockClientService` trait defines the service interface for the Woodstock Backup client.
//! It includes methods for authentication, executing commands, refreshing the cache, and launching backups.
//! The file also includes several modules for authentication, client configuration, commands, manifest handling, and scanning.
//!
#![recursion_limit = "256"]

use log::debug;
use tonic::codec::CompressionEncoding;
use tonic::transport::{Identity, Server, ServerTlsConfig};

use woodstock::client::config::{get_config_path, read_config};
use woodstock::client::grpc_logger::GrpcLogger;
use woodstock::client::server::WoodstockClient;
use woodstock::woodstock_client_service_server::WoodstockClientServiceServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let grpc_logger = Box::new(GrpcLogger::new());
    let env_logger = Box::new(env_logger::Builder::from_default_env().build());

    let rx_logger = grpc_logger.rx.clone();

    multi_log::MultiLogger::init(vec![grpc_logger, env_logger], log::Level::Debug).unwrap();

    let config_path = get_config_path();
    debug!("Config path: {}", config_path.display());
    let config_yml = config_path.join("config.yaml");
    let config = read_config(config_yml).expect("Failed to read config");

    let root_ca = config_path.join("rootCA.pem");
    let private_key = config_path.join(format!("{}_server.key", config.hostname));
    let public_key = config_path.join(format!("{}_server.pem", config.hostname));

    let root_ca = std::fs::read_to_string(root_ca).expect("Failed to read rootCA.pem");
    let public_key = std::fs::read_to_string(public_key).expect("Failed to public key");
    let private_key = std::fs::read_to_string(private_key).expect("Failed to private key");

    let addr = config.bind.parse()?;
    let woodstock_client =
        WoodstockClient::new(std::path::Path::new(&config_path), &config, rx_logger);

    let identity = Identity::from_pem(public_key, private_key);
    let client_ca_root = tonic::transport::Certificate::from_pem(root_ca);

    Server::builder()
        .tls_config(
            ServerTlsConfig::new()
                .identity(identity)
                .client_ca_root(client_ca_root),
        )?
        .add_service(
            WoodstockClientServiceServer::new(woodstock_client)
                .send_compressed(CompressionEncoding::Gzip)
                .accept_compressed(CompressionEncoding::Gzip),
        )
        .serve(addr)
        .await?;

    Ok(())
}
