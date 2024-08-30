use std::{collections::HashMap, ffi::OsString, io::Error, sync::Arc};

use async_stream::{stream, try_stream};
use async_trait::async_trait;
use backuppc_pool_reader::{
    decode_attribut::{FileAttributes, FileType},
    view::BackupPC,
};
use eyre::Result;
use futures::{pin_mut, Stream, StreamExt};
use log::{debug, error};
use sha3::{Digest, Sha3_256};
use tokio::{runtime::Runtime, sync::Mutex};
use woodstock::{
    config::{BUFFER_SIZE, CHUNK_SIZE, CHUNK_SIZE_U64},
    file_chunk,
    manifest::IndexManifest,
    refresh_cache_request,
    server::client::Client,
    utils::path::{osstr_to_vec, path_to_vec, vec_to_path},
    AuthenticateReply, ChunkHashReply, ChunkHashRequest, ChunkInformation, EntryState, EntryType,
    ExecuteCommandReply, FileChunk, FileChunkData, FileChunkEndOfFile, FileChunkFooter,
    FileChunkHeader, FileManifest, FileManifestJournalEntry, FileManifestStat, FileManifestType,
    FileManifestXAttr, RefreshCacheRequest, Share,
};

use crate::backuppc_manifest::{FileManifestBackupPC, BPC_DIGEST};

pub struct BackupPCClient {
    hostname: String,
    number: usize,
    view: Arc<Mutex<BackupPC>>,
}

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
            r#type: match file.type_ {
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
    pub fn new(view: BackupPC, hostname: &str, number: usize) -> Self {
        Self {
            hostname: hostname.to_string(),
            number,
            view: Arc::new(Mutex::new(view)),
        }
    }

    async fn one_level(
        &self,
        path: Vec<Vec<u8>>,
        to_visit: &mut Vec<Vec<Vec<u8>>>,
    ) -> Result<Vec<FileManifest>, Box<dyn std::error::Error>> {
        let path_join = vec_to_path(&path.join(&b'/'));
        debug!("Visit {:?}", path_join.display());

        let ref_path = &path
            .iter()
            .map(std::vec::Vec::as_slice)
            .collect::<Vec<&[u8]>>();

        let mut view = self.view.lock().await;
        let dir = view.list(ref_path)?;
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
}

#[async_trait]
impl Client for BackupPCClient {
    async fn ping(&self) -> Result<bool> {
        Ok(true)
    }

    async fn authenticate(&mut self, _password: &str) -> Result<AuthenticateReply> {
        unimplemented!("No authentication required for BackupPCClient");
    }

    async fn execute_command(&mut self, _command: &str) -> Result<ExecuteCommandReply> {
        unimplemented!("No command available for import");
    }

    fn synchronize_file_list(
        &mut self,
        stream: impl Stream<Item = RefreshCacheRequest> + Send + Sync + 'static,
    ) -> impl Stream<Item = Result<FileManifestJournalEntry>> + '_ {
        debug!("Start refreshing cache");
        pin_mut!(stream);

        let index: Arc<Mutex<IndexManifest<FileManifestBackupPC>>> =
            Arc::new(Mutex::new(IndexManifest::new()));
        let mut share: Option<Share> = None;

        let rt = Runtime::new().unwrap();

        let cache_index = index.clone();
        rt.block_on(async {
            let mut index = cache_index.lock().await;

            while let Some(request) = stream.next().await {
                match request.field {
                    Some(refresh_cache_request::Field::Header(header)) => {
                        debug!("Received header: {:?}", header);
                        if share.is_some() {
                            error!("Header already defined");
                            continue;
                        }

                        share = Some(header);
                    }
                    Some(refresh_cache_request::Field::FileManifest(manifest)) => {
                        debug!("Received file manifest for : {:?}", manifest.path());

                        index.add(FileManifestBackupPC::from(manifest));
                    }
                    None => {
                        error!("Unknown message in refresh_cache request");
                    }
                }
            }
        });

        debug!("Start downloading file list");
        if share.is_none() {
            error!("Share must be defined");
        }

        let share = share.unwrap();

        let share_path = osstr_to_vec(&OsString::from(&share.share_path));

        let stream = self.get_files(&share_path);

