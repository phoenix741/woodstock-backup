use futures::{pin_mut, Stream, TryStreamExt};
use log::{debug, error, info, trace};
use std::path::PathBuf;
use std::{path::Path, pin::Pin, sync::Arc};
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{metadata::MetadataMap, Response};

use crate::scanner::create_file_from_manifest;
use crate::utils::path::path_to_vec;
use crate::woodstock::{
    refresh_cache_request, woodstock_client_service_server::WoodstockClientService,
    AuthenticateReply, AuthenticateRequest, Empty as EmptyProto, EntryState, EntryType,
    ExecuteCommandReply, ExecuteCommandRequest, FileManifestJournalEntry,
};
use crate::{client::authentification::Service as AuthService, ChunkInformation};
use crate::{client::config::ClientConfig, FileChunk};
use crate::{client::exexcute_command::execute_command, scanner::CreateManifestOptions};
use crate::{manifest::FileManifestLight, PingRequest};
use crate::{manifest::IndexManifest, ChunkHashRequest};
use crate::{restore_file_request, FileManifest, RestoreFileReply, RestoreFileRequest};
use crate::{scanner::get_files_with_hash, ChunkHashReply};
use crate::{
    scanner::{calculate_chunk_hash_future, read_chunk},
    RefreshCacheRequest,
};
use crate::{
    utils::path::{list_to_globset, vec_to_str},
    Share,
};

/// The main context struct for the Woodstock Backup client.
/// It holds the authentication service and the client's context.
pub struct WoodstockClient {
    hostname: String,
    authentification_service: Arc<RwLock<AuthService>>,
    create_manifest_options: CreateManifestOptions,
}

impl WoodstockClient {
    /// Creates a new instance of `WoodstockClient`.
    ///
    /// # Arguments
    ///
    /// * `certificate_path` - The path to the certificate file.
    /// * `config` - The client configuration.
    ///
    /// # Returns
    ///
    /// A new instance of `WoodstockClient`.
    #[must_use]
    pub fn new(certificate_path: &Path, config: &ClientConfig) -> Self {
        let authentification_service = AuthService::new(certificate_path, config);

        Self {
            hostname: config.hostname.clone(),
            authentification_service: Arc::new(RwLock::new(authentification_service)),
            create_manifest_options: CreateManifestOptions {
                with_acl: config.acl,
                with_xattr: config.xattr,
            },
        }
    }

    /// Checks the context of the request and returns the session ID.
    ///
    /// # Arguments
    ///
    /// * `request` - The request to check.
    ///
    /// # Returns
    ///
    /// The session ID if the context is valid, otherwise an error.
    async fn check_context(&self, request_metadata: &MetadataMap) -> Result<String, tonic::Status> {
        // Get the value of x-session-id in metadata
        let session_id = request_metadata
            .get("x-session-id")
            .ok_or_else(|| tonic::Status::permission_denied("Session id not found"))?;

        debug!("Session ID: {:?}", session_id);

        let session_id = session_id
            .to_str()
            .map_err(|_err| tonic::Status::permission_denied("Invalid session id"))?;

        let service = self.authentification_service.read().await;
        let session_id = service
            .check_context(session_id)
            .await
            .map_err(|err| tonic::Status::permission_denied(err.to_string()))?;

        Ok(session_id)
    }

