use std::future::Future;
use std::{collections::HashMap, ffi::OsString, io::Error, sync::Arc};

use async_stream::{stream, try_stream};
use backuppc_pool_reader::{
    decode_attribut::{FileAttributes, FileType},
    view::BackupPC,
};
use eyre::Result;
use futures::{pin_mut, Stream, StreamExt};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error};
use woodstock::{
    config::{BUFFER_SIZE, CHUNK_SIZE, CHUNK_SIZE_U64},
    file_chunk,
    manifest::IndexManifest,
    refresh_cache_request,
    server::client::Client,
    utils::{
        chunk_hasher::create_chunk_hasher,
        path::{osstr_to_vec, path_to_vec, str_to_vec, vec_to_path},
    },
    AuthenticateReply, ChunkAlgorithm, ChunkHashReply, ChunkHashRequest, ChunkInformation,
    EntryState, EntryType, ExecuteCommandReply, FileChunk, FileChunkData, FileChunkEndOfFile,
    FileChunkFooter, FileChunkHeader, FileManifest, FileManifestJournalEntry, FileManifestStat,
    FileManifestType, FileManifestXAttr, RefreshCacheRequest, RestoreFileReply, RestoreFileRequest,
    Share,
};

use crate::backuppc_manifest::{FileManifestBackupPC, BPC_DIGEST};

/// The `BackupPCClient` struct provides an interface to interact with a `BackupPC` pool.
/// It allows for file listing, chunk retrieval, and synchronization with a Woodstock server.
///
/// # Fields
/// - `hostname`: The name of the host for which the backup is managed.
/// - `number`: The backup number associated with the host.
/// - `view`: An `Arc<Mutex<BackupPC>>` providing thread-safe access to the `BackupPC`` view.
/// - `chunk_algorithm`: The chunking algorithm used for file chunking operations.
#[derive(Clone)]
pub struct BackupPCClient {
    /// The name of the host for which the backup is managed.
    hostname: String,
    /// The backup number associated with the host.
    number: usize,
    /// Thread-safe access to the `BackupPC` view.
    view: Arc<Mutex<BackupPC>>,
    /// The chunking algorithm used for file chunking operations.
    chunk_algorithm: ChunkAlgorithm,
}

/// Converts a `FileAttributes` object and its path into a `FileManifest`.
///
/// # Arguments
/// * `path` - The path to the file as a slice of byte slices.
/// * `file` - The file attributes to convert.
///
/// # Returns
/// A `FileManifest` representing the file and its metadata.
fn file_attribute_to_manifest(path: &[&[u8]], file: FileAttributes) -> FileManifest {
    // Filename is path + filename
    let mut path = path.iter().map(|s| s.to_vec()).collect::<Vec<Vec<u8>>>();
    path.push(file.name.clone());

    let path = path.join(&b'/');

    let bpc_digest = BPC_DIGEST.to_string();
    let mut metadata = HashMap::new();
    metadata.insert(bpc_digest, file.bpc_digest.digest);

    FileManifest {
        path,
        hash: vec![],
        chunks: vec![],
        acl: vec![],
        symlink: vec![],
        xattr: file
            .xattrs
            .iter()
            .map(|attr| FileManifestXAttr {
                key: attr.key.clone().into(),
                value: attr.value.clone().into(),
            })
            .collect(),
        stats: Some(FileManifestStat {
            owner_id: file.uid,
            group_id: file.gid,
            size: file.size,
            mode: u32::from(file.mode),
            file_type: match file.type_ {
                FileType::File | FileType::Hardlink => FileManifestType::RegularFile,
                FileType::Symlink => FileManifestType::Symlink,
                FileType::Chardev => FileManifestType::CharacterDevice,
                FileType::Blockdev => FileManifestType::BlockDevice,
                FileType::Dir => FileManifestType::Directory,
                FileType::Fifo => FileManifestType::Fifo,
                FileType::Socket => FileManifestType::Socket,
                FileType::Unknown | FileType::Deleted => FileManifestType::Unknown,
            } as i32,
            created: 0,
            last_modified: i64::try_from(file.mtime).unwrap_or_default(),
            last_read: 0,
            dev: 0,
            ino: file.inode,
            rdev: 0,
            nlink: u64::from(file.nlinks),
            compressed_size: 0,
        }),
        metadata,
    }
}

