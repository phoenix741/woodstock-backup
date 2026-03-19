//! [`PoolManager`]: high-level owner of all pool operations.
//!
//! All logic previously spread across free functions in `pool/mod.rs` now lives
//! here. Construct once with an [`Arc<Configuration>`] and call methods.

use eyre::{Result, WrapErr};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::{create_dir_all, read_dir, remove_file};
use tracing::{error, warn};
use uuid::Uuid;

use crate::config::{Backups, Configuration};
use crate::pool::data::{self, CompactionTarget};
use crate::utils::compression::CompressionFormat;
use crate::utils::lock_redis::PoolLockRedis;

use super::{
    IndexedChunk, IndexedSegment, PoolChunkInformation, PoolIndex, PoolV3PendingFile,
    PoolV3PublicationChunkEntry, PoolV3PublicationFile, PoolV3RemovalChunkRecord,
    PoolV3RemovalFile, PoolV3StagingChunkRecord, PoolV3StagingFile,
};
use crate::pool::chunk::PreparedChunk;

const DEFAULT_SEGMENT_TARGET_SIZE: u64 = 512 * 1024 * 1024;

/// Aggregate result returned by one Pool V3 compaction run.
///
/// The values summarize only the segments that were successfully switched in the main index.
/// Physical cleanup of the replaced source segments still happens after that logical commit.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PoolCompactionSummary {
    /// Net number of bytes reclaimed after subtracting the new compacted segments.
    pub reclaimed_size: u64,
    /// Number of source segments removed from the logical index during the compaction commit.
    pub removed_segments: u64,
    /// Number of visible chunks rewritten into fresh target segments.
    pub rewritten_chunks: u64,
}

/// Incremental progress snapshot emitted while Pool V3 compaction is running.
///
/// Progress is reported after each processed source segment so callers can update job state
/// without waiting for the final checkpoint publish.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PoolCompactionProgress {
    /// Number of source segments already scanned and prepared for commit.
    pub processed_segments: usize,
    /// Total number of source segments selected as compaction candidates for this run.
    pub total_segments: usize,
    /// Number of visible chunks rewritten so far.
    pub rewritten_chunks: u64,
    /// Total stored bytes already copied into temporary compaction targets.
    pub rewritten_bytes: u64,
    /// Current reclaimed byte estimate based on processed source segments and temporary targets.
    pub reclaimed_size: u64,
}

#[derive(Debug, Clone)]
struct CompactionCandidate {
    segment: IndexedSegment,
    visible_chunks: Vec<IndexedChunk>,
    hidden_hashes: Vec<Vec<u8>>,
}

pub use data::SegmentReservation as ReservedPoolSegment;

/// High-level manager for all pool operations.
///
/// # Example
///
/// ```rust,no_run
/// # use std::sync::Arc;
/// # use woodstock::pool::PoolManager;
/// # use woodstock::config::Configuration;
/// # async fn example(config: Arc<Configuration>) -> eyre::Result<()> {
/// let pool = PoolManager::new(config);
/// pool.apply_pending_operations().await?;
/// # Ok(())
/// # }
/// ```
pub struct PoolManager {
    config: Arc<Configuration>,
}

impl PoolManager {
    /// Create a new [`PoolManager`] for the given configuration.
    #[must_use]
    pub fn new(config: Arc<Configuration>) -> Self {
        Self { config }
    }

    /// Reserves the first writable `OPEN` segment that is currently not locked.
    ///
    /// Candidates are processed in ascending `segment_id` order. If no `OPEN` segment can be
    /// reserved immediately, a brand-new segment is created, indexed, and returned already locked.
    ///
    /// # Errors
    /// Returns an error if the segment index cannot be opened, if Redis locking fails, or if the
    /// physical segment file cannot be created.
    pub async fn reserve_open_segment_or_create(
        &self,
        target_size: u64,
    ) -> Result<ReservedPoolSegment> {
        let segment_index = data::open_pool_index(&self.config.path.pool_path)?;

        data::reserve_open_or_create_segment(
            &self.config.path.pool_path,
            &self.config.redis_url(),
            &segment_index,
            target_size,
        )
        .await
    }

    /// Returns one visible chunk from the unified Pool V3 index.
    pub fn get_chunk_information(&self, hash: &[u8]) -> Result<Option<PoolChunkInformation>> {
        let pool_index = data::open_pool_index(&self.config.path.pool_path)?;
        Ok(pool_index
            .get_chunk(hash)?
            .map(Self::chunk_information_from_indexed_chunk))
    }

    /// Returns the number of Pool V3 segments that would currently be compacted.
    pub fn compact_pool_v3_max(&self) -> Result<usize> {
        let pool_index = data::open_pool_index(&self.config.path.pool_path)?;
        Ok(self.collect_compaction_candidates(&pool_index)?.len())
    }

