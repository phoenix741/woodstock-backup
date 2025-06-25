//! LocalClient: Woodstock Client implementation for local filesystem backup
//!
//! This client implements the `Client` trait and provides access to the local filesystem
//! for backup operations, using the configuration from HostConfiguration.

use std::path::Path;
use std::time::SystemTime;

use eyre::{eyre, Result};
use futures::pin_mut;
use futures::stream;
use futures::stream::StreamExt; // still needed for next()
use futures::Stream;
use log::debug;
use log::error;
use log::trace;
use tonic::async_trait;
use woodstock::client::config::ClientConfig;
use woodstock::client::exexcute_command::execute_command;
use woodstock::client::scanner::calculate_chunk_hash_future;
use woodstock::client::scanner::read_chunk;
use woodstock::client::scanner::{get_files_with_hash, CreateManifestOptions};
use woodstock::manifest::FileManifestLight;
use woodstock::manifest::IndexManifest;
use woodstock::refresh_cache_request;
use woodstock::server::client::Client;
use woodstock::utils::path::list_to_globset;
use woodstock::utils::path::vec_to_str;
use woodstock::EntryState;
use woodstock::EntryType;
use woodstock::FileManifest;
use woodstock::Share;
use woodstock::{
    AuthenticateReply, ChunkHashReply, ChunkHashRequest, ChunkInformation, ExecuteCommandReply,
    FileChunk, FileManifestJournalEntry, RefreshCacheRequest, RestoreFileReply, RestoreFileRequest,
};

use crate::standalone_config::StandaloneClientConfig;

pub struct LocalClient {
    /// Options for creating file manifests.
    create_manifest_options: CreateManifestOptions,
}

impl LocalClient {
    pub fn new(config: &StandaloneClientConfig) -> Self {
        Self {
            create_manifest_options: CreateManifestOptions {
                with_acl: config.acl,
                with_xattr: config.xattr,
            },
        }
    }
}

#[async_trait]
impl Client for LocalClient {
    async fn ping(&self) -> Result<bool> {
        Ok(true)
    }

    async fn authenticate(&self, _password: &str) -> Result<AuthenticateReply> {
        // No authentication needed for local backup
        Ok(AuthenticateReply {
            agent_version: ClientConfig::version(),
            session_id: String::new(),
        })
    }

    async fn execute_command(&self, command: &str) -> Result<ExecuteCommandReply> {
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

                Ok(reply)
            }
            Err(e) => {
                error!("Failed to execute command: {:?}", e);
                Ok(ExecuteCommandReply {
                    code: -1,
                    stdout: String::new(),
                    stderr: e.to_string(),
                })
            }
        }
    }

    fn synchronize_file_list(
        &self,
        stream: impl Stream<Item = RefreshCacheRequest> + Send + Sync + 'static,
    ) -> impl Stream<Item = Result<FileManifestJournalEntry>> + '_ {
        async_stream::stream!({
            let mut index: IndexManifest<FileManifestLight> = IndexManifest::new();
            let mut share: Option<Share> = None;

            pin_mut!(stream);
            while let Some(request) = stream.next().await {
                match request.field {
                    Some(refresh_cache_request::Field::Header(header)) => {
                        debug!("Received header: {:?}", header);
                        if share.is_some() {
                            error!("Header already defined");
                            yield Err(eyre!("Header already defined"));
                            break;
                        }

                        share = Some(header);
                    }
                    Some(refresh_cache_request::Field::FileManifest(manifest)) => {
                        trace!("Received manifest: {:?}", manifest);
                        index.add(FileManifestLight::from(manifest));
                    }
                    None => {
                        error!("Unknown message in refresh_cache request");
                        yield Err(eyre!("Unknown message"));
                        break;
                    }
                }
            }

            if share.is_none() {
                error!("Share must be defined");
                yield Err(eyre!("Share must be defined"));
                return;
            }
            let share = share.as_ref().unwrap().clone();

            debug!("Launch backup for share: {}", share.share_path);

            let includes = vec_to_str(&share.includes);
            let includes = list_to_globset(&includes)?;
            let excludes = vec_to_str(&share.excludes);
            let excludes = list_to_globset(&excludes)?;

            let create_manifest_options = self.create_manifest_options.clone();

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
                    yield Ok(file);
                }
            }

            // Remove file
            let file_to_remove = index.walk();
            for file in file_to_remove {
                if file.mark_viewed {
                    continue;
                }

                let xfer_start = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let entry = FileManifestJournalEntry {
                    r#type: EntryType::Remove as i32,
                    manifest: Some(FileManifest {
                        path: file.manifest.path.clone(),
                        ..Default::default()
                    }),

                    state: EntryState::Metadata as i32,
                    state_messages: Vec::new(),

                    xfer_start,
                    xfer_calculation: 0,
                    xfer_duration: 0,
                    xfer_check: 0,
                };
                yield Ok(entry);
            }
        })
    }

    async fn get_chunk_hash(&self, request: ChunkHashRequest) -> Result<ChunkHashReply> {
        let reply = calculate_chunk_hash_future(&request).await;
        Ok(reply)
    }

    fn get_chunk(
        &self,
        request: ChunkInformation,
    ) -> impl futures::Stream<Item = Result<FileChunk>> + '_ {
        let chunks = read_chunk(&request);
        let stream = chunks
            .map(|chunk| match chunk {
                Ok(data) => Ok(data),
                Err(e) => Err(eyre!(e)),
            })
            .boxed();
        stream
    }

    fn restore_file(
        &self,
        _requests: impl futures::Stream<Item = RestoreFileRequest> + Send + Sync + 'static,
    ) -> impl futures::Stream<Item = Result<RestoreFileReply>> + '_ {
        // Not needed for backup-only mode
        stream::empty()
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }
}