    async fn handle_restore_file(
        mut stream: tonic::Streaming<RestoreFileRequest>,
        tx: mpsc::Sender<Result<RestoreFileReply, tonic::Status>>,
    ) -> Result<(), tonic::Status> {
        let mut current_file = None;
        let mut error_occurred = false;

        while let Some(request) = stream.next().await {
            let request =
                request.map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;

            debug!("Content of current_file: {:?}", current_file);

            match request.request {
                Some(restore_file_request::Request::Manifest(file)) => {
                    info!("Received manifest for file: {:?}", file.path());

                    let option = Self::process_restore_manifest(&file, &tx).await?;
                    debug!("Option: {:?}", option);

                    let previous_path = if let Some((path, writer)) = option {
                        current_file.replace((path, writer))
                    } else {
                        current_file.take()
                    };

                    if let Some((previous_path, mut writer)) = previous_path {
                        let reply = RestoreFileReply {
                            path: path_to_vec(&previous_path),
                            restored: true,
                            message: String::new(),
                        };

                        writer.flush().await.map_err(|err| {
                            error!("Failed to flush file: {:?}", err);
                            tonic::Status::internal("Failed to flush file")
                        })?;

                        tx.send(Ok(reply)).await.map_err(|err| {
                            error!("Failed to send restore_file reply: {:?}", err);
                            tonic::Status::internal("Failed to send restore_file reply")
                        })?;
                    }
                }
                Some(restore_file_request::Request::Chunk(chunk)) => {
                    debug!("Receive chunk of length: {}", chunk.len());
                    if !error_occurred {
                        if let Some((current_path, current_file)) = &mut current_file {
                            error_occurred = Self::process_restore_chunk(
                                &chunk,
                                current_file,
                                &current_path,
                                &tx,
                            )
                            .await?;
                        } else {
                            debug!("No file to write chunk to");
                        }
                    }
                }
                None => {
                    error!("Unknown message in restore_file request");
                    break;
                }
            }
        }

        if let Some((path, mut writer)) = current_file.take() {
            writer.flush().await.map_err(|err| {
                error!("Failed to flush file: {:?}", err);
                tonic::Status::internal("Failed to flush file")
            })?;

            let reply = RestoreFileReply {
                path: path_to_vec(&path),
                restored: true,
                message: String::new(),
            };

            writer.flush().await.map_err(|err| {
                error!("Failed to flush file: {:?}", err);
                tonic::Status::internal("Failed to flush file")
            })?;

            tx.send(Ok(reply)).await.map_err(|err| {
                error!("Failed to send restore_file reply: {:?}", err);
                tonic::Status::internal("Failed to send restore_file reply")
            })?;
        }

        Ok(())
    }

    async fn process_restore_manifest(
        file: &FileManifest,
        tx: &mpsc::Sender<Result<RestoreFileReply, tonic::Status>>,
    ) -> Result<Option<(PathBuf, tokio::io::BufWriter<File>)>, tonic::Status> {
        let path = file.path();
        match create_file_from_manifest(file) {
            Ok(_) => {
                if file.is_regular_file() {
                    let file = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(&path)
                        .await;

                    match file {
                        Ok(file) => {
                            let writer = tokio::io::BufWriter::new(file);

                            return Ok(Some((path, writer)));
                        }
                        Err(err) => {
                            Self::send_restore_error_reply(
                                &path,
                                format!("Failed to open file: {:?}", err),
                                tx,
                            )
                            .await?;

                            return Ok(None);
                        }
                    }
                }

                Ok(None)
            }
            Err(err) => {
                error!("Failed to create file from manifest: {:?}", err);
                Self::send_restore_error_reply(
                    &file.path(),
                    format!("Failed to create file from manifest: {:?}", err),
                    tx,
                )
                .await?;

                Ok(None)
            }
        }
    }

    async fn process_restore_chunk<P: AsRef<Path>>(
        chunk: &[u8],
        current_file: &mut tokio::io::BufWriter<File>,
        current_path: P,
        tx: &mpsc::Sender<Result<RestoreFileReply, tonic::Status>>,
    ) -> Result<bool, tonic::Status> {
        let result = if let Err(err) = current_file.write_all(chunk).await {
            Self::send_restore_error_reply(
                current_path,
                format!("Failed to write chunk: {:?}", err),
                tx,
            )
            .await?;

            true
        } else {
            false
        };

        Ok(result)
    }

    async fn send_restore_error_reply<P: AsRef<Path>>(
        path: P,
        message: String,
        tx: &mpsc::Sender<Result<RestoreFileReply, tonic::Status>>,
    ) -> Result<(), tonic::Status> {
        let reply = RestoreFileReply {
            path: path_to_vec(path.as_ref()),
            restored: false,
            message,
        };
        tx.send(Ok(reply)).await.map_err(|err| {
            error!("Failed to send restore_file reply: {:?}", err);
            tonic::Status::internal("Failed to send restore_file reply")
        })
    }
}

