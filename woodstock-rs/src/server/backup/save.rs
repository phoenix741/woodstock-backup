use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use async_stream::stream;
use eyre::{eyre, Result};
use futures::{pin_mut, StreamExt};
use log::{debug, error, info};
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use crate::{
    config::{Backup, BackupStatus, Backups, Configuration, Context, DEFAULT_CHANNEL_BUFFER_SIZE},
    events::{create_event_backup_end, create_event_backup_start},
    file_chunk::{self, Field},
    pool::{add_refcnt_to_pool, PoolChunkInformation, PoolChunkWrapper, Refcnt},
    proto::{CompressedWriter, ProtobufWriter},
    refresh_cache_request,
    server::progression::FileListProgression,
    utils::{chunk_hasher::get_empty_hash, compression::CompressionFormat},
    ChunkAlgorithm, ChunkHashRequest, ChunkInformation, EntryState, EntryType, EventSource,
    EventStatus, ExecuteCommandReply, FileManifest, FileManifestJournalEntry, PoolRefCount,
    RefreshCacheRequest, Share,
};

use super::{super::client::Client, super::progression::BackupProgression};

pub struct BackupSave<Clt: Client> {
    /// The unique identifier for the save operation.
    uuid: Vec<u8>,
    /// The client responsible for the save operation.
    client: Clt,
    /// Indicates whether file consistency should be checked.
    check_file_consistency: bool,

    /// The hostname associated with the save operation.
    hostname: String,
    /// The ID of the current backup being saved.
    current_backup_id: usize,
    /// The version of the agent performing the save operation.
    agent_version: Arc<Mutex<Option<String>>>,
    /// Represents an optional fake date for testing purposes.
    /// This is used to override the current system time during backup operations.
    fake_date: Option<SystemTime>,
    /// Tracks the maximum progress for each task in the backup operation.
    /// This is a thread-safe structure to ensure consistency across multiple threads.
    progress_max: Arc<Mutex<HashMap<String, u64>>>,
    /// Represents the progression state of the save operation.
    /// This includes details such as the number of files processed and errors encountered.
    progression: Arc<Mutex<BackupProgression>>,
    /// Manages the reference count for the save operation.
    /// This ensures proper tracking of shared resources during the backup process.
    refcnt: Arc<Mutex<Refcnt>>,

    /// The source of events for the save operation.
    source: EventSource,
    /// The configuration for the save operation.
    config: Configuration,
    /// Specifies the algorithm used for chunking data during the backup.
    /// This determines how data is divided into smaller chunks for storage.
    algorithm: ChunkAlgorithm,
    /// Compression format of saved files
    compression_format: CompressionFormat,
}

impl<Clt: Client> BackupSave<Clt> {
    /// Creates a new instance of `BackupSave`.
    ///
    /// # Arguments
    /// * `client` - The client for the backup system.
    /// * `hostname` - The hostname for the backup.
    /// * `backup_number` - The backup number.
    /// * `ctxt` - The context for the backup.
    /// * `config` - The configuration for the backup system.
    ///
    /// # Returns
    ///
    /// A new instance of `BackupSave`.
    pub fn new(
        client: Clt,
        hostname: &str,
        backup_number: usize,
        ctxt: &Context,
        config: &Configuration,
    ) -> Self {
        // At first backup set the used algorithm
        let _ = config.fix_algorithm();

        let backups = Backups::new(config);
        let destination_directory =
            backups.get_backup_destination_directory(hostname, backup_number);

        info!(
            "Initialize client for backup {hostname}/{backup_number} in {destination_directory:?}"
        );
        let uuid = Uuid::new_v4();
        let uuid = uuid.as_bytes().to_vec();

        BackupSave {
            uuid,
            client,
            check_file_consistency: false,
            hostname: hostname.to_string(),
            current_backup_id: backup_number,
            agent_version: Arc::new(Mutex::new(None)),
            progress_max: Arc::new(Mutex::new(HashMap::new())),
            progression: Arc::new(Mutex::new(BackupProgression::default())),
            refcnt: Arc::new(Mutex::new(Refcnt::new(&destination_directory))),
            source: ctxt.source,
            config: config.clone(),
            algorithm: config.chunk_algorithm,
            compression_format: config.compression_format,
            fake_date: None,
        }
    }