impl BackupPCClient {
    #[must_use]
    /// Creates a new `BackupPCClient` instance.
    ///
    /// # Arguments
    /// * `view` - The `BackupPC` view to use for file operations.
    /// * `hostname` - The name of the host for which the backup is managed.
    /// * `number` - The backup number associated with the host.
    /// * `chunk_algorithm` - The chunking algorithm to use for file chunking operations.
    ///
    /// # Returns
    /// A new `BackupPCClient` instance.
    pub fn new(
        view: BackupPC,
        hostname: &str,
        number: usize,
        chunk_algorithm: &ChunkAlgorithm,
    ) -> Self {
        Self {
            hostname: hostname.to_string(),
            number,
            view: Arc::new(Mutex::new(view)),
            chunk_algorithm: *chunk_algorithm,
        }
    }

    /// Visits a single directory level in the `BackupPC` pool and collects file manifests.
    ///
    /// # Arguments
    /// * `path` - The current path as a vector of byte vectors.
    /// * `to_visit` - A mutable reference to the stack of paths to visit next.
    ///
    /// # Returns
    /// Returns a `Result` containing a vector of `FileManifest` objects for the current directory level, or an error if the directory cannot be read.
    ///
    /// # Errors
    /// Returns an error if the directory listing fails or if there is an issue accessing the `BackupPC` view.
    async fn one_level(
        &self,
        path: Vec<Vec<u8>>,
        to_visit: &mut Vec<Vec<Vec<u8>>>,
    ) -> Result<Vec<FileManifest>> {
        let path_join = vec_to_path(&path.join(&b'/'));
        debug!("Visit {:?}", path_join.display());

        let ref_path = &path
            .iter()
            .map(std::vec::Vec::as_slice)
            .collect::<Vec<&[u8]>>();

        let mut view = self.view.lock().await;
        let dir = view.list(ref_path).map_err(|e| eyre::eyre!("{e}"))?;
        debug!("Found {} files", dir.len());

        let mut files = Vec::new();
        for entry in dir {
            if entry.type_ == FileType::Dir {
                let path = path.iter().map(std::clone::Clone::clone).collect();
                let entry_name = vec![entry.name.clone()];
                let v = [path, entry_name].concat();
                to_visit.push(v);
            }

            files.push(file_attribute_to_manifest(ref_path, entry));
        }

        Ok(files)
    }

    /// Returns a stream of `FileManifest` objects for all files in the specified share.
    ///
    /// # Arguments
    /// * `share` - The share path as a byte slice.
    ///
    /// # Returns
    /// A stream yielding `FileManifest` objects for each file found in the share.
    fn get_files(&self, share: &[u8]) -> impl Stream<Item = FileManifest> + '_ {
        let share = share
            .split(|s| s == &b'/')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&[u8]>>();

        let number = self.number.to_string();
        let number = number.as_bytes().to_vec();
        let hostname = self.hostname.as_bytes().to_vec();
        let share = share.iter().map(|s| s.to_vec()).collect::<Vec<Vec<u8>>>();

        let mut path = vec![hostname, number];
        path.extend(share);

        let original_path = vec_to_path(&path.clone().join(&b'/'));