#[tonic::async_trait]
impl WoodstockClientService for WoodstockClient {
    /// Used by the server to check if the client is up and find the ip of the client.
    ///
    /// # Arguments
    ///
    /// * `request` - The request (with the hostname).
    ///
    /// # Returns
    ///
    /// The reply that tell nothing if it's the right hostname, and 404 else
    async fn ping(
        &self,
        request: tonic::Request<PingRequest>,
    ) -> std::result::Result<tonic::Response<EmptyProto>, tonic::Status> {
        let hostname = request.get_ref().hostname.clone();
        debug!("Ping for {}, current hostname {}", hostname, self.hostname);

        if hostname != self.hostname {
            return Err(tonic::Status::not_found("Wrong hostname"));
        }

        Ok(tonic::Response::new(EmptyProto {}))
    }

    /// Authenticates the client with the provided token.
    ///
    /// # Arguments
    ///
    /// * `request` - The authentication request.
    ///
    /// # Returns
    ///
    /// The authentication reply if successful, otherwise an error.
    async fn authenticate(
        &self,
        request: tonic::Request<AuthenticateRequest>,
    ) -> std::result::Result<tonic::Response<AuthenticateReply>, tonic::Status> {
        let version = request.get_ref().version;
        let token = &request.get_ref().token;

        debug!("Start authentification for {} version {}", token, version);

        // Get the version in request
        if version != 0 {
            error!("Unsupported version: {}", version);
            return Err(tonic::Status::invalid_argument("Unsupported version"));
        }

        let mut service = self.authentification_service.write().await;
        let session_id = service.authenticate(token);

        match session_id {
            Ok(session_id) => Ok(tonic::Response::new(AuthenticateReply {
                agent_version: ClientConfig::version(),
                session_id,
            })),
            Err(e) => {
                error!("Failed to authenticate: {:?}", e);

                Err(tonic::Status::permission_denied(e.to_string()))
            }
        }
    }

    /// Executes a command on the client.
    ///
    /// # Arguments
    ///
    /// * `request` - The execute command request.
    ///
    /// # Returns
    ///
    /// The execute command reply if successful, otherwise an error.
    async fn execute_command(
        &self,
        request: tonic::Request<ExecuteCommandRequest>,
    ) -> std::result::Result<tonic::Response<ExecuteCommandReply>, tonic::Status> {
        let command = &request.get_ref().command;

        debug!("Start execute command: {}", command);

        self.check_context(request.metadata()).await?;

        let output = execute_command(command);

        match output {
            Ok(output) => {
                let stdout = String::from_utf8(output.stdout).unwrap_or_default();
                let stderr = String::from_utf8(output.stderr).unwrap_or_default();

                let reply = ExecuteCommandReply {
                    code: output.status.code().unwrap_or_default(),
                    stdout,
                    stderr,
                };

                Ok(tonic::Response::new(reply))
            }
            Err(e) => {
                error!("Failed to execute command: {:?}", e);
                Ok(tonic::Response::new(ExecuteCommandReply {
                    code: -1,
                    stdout: String::new(),
                    stderr: e.to_string(),
                }))
            }
        }
    }

    type RestoreFileStream = ReceiverStream<Result<RestoreFileReply, tonic::Status>>;

