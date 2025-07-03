use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::fs::remove_file;
use tokio::sync::mpsc;

use futures::pin_mut;
use futures::StreamExt;

use crate::config::Backups;
use crate::config::Configuration;
use crate::config::Hosts;
use crate::proto::CompressedWriter;
use crate::proto::ProtobufWriter;
use crate::statistics::write_statistics;
use crate::statistics::PoolStatistics;
use crate::utils::chunk_hasher::is_empty_hash;
use crate::utils::compression::CompressionFormat;
use crate::PoolUnused;
use crate::{proto::ProtobufReader, ChunkAlgorithm, PoolRefCount};
use eyre::Result;

use super::PoolChunkWrapper;

#[derive(Debug)]
pub enum RefcntApplySens {
    Increase,
    Decrease,
}

/// # Pool Reference Count (Refcnt) Module
///
/// This module provides the [`Refcnt`] struct and associated methods for managing reference counts
/// and unused chunks in the Woodstock backup pool. It supports loading, saving, applying, and
/// reporting reference counts, as well as integration with backup and host metadata.
///
/// ## Main Structure
///
/// - [`Refcnt`]: Manages reference counts and unused chunks for the pool.
///
/// ## Main Methods
///
/// - [`Refcnt::new`]: Create a new reference count manager for a pool path.
/// - [`Refcnt::load_refcnt_from_path`]: Load reference counts from a specified path.
/// - [`Refcnt::load_refcnt`]: Load reference counts from disk.
/// - [`Refcnt::save_refcnt`]: Save reference counts to disk.
/// - [`Refcnt::apply`]: Apply a reference count change (increase or decrease).
/// - [`Refcnt::apply_all`]: Apply all reference counts from another instance.
/// - [`Refcnt::list_refcnt`]: List all reference counts.
/// - [`Refcnt::get_refcnt`]: Retrieve a reference count by hash.
/// - [`Refcnt::repair`]: Repair and validate reference count entries.
/// - [`Refcnt::remove_unused_files`]: Remove unused files from the pool.
///
/// ## Error Handling & Panics
///
/// - All async methods return `Result` and propagate errors using the `eyre` crate.
/// - Panics are not expected under normal operation.
///
/// ## See Also
///
/// - [`PoolRefCount`], [`PoolUnused`]: For reference count and unused chunk data
pub struct Refcnt {
    /// The path to the reference count file.
    path: PathBuf,
    /// The path to the unused chunks file.
    refcnt_path: PathBuf,
    /// The path to the unused chunks index file.
    unused_path: Option<PathBuf>,
    /// The index mapping chunk hashes to their reference counts.
    index: HashMap<Vec<u8>, PoolRefCount>,
}

// FIXME: Add lock (or add it on nodejs part ?)
impl Refcnt {
    /// Creates a new [`Refcnt`] instance for managing reference counts and unused chunks.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the reference count file.
    ///
    /// # Returns
    ///
    /// A new instance of [`Refcnt`].
    #[must_use]
    pub fn new<P: AsRef<Path>>(directory: P) -> Self {
        Self {
            path: directory.as_ref().to_path_buf(),
            refcnt_path: directory.as_ref().join("REFCNT"),
            unused_path: Some(directory.as_ref().join("unused")),
            index: HashMap::new(),
        }
    }

    /// Creates a new [`Refcnt`] instance by loading reference counts from a specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the reference count file.
    ///
    /// # Returns
    ///
    /// * `Ok(Refcnt)` - A new instance of [`Refcnt`] with loaded reference counts and unused chunks.
    /// * `Err(eyre::Report)` if an error occurs while loading the reference counts or unused chunks.
    ///
    /// # Errors
    ///
    /// Returns an error if the reference count file cannot be read or parsed, or if the unused chunks cannot be loaded.
    pub async fn load_refcnt_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut refcnt = Refcnt::new(path);
        refcnt.load_refcnt(false).await;