    /// Enables file consistency checks during the backup process.
    pub fn enable_file_consistency_check(&mut self) {
        self.check_file_consistency = true;
    }

    /// Disables file consistency checks during the backup process.
    pub fn disable_file_consistency_check(&mut self) {
        self.check_file_consistency = false;
    }

    /// Sets the agent version for the backup process.
    ///
    /// # Arguments
    /// * `agent_version` - The version of the agent to set.
    pub async fn set_agent_version(&self, agent_version: String) {
        let mut self_agent_version = self.agent_version.lock().await;
        self_agent_version.replace(agent_version);
    }

    /// Sets a fake date for the backup process.
    ///
    /// # Arguments
    /// * `fake_date` - An optional fake date to set.
    ///
    /// # Errors
    /// This function does not return errors.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn set_fake_date(&mut self, fake_date: Option<SystemTime>) {
        self.fake_date = fake_date;
    }

    /// Retrieves the fake date set for the backup process.
    ///
    /// # Returns
    ///
    /// The fake date as a `SystemTime`.
    pub fn get_fake_date(&self) -> SystemTime {
        self.fake_date.unwrap_or_else(SystemTime::now)
    }

    /// Retrieves the current progression state.
    ///
    /// # Returns
    ///
    /// The current `BackupProgression` state.
    pub async fn progress(&self) -> BackupProgression {
        *self.progression.lock().await
    }

    /// Converts the current state to a `Backup` object.
    ///
    /// # Arguments
    /// * `status` - The status of the backup.
    ///
    /// # Returns
    ///
    /// A `Backup` object representing the current state.
    async fn to_backup(&self, status: &BackupStatus) -> Backup {
        let now = SystemTime::now();
        let progression = *self.progression.lock().await;

        Backup {
            number: self.current_backup_id,
            agent_version: self.agent_version.lock().await.clone(),
            status: status.clone(),

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
            end_date: if status.is_finished() {
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

            error_count: progression.error_count,

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

    /// Authenticates the client with the provided password.
    ///
    /// # Arguments
    /// * `password` - The password for authentication.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the authentication succeeds.
    /// * `Err(eyre::Report)` if an error occurs during authentication.
    ///
    /// # Errors
    ///
    /// Returns an error if the authentication fails.
    pub async fn authenticate(&self, password: &str) -> Result<()> {
        info!("Authenticate to the server");

        let response = self.client.authenticate(password).await?;
        let mut self_agent_version = self.agent_version.lock().await;
        self_agent_version.replace(response.agent_version);

        Ok(())
    }

    /// Initializes the backup directory for the specified shares.
    ///
    /// # Arguments
    /// * `shares` - The list of shares to initialize.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the initialization succeeds.
    /// * `Err(eyre::Report)` if an error occurs during initialization.
    ///
    /// # Errors
    ///
    /// Returns an error if the initialization fails.
    pub async fn init_backup_directory(&self, shares: &[&str]) -> Result<()> {
        let backups = Backups::new(&self.config);
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

        self.save_backup(BackupStatus::InProgress).await?;

        // Register the event
        create_event_backup_start(
            &self.config.path.events_path,
            &self.uuid,
            self.source,
            &self.hostname,
            self.current_backup_id,
            shares,
        )
        .await?;

        Ok(())
    }

    /// Executes a command on the client.
    ///
    /// # Arguments
    /// * `command` - The command to execute.
    ///
    /// # Returns
    ///
    /// * `Ok(ExecuteCommandReply)` if the command execution succeeds.
    /// * `Err(eyre::Report)` if an error occurs during execution.
    ///
    /// # Errors
    ///
    /// Returns an error if the command execution fails.
    pub async fn execute_command(&self, command: &str) -> Result<ExecuteCommandReply> {
        info!("Execute command: {}", command);

        let result = self.client.execute_command(command).await?;

        self.save_backup(BackupStatus::InProgress).await?;

        Ok(result)
    }

    /// Synchronizes the file list for the backup process.
    ///
    /// # Arguments
    /// * `share` - The share to synchronize.
    /// * `file_list_tx` - An optional channel for sending file list progression updates.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the synchronization succeeds.
    /// * `Err(eyre::Report)` if an error occurs during synchronization.
    ///
    /// # Errors
    ///
    /// Returns an error if the synchronization fails.
    pub async fn synchronize_file_list(
        &self,
        share: &Share,
        file_list_tx: Option<mpsc::Sender<FileListProgression>>,
    ) -> Result<()> {
        info!("Download file list for {:?}", share);

        let hostname = self.hostname.clone();
        let current_backup_id = self.current_backup_id;

        let backups = Backups::new(&self.config);
        let manifest = backups.get_manifest(&hostname, current_backup_id, &share.share_path);

        let share_refresh_stream = share.clone();
        let manifest_refresh_stream = manifest.clone();

        let refresh_cache_stream = stream!({
            let header = RefreshCacheRequest {
                field: Some(refresh_cache_request::Field::Header(
                    share_refresh_stream.clone(),
                )),
            };

            yield header;

            let entries = manifest_refresh_stream.read_manifest_entries();
            pin_mut!(entries);

            while let Some(entry) = entries.next().await {
                let request = RefreshCacheRequest {
                    field: Some(refresh_cache_request::Field::FileManifest(entry)),
                };

                yield request;
            }
        });

        let response = self.client.synchronize_file_list(refresh_cache_stream);

        let progression = Arc::new(Mutex::new(FileListProgression::default()));

        let response = response.filter_map(|entry| {
            let progression = Arc::clone(&progression);
            let file_list_tx = file_list_tx.clone();
            async move {
                match entry {
                    Ok(entry) => {
                        let file_size = entry
                            .manifest
                            .as_ref()
                            .map(FileManifest::size)
                            .unwrap_or_default();

                        let mut progression = progression.lock().await;
                        progression.file_size += file_size;
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

                        // Send file list progress if a channel is provided
                        if let Some(tx) = &file_list_tx {
                            if let Err(e) = tx.send(progression.clone()).await {
                                error!("Failed to send file list progress: {}", e);
                            }
                        }

                        Some(entry.clone())
                    }
                    Err(e) => {
                        error!("Error while downloading file list: {}", e);
                        None
                    }
                }
            }
        });

        let result = manifest
            .save_filelist_entries(response, self.compression_format)
            .await;

        info!(
            "Download file list for {:?} completed (result = {})",
            share,
            result.as_ref().is_ok()
        );

        {
            let progression = progression.lock().await;

            let mut global_progression = self.progression.lock().await;
            global_progression.start_transfer_date = Some(SystemTime::now());
            global_progression.progress_max += progression.file_size;

            let mut progress_max = self.progress_max.lock().await;
            progress_max.insert(share.share_path.clone(), progression.file_size);
        }

        self.save_backup(BackupStatus::InProgress).await?;

        result
    }

    /// The goal is to download a zone and split the zone in multiple chunk.
    /// We suppose in this method that the the first byte of the zone is the first byte of the first chunk.
    /// We split the download stream into chunk of `CHUNK_SIZE_U64` bytes.
    ///
    /// # Arguments
    /// * `file_manifest` - The file manifest to download.
    /// * `chunks` - The map of chunks to download.
    /// * `chunk_information` - The chunk information for the download.
    /// * `tx` - The channel for sending chunk information.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the download succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the download.
    ///
    /// # Errors
    ///
    /// Returns an error if the download fails.
    async fn download_zone(
        &self,
        file_manifest: &mut FileManifest,
        chunks: &mut BTreeMap<usize, PoolChunkInformation>,
        chunk_information: ChunkInformation,
        tx: &mpsc::Sender<PoolChunkInformation>,
    ) -> Result<()> {
        let filename = chunk_information.filename.clone();
        let full = chunk_information.chunks_id.is_empty();

        let pool_path = &self.config.path.pool_path;
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
                    let writer = wrapper
                        .writer(self.algorithm, self.compression_format)
                        .await?;

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
                        let chunk_information = writer
                            .shutdown(&mut wrapper, &filename, self.compression_format)
                            .await?;
                        if let Err(e) = tx.send(chunk_information.clone()).await {
                            error!("Failed to send chunk information: {}", e);
                        }

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

    /// Retrieves the missing chunks from the pool.
    ///
    /// # Arguments
    /// * `chunks` - A map of chunk indices to their information.
    /// * `max` - The maximum number of chunks to retrieve.
    ///
    /// # Returns
    ///
    /// A vector of missing chunk indices.
    fn get_missing_chunks(
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

    /// Retrieves chunks from the specified share path.
    ///
    /// # Arguments
    /// * `share_path` - The path to the share.
    /// * `file_manifest` - The file manifest to update.
    ///
    /// # Returns
    ///
    /// A tuple containing the retrieved chunks and the missing chunks.
    async fn get_chunks(
        &self,
        share_path: &str,
        file_manifest: &mut FileManifest,
        filename: &[u8],
        tx: &mpsc::Sender<PoolChunkInformation>,
    ) -> Result<(BTreeMap<usize, PoolChunkInformation>, Vec<usize>)> {
        let pool_path = &self.config.path.pool_path;
        let reply = self
            .client
            .get_chunk_hash(ChunkHashRequest {
                share_path: share_path.to_string(),
                filename: filename.to_vec(),
                algorithm: self.algorithm as i32,
            })
            .await?;

        let mut chunks = BTreeMap::new();

        for chunk_number in 0..reply.chunks.len() {
            let hash = reply.chunks.get(chunk_number);
            if let Some(hash) = hash {
                let wrapper = PoolChunkWrapper::new(pool_path, Some(hash));
                if wrapper.exists() {
                    let chunk_information = wrapper.chunk_information().await?;
                    if let Err(e) = tx.send(chunk_information.clone()).await {
                        error!("Failed to send chunk information: {}", e);
                    }
                    chunks.insert(chunk_number, chunk_information);

                    continue;
                }
            }
        }

        file_manifest.hash = reply.hash;
        let missing_chunks = Self::get_missing_chunks(&chunks, reply.chunks.len());

        Ok((chunks, missing_chunks))
    }

    ///
    /// The goal of this method is to download the chunks of the manifest.
    /// The method will split the manifest in range of chunks that should be
    /// downloaded.
    ///
    /// The range of chunk will be downloaded sequentially
    async fn download_manifest_chunk(
        &self,
        share_path: &str,
        file_manifest: &mut FileManifest,
        is_add: bool,
        tx: &mpsc::Sender<PoolChunkInformation>,
    ) -> Result<(u64, u64, u64)> {
        debug!(
            "Download manifest chunk for {:?}, is_add = {:?}",
            file_manifest.path(),
            is_add
        );
        let chunk_count = file_manifest.chunk_count();
        if chunk_count == 0 {
            file_manifest.chunks = vec![];
            file_manifest.hash = get_empty_hash(self.algorithm);
            return Ok((0, 0, 0));
        }

        let filename = file_manifest.path.clone();

        let start_time = std::time::Instant::now();
        let (mut chunks, missing_chunks) = if is_add {
            (BTreeMap::new(), Vec::new())
        } else {
            self.get_chunks(share_path, file_manifest, &filename, tx)
                .await?
        };
        let xfer_calculation = start_time.elapsed();

        let start_time = std::time::Instant::now();
        if chunks.is_empty() || !missing_chunks.is_empty() {
            self.download_zone(
                file_manifest,
                &mut chunks,
                ChunkInformation {
                    share_path: share_path.to_string(),
                    filename,
                    chunks_id: missing_chunks
                        .iter()
                        .map(|x| u64::try_from(*x).unwrap_or_default())
                        .collect(),
                    algorithm: self.algorithm as i32,
                },
                tx,
            )
            .await?;
        }
        let xfer_duration = start_time.elapsed();

        let missing_chunks = Self::get_missing_chunks(&chunks, chunk_count);
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
                        format: 0,
                    },
                );
            }
        }

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
        if file_manifest.stats.is_none() {
            file_manifest.stats = Some(Default::default());
        }

        let stats = file_manifest.stats.as_mut().unwrap();
        stats.compressed_size = compressed_size;
        if stats.size != size {
            error!(
                "The manifest of file {:?}, size ({}) is not equal to the sum of the chunks ({})",
                &path, stats.size, size
            );
        }
        stats.size = size;
        file_manifest.chunks = chunks_hash;

        let start_time = std::time::Instant::now();
        if self.check_file_consistency {
            let hash = file_manifest
                .calculate_hash(&self.config.path.pool_path, self.algorithm)
                .await?;
            if file_manifest.hash.ne(&hash) {
                error!(
                    "The hash of the manifest of file {:?} is not equal to the calculated hash (corrupted file)",
                    &path
                );
            }
        }
        let xfer_check = start_time.elapsed();

        Ok((
            xfer_calculation.as_secs(),
            xfer_duration.as_secs(),
            xfer_check.as_secs(),
        ))
    }

    /// Creates a backup for the specified share path.
    ///
    /// # Arguments
    /// * `share_path` - The path of the share to back up.
    /// * `chunk_tx` - An optional channel for sending backup progression updates.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the backup creation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during backup creation.
    ///
    /// # Errors
    ///
    /// Returns an error if the backup creation fails.
    pub async fn create_backup(
        &self,
        share_path: &str,
        chunk_tx: Option<mpsc::Sender<BackupProgression>>,
    ) -> Result<()> {
        info!("Backup share {:?}", share_path);

        let mut error_count = 0;
        let mut abort: Option<eyre::Report> = None;

        let backups = Backups::new(&self.config);
        let manifest = backups.get_manifest(&self.hostname, self.current_backup_id, share_path);

        let progress_max = {
            let progress_max = self.progress_max.lock().await;
            progress_max.get(share_path).copied().unwrap_or_default()
        };
        let progression = Arc::new(Mutex::new(BackupProgression {
            progress_max,
            ..BackupProgression::default()
        }));

        let (internal_chunk_tx, mut chunk_rx) =
            mpsc::channel::<PoolChunkInformation>(DEFAULT_CHANNEL_BUFFER_SIZE);
        let progression_clone = progression.clone();

        let chunk_task = tokio::spawn(async move {
            while let Some(chunk) = chunk_rx.recv().await {
                let mut local_prog = progression_clone.lock().await;
                local_prog.progress_current += chunk.size;

                if let Some(chunk_tx) = &chunk_tx {
                    if let Err(e) = chunk_tx.send(*local_prog).await {
                        error!("Failed to send chunk progress: {}", e);
                    }
                }
            }
        });

        // Start by reading file list
        let mut journal_writer =
            ProtobufWriter::<CompressedWriter, FileManifestJournalEntry>::new_compressed(
                &manifest.journal_path,
                false,
                self.compression_format,
            )
            .await?;
        let file_list = manifest.read_filelist_entries();
        pin_mut!(file_list);

        while let Some(mut file_manifest_journal_entry) = file_list.next().await {
            let path = file_manifest_journal_entry.path();
            let is_add = file_manifest_journal_entry.r#type() == EntryType::Add;
            let is_remove = file_manifest_journal_entry.r#type() == EntryType::Remove;
            let is_special_file = file_manifest_journal_entry.is_special_file();
            let is_error = file_manifest_journal_entry.state() == EntryState::Error;

            // timestamp of the start of the transfer
            let xfer_start = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            file_manifest_journal_entry.xfer_start = xfer_start;

            if !is_remove && !is_special_file && !is_error {
                if let Some(file_manifest) = file_manifest_journal_entry.manifest.as_mut() {
                    // TODO: Parrallellise to download CHUNK_SIZE manifest max at the same time
                    let file_manifest = self
                        .download_manifest_chunk(
                            share_path,
                            file_manifest,
                            is_add,
                            &internal_chunk_tx,
                        )
                        .await;

                    match file_manifest {
                        Ok((xfer_calculation, xfer_duration, xfer_check)) => {
                            file_manifest_journal_entry.state = match file_manifest_journal_entry
                                .state()
                            {
                                EntryState::PartialMetadata => EntryState::ChunksPartialMetadata,
                                EntryState::Metadata => EntryState::Chunks,
                                _ => file_manifest_journal_entry.state(),
                            }
                                as i32;

                            file_manifest_journal_entry.xfer_calculation = xfer_calculation;
                            file_manifest_journal_entry.xfer_duration = xfer_duration;
                            file_manifest_journal_entry.xfer_check = xfer_check;
                        }
                        Err(e) => {
                            error!("Can't download chunk for {:?}: {}", path, e);

                            let tonic_status = e.downcast_ref::<tonic::Status>();
                            if let Some(tonic_status) = tonic_status {
                                // Si l'erreur est de type tonic (pas connecté, erreur d'authentification, ...) alors on abort
                                match tonic_status.code() {
                                    tonic::Code::Unavailable
                                    | tonic::Code::Unauthenticated
                                    | tonic::Code::PermissionDenied
                                    | tonic::Code::DeadlineExceeded
                                    | tonic::Code::Cancelled => {
                                        abort = Some(e);
                                        break;
                                    }
                                    _ => {}
                                }
                            }

                            file_manifest_journal_entry.state = EntryState::Error as i32;
                            file_manifest_journal_entry
                                .state_messages
                                .push(format!("{e:#}"));
                        }
                    };
                }
            }

            let is_error = file_manifest_journal_entry.state() == EntryState::Error;
            if is_error {
                error_count += 1;
            }

            if file_manifest_journal_entry.state() == EntryState::Chunks
                || file_manifest_journal_entry.state() == EntryState::ChunksPartialMetadata
            {
                match file_manifest_journal_entry.r#type() {
                    EntryType::Add => {
                        let size = file_manifest_journal_entry.size();
                        let compressed_size = file_manifest_journal_entry.compressed_size();

                        let mut progression = progression.lock().await;
                        progression.file_count += 1;
                        progression.file_size += size;
                        progression.compressed_file_size += compressed_size;

                        progression.new_file_count += 1;
                        progression.new_file_size += size;
                        progression.new_compressed_file_size += compressed_size;
                    }
                    EntryType::Modify => {
                        let size = file_manifest_journal_entry.size();
                        let compressed_size = file_manifest_journal_entry.compressed_size();

                        let mut progression = progression.lock().await;
                        progression.file_count += 1;
                        progression.file_size += size;
                        progression.compressed_file_size += compressed_size;

                        progression.modified_file_count += 1;
                        progression.modified_file_size += size;
                        progression.modified_compressed_file_size += compressed_size;
                    }
                    EntryType::Remove => {
                        let mut progression = progression.lock().await;
                        progression.removed_file_count += 1;
                    }
                }
            }

            let write_result = journal_writer.write(&file_manifest_journal_entry).await;
            if let Err(err) = write_result {
                journal_writer.cancel().await?;
                error!("Can't write journal entry for {:?}: {}", path, err);
                return Err(eyre!("Can't write to journal entry {err}"));
            }
        }

        drop(internal_chunk_tx);
        if let Err(e) = chunk_task.await {
            error!("Failed to join chunk task: {}", e);
        }

        journal_writer.flush().await?;

        {
            let progression = *progression.lock().await;

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

    /// Closes the backup process.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the closure succeeds.
    /// * `Err(eyre::Report)` if an error occurs during closure.
    ///
    /// # Errors
    ///
    /// Returns an error if the closure fails.
    pub async fn close(&self) -> Result<()> {
        info!("Close backup");

        self.progression.lock().await.end_transfer_date = Some(SystemTime::now());

        // FIXME: Manage abort

        self.client.close().await?;

        self.save_backup(BackupStatus::Finishing).await?;

        Ok(())
    }

    /// Compacts the specified share path.
    ///
    /// # Arguments
    /// * `share_path` - The path of the share to compact.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the compaction succeeds.
    /// * `Err(eyre::Report)` if an error occurs during compaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the compaction fails.
    pub async fn compact(&self, share_path: &str) -> Result<()> {
        info!("Compact share {:?}", share_path);

        let backups = Backups::new(&self.config);
        let manifest = backups.get_manifest(&self.hostname, self.current_backup_id, share_path);

        manifest
            .compact(
                &|manifest| async {
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
                },
                self.compression_format,
            )
            .await?;

        backups
            .add_backup_share_path(&self.hostname, self.current_backup_id, share_path)
            .await?;

        self.save_backup(BackupStatus::Finishing).await?;

        Ok(())
    }

    /// Counts the references for the backup process.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the reference counting succeeds.
    /// * `Err(eyre::Report)` if an error occurs during reference counting.
    ///
    /// # Errors
    ///
    /// Returns an error if the reference counting fails.
    pub async fn count_references(&self) -> Result<()> {
        info!("Count references");

        let backups = Backups::new(&self.config);

        let mut refcnt = self.refcnt.lock().await;
        refcnt
            .repair(&self.config.path.pool_path, self.algorithm)
            .await?;
        refcnt
            .save_refcnt(&self.get_fake_date(), false, self.compression_format)
            .await?;

        let host_refcnt_file = backups.get_host_path(&self.hostname);
        Refcnt::apply_all_from(
            &host_refcnt_file,
            &refcnt,
            &crate::pool::RefcntApplySens::Increase,
            &self.get_fake_date(),
            self.algorithm,
            self.compression_format,
        )
        .await?;

        self.save_backup(BackupStatus::Finishing).await?;

        Ok(())
    }

    /// Copy the references count from the backup to the pool.
    ///
    ///# Returns
    ///
    /// * `Ok(())` if the copy operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the copy operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the copy operation fails.
    pub async fn add_refcnt_to_pool(&self) -> Result<()> {
        info!("Add references count to pool");

        let backups = Backups::new(&self.config);
        let host_refcnt_file =
            backups.get_backup_destination_directory(&self.hostname, self.current_backup_id);

        add_refcnt_to_pool(
            &self.config,
            host_refcnt_file,
            &self.hostname,
            self.current_backup_id,
        )
        .await?;

        Ok(())
    }

    /// Saves the backup with the specified status.
    ///
    /// # Arguments
    /// * `status` - The status of the backup.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the save operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the save operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the save operation fails.
    pub async fn save_backup(&self, status: BackupStatus) -> Result<()> {
        info!("Save backup (complete = {status:?})");

        let backups = Backups::new(&self.config);
        let backup = self.to_backup(&status).await;

        backups
            .add_or_replace_backup(&self.hostname, &backup)
            .await?;

        if status.is_finished() {
            let shares = backups
                .get_backup_share_paths(&self.hostname, self.current_backup_id)
                .await;
            let shares = shares
                .iter()
                .map(std::string::String::as_str)
                .collect::<Vec<&str>>();

            // Register the event
            create_event_backup_end(
                &self.config.path.events_path,
                &self.uuid,
                self.source,
                &self.hostname,
                self.current_backup_id,
                &shares,
                EventStatus::Success,
            )
            .await?;
        }

        Ok(())
    }
}
