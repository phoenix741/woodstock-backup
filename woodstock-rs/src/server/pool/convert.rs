use std::{collections::HashMap, path::Path, time::SystemTime};
use tokio::sync::mpsc;

use eyre::Result;
use futures::StreamExt;
use log::error;
use tokio::fs::rename;
use uuid::Uuid;

use crate::{
    config::{Backups, Configuration, Hosts},
    events::append_events,
    manifest::Manifest,
    pool::{ConvertHashLink, PoolChunkWrapper, Refcnt, RefcntApplySens},
    proto::{save_file, ProtobufReader, ProtobufWriter, UnCompressedWriter},
    utils::files::copy_files,
    woodstock::event::Information,
    Event, EventSource, EventStatus, EventStep, EventType, HashConversionInformation, PoolRefCount,
};

#[derive(Clone)]
pub struct PoolConvert {
    /// The configuration used for the conversion process.
    config: Configuration,
}

impl PoolConvert {
    /// Creates a new instance of `PoolConvert`.
    ///
    /// # Arguments
    /// * `config` - The configuration for the pool conversion.
    ///
    /// # Returns
    ///
    /// A new instance of `PoolConvert`.
    #[must_use]
    pub fn new(config: &Configuration) -> Self {
        PoolConvert {
            config: config.clone(),
        }
    }

    /// Retrieves the maximum size of the pool.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` if the operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the operation.
    ///
    /// # Errors
    /// This function returns an error if the retrieval fails.
    pub async fn get_max(&self) -> Result<usize> {
        let mut pool_refcnt = Refcnt::new(&self.config.path.pool_path);
        pool_refcnt.load_refcnt(false).await;

        let mut count = 1 + pool_refcnt.size();

        let hosts = Hosts::new(&self.config);
        let backups = Backups::new(&self.config);

        for host in hosts.list_hosts().await? {
            let backups = backups.get_backups(&host).await;
            count += backups.len() + 1;
        }

        Ok(count)
    }

    /// Retrieves all chunks in the pool.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Vec<u8>>)` if the operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the operation.
    ///
    /// # Errors
    /// This function returns an error if the retrieval fails.
    pub async fn get_all_chunks(&self) -> Result<Vec<Vec<u8>>> {
        let mut pool_refcnt = Refcnt::new(&self.config.path.pool_path);
        pool_refcnt.load_refcnt(false).await;

        let chunks = pool_refcnt
            .list_refcnt()
            .map(|refcnt| refcnt.sha256.clone())
            .collect::<Vec<_>>();

        Ok(chunks)
    }

    /// Creates a start event for the hash conversion.
    ///
    /// # Arguments
    /// * `event_type` - The type of the event.
    /// * `source` - The source of the event.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` if the operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the operation.
    async fn create_event_start(
        &self,
        event_type: EventType,
        source: EventSource,
    ) -> Result<Vec<u8>> {
        let id = Uuid::new_v4();
        let id = id.as_bytes();

        let event = Event {
            id: id.to_vec(),
            r#type: event_type as i32,
            step: EventStep::Start as i32,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
            source: source as i32,
            user: String::new(),
            error_messages: Vec::new(),
            status: EventStatus::None as i32,

            information: None,
        };

        append_events(&self.config.path.events_path, &[&event]).await?;

        Ok(id.to_vec())
    }

    /// Creates an end event for the hash conversion.
    ///
    /// # Arguments
    /// * `id` - The ID of the event.
    /// * `information` - The information about the hash conversion.
    /// * `source` - The source of the event.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the operation.
    async fn create_event_hash_conversion_end(
        &self,
        id: &[u8],
        information: HashConversionInformation,
        source: EventSource,
    ) -> Result<()> {
        let event = Event {
            id: id.to_vec(),
            r#type: EventType::HashConversion as i32,
            step: EventStep::End as i32,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
            source: source as i32,
            user: String::new(),
            error_messages: Vec::new(),
            status: EventStatus::Success as i32,

            information: Some(Information::HashConversion(information)),
        };

        append_events(&self.config.path.events_path, &[&event]).await?;

        Ok(())
    }

