use hyper_util::rt::TokioIo;
use std::path::PathBuf;

use eyre::Result;
use futures::Future;
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};
use tonic::transport::{Endpoint, Server, Uri};
use tower::service_fn;
use woodstock::{
    client::{config::ClientConfig, server::WoodstockClient},
    config::{ConfigurationPath, Context, OptionalConfigurationPath},
    server::{backup_client::BackupClient, grpc_client::BackupGrpcClient},
    woodstock_client_service_client::WoodstockClientServiceClient,
    woodstock_client_service_server::WoodstockClientServiceServer,
    Share,
};

fn create_context() -> Context {
    Context {
        source: woodstock::EventSource::Cli,
        username: None,
        config: woodstock::config::Configuration {
            path: ConfigurationPath::new(
                PathBuf::from("./data/server"),
                OptionalConfigurationPath {
                    certificates_path: Some(PathBuf::from("./data")),
                    config_path: Some(PathBuf::from("./data")),
                    ..Default::default()
                },
            ),
            log_level: log::Level::Warn,
            cache_size: 1,
        },
    }
}

async fn server_and_client_stub(
    context: &Context,
) -> (impl Future<Output = ()>, BackupClient<BackupGrpcClient>) {
    let config_path = std::path::Path::new("./data");

    let config = ClientConfig {
        hostname: "localhost".to_string(),
        password: "password".to_string(),
        bind: "localhost".to_string(),
        secret: "secret".to_string(),
        acl: false,
        xattr: false,
        disable_mdns: true,
        backup_timeout: 1000,
        max_backup_seconds: 1000,
        auto_update: false,
        log_directory: Some(PathBuf::from("./data")),
        update_delay: 1000,
    };

    let woodstock_client = WoodstockClient::new(config_path, &config);

    let (client, server) = tokio::io::duplex(1024);

    let serve_future = async {
        let result = Server::builder()
            .add_service(WoodstockClientServiceServer::new(woodstock_client))
            .serve_with_incoming(tokio_stream::once(Ok::<_, std::io::Error>(server)))
            .await;

        assert!(result.is_ok());
    };

    // Connect to the server over a Unix socket
    // The URL will be ignored.
    let mut client = Some(client);
    let channel = Endpoint::try_from("http://any.url")
        .unwrap()
        .connect_with_connector(service_fn(move |_: Uri| {
            let client = client.take();

            async move {
                if let Some(client) = client {
                    Ok(TokioIo::new(client))
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Client already taken",
                    ))
                }
            }
        }))
        .await
        .unwrap();

    let client = WoodstockClientServiceClient::new(channel);
    let client = BackupGrpcClient::with_client("localhost", "127.0.0.1", client, config_path);
    let client = BackupClient::new(client, "localhost", 0, context);

    (serve_future, client)
}

/// Generate a file of 1Gb containing uint128 number increasing
async fn generate_big_data() -> Result<()> {
    let file = File::create("data/big_data").await?;
    let mut file = BufWriter::new(file);

    for i in 0u32..20_971_520 {
        let bytes = i.to_be_bytes();
        file.write_all(&bytes).await?;
    }

    file.flush().await?;

    Ok(())
}

struct CleanUpTest {}

impl Drop for CleanUpTest {
    fn drop(&mut self) {
        let _ = std::fs::remove_file("data/big_data");
        let _ = std::fs::remove_dir_all("data/server");
    }
}

#[tokio::test]
async fn test_server_backup() {
    let _clean_up = CleanUpTest {};

    let current_path = std::env::current_dir().unwrap();
    let share_path = current_path.to_str().unwrap().to_string();

    let context = create_context();
    let (serve_future, mut client) = server_and_client_stub(&context).await;

    tokio::spawn(async move {
        serve_future.await;
    });

    let request_future = async {
        generate_big_data().await.unwrap();

        client.authenticate("password").await.unwrap();

        client
            .init_backup_directory(&[(share_path.as_str())])
            .await
            .unwrap();

        let share = Share {
            includes: vec![],
            excludes: vec![],
            share_path: share_path.clone(),
        };

        client.synchronize_file_list(&share, &|_| {}).await.unwrap();

        client.create_backup(&share_path, &|_| {}).await.unwrap();

        client.close().await.unwrap();

        client.compact(&share_path).await.unwrap();

        client.count_references().await.unwrap();

        client.save_backup(true, true).await.unwrap();
    };

    // Wait for completion, when the client request future completes
    request_future.await;
}