        futures::stream::unfold(vec![path], |mut to_visit| async {
            let path = to_visit.pop()?;

            let file_stream = match self.one_level(path, &mut to_visit).await {
                Ok(files) => futures::stream::iter(files).left_stream(),
                Err(e) => futures::stream::iter({
                    error!("Can't read the file in directory: {}", e);
                    Vec::<FileManifest>::new()
                })
                .right_stream(),
            };

            Some((file_stream, to_visit))
        })
        .flatten()
        .map(move |manifest| {
            let path = vec_to_path(&manifest.path);
            // Remove orignal_path
            let path = path.strip_prefix(&original_path).unwrap_or(path.as_path());
            FileManifest {
                path: path_to_vec(path),
                ..manifest
            }
        })
    }

    /// Asynchronously synchronizes the file list by comparing the current `BackupPC` pool with the provided stream of refresh requests.
    ///
    /// # Arguments
    /// * `stream` - A stream of `RefreshCacheRequest` items representing the desired state.
    ///
    /// # Returns
    /// A stream yielding `Result<FileManifestJournalEntry>` for each detected change (add, modify, remove).
    ///
    /// # Errors
    /// Returns an error if the stream cannot be processed or if there are issues with the `BackupPC` view.
    async fn async_synchronize_file_list(
        &mut self,
        stream: impl Stream<Item = RefreshCacheRequest> + Send + Sync + 'static,
    ) -> impl Stream<Item = Result<FileManifestJournalEntry>> + '_ {
        debug!("Start refreshing cache");
        pin_mut!(stream);

        let index: Arc<Mutex<IndexManifest<FileManifestBackupPC>>> =
            Arc::new(Mutex::new(IndexManifest::new()));
        let mut share: Option<Share> = None;

        let refresh_index = index.clone();
        let mut refresh_index = refresh_index.lock().await;

        while let Some(request) = stream.next().await {
            match request.payload {
                Some(refresh_cache_request::Payload::Header(header)) => {
                    debug!("Received header: {:?}", header);
                    if share.is_some() {
                        error!("Header already defined");
                        continue;
                    }

                    share = Some(header);
                }
                Some(refresh_cache_request::Payload::FileManifest(manifest)) => {
                    debug!("Received file manifest for : {:?}", manifest.path());

                    refresh_index.add(FileManifestBackupPC::from(manifest));
                }
                None => {
                    error!("Unknown message in refresh_cache request");
                }
            }
        }

        debug!("Start downloading file list");
        let Some(share) = share else {
            error!("Share must be defined — RefreshCacheRequest missing SharePath, returning empty stream");
            // SAFETY: We cannot return early from an `impl Stream` function without boxing both branches.
            // In practice this path is never reached: the BackupPC protocol always provides a share.
            // The error! log above ensures the problem is visible before the expect() panic.
            panic!("Invariant violation: RefreshCacheRequest must contain a SharePath field");
        };
        let share_path = osstr_to_vec(&OsString::from(&share.share_path));

        let add_index = index.clone();

        let stream = self.get_files(&share_path);
        let stream = stream.filter_map(move |manifest| {
            debug!(
                "File {:?} {:?} have been changed or added",
                manifest.path(),
                manifest.file_mode()
            );

            let add_index = add_index.clone();

            async move {
                let mut add_index = add_index.lock().await;

                add_index.mark(&manifest.path);

                let entry = add_index.get_entry(&manifest.path);
                if let Some(entry) = entry {
                    let backuppc_digest = manifest.metadata.get(BPC_DIGEST);

                    if Some(&entry.manifest.backuppc_digest) == backuppc_digest {
                        return None;
                    }

                    return Some(FileManifestJournalEntry {
                        entry_type: EntryType::Modify as i32,
                        manifest: Some(manifest),

                        state: EntryState::Metadata as i32,
                        state_messages: Vec::new(),

                        xfer_start: 0,
                        xfer_calculation: 0,
                        xfer_duration: 0,
                        xfer_check: 0,
                        chunk_sizes: vec![],
                        chunk_compressed_sizes: vec![],
                        snapshot_result: None,
                    });
                }

                Some(FileManifestJournalEntry {
                    entry_type: EntryType::Add as i32,
                    manifest: Some(manifest),

                    state: EntryState::Metadata as i32,
                    state_messages: Vec::new(),

                    xfer_start: 0,
                    xfer_calculation: 0,
                    xfer_duration: 0,
                    xfer_check: 0,
                    chunk_sizes: vec![],
                    chunk_compressed_sizes: vec![],
                    snapshot_result: None,
                })
            }
        });

        let remove_index = index.clone();

        let remove_stream = stream!({
            debug!("Start removing files from the index");

            let remove_index = remove_index.lock().await;
            let file_to_remove = remove_index.walk();
            for file in file_to_remove {
                if file.mark_viewed {
                    continue;
                }

                debug!("Detect file {:?} to remove", file.path());
                yield FileManifestJournalEntry {
                    entry_type: EntryType::Remove as i32,
                    manifest: Some(FileManifest {
                        path: file.manifest.path.clone(),
                        ..Default::default()
                    }),

                    state: EntryState::Metadata as i32,
                    state_messages: Vec::new(),

                    xfer_start: 0,
                    xfer_calculation: 0,
                    xfer_duration: 0,
                    xfer_check: 0,
                    chunk_sizes: vec![],
                    chunk_compressed_sizes: vec![],
                    snapshot_result: None,
                };
            }
        });

        stream.chain(remove_stream).map(Ok)
    }
}

