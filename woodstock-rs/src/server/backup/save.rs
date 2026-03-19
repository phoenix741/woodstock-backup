use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use async_stream::stream;
use chrono::{DateTime, Local};
use eyre::{eyre, Result};
use futures::{pin_mut, StreamExt};
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error, info, Instrument};
use uuid::Uuid;

use crate::{
    config::{Backup, BackupStatus, Backups, Configuration, Context, DEFAULT_CHANNEL_BUFFER_SIZE},
    events::{create_event_backup_end, create_event_backup_start},
    file_chunk::{self, Payload},
    pool::{
        PoolChunkInformation, PoolChunkWrapper, PoolManager, PoolV3StagingChunkRecord,
        PoolV3StagingFile, PoolV3StagingWriter,
    },
    proto::{CompressedWriter, ProtobufWriter},
    refresh_cache_request,
    server::progression::FileListProgression,
    utils::{chunk_hasher::get_empty_hash, compression::CompressionFormat},
    ChunkAlgorithm, ChunkHashRequest, ChunkInformation, EntryState, EntryType, EventSource,
    EventStatus, ExecuteCommandReply, FileManifest, FileManifestJournalEntry, RefreshCacheRequest,
    Share,
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
    /// UUID v7 identifier — primary key, used for filesystem routing.
    backup_id: Uuid,
    /// Sequential display number for this backup (set at creation, immutable).
    number: usize,
    /// UUID v7 of the immediately preceding completed backup (used for incremental cloning).
    previous_backup_id: Option<Uuid>,
    /// The version of the agent performing the save operation.
    agent_version: Arc<Mutex<Option<String>>>,
    /// Represents an optional fake date for testing purposes.
    /// This is used to override the current system time during backup operations.
    fake_date: Option<DateTime<Local>>,
    /// Tracks the maximum progress for each task in the backup operation.
    /// This is a thread-safe structure to ensure consistency across multiple threads.
    progress_max: Arc<Mutex<HashMap<String, u64>>>,
    /// Represents the progression state of the save operation.
    /// This includes details such as the number of files processed and errors encountered.
    progression: Arc<Mutex<BackupProgression>>,
    /// The source of events for the save operation.
    source: EventSource,
    /// The configuration for the save operation.
    config: Arc<Configuration>,
    /// Specifies the algorithm used for chunking data during the backup.
    /// This determines how data is divided into smaller chunks for storage.
    algorithm: ChunkAlgorithm,
    /// Compression format of saved files
    compression_format: CompressionFormat,
    /// Backups configuration
    backups: Arc<Backups>,
    /// Long-lived buffered staging writer owned by this backup.
    staging_writer: Arc<Mutex<Option<PoolV3StagingWriter>>>,
}

