use hyper_util::rt::TokioIo;
use std::{collections::HashMap, path::Path};
use tokio_stream::StreamExt;

use futures::{pin_mut, Future};
use tonic::{
    transport::{Channel, Endpoint, Server, Uri},
    Request,
};
use tower::service_fn;
use woodstock::{
    client::{config::ClientConfig, server::WoodstockClient},
    file_chunk, refresh_cache_request,
    utils::encryption::create_authentification_token,
    woodstock_client_service_client::WoodstockClientServiceClient,
    woodstock_client_service_server::WoodstockClientServiceServer,
    AuthenticateRequest, ChunkHashRequest, ChunkInformation, ExecuteCommandRequest, FileManifest,
    RefreshCacheRequest, Share,
};

async fn server_and_client_stub() -> (
    impl Future<Output = ()>,
    WoodstockClientServiceClient<Channel>,
) {
    let config_path = std::path::Path::new("./data");
    let config = ClientConfig {
        hostname: "localhost".to_string(),
        password: "password".to_string(),
        bind: "localhost".to_string(),
        secret: "secret".to_string(),
        acl: false,
        xattr: false,
        disable_mdns: true,
        backup_timeout: 3600,
        max_backup_seconds: 3600,
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

    (serve_future, client)
}

async fn get_session_id(
    config_path: &Path,
    client: &mut WoodstockClientServiceClient<Channel>,
) -> std::result::Result<String, tonic::Status> {
    let token = create_authentification_token(config_path, "localhost", "password")
        .await
        .map_err(|_| tonic::Status::invalid_argument("Can't create authentification token"))?;

    let result = client
        .authenticate(Request::new(AuthenticateRequest { version: 0, token }))
        .await
        .map_err(|_| tonic::Status::invalid_argument("can't authentificate"))?;
    let result = result.get_ref();

    result
        .session_id
        .parse()
        .map_err(|_| tonic::Status::invalid_argument("Can't parse x-session-id"))
}

async fn create_request<T>(
    session_id: &str,
    request: T,
) -> std::result::Result<Request<T>, tonic::Status> {
    let mut request = Request::new(request);
    request.metadata_mut().insert(
        "x-session-id",
        session_id
            .parse()
            .map_err(|_| tonic::Status::invalid_argument("Can't parse x-session-id"))?,
    );

    Ok(request)
}
#[tokio::test]
async fn test_client_authentification() {
    let config_path = std::path::Path::new("./data");
    let (serve_future, mut client) = server_and_client_stub().await;

    tokio::spawn(async move {
        serve_future.await;
    });

    let request_future = async {
        let result = client
            .authenticate(Request::new(AuthenticateRequest {
                version: 0,
                token: "random_string".to_string(),
            }))
            .await;

        // Assert result is error
        assert!(result.is_err());

        // Generate a good token
        let token = create_authentification_token(config_path, "localhost", "password")
            .await
            .unwrap();

        let result = client
            .authenticate(Request::new(AuthenticateRequest { version: 0, token }))
            .await
            .unwrap();
        let result = result.get_ref();

        // Assert result is error
        assert_eq!(result.session_id.len(), 315);
    };

    // Wait for completion, when the client request future completes
    request_future.await;
}

#[tokio::test]
async fn test_client_execute_command() {
    let config_path = std::path::Path::new("./data");
    let (serve_future, mut client) = server_and_client_stub().await;

    tokio::spawn(async move {
        serve_future.await;
    });

    let request_future = async {
        let session_id = get_session_id(config_path, &mut client).await.unwrap();

        // Command depending on windows or unix
        let command = if cfg!(unix) {
            "ls"
        } else {
            "cmd /c \"dir /b\""
        }
        .to_string();

        let command = create_request(&session_id, ExecuteCommandRequest { command })
            .await
            .unwrap();

        let result = client.execute_command(command).await.unwrap();

        let sep = if cfg!(unix) { "\n" } else { "\r\n" };
        let stdout = result.get_ref().stdout.clone();
        let mut stdout = stdout
            .split(sep)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        stdout.sort_unstable();
        let stdout = stdout.join("\n");

        // Assert result is error
        let result_stdout = if cfg!(unix) {
            "Cargo.toml\nbuild.rs\ndata\nsrc\ntests\nwoodstock.proto"
        } else {
            ".gitignore\n.vscode\nCargo.toml\nbuild.rs\ndata\nsrc\ntests\nwoodstock.proto"
        };
        assert_eq!(result.get_ref().code, 0);
        assert_eq!(stdout, result_stdout);
    };

    // Wait for completion, when the client request future completes
    request_future.await;
}

#[tokio::test]
async fn test_client_download_file_list() {
    let current_path = std::env::current_dir().unwrap();
    let config_path = std::path::Path::new("./data");

    let (serve_future, mut client) = server_and_client_stub().await;

    tokio::spawn(async move {
        serve_future.await;
    });

    let request_future = async {
        let session_id = get_session_id(config_path, &mut client).await.unwrap();

        let share = Share {
            share_path: current_path.to_str().unwrap().to_string(),
            includes: Vec::new(),
            excludes: vec![
                "**/test-compact.*".to_string(),
                "data/server".to_string(),
                "**/home.filelist.test".to_string(),
                "data/big_data".to_string(),
            ],
        };

        let refresh_cache = tokio_stream::iter(vec![
            RefreshCacheRequest {
                field: Some(refresh_cache_request::Field::Header(share.clone())),
            },
            RefreshCacheRequest {
                field: Some(refresh_cache_request::Field::FileManifest(FileManifest {
                    path: "Cargo.toml".into(),
                    stats: None,
                    hash: Vec::new(),
                    symlink: Vec::new(),
                    acl: Vec::new(),
                    xattr: Vec::new(),
                    chunks: Vec::new(),
                    metadata: HashMap::new(),
                })),
            },
        ]);

        let refresh_cache = create_request(&session_id, refresh_cache).await.unwrap();

        let mut stream = client.synchronize_file_list(refresh_cache).await.unwrap();
        let result = stream.get_mut();

        let result = result.collect::<Result<Vec<_>, _>>().await.unwrap();

        assert!(result.len() > 50);
    };

    // Wait for completion, when the client request future completes
    request_future.await;
}

#[tokio::test]
async fn test_client_get_chunk_hash() {
    let current_path = std::env::current_dir().unwrap();
    let config_path = std::path::Path::new("./data");

    let (serve_future, mut client) = server_and_client_stub().await;

    tokio::spawn(async move {
        serve_future.await;
    });

    let request_future = async {
        let session_id = get_session_id(config_path, &mut client).await.unwrap();

        let share = Share {
            share_path: current_path.to_str().unwrap().to_string(),
            includes: Vec::new(),
            excludes: vec![
                "**/test-compact.*".to_string(),
                "data/server".to_string(),
                "**/home.filelist.test".to_string(),
                "data/big_data".to_string(),
            ],
        };

        let refresh_cache = tokio_stream::iter(vec![RefreshCacheRequest {
            field: Some(refresh_cache_request::Field::Header(share.clone())),
        }]);

        let refresh_cache = create_request(&session_id, refresh_cache).await.unwrap();

        let mut stream = client.synchronize_file_list(refresh_cache).await.unwrap();
        let result = stream.get_mut();

        let result = result.collect::<Result<Vec<_>, _>>().await.unwrap();

        let mut count = 0;
        for file in result {
            if file.is_special_file() {
                continue;
            }

            let path = current_path.join(file.path());
            let chunk_request = ChunkHashRequest {
                filename: path.to_str().unwrap().into(),
            };
            let chunk = create_request(&session_id, chunk_request).await.unwrap();

            let request = client.get_chunk_hash(chunk).await.unwrap();

            let hash = request.get_ref();

            println!("{path:?} {hash:?}");
            assert_eq!(hash.hash.len(), 32);
            count += 1;
        }

        assert!(count > 50);
    };

    // Wait for completion, when the client request future completes
    request_future.await;
}

#[tokio::test]
async fn test_client_get_chunk() {
    let current_path = std::env::current_dir().unwrap();
    let config_path = std::path::Path::new("./data");

    let (serve_future, mut client) = server_and_client_stub().await;

    tokio::spawn(async move {
        serve_future.await;
    });

    let request_future = async {
        let session_id = get_session_id(config_path, &mut client).await.unwrap();

        let share = Share {
            share_path: current_path.to_str().unwrap().to_string(),
            includes: Vec::new(),
            excludes: vec![
                "**/test-compact.*".to_string(),
                "data/server".to_string(),
                "**/home.filelist.test".to_string(),
                "data/big_data".to_string(),
            ],
        };

        let refresh_cache = tokio_stream::iter(vec![RefreshCacheRequest {
            field: Some(refresh_cache_request::Field::Header(share.clone())),
        }]);

        let refresh_cache = create_request(&session_id, refresh_cache).await.unwrap();

        let mut stream = client.synchronize_file_list(refresh_cache).await.unwrap();

        let result = stream.get_mut();

        let result = result.collect::<Result<Vec<_>, _>>().await.unwrap();

        let mut count = 0;
        for file in result {
            if file.is_special_file() {
                continue;
            }

            let path = current_path.join(file.path());
            let chunk_request = ChunkInformation {
                filename: path.to_str().unwrap().into(),
                chunks_id: Vec::new(),
            };
            let chunk = create_request(&session_id, chunk_request).await.unwrap();

            let mut request = client.get_chunk(chunk).await.unwrap();
            let result = request.get_mut();
            pin_mut!(result);

            let mut size = 0;
            let mut header_chunk_number = 0;
            let mut footer_chunk_number = 0;
            while let Some(chunk) = result.next().await {
                let chunk = chunk.unwrap();
                match chunk.field {
                    Some(file_chunk::Field::Data(data)) => {
                        size += data.data.len() as u64;
                    }
                    Some(file_chunk::Field::Header(_)) => {
                        header_chunk_number += 1;
                    }
                    Some(file_chunk::Field::Footer(_)) => {
                        footer_chunk_number += 1;
                    }
                    _ => (),
                }
            }
            assert_eq!(size, file.size());
            assert_eq!(header_chunk_number, footer_chunk_number);

            count += 1;
        }

        assert!(count > 50);
    };

    // Wait for completion, when the client request future completes
    request_future.await;
}