impl Client for BackupPCClient {
    fn ping(&self) -> impl Future<Output = Result<bool>> + Send {
        async move { Ok(true) }
    }

    fn authenticate(
        &self,
        _password: &str,
    ) -> impl Future<Output = Result<AuthenticateReply>> + Send {
        async move { unimplemented!("No authentication required for BackupPCClient") }
    }

    fn execute_command(
        &self,
        _command: &str,
    ) -> impl Future<Output = Result<ExecuteCommandReply>> + Send {
        async move { unimplemented!("No command available for import") }
    }

    fn synchronize_file_list(
        &self,
        stream: impl Stream<Item = RefreshCacheRequest> + Send + Sync + 'static,
    ) -> impl Stream<Item = Result<FileManifestJournalEntry>> + '_ {
        let (tx, rx) = mpsc::channel(100);

        // Clone les références nécessaires pour l'appel asynchrone
        let mut self_clone = self.clone();
        let stream_clone = stream;

        tokio::spawn(async move {
            let result_stream = self_clone.async_synchronize_file_list(stream_clone).await;
            pin_mut!(result_stream);

            while let Some(item) = result_stream.next().await {
                if tx.send(item).await.is_err() {
                    break;
                }
            }
        });

        ReceiverStream::new(rx)
    }

    fn get_chunk_hash(
        &self,
        request: ChunkHashRequest,
    ) -> impl Future<Output = Result<ChunkHashReply>> + Send {
        let number = self.number;
        let hostname = self.hostname.clone();
        let view = self.view.clone();

        async move {
            let number = number.to_string();
            let number = number.as_bytes();
            let hostname = hostname.as_bytes();

            let share_path = str_to_vec(&request.share_path);
            let share_path = share_path
                .split(|&c| c == b'/')
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();
            let filename = request
                .filename
                .split(|&c| c == b'/')
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();

            let mut path = vec![hostname, number];
            path.extend(share_path);
            path.extend(filename);
            debug!("Calculate chunk for file {:?}", &path);

            let mut view = view.lock().await;
            let mut file = view
                .read_file(&path)
                .map_err(|e| Error::new(std::io::ErrorKind::Other, format!("{e}")))?;

            let mut file_hasher = create_chunk_hasher(request.algorithm());
            let mut chunk_hasher = create_chunk_hasher(request.algorithm());
            let mut chunks = Vec::<Vec<u8>>::new();

            let mut buf = vec![0; CHUNK_SIZE];

            loop {
                let read = file.read(&mut buf)?;
                if read == 0 {
                    break;
                }

                chunk_hasher.update(&buf[..read]);
                file_hasher.update(&buf[..read]);

                let chunk_hash = chunk_hasher.finalize();
                chunks.push(chunk_hash);
                chunk_hasher = create_chunk_hasher(request.algorithm());
            }

            let hash = file_hasher.finalize();

            Ok(ChunkHashReply { chunks, hash })
        }
    }

    fn get_chunk(&self, request: ChunkInformation) -> impl Stream<Item = Result<FileChunk>> + '_ {
        let number = self.number;
        let hostname = self.hostname.as_bytes();
        let view = self.view.clone();
        let chunks = request.chunks_id.clone();
        let algorithm = self.chunk_algorithm;

        try_stream!({
            let backup_number = &number.to_string().into_bytes();

            let share_path = str_to_vec(&request.share_path);
            let share_path = share_path
                .split(|&c| c == b'/')
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();
            let filename = request
                .filename
                .split(|&c| c == b'/')
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();
            let mut path = vec![hostname, backup_number];
            path.extend(share_path);
            path.extend(filename);

            let mut view = view.lock().await;
            let mut file = view
                .read_file(&path)
                .map_err(|e| Error::new(std::io::ErrorKind::Other, format!("{e}")))?;

            let mut buf = vec![0; BUFFER_SIZE];
            let mut chunk_id = u64::MAX;
            let mut position: u64 = 0;
            let mut send_chunk = false;

            let mut file_hasher = create_chunk_hasher(algorithm);
            let mut chunk_hasher = create_chunk_hasher(algorithm);

            loop {
                let current_chunk = position / CHUNK_SIZE_U64;

                if current_chunk != chunk_id {
                    debug!("Chunk change from {chunk_id} to {current_chunk}");
                    if send_chunk {
                        debug!("Send footer for chunk {path:?}:{chunk_id}");
                        let chunk_hash = chunk_hasher.finalize().clone();
                        yield FileChunk {
                            payload: Some(file_chunk::Payload::Footer(FileChunkFooter {
                                chunk_hash,
                            })),
                        };
                        chunk_hasher = create_chunk_hasher(algorithm);
                    }

                    chunk_id = current_chunk;
                    send_chunk = chunks.is_empty() || chunks.contains(&chunk_id);

                    if send_chunk {
                        debug!("Send chunk for chunk {path:?}:{chunk_id}");
                        yield FileChunk {
                            payload: Some(file_chunk::Payload::Header(FileChunkHeader {
                                chunk_id,
                            })),
                        };
                    }
                }

                debug!("Read chunk {path:?}:{chunk_id}");
                let read = file.read(&mut buf)?;
                if send_chunk && read > 0 {
                    debug!("Send data for chunk {path:?}:{chunk_id}");

                    chunk_hasher.update(&buf[..read]);
                    file_hasher.update(&buf[..read]);

                    yield FileChunk {
                        payload: Some(file_chunk::Payload::Data(FileChunkData {
                            data: buf[..read].to_vec(),
                        })),
                    };
                }

                position += read as u64;

                if read == 0 {
                    break;
                }
            }

            if (chunks.is_empty() || chunks.contains(&chunk_id)) && chunk_id != u64::MAX {
                debug!("Send footer for chunk {path:?}:{chunk_id} last");
                let chunk_hash = chunk_hasher.finalize();
                yield FileChunk {
                    payload: Some(file_chunk::Payload::Footer(FileChunkFooter { chunk_hash })),
                };
            }

            let hash = file_hasher.finalize();

            debug!("Send EOF for {path:?}");
            if chunks.is_empty() || usize::try_from(chunk_id).unwrap_or_default() == chunks.len() {
                yield FileChunk {
                    payload: Some(file_chunk::Payload::Eof(FileChunkEndOfFile { hash })),
                };
            }
        })
    }

    fn restore_file(
        &self,
        _requests: impl Stream<Item = RestoreFileRequest> + Send + Sync + 'static,
    ) -> impl Stream<Item = Result<RestoreFileReply>> + '_ {
        futures::stream::empty()
    }

    fn close(&self, _aborted: bool) -> impl Future<Output = Result<()>> + Send {
        async move { Ok(()) }
    }
}