    /// Publishes a newly prepared chunk into the physical pool only.
    ///
    /// This method is intentionally limited to segment allocation and index insertion with
    /// `ref_count = 0`. Logical staging and publication are handled by the backup layer.
    pub(crate) async fn store_prepared_chunk(
        &self,
        prepared_chunk: PreparedChunk,
    ) -> Result<PoolChunkInformation> {
        let (prepared_path, mut chunk_information) = prepared_chunk.into_parts();
        if let Some(existing_chunk) = self.get_chunk_information(&chunk_information.chunk_hash)? {
            remove_file(&prepared_path).await?;
            return Ok(existing_chunk);
        }

        let reserved_segment = self
            .reserve_open_segment_or_create(DEFAULT_SEGMENT_TARGET_SIZE)
            .await?;
        let mut segment_file = data::open_existing_segment(reserved_segment.path()).await?;
        let segment_entry = segment_file
            .append_chunk_from_path(
                chunk_information.chunk_hash.clone(),
                chunk_information.size,
                chunk_information.compressed_size,
                CompressionFormat::try_from(chunk_information.format)?,
                &prepared_path,
            )
            .await?;
        remove_file(&prepared_path).await?;

        let updated_segment = IndexedSegment {
            segment_id: reserved_segment.segment().segment_id,
            state: segment_file.state(),
            size_total: segment_file.size_total(),
            size_effective: reserved_segment.segment().size_effective,
            size_limit: reserved_segment.segment().size_limit,
            chunk_count: reserved_segment.segment().chunk_count.saturating_add(1),
        };
        chunk_information.segment_id = updated_segment.segment_id;
        chunk_information.offset = segment_entry.header_offset;
        chunk_information.chunk_header_size = segment_entry.chunk_header_size;
        let pool_index = data::open_pool_index(&self.config.path.pool_path)?;
        pool_index.with_write_transaction(|write_txn| {
            PoolIndex::put_segment(write_txn, &updated_segment)?;
            PoolIndex::put_chunk(
                write_txn,
                &IndexedChunk {
                    hash: chunk_information.chunk_hash.clone(),
                    size: chunk_information.size,
                    compressed_size: chunk_information.compressed_size,
                    compression_format: CompressionFormat::try_from(chunk_information.format)?,
                    ref_count: 0,
                    segment_id: chunk_information.segment_id,
                    offset: chunk_information.offset,
                    chunk_header_size: chunk_information.chunk_header_size,
                },
            )?;
            Ok(())
        })?;

        Ok(chunk_information)
    }

    /// Atomically merges one backup staging log into the main Pool V3 index.
    pub async fn merge_backup_staging(&self, hostname: &str, backup_id: Uuid) -> Result<()> {
        let staging = self.staging_file(hostname, backup_id);
        if !staging.path().exists() {
            return Ok(());
        }

        let records = staging.read_chunks().await?;
        let pool_index = data::open_pool_index(&self.config.path.pool_path)?;
        let backup_key = backup_id.as_bytes().to_vec();

        pool_index.with_write_transaction(|write_txn| {
            if PoolIndex::is_backup_merged(write_txn, &backup_key)? {
                return Ok(true);
            }

            for chunk_record in &records {
                let mut existing_chunk =
                    PoolIndex::get_chunk_for_write(write_txn, &chunk_record.hash)?.ok_or_else(
                        || {
                            eyre::eyre!(
                                "staging references an unknown indexed chunk {}",
                                hex::encode(&chunk_record.hash)
                            )
                        },
                    )?;

                let was_zero = existing_chunk.ref_count == 0;
                existing_chunk.ref_count = existing_chunk
                    .ref_count
                    .checked_add(chunk_record.ref_count_delta)
                    .ok_or_else(|| {
                        eyre::eyre!(
                            "pool v3 ref_count overflow for chunk {}",
                            hex::encode(&chunk_record.hash)
                        )
                    })?;

                if was_zero {
                    let mut segment =
                        PoolIndex::get_segment_for_write(write_txn, existing_chunk.segment_id)?
                            .ok_or_else(|| {
                                eyre::eyre!(
                                    "segment {} missing for indexed chunk {}",
                                    existing_chunk.segment_id,
                                    hex::encode(&chunk_record.hash)
                                )
                            })?;
                    segment.size_effective = segment
                        .size_effective
                        .checked_add(stored_len(
                            existing_chunk.chunk_header_size,
                            existing_chunk.compressed_size,
                        )?)
                        .ok_or_else(|| {
                            eyre::eyre!("size_effective overflow on segment {}", segment.segment_id)
                        })?;
                    PoolIndex::put_segment(write_txn, &segment)?;
                }

                PoolIndex::put_chunk(write_txn, &existing_chunk)?;
            }

            PoolIndex::mark_backup_merged(write_txn, &backup_key)?;

            Ok(false)
        })?;

        self.write_backup_removal_artifact(hostname, backup_id, &records)
            .await?;

        remove_file(staging.path()).await?;
        Ok(())
    }