    /// Saves the hash conversion link to a file.
    ///
    /// # Arguments
    /// * `pool_path` - The path to the pool.
    /// * `map` - The map of old and new hashes.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the operation.
    async fn save_convert_hash_link<P: AsRef<Path>>(
        &self,
        pool_path: P,
        map: &HashMap<Vec<u8>, Vec<u8>>,
    ) -> Result<()> {
        let filename = pool_path.as_ref().join("convert_hash_link.bin");
        let mut writer =
            ProtobufWriter::<UnCompressedWriter, ConvertHashLink>::new(&filename, true).await?;

        for (chunk, new_chunk) in map {
            let hash = ConvertHashLink {
                old_hash: chunk.clone(),
                new_hash: new_chunk.clone(),
            };

            writer.write(&hash).await?;
        }

        writer.flush().await?;
        Ok(())
    }

    /// Reads the hash conversion link from a file.
    ///
    /// # Arguments
    /// * `pool_path` - The path to the pool.
    ///
    /// # Returns
    ///
    /// * `Ok(HashMap<Vec<u8>, Vec<u8>>)` if the operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the operation.
    async fn read_convert_hash_link<P: AsRef<Path>>(
        &self,
        pool_path: P,
    ) -> Result<HashMap<Vec<u8>, Vec<u8>>> {
        let filename = pool_path.as_ref().join("convert_hash_link.bin");

        // If the file doesn't exist yet, return an empty HashMap
        if !filename.exists() {
            return Ok(HashMap::new());
        }

        let mut reader = ProtobufReader::<ConvertHashLink>::new(&filename, false).await?;
        let mut map = HashMap::new();

        // Use the stream approach for better memory efficiency
        let mut stream = reader.into_stream();
        while let Some(result) = stream.next().await {
            match result {
                Ok(hash) => {
                    map.insert(hash.old_hash, hash.new_hash);
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }

        Ok(map)
    }

    /// Converts reference counts for chunks.
    ///
    /// # Arguments
    /// * `chunks_map` - A map of old to new chunk hashes.
    /// * `source_path` - The path to the source file.
    /// * `destination_path` - The path to the destination file.
    ///
    /// # Returns
    /// * `Ok(())` if the conversion was successful.
    /// * `Err(eyre::Report)` if an error occurred during the operation.
    ///
    /// # Errors
    /// This function returns an error if the conversion fails.
    pub async fn convert_refcnt<P1: AsRef<Path>, P2: AsRef<Path>>(
        chunks_map: &HashMap<Vec<u8>, Vec<u8>>,
        source_path: P1,
        destination_path: P2,
    ) -> Result<()> {
        let mut refcnt = Refcnt::new(source_path);
        refcnt.load_refcnt(false).await;

        let mut new_refcnt = Refcnt::new(destination_path);

        for refcnt in refcnt.list_refcnt() {
            let old_hash = &refcnt.sha256;
            let new_hash = chunks_map.get(old_hash);

            if let Some(new_hash) = new_hash {
                new_refcnt.apply(
                    &PoolRefCount {
                        sha256: new_hash.clone(),
                        size: refcnt.size,
                        compressed_size: refcnt.compressed_size,
                        ref_count: refcnt.ref_count,
                    },
                    &RefcntApplySens::Increase,
                );
            } else {
                error!(
                    "Unused chunk {} not found in the new pool",
                    hex::encode(old_hash)
                );
            }
        }

        new_refcnt.save_refcnt(&SystemTime::now(), true).await?;

        Ok(())
    }

    /// Converts a single chunk.
    ///
    /// # Arguments
    /// * `new_config` - The new configuration to apply.
    /// * `progress_tx` - An optional sender for progress updates.
    ///
    /// # Returns
    /// * `Ok(usize)` if the chunk was successfully converted.
    /// * `Err(eyre::Report)` if an error occurred during the operation.
    ///
    /// # Errors
    /// This function returns an error if the conversion fails.
    ///
    /// # Panics
    /// This function panics if the chunk map does not contain the required key.
    pub async fn convert_chunk(
        &self,
        new_config: &Configuration,
        progress_tx: Option<mpsc::Sender<usize>>,
    ) -> Result<usize> {
        let yaml_files = ["disk_history.yml", "history.yml", "statistics.yml"];

        let chunks = self.get_all_chunks().await?;
        let mut progress_current = 0;

        let mut chunks_map = self
            .read_convert_hash_link(&self.config.path.pool_path)
            .await?;

        for refcnt in chunks {
            let old_wrapper = PoolChunkWrapper::new(&self.config.path.pool_path, Some(&refcnt));
            let mut new_wrapper = PoolChunkWrapper::new(&new_config.path.pool_path, None);

            if !chunks_map.contains_key(old_wrapper.get_hash().as_ref().unwrap()) {
                old_wrapper
                    .copy(&mut new_wrapper, new_config.chunk_algorithm)
                    .await?;

                chunks_map.insert(
                    old_wrapper.get_hash().clone().unwrap(),
                    new_wrapper.get_hash().clone().unwrap(),
                );
            }

            progress_current += 1;

            if let Some(tx) = &progress_tx {
                if let Err(e) = tx.send(progress_current).await {
                    error!("Failed to send conversion progress: {}", e);
                }
            }
        }

        self.save_convert_hash_link(&self.config.path.pool_path, &chunks_map)
            .await?;

        // Convert REFCNT and unused
        let () = Self::convert_refcnt(
            &chunks_map,
            &self.config.path.pool_path,
            &new_config.path.pool_path,
        )
        .await?;

        // Next copy if exist all yaml file (disk_history.yml, history.yml, statistics.yml)
        copy_files(
            &self.config.path.pool_path,
            &new_config.path.pool_path,
            &yaml_files,
        )
        .await?;

        Ok(progress_current)
    }

    /// Converts a manifest using the provided chunks map.
    ///
    /// # Arguments
    /// * `chunks_map` - The map of old and new hashes.
    /// * `manifest` - The source manifest.
    /// * `destination_manifest` - The destination manifest.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the operation.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// * The manifest entries cannot be read or saved.
    /// * The log entries cannot be read or saved.
    async fn convert_manifest(
        &self,
        chunks_map: &HashMap<Vec<u8>, Vec<u8>>,
        manifest: Manifest,
        destination_manifest: Manifest,
    ) -> Result<()> {
        // Fonction helper pour remplacer les hash des chunks
        fn replace_chunks(chunks: &mut [Vec<u8>], chunks_map: &HashMap<Vec<u8>, Vec<u8>>) {
            if !chunks.is_empty() {
                for chunk in chunks.iter_mut() {
                    if let Some(new_hash) = chunks_map.get(chunk) {
                        *chunk = new_hash.clone();
                    }
                }
            }
        }

        // Process manifest entries
        let all_messages = manifest.read_manifest_entries().map(|message| {
            let mut message = message.clone();
            replace_chunks(&mut message.chunks, chunks_map);
            message
        });
        save_file(&destination_manifest.manifest_path, all_messages, false).await?;

        // Process log entries
        let all_messages = manifest.read_log_entries().map(|message| {
            let mut message = message.clone();
            if let Some(ref mut manifest) = message.manifest {
                replace_chunks(&mut manifest.chunks, chunks_map);
            }
            message
        });
        save_file(&destination_manifest.log_path, all_messages, false).await?;

        Ok(())
    }

    /// Converts all manifests in the pool.
    ///
    /// # Arguments
    /// * `new_config` - The new configuration to apply.
    /// * `progress_tx` - An optional sender for progress updates.
    ///
    /// # Returns
    /// * `Ok(())` if all manifests were successfully converted.
    /// * `Err(eyre::Report)` if an error occurred during the operation.
    ///
    /// # Errors
    /// This function returns an error if the conversion fails.
    pub async fn convert_all_manifests(
        &self,
        new_config: &Configuration,
        progress_tx: Option<mpsc::Sender<usize>>,
    ) -> Result<()> {
        let files = [
            "error",
            "log",
            "history.yml",
            "shares.yml",
            "statistics.yml",
            "backup.yml",
        ];

        let hosts_service = Hosts::new(&self.config);
        let backups_service = Backups::new(&self.config);
        let new_backups_service = Backups::new(new_config);

        let mut progress_current = 0;
        let chunks_map = self
            .read_convert_hash_link(&self.config.path.pool_path)
            .await?;

        // Progress
        for host in hosts_service.list_hosts().await? {
            let backups = backups_service.get_backups(&host).await;
            for backup in backups {
                // Convert file log, manifest + copy all other files
                let shares = backups_service
                    .get_backup_share_paths(&host, backup.number)
                    .await;

                for share in shares {
                    let manifest = backups_service.get_manifest(&host, backup.number, &share);
                    let destination_manifest =
                        new_backups_service.get_manifest(&host, backup.number, &share);

                    // Convert the manifest
                    self.convert_manifest(&chunks_map, manifest, destination_manifest)
                        .await?;

                    // Convert REFCNT, unused
                    let () = Self::convert_refcnt(
                        &chunks_map,
                        &backups_service.get_backup_destination_directory(&host, backup.number),
                        &new_backups_service.get_backup_destination_directory(&host, backup.number),
                    )
                    .await?;

                    // Copy error, log, history.yml, shares.yml, statistics.yml
                    copy_files(
                        &backups_service.get_backup_destination_directory(&host, backup.number),
                        &new_backups_service.get_backup_destination_directory(&host, backup.number),
                        &files,
                    )
                    .await?;

                    progress_current += 1;

                    if let Some(tx) = &progress_tx {
                        if let Err(e) = tx.send(progress_current).await {
                            error!("Failed to send manifest conversion progress: {}", e);
                        }
                    }
                }
            }

            // Convert REFCNT, unused
            let () = Self::convert_refcnt(
                &chunks_map,
                &backups_service.get_host_path(&host),
                &new_backups_service.get_host_path(&host),
            )
            .await?;

            copy_files(
                &backups_service.get_host_path(&host),
                &new_backups_service.get_host_path(&host),
                &files,
            )
            .await?;

            progress_current += 1;

            if let Some(tx) = &progress_tx {
                if let Err(e) = tx.send(progress_current).await {
                    error!("Failed to send manifest conversion progress: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Converts the backup directory.
    ///
    /// # Arguments
    /// * `new_config` - The new configuration to apply.
    /// * `source` - The source of the event.
    /// * `progress_tx` - An optional sender for progress updates.
    ///
    /// # Returns
    /// * `Ok(())` if the backup directory was successfully converted.
    /// * `Err(eyre::Report)` if an error occurred during the operation.
    ///
    /// # Errors
    /// This function returns an error if the conversion fails.
    pub async fn convert_backup_dir(
        &self,
        new_config: &Configuration,
        source: EventSource,
        progress_tx: Option<mpsc::Sender<usize>>,
    ) -> Result<()> {
        let id = self
            .create_event_start(EventType::HashConversion, source)
            .await?;

        let total_count = self.convert_chunk(new_config, progress_tx.clone()).await?;

        let () = self.convert_all_manifests(new_config, progress_tx).await?;

        new_config.overwrite_algorithm()?;

        // Rename new directory
        rename(
            &self.config.path.pool_path,
            &self.config.path.pool_path.with_extension("old"),
        )
        .await?;
        rename(
            &self.config.path.hosts_path,
            &self.config.path.hosts_path.with_extension("old"),
        )
        .await?;

        rename(&new_config.path.pool_path, &self.config.path.pool_path).await?;
        rename(&new_config.path.hosts_path, &self.config.path.hosts_path).await?;

        let informations = HashConversionInformation {
            algorithm: new_config.chunk_algorithm as i32,
            count: total_count as u64,
        };
        self.create_event_hash_conversion_end(&id, informations, source)
            .await?;

        Ok(())
    }
}
