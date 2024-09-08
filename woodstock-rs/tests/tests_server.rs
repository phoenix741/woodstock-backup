use std::{path::PathBuf, sync::Arc};

use eyre::Result;
use futures::Future;
use tempfile::NamedTempFile;
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};
use tonic::transport::{Endpoint, Server, Uri};
use tower::service_fn;
use woodstock::{
    client::{config::ClientConfig, server::WoodstockClient},
    config::{ConfigurationPath, Context},
    server::{backup_client::BackupClient, grpc_client::BackupGrpcClient},
    woodstock_client_service_client::WoodstockClientServiceClient,
    woodstock_client_service_server::WoodstockClientServiceServer,
    Share,
};

#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};
#[cfg(unix)]
use tokio_stream::wrappers::UnixListenerStream;

#[cfg(windows)]
use tokio::net::{TcpListener, TcpStream};
#[cfg(windows)]
use tokio_stream::wrappers::TcpListenerStream;

fn create_context() -> Context {
    Context {
        config: woodstock::config::Configuration {
            path: ConfigurationPath::new(
                PathBuf::from("./data/server"),
                Some(PathBuf::from("./data")),
                Some(PathBuf::from("./data")),
                None,
                None,
                None,
                None,
            ),
            log_level: log::Level::Warn,
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
    };

    let woodstock_client = WoodstockClient::new(config_path, &config);

    let socket = NamedTempFile::new().unwrap();
    let socket = Arc::new(socket.into_temp_path());
    std::fs::remove_file(socket.as_ref()).unwrap();

    #[cfg(unix)]
    let stream = {
        let uds = UnixListener::bind(&*socket).unwrap();
        UnixListenerStream::new(uds)
    };

    #[cfg(windows)]
    let random_port = rand::random::<u16>() % 1000 + 1000;

    #[cfg(windows)]
    let stream = {
        let listener = TcpListener::bind(format!("127.0.0.1:{random_port}"))
            .await
            .unwrap();
        TcpListenerStream::new(listener)
    };

    let serve_future = async {
        let result = Server::builder()
            .add_service(WoodstockClientServiceServer::new(woodstock_client))
            .serve_with_incoming(stream)
            .await;

        assert!(result.is_ok());
    };

    // Connect to the server over a Unix socket
    // The URL will be ignored.
    #[cfg(unix)]
    let socket = Arc::clone(&socket);
    let channel = Endpoint::try_from("http://any.url")
        .unwrap()
        .connect_with_connector(service_fn(move |_: Uri| {
            #[cfg(unix)]
            let socket = Arc::clone(&socket);

            async move {
                #[cfg(unix)]
                let stream = UnixStream::connect(&*socket);
                #[cfg(windows)]
                let stream = TcpStream::connect(format!("127.0.0.1:{random_port}"));

                stream.await
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
    tokio::select! {
        () = serve_future => panic!("server returned first"),
        () = request_future => (),
    }
}
