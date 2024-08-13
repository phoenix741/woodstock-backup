use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::Path,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use async_stream::stream;
use eyre::{eyre, Result};
use futures::{pin_mut, Future, StreamExt};
use log::{debug, error, info};
use tokio::sync::Mutex;

use crate::{
    config::{Backup, Backups, Context, SHA256_EMPTYSTRING},
    file_chunk::{self, Field},
    manifest::PathManifest,
    pool::{PoolChunkInformation, PoolChunkWrapper, Refcnt},
    proto::ProtobufWriter,
    refresh_cache_request,
    utils::path::path_to_vec,
    ChunkHashRequest, ChunkInformation, EntryType, ExecuteCommandReply, FileManifest,
    FileManifestJournalEntry, LaunchBackupRequest, PoolRefCount, RefreshCacheHeader,
    RefreshCacheRequest, Share,
};

use super::{client::Client, progression::BackupProgression};

pub struct BackupClient<Clt: Client> {
    client: Clt,

    hostname: String,
    current_backup_id: usize,
    fake_date: Option<SystemTime>,

    progress_max: HashMap<String, u64>,
    progression: Arc<Mutex<BackupProgression>>,
    refcnt: Arc<Mutex<Refcnt>>,

    context: Context,
}

impl<Clt: Client> BackupClient<Clt> {
    pub fn new(client: Clt, hostname: &str, backup_number: usize, ctxt: &Context) -> Self {
        let backups = Backups::new(ctxt);
        let destination_directory =
            backups.get_backup_destination_directory(hostname, backup_number);

        info!(
            "Initialize backup client for {hostname}/{backup_number} in {destination_directory:?}"
        );

        BackupClient {
            client,
            hostname: hostname.to_string(),
            current_backup_id: backup_number,
            progress_max: HashMap::new(),
            progression: Arc::new(Mutex::new(BackupProgression::default())),
            refcnt: Arc::new(Mutex::new(Refcnt::new(&destination_directory))),
            context: ctxt.clone(),
            fake_date: None,
        }
    }

    pub fn set_fake_date(&mut self, fake_date: Option<SystemTime>) {
        self.fake_date = fake_date;
    }

    pub fn get_fake_date(&self) -> SystemTime {
        self.fake_date.unwrap_or_else(SystemTime::now)
    }

    pub async fn progress(&self) -> BackupProgression {
        self.progression.lock().await.clone()
    }

    async fn to_backup(&self, is_complete: bool) -> Backup {
        let now = SystemTime::now();
        let progression = self.progression.lock().await.clone();

        Backup {
            number: self.current_backup_id,
            completed: is_complete,

            start_date: match self.fake_date {
                Some(fake_date) => fake_date
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                None => progression
                    .start_date
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            },
            end_date: if is_complete {
                match self.fake_date {
                    Some(fake_date) => {
                        let duration = if let Some(start_date) = progression.start_transfer_date {
                            now.duration_since(start_date).unwrap_or_default()
                        } else {
                            now.duration_since(UNIX_EPOCH).unwrap_or_default()
                        };
                        let end_date = fake_date + duration;
                        Some(
                            end_date
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                        )
                    }
                    None => Some(now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()),
                }
            } else {
                None
            },

            file_count: progression.file_count,
            new_file_count: progression.new_file_count,
            existing_file_count: progression
                .file_count
                .saturating_sub(progression.new_file_count),
            modified_file_count: progression.modified_file_count,
            removed_file_count: progression.removed_file_count,

            file_size: progression.file_size,
            new_file_size: progression.new_file_size,
            existing_file_size: progression
                .file_size
                .saturating_sub(progression.new_file_size),
            modified_file_size: progression.modified_file_size,

            compressed_file_size: progression.compressed_file_size,
            new_compressed_file_size: progression.new_compressed_file_size,
            existing_compressed_file_size: progression
                .compressed_file_size
                .saturating_sub(progression.new_compressed_file_size),
            modified_compressed_file_size: progression.modified_compressed_file_size,

            speed: progression.speed(),
        }
    }