    /// Finalizes one backup through durable publication and pending integration.
    pub async fn finalize_backup_publication(&self, hostname: &str, backup_id: Uuid) -> Result<()> {
        let staging = self.staging_file(hostname, backup_id);
        let publication = self.publication_file(hostname, backup_id);
        let backup_key = backup_id.as_bytes().to_vec();

        if !publication.path().exists() {
            if !staging.path().exists() {
                let pool_index = data::open_pool_index(&self.config.path.pool_path)?;
                let already_merged = pool_index.backup_is_merged(&backup_key)?;

                if already_merged {
                    return Ok(());
                }

                return Err(eyre::eyre!(
                    "pool v3 publication artifacts missing for backup {} on host {}",
                    backup_id,
                    hostname
                ));
            }

            let records = staging.read_chunks().await?;
            publication
                .create_with_records(hostname, backup_id.as_bytes(), &records)
                .await?;
            self.write_backup_removal_artifact(hostname, backup_id, &records)
                .await?;
        }

        self.enqueue_pending_operation("publication", hostname, backup_id, publication.path())
            .await?;
        self.apply_pool_v3_pending().await?;

        if staging.path().exists() {
            remove_file(staging.path()).await?;
        }

        Ok(())
    }

    /// Applies the logical removal of one backup against the Pool V3 index.
    pub async fn apply_backup_removal(&self, hostname: &str, backup_id: Uuid) -> Result<()> {
        let removal = self.removal_file(hostname, backup_id);
        let records = removal.read_records().await?;
        let pool_index = data::open_pool_index(&self.config.path.pool_path)?;
        let backup_key = backup_id.as_bytes().to_vec();

        pool_index.with_write_transaction(|write_txn| {
            if PoolIndex::is_backup_removed(write_txn, &backup_key)? {
                return Ok(());
            }

            for record in &records {
                let mut chunk = PoolIndex::get_chunk_for_write(write_txn, &record.hash)?
                    .ok_or_else(|| {
                        eyre::eyre!(
                            "pool v3 removal references an unknown chunk {}",
                            hex::encode(&record.hash)
                        )
                    })?;

                if chunk.ref_count < record.ref_count_delta {
                    return Err(eyre::eyre!(
                        "pool v3 removal underflow for chunk {}: {} < {}",
                        hex::encode(&record.hash),
                        chunk.ref_count,
                        record.ref_count_delta
                    ));
                }

                let previous_ref_count = chunk.ref_count;
                chunk.ref_count -= record.ref_count_delta;

                if previous_ref_count > 0 && chunk.ref_count == 0 {
                    let mut segment =
                        PoolIndex::get_segment_for_write(write_txn, chunk.segment_id)?.ok_or_else(
                            || {
                                eyre::eyre!(
                                    "segment {} missing for removed chunk {}",
                                    chunk.segment_id,
                                    hex::encode(&record.hash)
                                )
                            },
                        )?;
                    segment.size_effective = segment
                        .size_effective
                        .checked_sub(stored_len(chunk.chunk_header_size, chunk.compressed_size)?)
                        .ok_or_else(|| {
                            eyre::eyre!(
                                "size_effective underflow on segment {}",
                                segment.segment_id
                            )
                        })?;
                    PoolIndex::put_segment(write_txn, &segment)?;
                }

                PoolIndex::put_chunk(write_txn, &chunk)?;
            }

            PoolIndex::mark_backup_removed(write_txn, &backup_key)?;
            Ok(())
        })?;

        Ok(())
    }

    /// Finalizes one backup removal through durable pending integration.
    pub async fn finalize_backup_removal(&self, hostname: &str, backup_id: Uuid) -> Result<()> {
        let removal = self.removal_file(hostname, backup_id);
        if !removal.path().exists() {
            return Err(eyre::eyre!(
                "pool v3 removal artifact missing for backup {hostname}/{backup_id}"
            ));
        }

        self.enqueue_pending_operation("removal", hostname, backup_id, removal.path())
            .await?;
        self.apply_pool_v3_pending().await
    }

    /// Compacts Pool V3 segments by rewriting still-visible chunks into fresh segments.
    ///
    /// The public cleanup contract remains unchanged, but in Pool V3 the actual reclamation work
    /// is done by rewriting live chunks away from segments that still contain logically hidden
    /// entries. The main index switch happens inside one checkpointed index update; the physical
    /// deletion or quarantine of old segments only happens after that commit succeeds.
    pub async fn compact_pool_v3(&self, target: Option<&Path>) -> Result<PoolCompactionSummary> {
        self.compact_pool_v3_with_progress(target, |_| {}).await
    }