    async fn restore_file(
        &self,
        request: tonic::Request<tonic::Streaming<RestoreFileRequest>>,
    ) -> std::result::Result<tonic::Response<Self::RestoreFileStream>, tonic::Status> {
        debug!("Start restoring file");

        self.check_context(request.metadata()).await?;

        {
            let service = self.authentification_service.read().await;

            if service.is_restauration_disabled() {
                return Err(tonic::Status::permission_denied("Restauration is disabled"));
            }
        }

        let (tx, rx) = mpsc::channel::<Result<RestoreFileReply, tonic::Status>>(100_000);
        tokio::spawn(Self::handle_restore_file(request.into_inner(), tx));

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type SynchronizeFileListStream =
        ReceiverStream<Result<FileManifestJournalEntry, tonic::Status>>;

    /// Synchronizes the file list between the client and the server.
    ///
    /// # Arguments
    ///
    /// * `request` - The list of files known by the server
    ///
    /// # Returns
    ///
    /// The list of files known by the client
    async fn synchronize_file_list(
        &self,
        request: tonic::Request<tonic::Streaming<RefreshCacheRequest>>,
    ) -> std::result::Result<tonic::Response<Self::SynchronizeFileListStream>, tonic::Status> {
        debug!("Start refreshing cache");

        self.check_context(request.metadata()).await?;

        let mut stream = request.into_inner();

        let mut index: IndexManifest<FileManifestLight> = IndexManifest::new();
        let mut share: Option<Share> = None;

        while let Some(request) = stream.next().await {
            let request = request?;

            match request.field {
                Some(refresh_cache_request::Field::Header(header)) => {
                    debug!("Received header: {:?}", header);
                    if share.is_some() {
                        error!("Header already defined");
                        return Err(tonic::Status::invalid_argument("Header already defined"));
                    }

                    share = Some(header);
                }
                Some(refresh_cache_request::Field::FileManifest(manifest)) => {
                    trace!("Received manifest: {:?}", manifest);
                    index.add(FileManifestLight::from(manifest));
                }
                None => {
                    error!("Unknown message in refresh_cache request");
                    return Err(tonic::Status::invalid_argument("Unknown message"));
                }
            }
        }

        if share.is_none() {
            error!("Share must be defined");
            return Err(tonic::Status::invalid_argument("Share must be defined"));
        }
        let share = share.as_ref().unwrap().clone();

        debug!("Launch backup for share: {}", share.share_path);

        let includes = vec_to_str(&share.includes);
        let includes = list_to_globset(&includes)
            .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;
        let excludes = vec_to_str(&share.excludes);
        let excludes = list_to_globset(&excludes)
            .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;

        let create_manifest_options = self.create_manifest_options.clone();

        let (tx, rx) = mpsc::channel(100_000);
        tokio::spawn(async move {
            // Add and modify file
            {
                let share_path = Path::new(&share.share_path);
                let files = get_files_with_hash(
                    &mut index,
                    share_path,
                    &includes,
                    &excludes,
                    &create_manifest_options,
                );
                pin_mut!(files);

                while let Some(file) = files.next().await {
                    let result = tx.send(Ok(file)).await;
                    if let Err(err) = result {
                        error!("Failed to send file manifest journal entry: {err}");
                        break;
                    }
                }
            }

            // Remove file
            let file_to_remove = index.walk();
            for file in file_to_remove {
                if file.mark_viewed {
                    continue;
                }

                let entry = FileManifestJournalEntry {
                    r#type: EntryType::Remove as i32,
                    manifest: Some(FileManifest {
                        path: file.manifest.path.clone(),
                        ..Default::default()
                    }),

                    state: EntryState::Metadata as i32,
                    state_messages: Vec::new(),
                };
                let result = tx.send(Ok(entry)).await;
                if result.is_err() {
                    error!("Failed to send file manifest journal entry");
                    break;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn get_chunk_hash(
        &self,
        request: tonic::Request<ChunkHashRequest>,
    ) -> std::result::Result<tonic::Response<ChunkHashReply>, tonic::Status> {
        debug!("Get chunk hash");

        self.check_context(request.metadata()).await?;

        let chunk = request.get_ref();

        let reply = calculate_chunk_hash_future(chunk).await;

        Ok(Response::new(reply))
    }

    type GetChunkStream =
        Pin<Box<dyn Stream<Item = Result<FileChunk, tonic::Status>> + Send + 'static>>;

    /// Retrieves a chunk of data.
    ///
    /// # Arguments
    ///
    /// * `request` - The get chunk request stream.
    ///
    /// # Returns
    ///
    /// A stream of get chunk replies.
    async fn get_chunk(
        &self,
        request: tonic::Request<ChunkInformation>,
    ) -> std::result::Result<tonic::Response<Self::GetChunkStream>, tonic::Status> {
        debug!("Waiting for chunk");

        self.check_context(request.metadata()).await?;

        let chunk = request.get_ref();

        let replies = read_chunk(chunk).map_err(|f| tonic::Status::invalid_argument(f.to_string()));

        Ok(Response::new(Box::pin(replies) as Self::GetChunkStream))
    }

    /// Closes a backup operation.
    ///
    /// # Arguments
    ///
    /// * `request` - The close backup request.
    ///
    /// # Returns
    ///
    /// The close backup reply if successful, otherwise an error.
    async fn close_backup(
        &self,
        request: tonic::Request<EmptyProto>,
    ) -> std::result::Result<tonic::Response<EmptyProto>, tonic::Status> {
        debug!("Close backup");

        let session_id = self.check_context(request.metadata()).await?;

        self.authentification_service
            .write()
            .await
            .logout(&session_id)
            .map_err(|err| tonic::Status::permission_denied(err.to_string()))?;

        Ok(Response::new(EmptyProto {}))
    }
}