        Ok(refcnt)
    }

    /// Creates a new [`Refcnt`] instance by loading reference counts from a specified file.
    ///
    /// As a file is specified (not a path), the refcnt can't be saved.
    ///
    /// # Arguments
    ///
    /// * `file` - The file to the reference count file.
    ///
    /// # Returns
    ///
    /// * `Ok(Refcnt)` - A new instance of [`Refcnt`] with loaded reference counts and unused chunks.
    /// * `Err(eyre::Report)` if an error occurs while loading the reference counts or unused chunks.
    ///
    /// # Errors
    ///
    /// Returns an error if the reference count file cannot be read or parsed, or if the unused chunks cannot be loaded.
    pub async fn load_refcnt_from_file<P: AsRef<Path>>(file: P) -> Result<Self> {
        let mut refcnt = Refcnt {
            path: file.as_ref().to_path_buf(),
            refcnt_path: file.as_ref().to_path_buf(),
            unused_path: None,
            index: HashMap::new(),
        };
        refcnt.load_refcnt(false).await;

        Ok(refcnt)
    }

    /// Lists all reference counts in the pool.
    ///
    /// # Returns
    ///
    /// An iterator over all [`PoolRefCount`] entries.
    pub fn list_refcnt(&self) -> impl Iterator<Item = &PoolRefCount> {
        self.index.values()
    }

    /// Returns the total number of reference counts.
    ///
    /// # Returns
    ///
    /// The total count as a `usize`.
    #[must_use]
    pub fn size(&self) -> usize {
        self.index.len()
    }

    /// Returns the total number of unused reference counts .
    ///
    /// # Returns
    ///
    /// The total count as a `usize`.
    #[must_use]
    pub fn unused_size(&self) -> usize {
        self.index
            .iter()
            .filter(|(_, refcnt)| refcnt.ref_count == 0)
            .count()
    }

    /// Retrieves a reference count by its hash.
    ///
    /// # Arguments
    ///
    /// * `key` - The hash of the chunk.
    ///
    /// # Returns
    ///
    /// An optional reference to the [`PoolRefCount`] entry.
    #[must_use]
    pub fn get_refcnt(&self, key: &Vec<u8>) -> Option<&PoolRefCount> {
        self.index.get(key)
    }

    /// Creates a new [`Refcnt`] instance from a backup manifest.
    ///
    /// # Arguments
    ///
    /// * `hostname` - The hostname associated with the backup.
    /// * `backup_number` - The backup number to process.
    /// * `config` - Reference to the Woodstock [`Configuration`] struct.
    ///
    /// # Returns
    ///
    /// * `Ok(Refcnt)` - A new instance of [`Refcnt`] populated with reference counts from the manifest.
    /// * `Err(eyre::Report)` if an error occurs while loading the manifest or reference counts.
    ///
    /// # Errors
    ///
    /// Returns an error if the manifest cannot be loaded or processed.
    pub async fn new_backup_refcnt_from_manifest(
        hostname: &str,
        backup_number: usize,
        config: &Configuration,
    ) -> Result<Self> {
        let backups = Backups::new(config);
        let refcnt_file = backups.get_backup_destination_directory(hostname, backup_number);

        let manifests = backups.get_manifests(hostname, backup_number).await;
        let mut refcnt = Refcnt::new(&refcnt_file);
        refcnt.load_refcnt(true).await;

        for manifest in manifests {
            let counts = manifest.generate_refcnt();
            pin_mut!(counts);

            while let Some(count) = counts.next().await {
                refcnt.apply(&count, &crate::pool::RefcntApplySens::Increase);
            }
        }

        Ok(refcnt)
    }

    /// Creates a new [`Refcnt`] instance by aggregating reference counts from all backups of a specific host.
    ///
    /// # Arguments
    ///
    /// * `hostname` - The hostname associated with the backups.
    /// * `config` - Reference to the Woodstock [`Configuration`] struct.
    ///
    /// # Returns
    ///
    /// * `Ok(Refcnt)` - A new instance of [`Refcnt`] populated with aggregated reference counts from the host's backups.
    /// * `Err(eyre::Report)` if an error occurs while loading backup reference counts.
    ///
    /// # Errors
    ///
    /// Returns an error if the backup reference counts cannot be loaded or processed.
    pub async fn new_host_refcnt_from_backups(
        hostname: &str,
        config: &Configuration,
    ) -> Result<Self> {
        let backups_config = Backups::new(config);
        let refcnt_file = backups_config.get_host_path(hostname);

        let backups = backups_config.get_backups(hostname).await;

        let mut new_refcnt = Refcnt::new(&refcnt_file);
        new_refcnt.load_refcnt(true).await;

        for backup in backups {
            let destination_backup =
                backups_config.get_backup_destination_directory(hostname, backup.number);

            let mut backup_refcnt = Refcnt::new(&destination_backup);
            backup_refcnt.load_refcnt(false).await;

            for pool_refcnt in backup_refcnt.index.values() {
                new_refcnt.apply(pool_refcnt, &RefcntApplySens::Increase);
            }
        }

        Ok(new_refcnt)
    }

    /// Creates a new [`Refcnt`] instance by aggregating reference counts from all hosts.
    ///
    /// # Arguments
    ///
    /// * `config` - Reference to the Woodstock [`Configuration`] struct.
    ///
    /// # Returns
    ///
    /// * `Ok(Refcnt)` - A new instance of [`Refcnt`] populated with aggregated reference counts.
    /// * `Err(eyre::Report)` if an error occurs while loading host reference counts.
    ///
    /// # Errors
    ///
    /// Returns an error if the host reference counts cannot be loaded or processed.
    pub async fn new_pool_refcnt_from_host(config: &Configuration) -> Result<Self> {
        let hosts_config = Hosts::new(config);
        let backups_config = Backups::new(config);

        let refcnt_file = config.path.pool_path.clone();

        let hosts = hosts_config.list_hosts().await?;

        let mut new_refcnt = Refcnt::new(&refcnt_file);
        new_refcnt.load_refcnt(true).await;

        for host in hosts {
            let destination_host = backups_config.get_host_path(&host);

            let mut host_refcnt = Refcnt::new(&destination_host);
            host_refcnt.load_refcnt(false).await;

            for pool_refcnt in host_refcnt.index.values() {
                if pool_refcnt.ref_count > 0 {
                    new_refcnt.apply(pool_refcnt, &RefcntApplySens::Increase);
                }
            }
        }

        Ok(new_refcnt)
    }

    /// Applies all reference counts from one [`Refcnt`] instance to another.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the target reference count file.
    /// * `refcnt` - The source [`Refcnt`] instance to apply.
    /// * `sens` - The sensitivity (increase or decrease) for applying reference counts.
    /// * `date` - The current system time for logging purposes.
    /// * `algorithm` - The chunk algorithm used to identify empty file hashes.
    ///
    /// # Returns
    ///
    /// * `Ok(Refcnt)` - A new instance of [`Refcnt`] with the applied changes.
    /// * `Err(eyre::Report)` if an error occurs while applying reference counts.
    ///
    /// # Errors
    ///
    /// Returns an error if the reference counts cannot be applied or saved.
    pub async fn apply_all_from(
        path: &Path,
        refcnt: &Refcnt,
        sens: &RefcntApplySens,
        date: &SystemTime,
        algorithm: ChunkAlgorithm,
        compression_format: CompressionFormat,
    ) -> Result<Self> {
        let mut new_refcnt = Refcnt::load_refcnt_from_path(path).await?;

        debug!("Apply refcnt from {:?}", refcnt.path);
        new_refcnt.apply_all(&refcnt, sens);
        new_refcnt.repair(&path, algorithm).await?;
        new_refcnt
            .save_refcnt(date, false, compression_format)
            .await?;

        Ok(new_refcnt)
    }

    /// Loads reference counts from disk.
    ///
    /// # Arguments
    ///
    /// * `fill_zero` - If true, fills missing entries with zero counts.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub async fn load_refcnt(&mut self, fill_zero: bool) {
        debug!("Load refcnt from {:?}", self.refcnt_path);
        let messages = ProtobufReader::<PoolRefCount>::new(&self.refcnt_path, true).await;

        self.index.clear();
        self.load_unused().await;

        if let Ok(mut messages) = messages {
            let messages = messages.into_stream();
            pin_mut!(messages);

            while let Some(refcnt) = messages.next().await {
                if let Ok(mut refcnt) = refcnt {
                    let sha256_pool_refcnt = &refcnt.sha256;
                    if fill_zero {
                        refcnt.ref_count = 0;
                    }

                    self.index.insert(sha256_pool_refcnt.clone(), refcnt);
                }
            }
        } else {
            info!("No refcnt file found at {:?}", self.refcnt_path);
        }
    }

    /// Loads unused chunks from disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub async fn load_unused(&mut self) {
        let Some(unused_path) = &self.unused_path else {
            debug!("No unused path defined, skipping load unused");
            return;
        };

        debug!("Load unused from {:?}", self.unused_path);
        let messages = ProtobufReader::<PoolUnused>::new(unused_path, true).await;

        if let Ok(mut messages) = messages {
            let messages = messages.into_stream();
            pin_mut!(messages);

            while let Some(unused) = messages.next().await {
                if let Ok(unused) = unused {
                    self.index.insert(
                        unused.sha256.clone(),
                        PoolRefCount {
                            sha256: unused.sha256,
                            ref_count: 0,
                            size: unused.size,
                            compressed_size: unused.compressed_size,
                        },
                    );
                }
            }
        }
    }

    /// Removes unused files from the pool.
    ///
    /// # Arguments
    ///
    /// * `pool_path` - The root directory of the pool.
    /// * `target` - Optional target directory for removed files.
    /// * `progress_tx` - Channel for progress updates.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs.
    ///
    /// # Errors
    ///
    /// Returns an error if the removal process fails due to I/O issues.
    pub async fn remove_unused_files(
        &mut self,
        pool_path: &Path,
        target: Option<PathBuf>,
        progress_tx: mpsc::Sender<Option<PoolUnused>>,
    ) -> Result<()> {
        let Some(unused_path) = &self.unused_path else {
            debug!("No unused path defined, skipping remove unused files");
            return Ok(());
        };

        debug!("Remove unused files in {:?}", self.unused_path);
        for pool_unused in self.index.values().filter(|r| r.ref_count == 0) {
            let wrapper = PoolChunkWrapper::new(pool_path, Some(&pool_unused.sha256));
            if let Some(target) = &target {
                if let Err(e) = wrapper.mv(target).await {
                    error!("Error while moving chunk: {:?}", e);
                }
            } else {
                wrapper.remove().await;
            }

            if let Err(e) = progress_tx
                .send(Some(PoolUnused {
                    sha256: pool_unused.sha256.clone(),
                    size: pool_unused.size,
                    compressed_size: pool_unused.compressed_size,
                }))
                .await
            {
                error!("Failed to send unused file removal progress: {}", e);
            }
        }

        // Remove unused file if exists
        let _ = remove_file(&unused_path).await;

        Ok(())
    }

    /// Applies all reference counts from another [`Refcnt`] instance.
    ///
    /// # Arguments
    ///
    /// * `refcnt` - The source [`Refcnt`] instance containing reference counts to apply.
    /// * `sens` - The sensitivity (increase or decrease) for applying reference counts.
    pub fn apply_all(&mut self, refcnt: &Refcnt, sens: &RefcntApplySens) {
        for pool_refcnt in refcnt.index.values() {
            self.apply(pool_refcnt, sens);
        }
    }

    /// Applies a reference count change (increase or decrease).
    ///
    /// # Arguments
    ///
    /// * `refcnt` - The reference count to apply.
    /// * `sens` - The sensitivity (increase or decrease).
    pub fn apply(&mut self, refcnt: &PoolRefCount, sens: &RefcntApplySens) {
        let sha256_pool_refcnt = refcnt.sha256.clone();
        let hash_str = hex::encode(&sha256_pool_refcnt);

        let cnt = self
            .index
            .get(&sha256_pool_refcnt)
            .cloned()
            .unwrap_or_default();

        debug!(
            "Apply refcnt {} in {:?} {} {:?}",
            hash_str, sens, refcnt.ref_count, self.refcnt_path
        );

        let ref_count = match sens {
            RefcntApplySens::Increase => cnt.ref_count + refcnt.ref_count,
            RefcntApplySens::Decrease => cnt.ref_count.saturating_sub(refcnt.ref_count),
        };

        if cnt.compressed_size != refcnt.compressed_size
            && cnt.compressed_size != 0
            && refcnt.compressed_size != 0
        {
            error!(
                "Registered compressed size {} (refcnt {}) is different {} (refcnt {}) for {hash_str}  in {:?}",
                cnt.compressed_size, cnt.ref_count, refcnt.compressed_size, refcnt.ref_count, self.refcnt_path
            );
        }

        if cnt.size != refcnt.size && cnt.size != 0 && refcnt.size != 0 {
            error!(
                "Registered size is different for {hash_str} in {:?}",
                self.refcnt_path
            );
        }

        self.index.insert(
            sha256_pool_refcnt.clone(),
            PoolRefCount {
                sha256: sha256_pool_refcnt,
                ref_count,
                size: if refcnt.size == 0 {
                    cnt.size
                } else {
                    refcnt.size
                },
                compressed_size: if refcnt.compressed_size == 0 {
                    cnt.compressed_size
                } else {
                    refcnt.compressed_size
                },
            },
        );
    }

    /// Repairs and validates reference count entries by filling missing chunk information.
    ///
    /// This method iterates through all reference count entries and checks for missing
    /// size or compressed_size information. When these values are zero, it reads the
    /// actual chunk information from the pool and updates the reference count entry.
    /// For empty file hashes, no chunk information is expected and the entry is skipped.
    ///
    /// # Arguments
    ///
    /// * `pool_path` - The root directory of the pool where chunk files are stored.
    /// * `algorithm` - The chunk algorithm used to identify empty file hashes.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the repair operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Chunk information cannot be read from the pool
    /// * I/O operations fail during chunk inspection
    pub async fn repair(&mut self, pool_path: &Path, algorithm: ChunkAlgorithm) -> Result<()> {
        debug!("Read chunk informations for {:?}", self.refcnt_path);
        // For each value check chunk informations
        for (sha256_pool_refcnt, pool_refcnt) in &mut self.index {
            if pool_refcnt.size == 0 || pool_refcnt.compressed_size == 0 {
                let wrapper = PoolChunkWrapper::new(pool_path, Some(sha256_pool_refcnt));
                let hash_str = wrapper.get_hash_str().as_ref().map_or("None", |v| v);

                // Check if this is an empty file hash - this is normal and expected
                if is_empty_hash(sha256_pool_refcnt, algorithm) {
                    debug!(
                        "Empty file hash detected, skipping metadata check: {}",
                        hash_str
                    );
                    continue;
                }

                warn!(
                    "Missing size or compressed size for non-empty hash: {}",
                    hash_str
                );

                if let Ok(informations) = wrapper.chunk_information().await {
                    pool_refcnt.size = informations.size;
                    pool_refcnt.compressed_size = informations.compressed_size;
                    info!(
                        "Repaired metadata for {}: size={}, compressed_size={}",
                        hash_str, informations.size, informations.compressed_size
                    );
                } else {
                    error!("Cannot read chunk information for {}", hash_str);
                }
            }
        }

        Ok(())
    }

    /// Saves the reference counts to disk, with unused part.
    ///
    /// # Arguments
    ///
    /// * `date` - The current system time for logging purposes.
    /// * `with_unused` - If true, includes unused chunks in the saved reference counts.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs.
    ///
    /// # Errors
    ///
    /// Returns an error if the save operation fails due to I/O issues.
    pub async fn save_refcnt(
        &self,
        date: &SystemTime,
        with_unused: bool,
        compression_format: CompressionFormat,
    ) -> Result<()> {
        debug!("Save refcnt in {:?}", self.refcnt_path);
        let mut refcnt_writer = ProtobufWriter::<CompressedWriter, PoolRefCount>::new_compressed(
            &self.refcnt_path,
            true,
            compression_format,
        )
        .await?;
        let mut unused_writer = if with_unused && self.unused_path.is_some() {
            Some(
                ProtobufWriter::<CompressedWriter, PoolUnused>::new_compressed(
                    self.unused_path.as_ref().unwrap(),
                    true,
                    compression_format,
                )
                .await?,
            )
        } else {
            if let Some(unused_path) = &self.unused_path {
                debug!("Remove unused file if exist {:?}", unused_path);
                let _ = remove_file(unused_path).await;
            }
            None
        };

        for pool_refcnt in self.index.values() {
            if pool_refcnt.ref_count > 0 {
                refcnt_writer.write(pool_refcnt).await?;
            } else if let Some(unused_writer) = &mut unused_writer {
                unused_writer
                    .write(&PoolUnused {
                        sha256: pool_refcnt.sha256.clone(),
                        size: pool_refcnt.size,
                        compressed_size: pool_refcnt.compressed_size,
                    })
                    .await?;
            }
        }

        refcnt_writer.flush().await?;
        if let Some(unused_writer) = &mut unused_writer {
            unused_writer.flush().await?;
        }

        debug!("Save statistics in {:?}", self.path);
        let statistics = self.calculate_statistics();
        write_statistics(&statistics, &self.path, date).await?;

        Ok(())
    }

    /// Prepares the statistics for saving, calculating the total size, reference
    /// counts, and longest chain.
    ///
    /// # Returns
    ///
    /// A [`PoolStatistics`] struct containing the calculated statistics.
    fn calculate_statistics(&self) -> PoolStatistics {
        let mut statistics = PoolStatistics::default();

        debug!("Calculate statistics for {:?}", self.refcnt_path);
        // For each value in the index
        for pool_refcnt in self.index.values() {
            statistics.nb_ref += pool_refcnt.ref_count;

            if pool_refcnt.ref_count > statistics.longest_chain {
                statistics.longest_chain = pool_refcnt.ref_count;
            }

            if pool_refcnt.ref_count > 0 {
                statistics.nb_chunk += 1;
                statistics.size += pool_refcnt.size;
                statistics.compressed_size += pool_refcnt.compressed_size;
            } else {
                statistics.unused_size += pool_refcnt.size;
            }
        }

        statistics
    }
}

