use std::{
    path::Path,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use async_stream::stream;
use futures::{pin_mut, Future, StreamExt};
use log::error;
use tokio::sync::Mutex;

use crate::{
    config::{Backup, Backups, Configuration, ConfigurationPath},
    manifest::PathManifest,
    pool::{PoolChunkInformation, PoolChunkWrapper, Refcnt},
    proto::ProtobufWriter,
    refresh_cache_request,
    scanner::{CHUNK_SIZE_U64, SHA256_EMPTYSTRING},
    utils::path::path_to_vec,
    ChunkInformation, EntryType, ExecuteCommandReply, FileManifest, FileManifestJournalEntry,
    LaunchBackupRequest, PoolRefCount, RefreshCacheHeader, RefreshCacheRequest, Share,
};

use super::{grpc_client::BackupGrpcClient, progression::BackupProgression};

pub struct BackupClient {
    client: BackupGrpcClient,

    hostname: String,
    current_backup_id: usize,

    progression: Arc<Mutex<BackupProgression>>,
    refcnt: Arc<Mutex<Refcnt>>,
}

impl BackupClient {
    pub async fn new(
        hostname: &str,
        ip_address: &str,
        backup_number: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let configuration = Configuration::default();
        let backups = Backups::new(&configuration.path);
        let destination_directory =
            backups.get_backup_destination_directory(hostname, backup_number);

        Ok(BackupClient {
            client: BackupGrpcClient::new(hostname, ip_address).await?,
            hostname: hostname.to_string(),
            current_backup_id: backup_number,
            progression: Arc::new(Mutex::new(BackupProgression::default())),
            refcnt: Arc::new(Mutex::new(Refcnt::new(&destination_directory))),
        })
    }

    pub async fn progress(&self) -> BackupProgression {
        self.progression.lock().await.clone()
    }

    pub fn get_client(&self) -> BackupGrpcClient {
        self.client.clone()
    }

    async fn to_backup(&self, is_complete: bool) -> Backup {
        let now = SystemTime::now();
        let progression = self.progression.lock().await.clone();

        Backup {
            number: self.current_backup_id,
            completed: is_complete,

            start_date: progression
                .start_date
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            end_date: if is_complete {
                Some(now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())
            } else {
                None
            },

            file_count: progression.file_count,
            new_file_count: progression.new_file_count,
            existing_file_count: progression
                .file_count
                .saturating_sub(progression.new_file_count),

            file_size: progression.file_size,
            new_file_size: progression.new_file_size,
            existing_file_size: progression
                .file_size
                .saturating_sub(progression.new_file_size),

            compressed_file_size: progression.compressed_file_size,
            new_compressed_file_size: progression.new_compressed_file_size,
            existing_compressed_file_size: progression
                .compressed_file_size
                .saturating_sub(progression.new_compressed_file_size),

            speed: progression.speed(),
        }
    }

    pub async fn authenticate(&mut self, password: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.client.authenticate(password).await?;

        Ok(())
    }

    pub async fn init_backup_directory(&self) -> Result<(), Box<dyn std::error::Error>> {
        let backups = Backups::new(&ConfigurationPath::default());
        let previous_backup = backups
            .get_previous_backup(&self.hostname, self.current_backup_id)
            .map(|b| b.number);

        backups
            .clone_backup(&self.hostname, previous_backup, self.current_backup_id)
            .await?;

        // Load Reference count
        self.refcnt.lock().await.load_refcnt(true).await;

        self.save_backup(false).await?;

        Ok(())
    }

    pub async fn execute_command(
        &mut self,
        command: &str,
    ) -> Result<ExecuteCommandReply, Box<dyn std::error::Error>> {
        let result = self.client.execute_command(command).await?;

        self.save_backup(false).await?;

        Ok(result)
    }

    pub async fn upload_file_list(
        &mut self,
        shares: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let hostname = self.hostname.clone();
        let current_backup_id = self.current_backup_id;

        let refresh_cache_stream = stream!({
            let configuration = Configuration::default();
            let backups = Backups::new(&configuration.path);
            for share in shares {
                let manifest = backups.get_manifest(&hostname, current_backup_id, &share);
                let header = RefreshCacheRequest {
                    field: Some(refresh_cache_request::Field::Header(RefreshCacheHeader {
                        share_path: share,
                    })),
                };

                yield header;

                let entries = manifest.read_manifest_entries();
                pin_mut!(entries);

                while let Some(entry) = entries.next().await {
                    let request = RefreshCacheRequest {
                        field: Some(refresh_cache_request::Field::FileManifest(entry)),
                    };

                    yield request;
                }
            }
        });

        self.client.refresh_cache(refresh_cache_stream).await?;

        self.save_backup(false).await?;

        Ok(())
    }

    pub async fn download_file_list(
        &mut self,
        share: &Share,
        callback: &impl Fn(&BackupProgression),
    ) -> Result<(), Box<dyn std::error::Error>> {
        let hostname = self.hostname.clone();
        let current_backup_id = self.current_backup_id;

        let configuration = Configuration::default();
        let backups = Backups::new(&configuration.path);
        let manifest = backups.get_manifest(&hostname, current_backup_id, &share.share_path);

        let response = self.client.download_file_list(LaunchBackupRequest {
            share: Some(share.clone()),
        });

        let response = response.filter_map(|entry| async {
            match entry {
                Ok(entry) => {
                    let file_size = entry
                        .manifest
                        .as_ref()
                        .map(FileManifest::size)
                        .unwrap_or_default();

                    let mut progression = self.progression.lock().await;
                    progression.progress_max += file_size;

                    callback(&progression);

                    Some(entry.clone())
                }
                Err(e) => {
                    error!("Error while downloading file list: {}", e);
                    None
                }
            }
        });

        let result = manifest.save_filelist_entries(response).await;

        self.progression.lock().await.start_transfer_date = Some(SystemTime::now());

        self.save_backup(false).await?;

        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }

    async fn copy_chunk(
        &self,
        share_path: &str,
        file_manifest: &FileManifest,
        chunk_number: usize,
    ) -> Result<PoolChunkInformation, Box<dyn std::error::Error>> {
        let sha256: Option<&Vec<u8>> = if chunk_number < file_manifest.chunks.len() {
            Some(&file_manifest.chunks[chunk_number])
        } else {
            None
        };

        let position = CHUNK_SIZE_U64 * u64::try_from(chunk_number)?;
        let rest_size = file_manifest.size() - position;
        let size = u32::try_from(if CHUNK_SIZE_U64 > rest_size {
            rest_size
        } else {
            CHUNK_SIZE_U64
        })?;

        if size == 0 {
            return Ok(PoolChunkInformation {
                sha256: SHA256_EMPTYSTRING.to_vec(),
                size: 0,
                compressed_size: 0,
            });
        }

        let filename = Path::new(share_path).join(file_manifest.path());
        let filename = path_to_vec(filename.as_path());

        let configuration = Configuration::default();
        let mut wrapper = PoolChunkWrapper::new(&configuration.path.pool_path, sha256);
        if wrapper.exists() {
            // Return the information on existing chunk
            let old_chunk = wrapper.chunk_information().await?;

            if let Some(hash) = sha256 {
                if hash.ne(&old_chunk.sha256) {
                    error!(
                        "{:?}:{} Chunk {} is not the same that {}",
                        file_manifest.path(),
                        chunk_number,
                        hex::encode(hash),
                        hex::encode(&old_chunk.sha256)
                    );
                }
            }

            return Ok(old_chunk);
        }

        let chunk_information = ChunkInformation {
            size,
            position,
            filename: filename.clone(),
        };

        // Download and write chunk
        let readable = self.client.get_chunk(chunk_information);
        let readable = readable.map(|chunk| chunk.map(|chunk| chunk.data));
        let chunk = wrapper.write(readable, &filename).await?;

        Ok(chunk)
    }

    async fn download_manifest_chunk<Fut, F>(
        &self,
        share_path: &str,
        mut file_manifest: FileManifest,
        callback: &F,
    ) -> Result<FileManifest, Box<dyn std::error::Error>>
    where
        F: Fn(PoolChunkInformation) -> Fut,
        Fut: Future<Output = ()>,
    {
        let chunk_count = file_manifest.chunk_count();

        let mut compressed_size: u64 = 0;
        let mut size: u64 = 0;
        let mut chunks = Vec::with_capacity(chunk_count);

        for chunk_number in 0..chunk_count {
            let chunk = self
                .copy_chunk(share_path, &file_manifest, chunk_number)
                .await?;

            callback(chunk.clone()).await;

            self.refcnt.lock().await.apply(
                &PoolRefCount {
                    sha256: chunk.sha256.clone(),
                    size: chunk.size,
                    compressed_size: chunk.compressed_size,
                    ref_count: 0,
                },
                &crate::pool::RefcntApplySens::Increase,
            );

            compressed_size += chunk.compressed_size;
            size += chunk.size;
            assert!(chunks.len() == chunk_number);
            chunks.push(chunk.sha256);
        }

        let path = file_manifest.path();
        let mut stats = file_manifest.stats.unwrap_or_default();
        stats.compressed_size = compressed_size;
        if stats.size != size {
            error!(
                "The manifest of dile {:?}, size ({}) is not equal to the sum of the chunks ({})",
                &path, stats.size, size
            );
        }
        stats.size = size;
        file_manifest.stats = Some(stats);
        file_manifest.chunks = chunks;

        Ok(file_manifest)
    }

    pub async fn create_backup(
        &self,
        share_path: &str,
        callback: &impl Fn(&BackupProgression),
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut error_count = 0;
        let mut abort: Option<Box<dyn std::error::Error>> = None;

        let configuration = Configuration::default();
        let backups = Backups::new(&configuration.path);
        let manifest = backups.get_manifest(&self.hostname, self.current_backup_id, share_path);

        // Start by reading file list
        let mut journal_writer =
            ProtobufWriter::<FileManifestJournalEntry>::new(&manifest.journal_path, true, true)
                .await?;
        let file_list = manifest.read_filelist_entries();
        pin_mut!(file_list);

        while let Some(mut file_manifest_journal_entry) = file_list.next().await {
            let path = file_manifest_journal_entry.path();
            let is_remove = file_manifest_journal_entry.r#type() == EntryType::Remove;
            let is_special_file = file_manifest_journal_entry.is_special_file();
            if !is_remove && !is_special_file {
                if let Some(file_manifest) = file_manifest_journal_entry.manifest {
                    let file_manifest = self
                        .download_manifest_chunk(share_path, file_manifest, &|chunk| async move {
                            let mut progression = self.progression.lock().await;
                            progression.progress_current += chunk.size;

                            callback(&progression);
                        })
                        .await;

                    match file_manifest {
                        Ok(file_manifest) => {
                            file_manifest_journal_entry.manifest = Some(file_manifest);

                            match file_manifest_journal_entry.r#type() {
                                EntryType::Add => {
                                    let size = file_manifest_journal_entry.size();
                                    let compressed_size =
                                        file_manifest_journal_entry.compressed_size();

                                    let mut progression = self.progression.lock().await;
                                    progression.new_file_count += 1;
                                    progression.new_file_size += size;
                                    progression.new_compressed_file_size += compressed_size;
                                }
                                EntryType::Modify => {
                                    let size = file_manifest_journal_entry.size();
                                    let compressed_size =
                                        file_manifest_journal_entry.compressed_size();

                                    let mut progression = self.progression.lock().await;
                                    progression.file_size += size;
                                    progression.compressed_file_size += compressed_size;
                                }
                                EntryType::Remove => {}
                            }
                        }
                        Err(e) => {
                            error!("Can't download chunk for {:?}: {}", path, e);
                            let tonic_status = e.downcast_ref::<tonic::Status>();
                            if let Some(tonic_status) = tonic_status {
                                // Si l'erreur est de type tonic (pas connecté, erreur d'authentification, ...) alors on abort
                                match tonic_status.code() {
                                    tonic::Code::Unavailable
                                    | tonic::Code::Unauthenticated
                                    | tonic::Code::PermissionDenied => {
                                        abort = Some(e);
                                        break;
                                    }
                                    _ => {}
                                }
                            }

                            // FIXME: What to do in case of error (delete file, vanished file)
                            error_count += 1;
                            continue;
                        }
                    };
                }
            }

            let write_result = journal_writer.write(&file_manifest_journal_entry).await;
            if let Err(err) = write_result {
                journal_writer.cancel().await?;
                error!("Can't write journal entry for {:?}: {}", path, err);
                return Err(Box::new(err));
            }
        }

        journal_writer.flush().await?;

        self.progression.lock().await.error_count = error_count;

        if let Some(e) = abort {
            Err(e)
        } else {
            Ok(())
        }
    }

    pub async fn close(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.progression.lock().await.end_transfer_date = Some(SystemTime::now());

        // FIXME: Manage abort

        self.client.close().await?;

        self.save_backup(false).await?;

        Ok(())
    }

    pub async fn compact(&self, share_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let configuration = Configuration::default();
        let backups = Backups::new(&configuration.path);
        let manifest = backups.get_manifest(&self.hostname, self.current_backup_id, share_path);

        manifest
            .compact(&|manifest| async {
                let mut progression = self.progression.lock().await;
                progression.file_count += 1;
                progression.file_size += manifest.size();
                progression.compressed_file_size += manifest.compressed_size();

                let mut refcnt = self.refcnt.lock().await;
                for sha256 in &manifest.chunks {
                    refcnt.apply(
                        &PoolRefCount {
                            sha256: sha256.clone(),
                            ref_count: 1,
                            size: 0,
                            compressed_size: 0,
                        },
                        &crate::pool::RefcntApplySens::Increase,
                    );
                }

                Some(manifest)
            })
            .await?;

        backups
            .add_backup_share_path(&self.hostname, self.current_backup_id, share_path)
            .await?;

        self.save_backup(false).await?;

        Ok(())
    }

    pub async fn count_references(&self) -> Result<(), Box<dyn std::error::Error>> {
        let configuration = Configuration::default();
        let backups = Backups::new(&configuration.path);

        let mut refcnt = self.refcnt.lock().await;
        refcnt.finish().await?;
        refcnt.save_refcnt().await?;

        let host_refcnt_file = backups.get_host_path(&self.hostname);
        Refcnt::apply_all_from(
            &host_refcnt_file,
            &refcnt,
            &crate::pool::RefcntApplySens::Increase,
        )
        .await?;

        self.save_backup(false).await?;

        Ok(())
    }

    pub async fn save_backup(&self, is_complete: bool) -> Result<(), Box<dyn std::error::Error>> {
        let backups = Backups::new(&ConfigurationPath::default());
        let backup = self.to_backup(is_complete).await;

        backups
            .add_or_replace_backup(&self.hostname, &backup)
            .await?;

        Ok(())
    }
}
