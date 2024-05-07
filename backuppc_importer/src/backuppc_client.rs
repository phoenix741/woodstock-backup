use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    path::PathBuf,
    sync::Arc,
};

use async_stream::{stream, try_stream};
use async_trait::async_trait;
use backuppc_pool_reader::{
    attribute_file::Search,
    decode_attribut::{FileAttributes, FileType},
    hosts::Hosts,
    view::BackupPC,
};
use futures::{pin_mut, stream, Stream, StreamExt};
use log::{debug, error, info};
use tokio::sync::Mutex;
use woodstock::{
    config::BUFFER_SIZE,
    manifest::IndexManifest,
    refresh_cache_request,
    server::client::Client,
    utils::path::{path_to_vec, vec_to_path},
    AuthenticateReply, ChunkInformation, Empty as ProtoEmpty, EntryType, ExecuteCommandReply,
    FileChunk, FileManifest, FileManifestJournalEntry, FileManifestStat, FileManifestType,
    FileManifestXAttr, LaunchBackupRequest, LogEntry, RefreshCacheRequest,
};

use crate::backuppc_manifest::{FileManifestBackupPC, BPC_DIGEST};

pub struct BackupPCClient {
    hostname: String,
    number: usize,
    index: Arc<Mutex<HashMap<String, IndexManifest<FileManifestBackupPC>>>>,
    view: Arc<Mutex<BackupPC>>,
}