#[cfg(test)]
mod tests {
    use std::fs::remove_dir_all;

    use super::*;

    const SHA3_256_1: [u8; 1] = [0x1];
    const SHA3_256_2: [u8; 1] = [0x2];
    const SHA3_256_3: [u8; 1] = [0x3];
    const SHA3_256_4: [u8; 1] = [0x4];
    const SHA3_256_5: [u8; 1] = [0x5];
    const SHA3_256_6: [u8; 1] = [0x6];

    struct CleanUp;

    impl Drop for CleanUp {
        fn drop(&mut self) {
            // Clean up fixtures
            let _ = remove_dir_all("./data/pool_refcnt");
        }
    }

    fn create_refcnt(hash: Vec<&[u8]>) -> Refcnt {
        let path = Path::new("/tmp").to_path_buf();
        let mut refcnt = Refcnt::new(&path);

        for sha256 in hash {
            refcnt.apply(
                &PoolRefCount {
                    sha256: sha256.to_vec(),
                    ref_count: 1,
                    size: 10,
                    compressed_size: 5,
                },
                &RefcntApplySens::Increase,
            );
        }

        refcnt
    }

    #[tokio::test]
    async fn test_refcnt() {
        // The test should create a first Refcnt file, check it
        // Then create a second Refcnt file, check it
        let refcnt1 = create_refcnt(vec![&SHA3_256_1, &SHA3_256_2, &SHA3_256_3]);
        let refcnt2 = create_refcnt(vec![
            &SHA3_256_1,
            &SHA3_256_1,
            &SHA3_256_1,
            &SHA3_256_1,
            &SHA3_256_5,
        ]);

        let mut refcnt1_values = refcnt1.index.values().collect::<Vec<&PoolRefCount>>();
        refcnt1_values.sort_by(|a, b| a.sha256.cmp(&b.sha256));
        assert_eq!(
            refcnt1_values,
            vec![
                &PoolRefCount {
                    sha256: SHA3_256_1.to_vec(),
                    ref_count: 1,
                    size: 10,
                    compressed_size: 5
                },
                &PoolRefCount {
                    sha256: SHA3_256_2.to_vec(),
                    ref_count: 1,
                    size: 10,
                    compressed_size: 5
                },
                &PoolRefCount {
                    sha256: SHA3_256_3.to_vec(),
                    ref_count: 1,
                    size: 10,
                    compressed_size: 5
                }
            ]
        );

        let mut refcnt2_values = refcnt2.index.values().collect::<Vec<&PoolRefCount>>();
        refcnt2_values.sort_by(|a, b| a.sha256.cmp(&b.sha256));
        assert_eq!(
            refcnt2_values,
            vec![
                &PoolRefCount {
                    sha256: SHA3_256_1.to_vec(),
                    ref_count: 4,
                    size: 10,
                    compressed_size: 5
                },
                &PoolRefCount {
                    sha256: SHA3_256_5.to_vec(),
                    ref_count: 1,
                    size: 10,
                    compressed_size: 5
                },
            ]
        );
    }