        let added_index = index.clone();
        let stream = stream.filter_map(move |manifest| {
            debug!(
                "File {:?} {:?} have been changed or added",
                manifest.path(),
                manifest.file_mode()
            );

            let added_index = added_index.clone();
            async move {
                let mut index = added_index.lock().await;
                index.mark(&manifest.path);

                let entry = index.get_entry(&manifest.path);
                if let Some(entry) = entry {
                    let backuppc_digest = manifest.metadata.get(BPC_DIGEST);

                    if Some(&entry.manifest.backuppc_digest) == backuppc_digest {
                        return None;
                    }

                    return Some(FileManifestJournalEntry {
                        r#type: EntryType::Modify as i32,
                        manifest: Some(manifest),

                        state: EntryState::Metadata as i32,
                        state_messages: Vec::new(),
                    });
                }

                Some(FileManifestJournalEntry {
                    r#type: EntryType::Add as i32,
                    manifest: Some(manifest),

                    state: EntryState::Metadata as i32,
                    state_messages: Vec::new(),
                })
            }
        });

        let remove_index = index.clone();
        let remove_stream = stream!({
            debug!("Start removing files from the index");

            let index = remove_index.lock().await;
            let file_to_remove = index.walk();
            for file in file_to_remove {
                if file.mark_viewed {
                    continue;
                }

                debug!("Detect file {:?} to remove", file.path());
                yield FileManifestJournalEntry {
                    r#type: EntryType::Remove as i32,
                    manifest: Some(FileManifest {
                        path: file.manifest.path.clone(),
                        ..Default::default()
                    }),

                    state: EntryState::Metadata as i32,
                    state_messages: Vec::new(),
                };
            }
        });

        stream.chain(remove_stream).map(Ok)
    }

    async fn get_chunk_hash(&self, request: ChunkHashRequest) -> Result<ChunkHashReply> {
        let number = self.number;
        let number = number.to_string();
        let number = number.as_bytes();
        let hostname = self.hostname.as_bytes();
        let view = self.view.clone();

        let filename = request
            .filename
            .split(|&c| c == b'/')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();

        let mut path = vec![hostname, number];
        path.extend(filename);
        debug!("Calculate chunk for file {:?}", &path);

        let mut view = view.lock().await;
        let mut file = view
            .read_file(&path)
            .map_err(|e| Error::new(std::io::ErrorKind::Other, format!("{e}")))?;

        let mut file_hasher = Sha3_256::new();
        let mut chunk_hasher = Sha3_256::new();
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
            chunks.push(chunk_hash.to_vec());
            chunk_hasher = Sha3_256::new();
        }

        let hash = file_hasher.finalize().to_vec();

        Ok(ChunkHashReply { chunks, hash })
    }

    fn get_chunk(&self, request: ChunkInformation) -> impl Stream<Item = Result<FileChunk>> + '_ {
        let number = self.number;
        let hostname = self.hostname.as_bytes();
        let view = self.view.clone();
        let chunks = request.chunks_id.clone();

        try_stream!({
            let backup_number = &number.to_string().into_bytes();

            let filename = request
                .filename
                .split(|&c| c == b'/')
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();
            let mut path = vec![hostname, backup_number];
            path.extend(filename);

            let mut view = view.lock().await;
            let mut file = view
                .read_file(&path)
                .map_err(|e| Error::new(std::io::ErrorKind::Other, format!("{e}")))?;

            let mut buf = vec![0; BUFFER_SIZE];
            let mut chunk_id = u64::MAX;
            let mut position: u64 = 0;
            let mut send_chunk = false;

            let mut file_hasher = Sha3_256::new();
            let mut chunk_hasher = Sha3_256::new();

            loop {
                let current_chunk = position / CHUNK_SIZE_U64;

                if current_chunk != chunk_id {
                    debug!("Chunk change from {chunk_id} to {current_chunk}");
                    if send_chunk {
                        debug!("Send footer for chunk {path:?}:{chunk_id}");
                        let chunk_hash = chunk_hasher.finalize().to_vec();
                        yield FileChunk {
                            field: Some(file_chunk::Field::Footer(FileChunkFooter { chunk_hash })),
                        };
                        chunk_hasher = Sha3_256::new();
                    }

                    chunk_id = current_chunk;
                    send_chunk = chunks.is_empty() || chunks.contains(&chunk_id);

                    if send_chunk {
                        debug!("Send chunk for chunk {path:?}:{chunk_id}");
                        yield FileChunk {
                            field: Some(file_chunk::Field::Header(FileChunkHeader { chunk_id })),
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
                        field: Some(file_chunk::Field::Data(FileChunkData {
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
                let chunk_hash = chunk_hasher.finalize().to_vec();
                yield FileChunk {
                    field: Some(file_chunk::Field::Footer(FileChunkFooter { chunk_hash })),
                };
            }

            let hash = file_hasher.finalize().to_vec();

            debug!("Send EOF for {path:?}");
            if chunks.is_empty() || usize::try_from(chunk_id).unwrap_or_default() == chunks.len() {
                yield FileChunk {
                    field: Some(file_chunk::Field::Eof(FileChunkEndOfFile { hash })),
                };
            }
        })
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }
}