    /// Same as [`Self::compact_pool_v3`] but emits one synchronous progress snapshot after each
    /// processed source segment.
    pub async fn compact_pool_v3_with_progress<F>(
        &self,
        target: Option<&Path>,
        mut on_progress: F,
    ) -> Result<PoolCompactionSummary>
    where
        F: FnMut(PoolCompactionProgress),
    {
        let pool_index = data::open_pool_index(&self.config.path.pool_path)?;
        let candidates = self.collect_compaction_candidates(&pool_index)?;
        if candidates.is_empty() {
            return Ok(PoolCompactionSummary::default());
        }
        let total_segments = candidates.len();

        let segment_index = data::open_pool_index(&self.config.path.pool_path)?;
        let candidate_ids: BTreeSet<u64> = candidates
            .iter()
            .map(|candidate| candidate.segment.segment_id)
            .collect();

        let mut source_segments = Vec::new();
        let mut hidden_hashes = Vec::new();
        let mut moved_chunks = Vec::new();
        let mut created_targets = Vec::new();
        let mut active_target: Option<CompactionTarget> = None;
        let mut rewritten_bytes = 0_u64;
        let mut processed_source_bytes = 0_u64;

        for (processed_segments, candidate) in candidates.iter().enumerate() {
            let compaction_lock = self
                .try_reserve_segment_compaction_lock(candidate.segment.segment_id)
                .await?
                .ok_or_else(|| {
                    eyre::eyre!(
                        "segment {} is already reserved for compaction",
                        candidate.segment.segment_id
                    )
                })?;

            let source_path =
                data::resolve_segment_path(&self.config.path.pool_path, &candidate.segment);
            let source_segment = data::open_existing_segment(&source_path)
                .await
                .wrap_err_with(|| {
                    format!(
                        "failed to open source segment {} for compaction",
                        candidate.segment.segment_id
                    )
                })?;

            let mut visible_chunks = candidate.visible_chunks.clone();
            visible_chunks.sort_by_key(|chunk| chunk.offset);

            for chunk in &visible_chunks {
                let required_len = stored_len(chunk.chunk_header_size, chunk.compressed_size)?;

                if active_target.as_ref().is_some_and(|current| {
                    current.segment_file().remaining_capacity() < required_len
                }) {
                    created_targets.push(active_target.take().unwrap());
                }

                if active_target.is_none() {
                    active_target = Some(
                        self.create_reserved_compaction_target(&segment_index)
                            .await?,
                    );
                }

                let (entry, mut reader) = source_segment.read_chunk_at(chunk.offset).await?;
                if entry.hash != chunk.hash {
                    return Err(eyre::eyre!(
                        "segment {} chunk hash mismatch at offset {}",
                        candidate.segment.segment_id,
                        chunk.offset
                    ));
                }

                let target_segment = active_target.as_mut().expect("active target just created");
                let new_entry = target_segment
                    .segment_file_mut()
                    .append_chunk_from_reader(
                        chunk.hash.clone(),
                        chunk.size,
                        chunk.compressed_size,
                        chunk.compression_format,
                        &mut reader,
                    )
                    .await?;

                target_segment.segment_mut().state = target_segment.segment_file().state();
                target_segment.segment_mut().size_total =
                    target_segment.segment_file().size_total();
                target_segment.segment_mut().size_effective = target_segment
                    .segment()
                    .size_effective
                    .checked_add(required_len)
                    .ok_or_else(|| {
                        eyre::eyre!(
                            "size_effective overflow on compaction target {}",
                            target_segment.segment().segment_id
                        )
                    })?;
                target_segment.segment_mut().chunk_count = target_segment
                    .segment()
                    .chunk_count
                    .checked_add(1)
                    .ok_or_else(|| {
                        eyre::eyre!(
                            "chunk_count overflow on compaction target {}",
                            target_segment.segment().segment_id
                        )
                    })?;
                rewritten_bytes = rewritten_bytes.checked_add(required_len).ok_or_else(|| {
                    eyre::eyre!("rewritten byte count overflow during pool v3 compaction")
                })?;

                moved_chunks.push(IndexedChunk {
                    hash: chunk.hash.clone(),
                    size: chunk.size,
                    compressed_size: chunk.compressed_size,
                    compression_format: chunk.compression_format,
                    ref_count: chunk.ref_count,
                    segment_id: target_segment.segment().segment_id,
                    offset: new_entry.header_offset,
                    chunk_header_size: new_entry.chunk_header_size,
                });
            }

            hidden_hashes.extend(candidate.hidden_hashes.iter().cloned());
            source_segments.push((candidate.segment.clone(), source_path, compaction_lock));
            processed_source_bytes = processed_source_bytes
                .checked_add(candidate.segment.size_total)
                .ok_or_else(|| {
                    eyre::eyre!("source byte count overflow during pool v3 compaction")
                })?;

            let current_target_bytes = created_targets
                .iter()
                .map(|segment| segment.segment().size_total)
                .sum::<u64>()
                .checked_add(
                    active_target
                        .as_ref()
                        .map(|segment| segment.segment_file().size_total())
                        .unwrap_or_default(),
                )
                .ok_or_else(|| {
                    eyre::eyre!("target byte count overflow during pool v3 compaction")
                })?;
            on_progress(PoolCompactionProgress {
                processed_segments: processed_segments + 1,
                total_segments,
                rewritten_chunks: u64::try_from(moved_chunks.len())?,
                rewritten_bytes,
                reclaimed_size: processed_source_bytes.saturating_sub(current_target_bytes),
            });
        }

        if let Some(target_segment) = active_target.take() {
            created_targets.push(target_segment);
        }

        for target in &mut created_targets {
            target.publish().await?;
        }

        let removed_segment_ids: Vec<u64> = source_segments
            .iter()
            .map(|(segment, _, _)| segment.segment_id)
            .collect();

        let commit_result = pool_index.with_write_transaction(|write_txn| {
            for chunk in &moved_chunks {
                let current_chunk = PoolIndex::get_chunk_for_write(write_txn, &chunk.hash)?
                    .ok_or_else(|| {
                        eyre::eyre!(
                            "chunk {} disappeared before compaction commit",
                            hex::encode(&chunk.hash)
                        )
                    })?;
                if !candidate_ids.contains(&current_chunk.segment_id) {
                    return Err(eyre::eyre!(
                        "chunk {} moved outside candidate segments during compaction",
                        hex::encode(&chunk.hash)
                    ));
                }
                if current_chunk.ref_count == 0 {
                    return Err(eyre::eyre!(
                        "chunk {} became hidden during compaction",
                        hex::encode(&chunk.hash)
                    ));
                }
                PoolIndex::put_chunk(write_txn, chunk)?;
            }

            for hash in &hidden_hashes {
                let Some(hidden_chunk) = PoolIndex::get_chunk_for_write(write_txn, hash)? else {
                    continue;
                };
                if hidden_chunk.ref_count > 0 {
                    return Err(eyre::eyre!(
                        "hidden chunk {} became visible during compaction",
                        hex::encode(hash)
                    ));
                }
                PoolIndex::delete_chunk(write_txn, hash)?;
            }

            for target_segment in &created_targets {
                PoolIndex::put_segment(write_txn, target_segment.segment())?;
            }

            for segment_id in &removed_segment_ids {
                PoolIndex::delete_segment(write_txn, *segment_id)?;
            }

            Ok(())
        });

        if let Err(error) = commit_result {
            data::cleanup_unpublished_compaction_targets(&created_targets).await;
            return Err(error);
        }

        let removed_size_total: u64 = source_segments
            .iter()
            .map(|(segment, _, _)| segment.size_total)
            .sum();
        let added_size_total: u64 = created_targets
            .iter()
            .map(|segment| segment.segment().size_total)
            .sum();
        let reclaimed_size = removed_size_total.saturating_sub(added_size_total);
        let removed_segments = u64::try_from(source_segments.len())?;
        let rewritten_chunks = u64::try_from(moved_chunks.len())?;

        for (_, source_path, _) in &source_segments {
            if let Err(error) = data::cleanup_compacted_segment(source_path, target).await {
                warn!(
                    "failed to cleanup compacted segment {}: {}",
                    source_path.display(),
                    error
                );
            }
        }

        Ok(PoolCompactionSummary {
            reclaimed_size,
            removed_segments,
            rewritten_chunks,
        })
    }