    #[tokio::test]
    async fn test_refcnt_pool() {
        let _clean_up = CleanUp;

        let refcnt1 = create_refcnt(vec![&SHA3_256_1, &SHA3_256_2, &SHA3_256_3]);
        let refcnt2 = create_refcnt(vec![
            &SHA3_256_1,
            &SHA3_256_1,
            &SHA3_256_1,
            &SHA3_256_1,
            &SHA3_256_5,
        ]);
        let refcnt3 = create_refcnt(vec![&SHA3_256_3, &SHA3_256_4, &SHA3_256_5]);
        let refcnt4 = create_refcnt(vec![&SHA3_256_6, &SHA3_256_3]);

        // Add all refcnt to the pool
        let now = SystemTime::now();
        let pool_path = Path::new("./data/pool_refcnt");
        Refcnt::apply_all_from(
            pool_path,
            &refcnt1,
            &RefcntApplySens::Increase,
            &now,
            ChunkAlgorithm::Blake3,
            CompressionFormat::Zstd,
        )
        .await
        .unwrap();
        Refcnt::apply_all_from(
            pool_path,
            &refcnt2,
            &RefcntApplySens::Increase,
            &now,
            ChunkAlgorithm::Blake3,
            CompressionFormat::Zstd,
        )
        .await
        .unwrap();
        Refcnt::apply_all_from(
            pool_path,
            &refcnt3,
            &RefcntApplySens::Increase,
            &now,
            ChunkAlgorithm::Blake3,
            CompressionFormat::Zstd,
        )
        .await
        .unwrap();
        Refcnt::apply_all_from(
            pool_path,
            &refcnt4,
            &RefcntApplySens::Increase,
            &now,
            ChunkAlgorithm::Blake3,
            CompressionFormat::Zstd,
        )
        .await
        .unwrap();

        // Load refcnt from the pool
        let mut pool_refcnt = Refcnt::new(pool_path);
        pool_refcnt.load_refcnt(false).await;

        let mut pool_refcnt_values = pool_refcnt.index.values().collect::<Vec<&PoolRefCount>>();
        pool_refcnt_values.sort_by(|a, b| a.sha256.cmp(&b.sha256));

        assert_eq!(
            pool_refcnt_values,
            vec![
                &PoolRefCount {
                    sha256: SHA3_256_1.to_vec(),
                    ref_count: 5,
                    size: 10,
                    compressed_size: 5
                },
                &PoolRefCount {
                    sha256: SHA3_256_2.to_vec(),
                    ref_count: 1,
                    size: 10,
                    compressed_size: 5
                },
                &PoolRefCount {
                    sha256: SHA3_256_3.to_vec(),
                    ref_count: 3,
                    size: 10,
                    compressed_size: 5
                },
                &PoolRefCount {
                    sha256: SHA3_256_4.to_vec(),
                    ref_count: 1,
                    size: 10,
                    compressed_size: 5
                },
                &PoolRefCount {
                    sha256: SHA3_256_5.to_vec(),
                    ref_count: 2,
                    size: 10,
                    compressed_size: 5
                },
                &PoolRefCount {
                    sha256: SHA3_256_6.to_vec(),
                    ref_count: 1,
                    size: 10,
                    compressed_size: 5
                },
            ]
        );

        let pool = Refcnt::apply_all_from(
            pool_path,
            &refcnt2,
            &RefcntApplySens::Decrease,
            &now,
            ChunkAlgorithm::Blake3,
            CompressionFormat::Zstd,
        )
        .await
        .unwrap();
        let mut pool_refcnt_values = pool.index.values().collect::<Vec<&PoolRefCount>>();
        pool_refcnt_values.sort_by(|a, b| a.sha256.cmp(&b.sha256));

        assert_eq!(
            pool_refcnt_values,
            vec![
                &PoolRefCount {
                    sha256: SHA3_256_1.to_vec(),
                    ref_count: 1,
                    size: 10,
                    compressed_size: 5
                },
                &PoolRefCount {
                    sha256: SHA3_256_2.to_vec(),
                    ref_count: 1,
                    size: 10,
                    compressed_size: 5
                },
                &PoolRefCount {
                    sha256: SHA3_256_3.to_vec(),
                    ref_count: 3,
                    size: 10,
                    compressed_size: 5
                },
                &PoolRefCount {
                    sha256: SHA3_256_4.to_vec(),
                    ref_count: 1,
                    size: 10,
                    compressed_size: 5
                },
                &PoolRefCount {
                    sha256: SHA3_256_5.to_vec(),
                    ref_count: 1,
                    size: 10,
                    compressed_size: 5
                },
                &PoolRefCount {
                    sha256: SHA3_256_6.to_vec(),
                    ref_count: 1,
                    size: 10,
                    compressed_size: 5
                },
            ]
        );

        let pool = Refcnt::apply_all_from(
            pool_path,
            &refcnt3,
            &RefcntApplySens::Decrease,
            &now,
            ChunkAlgorithm::Blake3,
            CompressionFormat::Zstd,
        )
        .await
        .unwrap();
        let mut pool_refcnt_values = pool.index.values().collect::<Vec<&PoolRefCount>>();
        pool_refcnt_values.sort_by(|a, b| a.sha256.cmp(&b.sha256));

        assert_eq!(
            pool_refcnt_values,
            vec![
                &PoolRefCount {
                    sha256: SHA3_256_1.to_vec(),
                    ref_count: 1,
                    size: 10,
                    compressed_size: 5
                },
                &PoolRefCount {
                    sha256: SHA3_256_2.to_vec(),
                    ref_count: 1,
                    size: 10,
                    compressed_size: 5
                },
                &PoolRefCount {
                    sha256: SHA3_256_3.to_vec(),
                    ref_count: 2,
                    size: 10,
                    compressed_size: 5
                },
                &PoolRefCount {
                    sha256: SHA3_256_4.to_vec(),
                    ref_count: 0,
                    size: 10,
                    compressed_size: 5
                },
                &PoolRefCount {
                    sha256: SHA3_256_5.to_vec(),
                    ref_count: 0,
                    size: 10,
                    compressed_size: 5
                },
                &PoolRefCount {
                    sha256: SHA3_256_6.to_vec(),
                    ref_count: 1,
                    size: 10,
                    compressed_size: 5
                },
            ]
        );
    }
}