fn file_attribute_to_manifest(path: &[&str], file: FileAttributes) -> FileManifest {
    // Filename is path + filename
    let mut path = path
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    path.push(file.name.clone());
    let path = path.join("/");

    let bpc_digest = BPC_DIGEST.to_string();
    let mut metadata = HashMap::new();
    metadata.insert(bpc_digest, file.bpc_digest.digest);

    FileManifest {
        path: path_to_vec(&PathBuf::from(path)),
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
    pub fn new(backuppc_pool: &str, hostname: &str, number: usize) -> Self {
        let hosts = Hosts::new(backuppc_pool);
        let search = Search::new(backuppc_pool);
        let view = BackupPC::new(backuppc_pool, Box::new(hosts), Box::new(search));

        Self {
            hostname: hostname.to_string(),
            number,
            index: Arc::new(Mutex::new(HashMap::new())),
            view: Arc::new(Mutex::new(view)),
        }
    }

    async fn one_level(
        &self,
        path: Vec<String>,
        to_visit: &mut Vec<Vec<String>>,
    ) -> Result<Vec<FileManifest>, Box<dyn std::error::Error>> {
        debug!("Visit {:?}", path.join("/"));
        let ref_path = &path
            .iter()
            .map(std::string::String::as_str)
            .collect::<Vec<&str>>();
        let mut view = self.view.lock().await;
        let dir = view.list(ref_path)?;
        debug!("Found {} files", dir.len());

        let mut files = Vec::new();
        for entry in dir {
            if entry.type_ == FileType::Dir {
                to_visit.push([path.clone(), vec![entry.name.clone()]].concat());
            }

            files.push(file_attribute_to_manifest(ref_path, entry));
        }

        Ok(files)
    }

    fn get_files(&self, share: &str) -> impl Stream<Item = FileManifest> + '_ {
        let share = share
            .split('/')
            .filter_map(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(s.to_string())
                }
            })
            .collect::<Vec<String>>();
        let backup_number = self.number.to_string();
        let mut path = vec![self.hostname.clone(), backup_number];
        path.extend(share);

        let original_path = PathBuf::from(path.clone().join("/"));

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
    async fn authenticate(
        &mut self,
        _password: &str,
    ) -> Result<AuthenticateReply, Box<dyn std::error::Error>> {
        unimplemented!("No authentication required for BackupPCClient");
    }

    fn stream_log(
        &mut self,
    ) -> impl Stream<Item = Result<LogEntry, Box<dyn std::error::Error + Send + Sync>>> + '_ {
        stream::empty()
    }

    async fn execute_command(
        &mut self,
        _command: &str,
    ) -> Result<ExecuteCommandReply, Box<dyn std::error::Error>> {
        unimplemented!("No command available for import");
    }

    async fn refresh_cache(
        &mut self,
        stream: impl Stream<Item = RefreshCacheRequest> + Send + Sync + 'static,
    ) -> Result<ProtoEmpty, Box<dyn std::error::Error>> {
        debug!("Start refreshing cache");
        pin_mut!(stream);

        let mut index_map: HashMap<String, IndexManifest<FileManifestBackupPC>> = HashMap::new();
        let mut current_path: Option<String> = None;

        while let Some(request) = stream.next().await {
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
                    debug!("Received file manifest for : {:?}", manifest.path());

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
                        return Err(Box::new(Error::new(
                            ErrorKind::InvalidData,
                            "Missing header in refresh_cache request",
                        )));
                    }
                }
                None => {
                    error!("Unknown message in refresh_cache request");
                    return Err(Box::new(Error::new(
                        ErrorKind::InvalidData,
                        "Unknown message",
                    )));
                }
            }
        }

        *self.index.lock().await = index_map;

        Ok(ProtoEmpty {})
    }

    fn download_file_list(
        &mut self,
        request: LaunchBackupRequest,
    ) -> impl Stream<Item = Result<FileManifestJournalEntry, Box<dyn std::error::Error + Send + Sync>>>
           + '_ {
        debug!("Start downloading file list");
        let share = request.share.unwrap();
        let remove_share = share.share_path.clone();

        let stream = self.get_files(share.share_path.clone().as_str());

        let index = self.index.clone();
        let remove_index = self.index.clone();

        let stream = stream.filter_map(move |manifest| {
            debug!(
                "File {:?} {:?} have been changed or added",
                manifest.path(),
                manifest.file_mode()
            );

            let share = share.share_path.clone();
            let index = index.clone();

            async move {
                let mut index = index.lock().await;
                let index = index.get_mut(&share);
                if let Some(index) = index {
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
                        });
                    }
                }

                Some(FileManifestJournalEntry {
                    r#type: EntryType::Add as i32,
                    manifest: Some(manifest),
                })
            }
        });

        let remove_stream = stream!({
            debug!("Start removing files from the index");

            let mut index = remove_index.lock().await;
            let index = index.get_mut(&remove_share);
            if let Some(index) = index {
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
                    };
                }
            }
        });

        stream.chain(remove_stream).map(Ok)
    }

    fn get_chunk(
        &self,
        request: ChunkInformation,
    ) -> impl Stream<Item = Result<FileChunk, Box<dyn std::error::Error + Send + Sync>>> + '_ {
        let number = self.number;
        let hostname = self.hostname.clone();
        let view = self.view.clone();

        try_stream!({
            let backup_number = number.to_string();

            let filename = vec_to_path(&request.filename);
            let filename = filename.to_str().unwrap_or_default();
            let filename = filename
                .split('/')
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();
            let mut path = vec![hostname.as_str(), backup_number.as_str()];
            path.extend(filename);
            debug!(
                "Download chunk {:?} at position {}",
                &path, request.position
            );
            let mut view = view.lock().await;
            let mut file = view
                .read_file(&path)
                .map_err(|e| Error::new(std::io::ErrorKind::Other, format!("{e}")))?;

            let mut buf = vec![0; BUFFER_SIZE];

            // Read and skip until request position
            let mut remaining = request.position;

            while remaining > 0 {
                let to_read = usize::try_from(std::cmp::min(remaining, BUFFER_SIZE as u64))?;
                let read = file.read(&mut buf[..to_read])?;
                remaining -= read as u64;

                if read == 0 {
                    info!("End of file");
                    break;
                }
            }

            // Read the requested chunk
            let request_size = request.size as usize;
            let mut readed = 0;
            while readed < request_size {
                let to_read = std::cmp::min(request_size - readed, BUFFER_SIZE);

                let read = file.read(&mut buf[..to_read])?;
                readed += read;

                yield FileChunk {
                    data: buf[..read].to_vec(),
                };

                if read == 0 {
                    info!("End of file");
                    break;
                }
            }
        })
    }

    async fn close(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