    /// Applies all pending Pool V3 publication/removal descriptors.
    ///
    /// The caller should hold the appropriate pool lock before calling this in a
    /// multi-writer environment.
    pub async fn apply_pending_operations(&self) -> Result<()> {
        self.assert_clean("apply_pool_v3_pending_operations")
            .await?;
        self.apply_pool_v3_pending().await
    }

    /// Count the number of pending Pool V3 operations currently queued for integration.
    pub async fn count_pending_operations(&self) -> Result<usize> {
        let pending_path = data::pending_directory_path(&self.config.path.pool_path);

        if !pending_path.exists() {
            return Ok(0);
        }

        let mut count = 0;
        let mut entries = read_dir(&pending_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.path().is_file() {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Check whether the pool is currently in an incomplete transactional state.
    ///
    /// The heed-backed index no longer uses temporary checkpoint/shard files, so only pending
    /// operations can make the pool operationally incomplete.
    pub async fn is_dirty(&self) -> Result<Option<PathBuf>> {
        Ok(None)
    }

    /// Assert that the pool is clean, returning a descriptive error if dirty.
    ///
    /// # Arguments
    ///
    /// * `operation` — Name of the planned operation (used in the error message).
    ///
    /// # Errors
    ///
    /// Returns an error if the pool is in an incomplete transactional state.
    pub async fn assert_clean(&self, operation: &str) -> Result<()> {
        if let Some(dirty_path) = self.is_dirty().await? {
            error!(
                "Pool V3 index is in an incomplete transactional state - {} cannot proceed until fsck repairs {}",
                operation,
                dirty_path.display()
            );
            return Err(eyre::eyre!(
                "Pool V3 index is incomplete - cannot execute {}. Run fsck --repair to recover.",
                operation
            ));
        }
        Ok(())
    }

    fn staging_file(&self, hostname: &str, backup_id: Uuid) -> PoolV3StagingFile {
        PoolV3StagingFile::new(
            Backups::new(self.config.clone()).get_pool_v3_staging_path(hostname, backup_id),
        )
    }

    fn removal_file(&self, hostname: &str, backup_id: Uuid) -> PoolV3RemovalFile {
        PoolV3RemovalFile::new(
            Backups::new(self.config.clone()).get_pool_v3_removal_path(hostname, backup_id),
        )
    }

    fn publication_file(&self, hostname: &str, backup_id: Uuid) -> PoolV3PublicationFile {
        PoolV3PublicationFile::new(
            Backups::new(self.config.clone()).get_pool_v3_publication_path(hostname, backup_id),
        )
    }

    fn pending_file(
        &self,
        operation_type: &str,
        hostname: &str,
        backup_id: Uuid,
    ) -> PoolV3PendingFile {
        let operation_id = format!("{operation_type}-{hostname}-{backup_id}");
        PoolV3PendingFile::new(
            data::pending_directory_path(&self.config.path.pool_path).join(operation_id),
        )
    }

    async fn try_reserve_segment_compaction_lock(
        &self,
        segment_id: u64,
    ) -> Result<Option<PoolLockRedis>> {
        data::try_reserve_segment_compaction_lock(&self.config.redis_url(), segment_id).await
    }

    fn collect_compaction_candidates(
        &self,
        pool_index: &PoolIndex,
    ) -> Result<Vec<CompactionCandidate>> {
        let mut segments = pool_index.list_segments()?;
        segments.sort_by_key(|segment| segment.segment_id);

        let mut by_segment = BTreeMap::<u64, Vec<IndexedChunk>>::new();
        for chunk in pool_index.list_all_chunks()? {
            by_segment.entry(chunk.segment_id).or_default().push(chunk);
        }

        let mut candidates = Vec::new();
        for segment in segments {
            let Some(chunks) = by_segment.remove(&segment.segment_id) else {
                candidates.push(CompactionCandidate {
                    segment,
                    visible_chunks: Vec::new(),
                    hidden_hashes: Vec::new(),
                });
                continue;
            };

            let mut visible_chunks = Vec::new();
            let mut hidden_hashes = Vec::new();
            let mut has_hidden = false;

            for chunk in chunks {
                if chunk.ref_count > 0 {
                    visible_chunks.push(chunk);
                } else {
                    has_hidden = true;
                    hidden_hashes.push(chunk.hash.clone());
                }
            }

            if !has_hidden && !visible_chunks.is_empty() {
                continue;
            }

            candidates.push(CompactionCandidate {
                segment,
                visible_chunks,
                hidden_hashes,
            });
        }

        Ok(candidates)
    }

    async fn create_reserved_compaction_target(
        &self,
        segment_index: &PoolIndex,
    ) -> Result<CompactionTarget> {
        data::create_compaction_target(
            &self.config.path.pool_path,
            &self.config.redis_url(),
            segment_index,
            DEFAULT_SEGMENT_TARGET_SIZE,
        )
        .await
    }

    fn chunk_information_from_indexed_chunk(chunk: IndexedChunk) -> PoolChunkInformation {
        PoolChunkInformation {
            chunk_hash: chunk.hash,
            size: chunk.size,
            compressed_size: chunk.compressed_size,
            format: chunk.compression_format.as_u32(),
            segment_id: chunk.segment_id,
            offset: chunk.offset,
            chunk_header_size: chunk.chunk_header_size,
        }
    }

    async fn write_backup_removal_artifact(
        &self,
        hostname: &str,
        backup_id: Uuid,
        records: &[PoolV3StagingChunkRecord],
    ) -> Result<()> {
        let aggregated = Self::aggregate_backup_removal_records(records)?;

        self.removal_file(hostname, backup_id)
            .create_with_records(&aggregated.into_values().collect::<Vec<_>>())
            .await
    }

    async fn enqueue_pending_operation(
        &self,
        operation_type: &str,
        hostname: &str,
        backup_id: Uuid,
        journal_path: &Path,
    ) -> Result<()> {
        let pending = self.pending_file(operation_type, hostname, backup_id);
        if pending.path().exists() {
            return Ok(());
        }

        create_dir_all(data::pending_directory_path(&self.config.path.pool_path)).await?;
        let operation_id = format!("{operation_type}-{hostname}-{backup_id}");
        pending
            .create(
                &operation_id,
                operation_type,
                hostname,
                backup_id.as_bytes(),
                journal_path,
            )
            .await
    }

    async fn apply_pool_v3_pending(&self) -> Result<()> {
        let pending_directory = data::pending_directory_path(&self.config.path.pool_path);
        create_dir_all(&pending_directory).await?;

        let mut entries = read_dir(&pending_directory).await?;
        let mut pending_paths = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                pending_paths.push(path);
            }
        }
        pending_paths.sort();

        for path in pending_paths {
            let pending = PoolV3PendingFile::new(&path);
            let Some(header) = pending.read_header().await? else {
                continue;
            };

            match header.operation_type.as_str() {
                "publication" => {
                    let backup_id = Uuid::from_slice(&header.backup_id)?;
                    let publication = PoolV3PublicationFile::new(&header.journal_path);
                    let records = publication.read_records().await?;
                    self.integrate_publication_records(backup_id, &records)?;
                }
                "removal" => {
                    let backup_id = Uuid::from_slice(&header.backup_id)?;
                    self.apply_backup_removal(&header.hostname, backup_id)
                        .await?;
                }
                other => {
                    return Err(eyre::eyre!(
                        "unknown pool v3 pending operation type: {other}"
                    ));
                }
            }

            remove_file(&path).await?;
        }

        Ok(())
    }

    fn integrate_publication_records(
        &self,
        backup_id: Uuid,
        records: &[PoolV3PublicationChunkEntry],
    ) -> Result<()> {
        let pool_index = data::open_pool_index(&self.config.path.pool_path)?;
        let backup_key = backup_id.as_bytes().to_vec();

        pool_index.with_write_transaction(|write_txn| {
            if PoolIndex::is_backup_merged(write_txn, &backup_key)? {
                return Ok(());
            }

            for chunk_record in records {
                let mut existing_chunk =
                    PoolIndex::get_chunk_for_write(write_txn, &chunk_record.hash)?.ok_or_else(
                        || {
                            eyre::eyre!(
                                "publication references an unknown indexed chunk {}",
                                hex::encode(&chunk_record.hash)
                            )
                        },
                    )?;

                let was_zero = existing_chunk.ref_count == 0;
                existing_chunk.ref_count = existing_chunk
                    .ref_count
                    .checked_add(chunk_record.ref_count_delta)
                    .ok_or_else(|| {
                        eyre::eyre!(
                            "pool v3 ref_count overflow for chunk {}",
                            hex::encode(&chunk_record.hash)
                        )
                    })?;

                if was_zero {
                    let mut segment =
                        PoolIndex::get_segment_for_write(write_txn, existing_chunk.segment_id)?
                            .ok_or_else(|| {
                                eyre::eyre!(
                                    "segment {} missing for indexed chunk {}",
                                    existing_chunk.segment_id,
                                    hex::encode(&chunk_record.hash)
                                )
                            })?;
                    segment.size_effective = segment
                        .size_effective
                        .checked_add(stored_len(
                            existing_chunk.chunk_header_size,
                            existing_chunk.compressed_size,
                        )?)
                        .ok_or_else(|| {
                            eyre::eyre!("size_effective overflow on segment {}", segment.segment_id)
                        })?;
                    PoolIndex::put_segment(write_txn, &segment)?;
                }

                PoolIndex::put_chunk(write_txn, &existing_chunk)?;
            }

            PoolIndex::mark_backup_merged(write_txn, &backup_key)?;
            Ok(())
        })
    }

    fn aggregate_backup_removal_records(
        records: &[PoolV3StagingChunkRecord],
    ) -> Result<BTreeMap<Vec<u8>, PoolV3RemovalChunkRecord>> {
        let mut aggregated = BTreeMap::<Vec<u8>, PoolV3RemovalChunkRecord>::new();

        for chunk_record in records {
            match aggregated.entry(chunk_record.hash.clone()) {
                std::collections::btree_map::Entry::Occupied(mut entry) => {
                    let existing = entry.get_mut();
                    if existing.size != chunk_record.size
                        || existing.compressed_size != chunk_record.compressed_size
                        || existing.chunk_header_size != chunk_record.chunk_header_size
                    {
                        return Err(eyre::eyre!(
                            "pool v3 removal artifact metadata mismatch for chunk {}",
                            hex::encode(&chunk_record.hash)
                        ));
                    }

                    existing.ref_count_delta = existing
                        .ref_count_delta
                        .checked_add(chunk_record.ref_count_delta)
                        .ok_or_else(|| {
                            eyre::eyre!(
                                "pool v3 removal artifact overflow for chunk {}",
                                hex::encode(&chunk_record.hash)
                            )
                        })?;
                }
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(PoolV3RemovalChunkRecord {
                        hash: chunk_record.hash.clone(),
                        size: chunk_record.size,
                        compressed_size: chunk_record.compressed_size,
                        chunk_header_size: chunk_record.chunk_header_size,
                        ref_count_delta: chunk_record.ref_count_delta,
                    });
                }
            }
        }

        Ok(aggregated)
    }
}

