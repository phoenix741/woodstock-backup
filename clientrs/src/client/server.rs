use futures::{pin_mut, Stream, TryStreamExt};
use log::{debug, error};
use std::{
    collections::HashMap,
    path::Path,
    pin::Pin,
    sync::{Arc, Mutex},
};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{metadata::MetadataMap, Response};

use crate::manifest::FileManifestLight;
use crate::scanner::{calculate_chunk_hash_future, read_chunk};
use crate::utils::path::{list_to_globset, vec_to_str};
use crate::woodstock::{
    refresh_cache_request, woodstock_client_service_server::WoodstockClientService,
    AuthenticateReply, AuthenticateRequest, Empty as EmptyProto, EntryType, ExecuteCommandReply,
    ExecuteCommandRequest, FileManifestJournalEntry, LaunchBackupRequest, RefreshCacheRequest,
};
use crate::FileManifest;
use crate::{client::authentification::Service as AuthService, ChunkInformation};
use crate::{client::config::ClientConfig, FileChunk};
use crate::{client::exexcute_command::execute_command, scanner::CreateManifestOptions};
use crate::{manifest::IndexManifest, ChunkHashRequest};
use crate::{scanner::get_files_with_hash, ChunkHashReply};

#[derive(Default)]
struct WoodstockContext {
    index: HashMap<String, IndexManifest<FileManifestLight>>,
}

/// The main context struct for the Woodstock Backup client.
/// It holds the authentication service and the client's context.
pub struct WoodstockClient {
    authentification_service: Arc<RwLock<AuthService>>,
    context: Arc<Mutex<HashMap<String, WoodstockContext>>>,
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
            authentification_service: Arc::new(RwLock::new(authentification_service)),
            context: Arc::new(Mutex::new(HashMap::new())),
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
            .map_err(|err| tonic::Status::permission_denied(err.to_string()))?;

        Ok(session_id)
    }

    /// Updates the index in the client's context.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID.
    /// * `index` - The index to update.
    ///
    /// # Returns
    ///
    /// An empty result if the update is successful, otherwise an error.
    fn update_index_in_context(
        &self,
        session_id: &str,
        index: HashMap<String, IndexManifest<FileManifestLight>>,
    ) -> Result<(), tonic::Status> {
        debug!("Updating index in context for session: {}", session_id);

        let mut context_map = self
            .context
            .lock()
            .map_err(|_err| tonic::Status::permission_denied("Can't get session"))?;

        let context = context_map.get_mut(session_id);

        if let Some(context) = context {
            context.index = index;
        } else {
            let context = WoodstockContext { index };
            context_map.insert(session_id.to_string(), context);
        };

        Ok(())
    }

    /// Gets the index from the client's context.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID.
    /// * `share_path` - The share path.
    ///
    /// # Returns
    ///
    /// The index if found, otherwise an error.
    fn get_index_in_context(
        &self,
        session_id: &str,
        share_path: &String,
    ) -> Result<IndexManifest<FileManifestLight>, tonic::Status> {
        let mut context_map = self
            .context
            .lock()
            .map_err(|_err| tonic::Status::permission_denied("Can't get session"))?;
        let context = context_map.get_mut(session_id);

        let index = match context {
            Some(context) => {
                let index = context.index.remove(share_path);
                match index {
                    Some(index) => index,
                    None => return Err(tonic::Status::permission_denied("Index not found")),
                }
            }
            None => return Err(tonic::Status::permission_denied("Session not found")),
        };

        Ok(index)
    }
}

#[tonic::async_trait]
impl WoodstockClientService for WoodstockClient {
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
            Ok(session_id) => Ok(tonic::Response::new(AuthenticateReply { session_id })),
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

    /// Refreshes the cache with the provided cache requests.
    ///
    /// # Arguments
    ///
    /// * `request` - The refresh cache request stream.
    ///
    /// # Returns
    ///
    /// The refresh cache reply if successful, otherwise an error.
    async fn refresh_cache(
        &self,
        request: tonic::Request<tonic::Streaming<RefreshCacheRequest>>,
    ) -> std::result::Result<tonic::Response<EmptyProto>, tonic::Status> {
        debug!("Start refreshing cache");

        let session_id = self.check_context(request.metadata()).await?;

        let mut stream = request.into_inner();

        let mut index_map: HashMap<String, IndexManifest<FileManifestLight>> = HashMap::new();
        let mut current_path: Option<String> = None;

        while let Some(request) = stream.next().await {
            let request = request?;

            match request.field {
                Some(refresh_cache_request::Field::Header(header)) => {
                    debug!("Received header: {:?}", header);
                    let path = header.share_path.clone();
                    if !index_map.contains_key(&path) {
                        index_map.insert(path.clone(), IndexManifest::new());
                    }
                    current_path = Some(path);
                }
                Some(refresh_cache_request::Field::FileManifest(manifest)) => {
                    // debug!("Received manifest: {:?}", manifest);

                    let current_path = current_path.clone();
                    let current_path = current_path.unwrap_or_default();
                    let index = index_map.get_mut(&current_path);
                    if let Some(index) = index {
                        index.apply(FileManifestJournalEntry {
                            r#type: EntryType::Add as i32,
                            manifest: Some(manifest),
                        });
                    } else {
                        error!("Missing header in refresh_cache request");
                        return Err(tonic::Status::invalid_argument(
                            "Missing header in refresh_cache request",
                        ));
                    }
                }
                None => {
                    error!("Unknown message in refresh_cache request");
                    return Err(tonic::Status::invalid_argument("Unknown message"));
                }
            }
        }

        self.update_index_in_context(&session_id, index_map)?;

        Ok(Response::new(EmptyProto {}))
    }

    type LaunchBackupStream = ReceiverStream<Result<FileManifestJournalEntry, tonic::Status>>;

    /// Launches a backup operation.
    ///
    /// # Arguments
    ///
    /// * `request` - The launch backup request.
    ///
    /// # Returns
    ///
    /// A stream of launch backup replies.
    async fn launch_backup(
        &self,
        request: tonic::Request<LaunchBackupRequest>,
    ) -> std::result::Result<tonic::Response<Self::LaunchBackupStream>, tonic::Status> {
        let share = &request.get_ref().share;
        if share.is_none() {
            error!("Share must be defined");
            return Err(tonic::Status::invalid_argument("Share must be defined"));
        }
        let share = share.as_ref().unwrap().clone();

        debug!("Launch backup for share: {}", share.share_path);

        let session_id = self.check_context(request.metadata()).await?;

        let includes = vec_to_str(&share.includes);
        let includes = list_to_globset(&includes)
            .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;
        let excludes = vec_to_str(&share.excludes);
        let excludes = list_to_globset(&excludes)
            .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;

        let mut index = self.get_index_in_context(&session_id, &share.share_path)?;
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
                    if result.is_err() {
                        error!("Failed to send file manifest journal entry");
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

        self.context
            .lock()
            .map_err(|_err| tonic::Status::permission_denied("Can't lock context"))?
            .remove(&session_id);

        self.authentification_service
            .write()
            .await
            .logout(&session_id)
            .map_err(|err| tonic::Status::permission_denied(err.to_string()))?;

        Ok(Response::new(EmptyProto {}))
    }
}