impl<Clt: Client> BackupSave<Clt> {
    /// Creates a new instance of `BackupSave`.
    ///
    /// # Arguments
    /// * `client` - The client for the backup system.
    /// * `hostname` - The hostname for the backup.
    /// * `backup_id` - The UUID v7 identifier for this backup.
    /// * `number` - The sequential display number for this backup.
    /// * `ctxt` - The context for the backup.
    /// * `config` - The configuration for the backup system.
    ///
    /// # Returns
    ///
    /// A new instance of `BackupSave`.
    pub fn new(
        client: Clt,
        hostname: &str,
        backup_id: Uuid,
        number: usize,
        previous_backup_id: Option<Uuid>,
        ctxt: &Context,
        config: Arc<Configuration>,
        backups: Arc<Backups>,
    ) -> Self {
        // At first backup set the used algorithm
        let _ = config.fix_algorithm();

        let destination_directory = backups.get_backup_destination_directory(hostname, backup_id);

        info!("Initialize client for backup {hostname}/{backup_id} in {destination_directory:?}");
        let uuid = Uuid::new_v4();
        let uuid = uuid.as_bytes().to_vec();

        BackupSave {
            uuid,
            client,
            check_file_consistency: false,
            hostname: hostname.to_string(),
            backup_id,
            number,
            previous_backup_id,
            agent_version: Arc::new(Mutex::new(None)),
            progress_max: Arc::new(Mutex::new(HashMap::new())),
            progression: Arc::new(Mutex::new(BackupProgression::default())),
            source: ctxt.source,
            config: config.clone(),
            algorithm: config.chunk_algorithm,
            compression_format: config.compression_format,
            fake_date: None,
            backups,
            staging_writer: Arc::new(Mutex::new(None)),
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
    pub fn set_fake_date(&mut self, fake_date: Option<DateTime<Local>>) {
        self.fake_date = fake_date;
    }

    /// Retrieves the fake date set for the backup process.
    ///
    /// # Returns
    ///
    /// The fake date as a `SystemTime`.
    pub fn get_fake_date(&self) -> DateTime<Local> {
        self.fake_date.unwrap_or_else(Local::now)
    }

    async fn ensure_staging_writer(&self) -> Result<()> {
        let mut staging_writer = self.staging_writer.lock().await;
        if staging_writer.is_some() {
            return Ok(());
        }

        let staging_file = PoolV3StagingFile::new(
            self.backups
                .get_pool_v3_staging_path(&self.hostname, self.backup_id),
        );
        let writer = staging_file
            .create_or_open_writer(&self.hostname, self.backup_id.as_bytes())
            .await?;
        staging_writer.replace(writer);
        Ok(())
    }

    async fn append_staging_chunk(
        &self,
        chunk_information: &PoolChunkInformation,
        publishes_new_chunk: bool,
    ) -> Result<()> {
        self.ensure_staging_writer().await?;
        let mut staging_writer = self.staging_writer.lock().await;
        let writer = staging_writer
            .as_mut()
            .ok_or_else(|| eyre!("staging writer not initialized"))?;
        writer
            .append_chunk(&PoolV3StagingChunkRecord {
                hash: chunk_information.chunk_hash.clone(),
                size: chunk_information.size,
                compressed_size: chunk_information.compressed_size,
                chunk_header_size: chunk_information.chunk_header_size,
                compression_format: chunk_information.format,
                ref_count_delta: 1,
                publishes_new_chunk,
                segment_id: chunk_information.segment_id,
                offset: chunk_information.offset,
            })
            .await
    }

    async fn flush_staging_writer(&self) -> Result<()> {
        let mut staging_writer = self.staging_writer.lock().await;
        if let Some(writer) = staging_writer.as_mut() {
            writer.flush().await?;
        }
        staging_writer.take();
        Ok(())
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
        let now = Local::now();
        let progression = *self.progression.lock().await;

        Backup {
            id: self.backup_id,
            number: self.number,
            agent_version: self.agent_version.lock().await.clone(),
            status: status.clone(),

            start_date: match self.fake_date {
                Some(fake_date) => fake_date,
                None => progression.start_date,
            },
            end_date: if status.is_finished() {
                match self.fake_date {
                    Some(fake_date) => {
                        let duration = if let Some(start_date) = progression.start_transfer_date {
                            now - start_date
                        } else {
                            now - now
                        };
                        Some(fake_date + duration)
                    }
                    None => Some(now),
                }
            } else {
                None
            },

            error_count: progression.error_count,
            error_message: None,

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
        let previous_backup = self.previous_backup_id;

        info!(
            "Prepare backup directory for {hostname}/{backup_id} with shares {shares:?} from previous backup {previous_backup:?}",
            hostname = self.hostname,
            backup_id = self.backup_id,
        );

        self.backups
            .clone_backup(&self.hostname, previous_backup, self.backup_id, shares)
            .await?;
        self.ensure_staging_writer().await?;
        self.save_backup(BackupStatus::InProgress).await?;

        // Register the event
        create_event_backup_start(
            &self.config,
            &self.config.path.events_path,
            &self.uuid,
            self.source,
            &self.hostname,
            self.backup_id,
            self.number,
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
        let current_backup_id = self.backup_id;

        let manifest = self
            .backups
            .get_manifest(&hostname, current_backup_id, &share.share_path);

        let share_refresh_stream = share.clone();
        let manifest_refresh_stream = manifest.clone();

        let refresh_cache_stream = stream!({
            let header = RefreshCacheRequest {
                payload: Some(refresh_cache_request::Payload::Header(
                    share_refresh_stream.clone(),
                )),
            };

            yield header;

            let entries = manifest_refresh_stream.read_manifest_entries();
            pin_mut!(entries);

            while let Some(entry) = entries.next().await {
                let request = RefreshCacheRequest {
                    payload: Some(refresh_cache_request::Payload::FileManifest(entry)),
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
                        match &entry.entry_type() {
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
            global_progression.start_transfer_date = Some(Local::now());
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

        let readable = self.client.get_chunk(chunk_information);
        pin_mut!(readable);

        let mut current_chunk_id = 0;
        let mut current_chunk = None;

        while let Some(message) = readable.next().await {
            let message = message?;
            match message.payload {
                Some(file_chunk::Payload::Header(header)) => {
                    current_chunk_id = usize::try_from(header.chunk_id)?;

                    debug!("Download chunk {}", current_chunk_id);

                    let wrapper = PoolChunkWrapper::new(&self.config.path.pool_path, None);
                    let writer = wrapper
                        .writer(self.algorithm, self.compression_format)
                        .await?;

                    current_chunk = Some((wrapper, writer));
                }
                Some(Payload::Data(chunk)) => {
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
                Some(Payload::Footer(_)) => {
                    debug!("Download chunk footer {}", current_chunk_id);

                    if let Some((mut wrapper, mut writer)) = current_chunk.take() {
                        let prepared_chunk = writer
                            .shutdown(&mut wrapper, &filename, self.compression_format)
                            .await?;
                        let chunk_information = PoolManager::new(self.config.clone())
                            .store_prepared_chunk(prepared_chunk)
                            .await?;
                        self.append_staging_chunk(&chunk_information, true).await?;
                        if let Err(e) = tx.send(chunk_information.clone()).await {
                            error!("Failed to send chunk information: {}", e);
                        }

                        chunks.insert(current_chunk_id, chunk_information);
                    } else {
                        error!("No chunk header before footer");
                    }
                }
                Some(Payload::Eof(eof)) => {
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
                let wrapper = PoolChunkWrapper::new(&self.config.path.pool_path, Some(hash));
                if wrapper.exists() {
                    let chunk_information = wrapper.chunk_information().await?;
                    self.append_staging_chunk(&chunk_information, false).await?;
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
    ) -> Result<(u64, u64, u64, Vec<u64>, Vec<u64>)> {
        debug!(
            "Download manifest chunk for {:?}, is_add = {:?}",
            file_manifest.path(),
            is_add
        );
        let chunk_count = file_manifest.chunk_count();
        if chunk_count == 0 {
            file_manifest.chunks = vec![];
            file_manifest.hash = get_empty_hash(self.algorithm);
            return Ok((0, 0, 0, vec![], vec![]));
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
                        chunk_hash: vec![],
                        size: 0,
                        compressed_size: 0,
                        format: 0,
                        segment_id: 0,
                        offset: 0,
                        chunk_header_size: 0,
                    },
                );
            }
        }

        let mut compressed_size: u64 = 0;
        let mut size: u64 = 0;
        let mut chunks_hash = Vec::with_capacity(chunk_count);
        let mut chunk_sizes = Vec::with_capacity(chunk_count);
        let mut chunk_compressed_sizes = Vec::with_capacity(chunk_count);
        for chunk in chunks.values() {
            compressed_size += chunk.compressed_size;
            size += chunk.size;
            chunks_hash.push(chunk.chunk_hash.clone());
            chunk_sizes.push(chunk.size);
            chunk_compressed_sizes.push(chunk.compressed_size);
        }

        let path = file_manifest.path();
        if file_manifest.stats.is_none() {
            file_manifest.stats = Some(Default::default());
        }
        // SAFETY: stats is always Some here — set just above if it was None
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
            chunk_sizes,
            chunk_compressed_sizes,
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

        let manifest = self
            .backups
            .get_manifest(&self.hostname, self.backup_id, share_path);

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

        let chunk_task = tokio::spawn(
            async move {
                while let Some(chunk) = chunk_rx.recv().await {
                    let mut local_prog = progression_clone.lock().await;
                    local_prog.progress_current += chunk.size;

                    if let Some(chunk_tx) = &chunk_tx {
                        if let Err(e) = chunk_tx.send(*local_prog).await {
                            error!("Failed to send chunk progress: {}", e);
                        }
                    }
                }
            }
            .in_current_span(),
        );

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
            let is_add = file_manifest_journal_entry.entry_type() == EntryType::Add;
            let is_remove = file_manifest_journal_entry.entry_type() == EntryType::Remove;
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
                        Ok((
                            xfer_calculation,
                            xfer_duration,
                            xfer_check,
                            chunk_sizes,
                            chunk_compressed_sizes,
                        )) => {
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
                            file_manifest_journal_entry.chunk_sizes = chunk_sizes;
                            file_manifest_journal_entry.chunk_compressed_sizes =
                                chunk_compressed_sizes;
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
                match file_manifest_journal_entry.entry_type() {
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

        self.progression.lock().await.end_transfer_date = Some(Local::now());

        // FIXME: Manage abort

        self.client.close().await?;

        Ok(())
    }

    /// Reconstructs in-memory progression stats from on-disk data after a server crash.
    ///
    /// Called during [`BackupPhase::Recover`] before resuming from a partial backup.
    /// The source of truth depends on how far the previous run progressed:
    ///
    /// - **Journal present** (crash before/during compact): reads journal entries (including
    ///   errors) to rebuild the breakdown stats (`new_file_count`, `modified_file_count`, etc.).
    ///   The totals (`file_count`, `file_size`) are left to the subsequent compact phase, which
    ///   iterates the final merged manifest.
    ///
    /// - **Log present, no journal** (crash after compact): compact will be skipped due to the
    ///   idempotence check. Reads log entries for the breakdown and manifest entries for the
    ///   totals (`file_count`, `file_size`, `compressed_file_size`).
    ///
    /// - **Neither present**: nothing to recover; progression stays at zero.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from disk fails unrecoverably.
    pub async fn recover_progression(&self, share_path: &str) -> Result<()> {
        info!(
            "Recovering progression for {}/{} share {:?}",
            self.hostname, self.backup_id, share_path
        );

        let manifest = self
            .backups
            .get_manifest(&self.hostname, self.backup_id, share_path);

        let mut new_file_count = 0usize;
        let mut new_file_size = 0u64;
        let mut new_compressed_file_size = 0u64;
        let mut modified_file_count = 0usize;
        let mut modified_file_size = 0u64;
        let mut modified_compressed_file_size = 0u64;
        let mut removed_file_count = 0usize;
        let mut error_count = 0usize;
        let mut file_count = 0usize;
        let mut file_size = 0u64;
        let mut compressed_file_size = 0u64;

        // Determine the source stream and whether the manifest totals must also be read.
        // - Journal present  → crash before compact; compact will rebuild file_count/file_size.
        //   Use read_journal_entries_all() so that error entries are also counted.
        // - Log present      → crash after compact; compact is idempotent-skipped, so totals
        //   must be read from the final manifest as well.
        // - Neither present  → nothing to recover.
        let (entries_stream, read_manifest_totals): (
            std::pin::Pin<Box<dyn futures::Stream<Item = FileManifestJournalEntry> + Send + '_>>,
            bool,
        ) = if manifest.journal_path.exists() {
            (Box::pin(manifest.read_journal_entries_all()), false)
        } else if manifest.log_path.exists() {
            (Box::pin(manifest.read_log_entries()), true)
        } else {
            info!(
                "No journal or log for {}/{} share {:?} — nothing to recover",
                self.hostname, self.backup_id, share_path
            );
            return Ok(());
        };

        // Shared loop: accumulates breakdown stats from journal entries or log entries
        // (both have the same `FileManifestJournalEntry` format).
        pin_mut!(entries_stream);
        while let Some(entry) = entries_stream.next().await {
            match entry.state() {
                EntryState::Error => error_count += 1,
                EntryState::Chunks | EntryState::ChunksPartialMetadata => {
                    let size = entry.size();
                    let compressed = entry.compressed_size();
                    match entry.entry_type() {
                        EntryType::Add => {
                            new_file_count += 1;
                            new_file_size += size;
                            new_compressed_file_size += compressed;
                        }
                        EntryType::Modify => {
                            modified_file_count += 1;
                            modified_file_size += size;
                            modified_compressed_file_size += compressed;
                        }
                        EntryType::Remove => removed_file_count += 1,
                    }
                }
                _ => {}
            }
        }

        // Totals from the final manifest — only needed when compact was already done
        // (log case). In the journal case, compact will set these itself.
        if read_manifest_totals {
            let manifest_entries = manifest.read_manifest_entries();
            pin_mut!(manifest_entries);
            while let Some(fm) = manifest_entries.next().await {
                file_count += 1;
                file_size += fm.size();
                compressed_file_size += fm.compressed_size();
            }
        }

        info!(
            "Recovered {}/{} share {:?}: new={}, modified={}, removed={}, errors={}, file_count={}",
            self.hostname,
            self.backup_id,
            share_path,
            new_file_count,
            modified_file_count,
            removed_file_count,
            error_count,
            file_count
        );

        let mut progression = self.progression.lock().await;
        progression.error_count += error_count;
        progression.new_file_count += new_file_count;
        progression.new_file_size += new_file_size;
        progression.new_compressed_file_size += new_compressed_file_size;
        progression.modified_file_count += modified_file_count;
        progression.modified_file_size += modified_file_size;
        progression.modified_compressed_file_size += modified_compressed_file_size;
        progression.removed_file_count += removed_file_count;
        // Totals: non-zero only in the log case (crash after compact). In the journal case,
        // compact will set these when it iterates the final manifest.
        progression.file_count += file_count;
        progression.file_size += file_size;
        progression.compressed_file_size += compressed_file_size;

        Ok(())
    }

    /// Compacts the backup manifest for the specified share path.
    ///
    /// Reads chunks from the agent, merges the journal into the manifest, and
    /// writes the final manifest. This operation is idempotent: if the journal has
    /// already been processed (renamed to `.log`), the compaction is skipped.
    ///
    /// # Arguments
    /// * `share_path` - The share path to compact.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the compaction succeeds or was already done.
    /// * `Err(eyre::Report)` if an error occurs during compaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the compaction fails.
    pub async fn compact(&self, share_path: &str) -> Result<()> {
        info!("Compact share {:?}", share_path);

        let manifest = self
            .backups
            .get_manifest(&self.hostname, self.backup_id, share_path);

        // Idempotence check: if journal doesn't exist, compact has already completed successfully
        if !manifest.journal_path.exists() {
            info!(
                "Compact already done for {}/{} share {:?} (journal already processed), skipping",
                self.hostname, self.backup_id, share_path
            );
            return Ok(());
        }

        manifest
            .compact(
                &|manifest| async {
                    let mut progression = self.progression.lock().await;
                    progression.file_count += 1;
                    progression.file_size += manifest.size();
                    progression.compressed_file_size += manifest.compressed_size();

                    Some(manifest)
                },
                self.compression_format,
            )
            .await?;

        self.backups
            .add_backup_share_path(&self.hostname, self.backup_id, share_path)
            .await?;

        Ok(())
    }

    /// Finalizes the local staging journal for the backup process.
    ///
    /// This operation is idempotent: it can be safely called multiple times.
    /// Re-running it only ensures the buffered Pool V3 staging writer is fully
    /// flushed before publication is finalized.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the staging flush succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the flush.
    ///
    /// # Errors
    ///
    /// Returns an error if the staging flush fails.
    pub async fn count_references(&self) -> Result<()> {
        info!("Finalize pool v3 staging before merge");
        self.flush_staging_writer().await?;
        Ok(())
    }

    /// Finalizes the Pool V3 publication for this backup.
    ///
    ///# Returns
    ///
    /// * `Ok(())` if the copy operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the copy operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the copy operation fails.
    pub async fn finalize_pool_publication(&self) -> Result<()> {
        info!("Finalize pool v3 backup publication");

        PoolManager::new(self.config.clone())
            .finalize_backup_publication(&self.hostname, self.backup_id)
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

        let backup = self.to_backup(&status).await;

        self.backups
            .add_or_replace_backup(&self.hostname, &backup)
            .await?;

        if status.is_finished() {
            let shares = self
                .backups
                .get_backup_share_paths(&self.hostname, self.backup_id)
                .await;
            let shares = shares
                .iter()
                .map(std::string::String::as_str)
                .collect::<Vec<&str>>();

            // Register the event
            create_event_backup_end(
                &self.config,
                &self.config.path.events_path,
                &self.uuid,
                self.source,
                &self.hostname,
                self.backup_id,
                self.number,
                &shares,
                EventStatus::Success,
            )
            .await?;
        }

        Ok(())
    }
}