fn stored_len(chunk_header_size: u64, compressed_size: u64) -> Result<u64> {
    chunk_header_size
        .checked_add(compressed_size)
        .ok_or_else(|| eyre::eyre!("pool v3 stored length overflow"))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use eyre::Result;
    use tempfile::tempdir;

    use super::PoolManager;
    use crate::config::Configuration;
    use crate::pool::data::SegmentFileState;
    use crate::pool::{data, IndexedChunk, IndexedSegment, PoolV3StagingChunkRecord};
    use crate::utils::compression::CompressionFormat;

    fn create_manager() -> Result<(tempfile::TempDir, PoolManager)> {
        let directory = tempdir()?;
        let config = Arc::new(Configuration::from_backup_path(
            directory.path().to_path_buf(),
        ));
        Ok((directory, PoolManager::new(config)))
    }

    fn sample_segment(segment_id: u64, state: SegmentFileState) -> IndexedSegment {
        IndexedSegment {
            segment_id,
            state,
            size_total: 1024,
            size_effective: 512,
            size_limit: 4096,
            chunk_count: 2,
        }
    }

    fn sample_chunk(hash_byte: u8, segment_id: u64, ref_count: u64, offset: u64) -> IndexedChunk {
        IndexedChunk {
            hash: vec![hash_byte; 32],
            size: 128,
            compressed_size: 64,
            compression_format: CompressionFormat::Zstd,
            ref_count,
            segment_id,
            offset,
            chunk_header_size: 12,
        }
    }

    fn staging_chunk_record(
        hash_byte: u8,
        ref_count_delta: u64,
        size: u64,
    ) -> PoolV3StagingChunkRecord {
        PoolV3StagingChunkRecord {
            hash: vec![hash_byte; 32],
            size,
            compressed_size: size / 2,
            chunk_header_size: 12,
            compression_format: 2,
            ref_count_delta,
            publishes_new_chunk: true,
            segment_id: 1,
            offset: 32,
        }
    }

    #[test]
    fn reserve_segment_paths_use_pool_layout() -> Result<()> {
        let (_directory, manager) = create_manager()?;

        assert!(data::pool_index_path(&manager.config.path.pool_path).ends_with("pool/index"));
        assert!(data::segment_path(&manager.config.path.pool_path, 12)
            .ends_with("pool/segments/seg-00000000012.seg"));
        assert_eq!(
            data::segment_relative_path(12),
            "segments/seg-00000000012.seg"
        );

        Ok(())
    }

    #[test]
    fn segment_lock_resource_matches_v3_convention() {
        assert_eq!(
            data::segment_write_lock_resource(44),
            "pool:segment:write:44"
        );
    }

    #[test]
    fn segment_index_lists_only_open_candidates_in_order() -> Result<()> {
        let (_directory, manager) = create_manager()?;
        let segment_index = data::open_pool_index(&manager.config.path.pool_path)?;

        segment_index.add_segment(&sample_segment(9, SegmentFileState::Full))?;
        segment_index.add_segment(&sample_segment(3, SegmentFileState::Open))?;
        segment_index.add_segment(&sample_segment(1, SegmentFileState::Open))?;

        let open_ids: Vec<u64> = segment_index
            .list_open_segments()?
            .into_iter()
            .map(|segment| segment.segment_id)
            .collect();

        assert_eq!(open_ids, vec![1, 3]);

        Ok(())
    }

    #[test]
    fn collect_compaction_candidates_keeps_hidden_and_empty_segments() -> Result<()> {
        let (_directory, manager) = create_manager()?;
        let pool_index = data::open_pool_index(&manager.config.path.pool_path)?;

        pool_index.add_segment(&sample_segment(1, SegmentFileState::Full))?;
        pool_index.add_segment(&sample_segment(2, SegmentFileState::Full))?;
        pool_index.add_segment(&sample_segment(3, SegmentFileState::Open))?;

        pool_index.add_chunk(&sample_chunk(0x01, 1, 2, 10))?;
        pool_index.add_chunk(&sample_chunk(0x02, 1, 0, 90))?;
        pool_index.add_chunk(&sample_chunk(0x03, 2, 1, 20))?;

        let candidates = manager.collect_compaction_candidates(&pool_index)?;
        let candidate_ids: Vec<u64> = candidates
            .iter()
            .map(|candidate| candidate.segment.segment_id)
            .collect();

        assert_eq!(candidate_ids, vec![1, 3]);
        assert_eq!(candidates[0].visible_chunks.len(), 1);
        assert_eq!(candidates[0].hidden_hashes.len(), 1);
        assert!(candidates[1].visible_chunks.is_empty());

        Ok(())
    }

    #[test]
    fn aggregate_backup_removal_records_rejects_metadata_mismatch() {
        let records = vec![
            staging_chunk_record(0x11, 1, 64),
            staging_chunk_record(0x11, 2, 128),
        ];

        let error = PoolManager::aggregate_backup_removal_records(&records).unwrap_err();
        assert!(error
            .to_string()
            .contains("pool v3 removal artifact metadata mismatch"));
    }

    #[test]
    fn aggregate_backup_removal_records_accumulates_deltas() -> Result<()> {
        let records = vec![
            staging_chunk_record(0x22, 1, 64),
            staging_chunk_record(0x22, 2, 64),
        ];

        let aggregated = PoolManager::aggregate_backup_removal_records(&records)?;
        assert_eq!(aggregated.len(), 1);
        assert_eq!(aggregated.values().next().unwrap().ref_count_delta, 3);

        Ok(())
    }

    #[test]
    fn compaction_temp_segment_path_is_hidden_from_normal_layout() -> Result<()> {
        let (_directory, manager) = create_manager()?;
        let temp_path = data::compaction_temp_segment_path(&manager.config.path.pool_path, 42);

        assert!(temp_path.to_string_lossy().contains(".seg-00000000042."));
        assert!(temp_path.to_string_lossy().contains(".compacting"));

        Ok(())
    }
}