    pub async fn authenticate(&mut self, password: &str) -> Result<()> {
        info!("Authenticate to the server");

        self.client.authenticate(password).await?;

        Ok(())
    }

    pub async fn init_backup_directory(&self, shares: &[&str]) -> Result<()> {
        let backups = Backups::new(&self.context);
        let previous_backup = backups
            .get_previous_backup(&self.hostname, self.current_backup_id)
            .await
            .map(|b| b.number);

        info!(
            "Prepare backup directory for {hostname}/{backup_number} with shares {shares:?} from previous backup {previous_backup:?}",
            hostname = self.hostname,
            backup_number = self.current_backup_id,
        );

        backups
            .clone_backup(
                &self.hostname,
                previous_backup,
                self.current_backup_id,
                shares,
            )
            .await?;

        // Load Reference count
        self.refcnt.lock().await.load_refcnt(true).await;

        self.save_backup(false).await?;

        Ok(())
    }

    pub async fn execute_command(&mut self, command: &str) -> Result<ExecuteCommandReply> {
        info!("Execute command: {}", command);

        let result = self.client.execute_command(command).await?;

        self.save_backup(false).await?;

        Ok(result)
    }

    pub async fn upload_file_list(&mut self, shares: Vec<String>) -> Result<()> {
        info!("Upload file list for {:?}", shares);

        let hostname = self.hostname.clone();
        let current_backup_id = self.current_backup_id;
        let context = self.context.clone().clone();

        let refresh_cache_stream = stream!({
            let backups = Backups::new(&context);
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
    ) -> Result<()> {
        info!("Download file list for {:?}", share);

        let hostname = self.hostname.clone();
        let current_backup_id = self.current_backup_id;

        let backups = Backups::new(&self.context);
        let manifest = backups.get_manifest(&hostname, current_backup_id, &share.share_path);

        let response = self.client.download_file_list(LaunchBackupRequest {
            share: Some(share.clone()),
        });

        let progression = Arc::new(Mutex::new(BackupProgression::default()));

        let response = response.filter_map(|entry| async {
            match entry {
                Ok(entry) => {
                    let file_size = entry
                        .manifest
                        .as_ref()
                        .map(FileManifest::size)
                        .unwrap_or_default();

                    let mut progression = progression.lock().await;
                    progression.progress_max += file_size;
                    match &entry.r#type() {
                        EntryType::Add => {
                            progression.new_file_count += 1;
                            progression.new_file_size += file_size;
                        }
                        EntryType::Modify => {
                            progression.modified_file_count += 1;
                            progression.modified_file_size += file_size;
                        }
                        EntryType::Remove => {
                            progression.removed_file_count += 1;
                        }
                    }

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

        {
            let progression = progression.lock().await;

            let mut global_progression = self.progression.lock().await;
            global_progression.start_transfer_date = Some(SystemTime::now());
            global_progression.progress_max += progression.progress_max;

            self.progress_max
                .insert(share.share_path.clone(), progression.progress_max);
        }

        self.save_backup(false).await?;

        Ok(result?)
    }

    /// The goal is to download a zone and split the zone in multiple chunk.
    /// We suppose in this method that the the first byte of the zone is the first byte of the first chunk.
    /// We split the download stream into chunk of `CHUNK_SIZE_U64` bytes.
    async fn download_zone<Fut, F>(
        &self,
        file_manifest: &mut FileManifest,
        chunks: &mut BTreeMap<usize, PoolChunkInformation>,
        chunk_information: ChunkInformation,
        callback: &F,
    ) -> Result<()>
    where
        F: Fn(PoolChunkInformation) -> Fut,
        Fut: Future<Output = ()>,
    {
        let filename = chunk_information.filename.clone();
        let full = chunk_information.chunks_id.is_empty();

        let pool_path = &self.context.config.path.pool_path;
        let readable = self.client.get_chunk(chunk_information);
        pin_mut!(readable);

        let mut current_chunk_id = 0;
        let mut current_chunk = None;

        while let Some(message) = readable.next().await {
            let message = message?;
            match message.field {
                Some(file_chunk::Field::Header(header)) => {
                    current_chunk_id = usize::try_from(header.chunk_id)?;

                    debug!("Download chunk {}", current_chunk_id);

                    let wrapper = PoolChunkWrapper::new(pool_path, None);
                    let writer = wrapper.writer().await?;

                    current_chunk = Some((wrapper, writer));
                }
                Some(Field::Data(chunk)) => {
                    debug!(
                        "Download chunk data {}, len = {}",
                        current_chunk_id,
                        chunk.data.len()
                    );

                    if let Some((_wrapper, writer)) = &mut current_chunk {
                        writer.write(&chunk.data).await?;
                    } else {
                        error!("No chunk header before data");
                    }
                }
                Some(Field::Footer(_)) => {
                    debug!("Download chunk footer {}", current_chunk_id);

                    if let Some((mut wrapper, mut writer)) = current_chunk.take() {
                        let chunk_information = writer.shutdown(&mut wrapper, &filename).await?;
                        callback(chunk_information.clone()).await;

                        chunks.insert(current_chunk_id, chunk_information);
                    } else {
                        error!("No chunk header before footer");
                    }
                }
                Some(Field::Eof(eof)) => {
                    debug!("Download chunk eof {}", current_chunk_id);

                    if full {
                        file_manifest.hash = eof.hash;
                    }
                }
                None => {
                    error!("No field in message");
                }
            }
        }

        Ok(())
    }

    fn get_missing_chunks(
        &self,
        chunks: &BTreeMap<usize, PoolChunkInformation>,
        max: usize,
    ) -> Vec<usize> {
        let all_numbers: BTreeSet<usize> = (0..max).collect();
        let map_keys: BTreeSet<usize> = chunks.keys().copied().collect();

        all_numbers
            .difference(&map_keys)
            .copied()
            .collect::<Vec<usize>>()
    }

    async fn get_chunks<Fut, F>(
        &self,
        file_manifest: &mut FileManifest,
        filename: &[u8],
        callback: &F,
    ) -> Result<(BTreeMap<usize, PoolChunkInformation>, Vec<usize>)>
    where
        F: Fn(PoolChunkInformation) -> Fut,
        Fut: Future<Output = ()>,
    {
        let pool_path = &self.context.config.path.pool_path;
        let reply = self
            .client
            .get_chunk_hash(ChunkHashRequest {
                filename: filename.to_vec(),
            })
            .await?;

        let mut chunks = BTreeMap::new();

        for chunk_number in 0..reply.chunks.len() {
            let hash = reply.chunks.get(chunk_number);
            if let Some(hash) = hash {
                let wrapper = PoolChunkWrapper::new(pool_path, Some(hash));
                if wrapper.exists() {
                    let chunk_information = wrapper.chunk_information().await?;
                    callback(chunk_information.clone()).await;
                    chunks.insert(chunk_number, chunk_information);

                    continue;
                }
            }
        }

        file_manifest.hash = reply.hash;
        let missing_chunks = self.get_missing_chunks(&chunks, reply.chunks.len());

        Ok((chunks, missing_chunks))
    }

    ///
    /// The goal of this method is to download the chunks of the manifest.
    /// The method will split the manifest in range of chunks that should be
    /// downloaded.
    ///
    /// The range of chunk will be downloaded sequentially
    async fn download_manifest_chunk<Fut, F>(
        &self,
        share_path: &str,
        mut file_manifest: FileManifest,
        is_add: bool,
        callback: &F,
    ) -> Result<FileManifest>
    where
        F: Fn(PoolChunkInformation) -> Fut,
        Fut: Future<Output = ()>,
    {
        info!(
            "Download manifest chunk for {:?}, is_add = {:?}",
            file_manifest.path(),
            is_add
        );
        let chunk_count = file_manifest.chunk_count();
        if chunk_count == 0 {
            file_manifest.chunks = vec![];
            file_manifest.hash = SHA256_EMPTYSTRING.to_vec();
            return Ok(file_manifest);
        }

        let filename = Path::new(share_path).join(file_manifest.path());
        let filename = path_to_vec(filename.as_path());

        let (mut chunks, missing_chunks) = if is_add {
            (BTreeMap::new(), Vec::new())
        } else {
            self.get_chunks(&mut file_manifest, &filename, callback)
                .await?
        };

        if chunks.is_empty() || !missing_chunks.is_empty() {
            self.download_zone(
                &mut file_manifest,
                &mut chunks,
                ChunkInformation {
                    filename,
                    chunks_id: missing_chunks
                        .iter()
                        .map(|x| u64::try_from(*x).unwrap_or_default())
                        .collect(),
                },
                callback,
            )
            .await?;
        }

        let missing_chunks = self.get_missing_chunks(&chunks, chunk_count);
        if !missing_chunks.is_empty() {
            error!(
                "Missing chunks for {:?}: {:?}",
                file_manifest.path(),
                missing_chunks
            );
            for chunk in missing_chunks {
                chunks.insert(
                    chunk,
                    PoolChunkInformation {
                        sha256: vec![],
                        size: 0,
                        compressed_size: 0,
                    },
                );
            }
        }

        debug!("Chunks = {:?}", chunks);

        let mut compressed_size: u64 = 0;
        let mut size: u64 = 0;
        let mut chunks_hash = Vec::with_capacity(chunk_count);
        {
            let mut refcnt = self.refcnt.lock().await;
            for chunk in chunks.values() {
                compressed_size += chunk.compressed_size;
                size += chunk.size;
                chunks_hash.push(chunk.sha256.clone());

                refcnt.apply(
                    &PoolRefCount {
                        sha256: chunk.sha256.clone(),
                        size: chunk.size,
                        compressed_size: chunk.compressed_size,
                        ref_count: 0,
                    },
                    &crate::pool::RefcntApplySens::Increase,
                );
            }
        }

        let path = file_manifest.path();
        let mut stats = file_manifest.stats.unwrap_or_default();
        stats.compressed_size = compressed_size;
        if stats.size != size {
            error!(
                "The manifest of file {:?}, size ({}) is not equal to the sum of the chunks ({})",
                &path, stats.size, size
            );
        }
        stats.size = size;
        file_manifest.stats = Some(stats);
        file_manifest.chunks = chunks_hash;

        // TODO: Add optional coherence check (in another thread ?)
        // TODO: Add if
        // if coherence_check {
        {
            let hash = file_manifest
                .calculate_hash(&self.context.config.path.pool_path)
                .await?;
            if file_manifest.hash.ne(&hash) {
                error!(
                    "The hash of the manifest of file {:?} is not equal to the calculated hash (corrupted file)",
                    &path
                );
            }
        }
        //}

        Ok(file_manifest)
    }

    pub async fn create_backup(
        &self,
        share_path: &str,
        callback: &impl Fn(&BackupProgression),
    ) -> Result<()> {
        info!("Backup share {:?}", share_path);

        let mut error_count = 0;
        let mut abort: Option<eyre::Report> = None;

        let backups = Backups::new(&self.context);
        let manifest = backups.get_manifest(&self.hostname, self.current_backup_id, share_path);

        let progress_max = self
            .progress_max
            .get(share_path)
            .copied()
            .unwrap_or_default();
        let progression = Arc::new(Mutex::new(BackupProgression {
            progress_max,
            ..BackupProgression::default()
        }));

        // Start by reading file list
        let mut journal_writer =
            ProtobufWriter::<FileManifestJournalEntry>::new(&manifest.journal_path, true, false)
                .await?;
        let file_list = manifest.read_filelist_entries();
        pin_mut!(file_list);

        while let Some(mut file_manifest_journal_entry) = file_list.next().await {
            let path = file_manifest_journal_entry.path();
            let is_add = file_manifest_journal_entry.r#type() == EntryType::Add;
            let is_remove = file_manifest_journal_entry.r#type() == EntryType::Remove;
            let is_special_file = file_manifest_journal_entry.is_special_file();
            if !is_remove && !is_special_file {
                if let Some(file_manifest) = file_manifest_journal_entry.manifest {
                    // TODO: Parrallellise to download CHUNK_SIZE manifest max at the same time
                    let progression = Arc::clone(&progression);

                    let file_manifest = self
                        .download_manifest_chunk(share_path, file_manifest, is_add, &move |chunk| {
                            let progression = Arc::clone(&progression);
                            async move {
                                let mut progression = progression.lock().await;
                                progression.progress_current += chunk.size;

                                callback(&progression);
                            }
                        })
                        .await;

                    match file_manifest {
                        Ok(file_manifest) => {
                            file_manifest_journal_entry.manifest = Some(file_manifest);
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

            match file_manifest_journal_entry.r#type() {
                EntryType::Add => {
                    let size = file_manifest_journal_entry.size();
                    let compressed_size = file_manifest_journal_entry.compressed_size();

                    let mut progression = progression.lock().await;
                    progression.new_file_count += 1;
                    progression.new_file_size += size;
                    progression.new_compressed_file_size += compressed_size;
                }
                EntryType::Modify => {
                    let size = file_manifest_journal_entry.size();
                    let compressed_size = file_manifest_journal_entry.compressed_size();

                    let mut progression = progression.lock().await;
                    progression.modified_file_count += 1;
                    progression.modified_file_size += size;
                    progression.modified_compressed_file_size += compressed_size;
                }
                EntryType::Remove => {
                    let mut progression = progression.lock().await;
                    progression.removed_file_count += 1;
                }
            }

            let write_result = journal_writer.write(&file_manifest_journal_entry).await;
            if let Err(err) = write_result {
                journal_writer.cancel().await?;
                error!("Can't write journal entry for {:?}: {}", path, err);
                return Err(eyre!("Can't write to journal entry {err}"));
            }
        }

        journal_writer.flush().await?;

        {
            let progression = progression.lock().await;

            let mut global_progression = self.progression.lock().await;
            global_progression.error_count += error_count;
            global_progression.progress_current += progression.progress_current;

            global_progression.new_file_count += progression.new_file_count;
            global_progression.new_file_size += progression.new_file_size;
            global_progression.new_compressed_file_size += progression.new_compressed_file_size;

            global_progression.modified_file_count += progression.modified_file_count;
            global_progression.modified_file_size += progression.modified_file_size;
            global_progression.modified_compressed_file_size +=
                progression.modified_compressed_file_size;

            global_progression.removed_file_count += progression.removed_file_count;
        }

        if let Some(e) = abort {
            Err(e)
        } else {
            Ok(())
        }
    }

    pub async fn close(&mut self) -> Result<()> {
        info!("Close backup");

        self.progression.lock().await.end_transfer_date = Some(SystemTime::now());

        // FIXME: Manage abort

        self.client.close().await?;

        self.save_backup(false).await?;

        Ok(())
    }

    pub async fn compact(&self, share_path: &str) -> Result<()> {
        info!("Compact share {:?}", share_path);

        let backups = Backups::new(&self.context);
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

    pub async fn count_references(&self) -> Result<()> {
        info!("Count references");

        let backups = Backups::new(&self.context);

        let mut refcnt = self.refcnt.lock().await;
        refcnt.finish(&self.context.config.path.pool_path).await?;
        refcnt.save_refcnt(&self.get_fake_date()).await?;

        let host_refcnt_file = backups.get_host_path(&self.hostname);
        Refcnt::apply_all_from(
            &host_refcnt_file,
            &refcnt,
            &crate::pool::RefcntApplySens::Increase,
            &self.get_fake_date(),
            &self.context,
        )
        .await?;

        self.save_backup(false).await?;

        Ok(())
    }

    pub async fn save_backup(&self, is_complete: bool) -> Result<()> {
        info!("Save backup (complete = {is_complete})");

        let backups = Backups::new(&self.context);
        let backup = self.to_backup(is_complete).await;

        backups
            .add_or_replace_backup(&self.hostname, &backup)
            .await?;

        Ok(())
    }
}
