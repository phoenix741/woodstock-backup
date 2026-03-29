//! [`Segments`]: manager for the collection of append-only segment files.
//!
//! Each segment file is named `segment-{id:016x}.seg` and lives in
//! `{pool_path}/segments/`.  The manager can open any segment for reading by
//! its numeric ID, and always returns a writable handle to the oldest
//! non-full segment (creating a new one when all existing segments have
//! reached their target size).
//!
//! ## `segments.info` — directory-wide metadata
//!
//! To avoid scanning every `.seg` file on each writer request, the manager
//! maintains a small protobuf file `segments/segments.info` that caches:
//!
//! - the oldest still-open segment ID,
//! - the next ID to allocate for a new segment,
//! - a monotone generation counter.
//!
//! Reads and writes to this file are coordinated with an exclusive Redis lock
//! (`LockOperation::Segment(SegmentLockTarget::Info)`) so that concurrent
//! processes never corrupt it and never allocate the same segment ID.

use std::path::PathBuf;
use std::sync::Arc;

use eyre::{eyre, Result};
use tokio::fs::{create_dir_all, read_dir, remove_file};
use tracing::{debug, warn};

use super::segment_protobuf::SegmentsInformation;
use crate::config::Configuration;
use crate::utils::lock_redis::{LockOperation, PoolLockRedis, SegmentLockTarget};

use super::segment_metadata::{
    read_persisted_segment_file_metadata, segment_sidecar_metadata_path,
    write_segment_file_metadata,
};
use super::segment_model::{SegmentFileMetadata, SegmentFileState, SegmentFillReport};
use super::segments_info::{read_segments_info, segments_info_path, write_segments_info_atomic};
use super::segments_writer::SegmentsWriter;
use super::{SegmentReader, SegmentWriter};

/// Default target size for a single segment file: 512 MiB.
pub const DEFAULT_SEGMENT_TARGET_SIZE: u64 = 512 * 1024 * 1024;

/// Manages the collection of append-only segment files that make up the pool
/// storage layer.
///
/// Segment files are named `segment-{id:016x}.seg` and stored under
/// `{pool_path}/segments/`.
///
/// # Usage
///
/// ```rust,no_run
/// # use std::sync::Arc;
/// # use woodstock::pool::Segments;
/// # use woodstock::config::Configuration;
/// # async fn example(config: Arc<Configuration>) -> eyre::Result<()> {
/// let segments = Segments::new(config);
///
/// // Open a segment for reading
/// let reader = segments.get_reader(0).await?;
///
/// // Obtain an append-only writer (creates a segment if none exist)
/// let writer = segments.get_writer().await?;
/// # Ok(())
/// # }
/// ```
pub struct Segments {
    config: Arc<Configuration>,
}

impl Segments {
    /// Creates a new [`Segments`] manager for the given configuration.
    #[must_use]
    pub fn new(config: Arc<Configuration>) -> Self {
        Self { config }
    }

    /// Returns the filesystem path for the segment with the given `id`.
    fn segment_path(&self, segment_id: u64) -> PathBuf {
        self.config
            .path
            .pool_segments_path
            .join(format!("segment-{segment_id:016x}.seg"))
    }

    /// Scans the segment directory and returns all valid segment files, sorted
    /// by ID in ascending order.
    ///
    /// Each entry is `(segment_id, path, is_full)`.  Only files with the `.seg`
    /// extension are considered; sidecar files (`.seg.meta`) and any other
    /// entries are silently skipped.  The segment ID is read from the file
    /// header rather than parsed from the filename.  Files that cannot be
    /// opened (corrupt header, …) are logged as warnings and skipped.
    async fn list_segments(&self) -> Result<Vec<(u64, PathBuf, bool)>> {
        let dir = &self.config.path.pool_segments_path;

        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut segments: Vec<(u64, PathBuf, bool)> = Vec::new();
        let mut entries = read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            // Only process .seg files; sidecar (.seg.meta) and others are ignored.
            if path.extension().and_then(|e| e.to_str()) != Some("seg") {
                continue;
            }

            match SegmentReader::open(&path).await {
                Ok(reader) => {
                    let segment_id = reader.header().segment_id;
                    segments.push((segment_id, path, reader.is_full()));
                }
                Err(e) => {
                    warn!(
                        "Cannot open segment {:?} for inspection, skipping: {e}",
                        path.file_name().unwrap_or_default()
                    );
                }
            }
        }

        segments.sort_by_key(|&(id, _, _)| id);
        Ok(segments)
    }

    /// Opens an existing segment for reading by its numeric `segment_id`.
    ///
    /// # Errors
    ///
    /// Returns an error if the segment file does not exist or cannot be read.
    pub async fn get_reader(&self, segment_id: u64) -> Result<SegmentReader> {
        let path = self.segment_path(segment_id);
        SegmentReader::open(&path)
            .await
            .map_err(|e| eyre!("Segment {segment_id} not found or unreadable: {e}"))
    }

    /// Returns a [`SegmentWriter`] and its associated exclusive Redis lock for the
    /// oldest non-full, unlocked segment.  If the open segment is currently locked
    /// by another writer, a new segment is created immediately.
    ///
    /// **Coordination** — before choosing or allocating a segment, this method
    /// acquires a blocking exclusive Redis lock on `segments.info`
    /// (`LockOperation::Segment(SegmentLockTarget::Info)`).  The lock is held only
    /// for the duration of the metadata read-modify-write cycle (a few
    /// microseconds), so contention is minimal. The per-segment lock
    /// (`SegmentLockTarget::File`) is acquired separately and returned to the
    /// caller, who holds it for the entire write session.
    ///
    /// # Errors
    ///
    /// Returns an error if the segment directory cannot be created, the metadata
    /// file cannot be read or written, or a segment cannot be opened/created.
    pub(super) async fn get_segment_writer(&self) -> Result<(SegmentWriter, PoolLockRedis)> {
        let dir = &self.config.path.pool_segments_path;
        create_dir_all(dir).await?;

        let redis_url = self.config.redis_url();

        // ── Step 1: acquire the exclusive metadata lock ───────────────────────
        let info_lock = PoolLockRedis::new_with_path(
            &redis_url,
            segments_info_path(dir),
            LockOperation::Segment(SegmentLockTarget::Info),
        )
        .await?
        .lock_exclusive()
        .await?;

        // ── Step 2: read or bootstrap SegmentsInformation ────────────────────
        let info = read_segments_info(dir).await?;

        let (mut first_open, mut next_id) = match &info {
            Some(i) => (i.first_open_segment_id, i.next_segment_id),
            None => {
                // First run or legacy directory — fall back to a full scan.
                let segments = self.list_segments().await?;
                let first_open = segments
                    .iter()
                    .find(|&&(_, _, full)| !full)
                    .map_or(0, |&(id, _, _)| id);
                let next_id = segments.last().map_or(0, |&(id, _, _)| id + 1);
                (first_open, next_id)
            }
        };
        let generation = info.as_ref().map_or(0, |i| i.generation);

        // ── Step 3: try the open segment, fall through to creation if locked ──
        let seg_lock_op = LockOperation::Segment(SegmentLockTarget::File);

        // `cursor` walks forward independently of `first_open`.
        // `first_open` is the persisted hint and is only advanced when the segment
        // AT the current `first_open` boundary is confirmed full — never when it is
        // merely locked by another writer.
        //
        // Example layout: FFFFFOOOFFFOOO (F=full, O=open)
        //   first_open starts at 5 (first O). If 5 is locked, cursor moves to 6 but
        //   first_open stays at 5. Encountering a full segment at 8 should NOT move
        //   first_open because segment 5 is still open.
        let mut cursor = first_open;

        let (writer, seg_lock) = 'pick: {
            loop {
                // All existing segments scanned — none was available.
                if cursor >= next_id {
                    break;
                }

                let candidate_path = self.segment_path(cursor);

                let is_open = match SegmentReader::open(&candidate_path).await {
                    Ok(reader) => !reader.is_full(),
                    Err(_) => false, // file missing or corrupt — treat as done
                };

                if !is_open {
                    // Only advance the persisted hint when we are at its boundary;
                    // a full segment further ahead must not move first_open past
                    // still-open (but locked) segments that precede it.
                    if cursor == first_open {
                        first_open += 1;
                    }
                    cursor += 1;
                    continue;
                }

                // Segment is open — try to grab the per-file lock without blocking.
                let lock =
                    PoolLockRedis::new_with_path(&redis_url, &candidate_path, seg_lock_op.clone())
                        .await?
                        .try_lock_exclusive_nowait()
                        .await?;

                if let Some(lock) = lock {
                    debug!(
                        segment_id = cursor,
                        "Reusing open segment {:?}",
                        candidate_path.file_name().unwrap_or_default()
                    );
                    let writer = SegmentWriter::open(&candidate_path).await?;
                    break 'pick (writer, lock);
                }

                // Segment is locked by another writer — try the next one.
                // Do NOT advance first_open: the segment is open, just busy.
                debug!(
                    segment_id = cursor,
                    "Open segment {:?} locked by another writer — trying next",
                    candidate_path.file_name().unwrap_or_default()
                );
                cursor += 1;
            }

            // No available open segment found — create a new one at next_id.
            let new_path = self.segment_path(next_id);
            debug!(
                segment_id = next_id,
                "No available open segment — creating new segment {:?}",
                new_path.file_name().unwrap_or_default()
            );

            let lock = PoolLockRedis::new_with_path(&redis_url, &new_path, seg_lock_op)
                .await?
                .lock_exclusive()
                .await?;

            let writer =
                SegmentWriter::create(&new_path, next_id, DEFAULT_SEGMENT_TARGET_SIZE).await?;

            // Update first_open only when all previous segments were full (first_open
            // advanced all the way to next_id). If first_open < next_id there are
            // still open (but locked) segments earlier — keep pointing at the first one.
            if first_open >= next_id {
                first_open = next_id;
            }
            next_id += 1;

            (writer, lock)
        };

        // ── Step 4: persist updated SegmentsInformation and release meta lock ─
        let updated = SegmentsInformation {
            first_open_segment_id: first_open,
            next_segment_id: next_id,
            generation: generation + 1,
        };
        write_segments_info_atomic(dir, &updated).await?;
        drop(info_lock);

        Ok((writer, seg_lock))
    }

    /// Returns a [`SegmentsWriter`] backed by this segment collection.
    ///
    /// The writer starts on the oldest non-full segment (creating one if
    /// needed) and automatically rotates to the next segment once the current
    /// one reaches its target size.
    ///
    /// # Errors
    ///
    /// Returns an error if the segment directory cannot be created, an
    /// existing segment cannot be reopened, or a new segment cannot be
    /// created.
    pub async fn get_writer(&self) -> Result<SegmentsWriter<'_>> {
        SegmentsWriter::new(self).await
    }

    // ── Compaction helpers ─────────────────────────────────────────────────

    /// Returns the persisted sidecar metadata for every segment file found in
    /// the segment directory.
    ///
    /// Files whose sidecar (`.seg.meta`) is missing or unreadable are silently
    /// skipped with a warning.  The list is sorted by `segment_id` ascending.
    ///
    /// # Errors
    ///
    /// Returns an error only if the segment directory itself cannot be read.
    pub async fn list_segments_metadata(&self) -> Result<Vec<SegmentFileMetadata>> {
        let dir = &self.config.path.pool_segments_path;

        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut result: Vec<SegmentFileMetadata> = Vec::new();
        let mut entries = read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("seg") {
                continue;
            }

            match read_persisted_segment_file_metadata(&path).await {
                Ok(meta) => result.push(meta),
                Err(e) => {
                    warn!(
                        "Cannot read sidecar for {:?}, skipping: {e}",
                        path.file_name().unwrap_or_default()
                    );
                }
            }
        }

        result.sort_by_key(|m| m.segment_id);
        Ok(result)
    }

    /// Returns fill reports for all segments in the directory (sorted by id).
    ///
    /// # Errors
    ///
    /// Returns an error only if the segment directory cannot be read.
    pub async fn list_fill_reports(&self) -> Result<Vec<SegmentFillReport>> {
        let metas = self.list_segments_metadata().await?;
        Ok(metas.iter().map(SegmentFillReport::from).collect())
    }

    /// Returns only the segments that are candidates for compaction.
    ///
    /// A segment is a candidate when:
    /// - its state is `Full` (sealed, no new writes expected), and
    /// - its fill rate is strictly below `threshold` (0.0–1.0).
    ///
    /// The list is sorted by fill rate ascending (worst-utilised first).
    ///
    /// # Errors
    ///
    /// Returns an error if the segment directory cannot be read.
    pub async fn find_compaction_candidates(
        &self,
        threshold: f64,
    ) -> Result<Vec<SegmentFileMetadata>> {
        let mut candidates: Vec<SegmentFileMetadata> = self
            .list_segments_metadata()
            .await?
            .into_iter()
            .filter(|m| {
                m.state == SegmentFileState::Full
                    && SegmentFillReport::from(m).fill_rate() < threshold
            })
            .collect();

        // Worst-utilised segments first so the caller can compact in priority order.
        candidates.sort_by(|a, b| {
            SegmentFillReport::from(a)
                .fill_rate()
                .partial_cmp(&SegmentFillReport::from(b).fill_rate())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(candidates)
    }

    /// Marks a segment as `Compacted` by rewriting its sidecar metadata.
    ///
    /// After this call the segment file is safe to delete: its live chunks have
    /// already been copied to a new segment and the index has been updated.
    ///
    /// # Errors
    ///
    /// Returns an error if the sidecar cannot be read or written.
    pub async fn mark_segment_compacted(&self, segment_id: u64) -> Result<()> {
        let seg_path = self.segment_path(segment_id);
        let mut meta = read_persisted_segment_file_metadata(&seg_path).await?;
        meta.state = SegmentFileState::Compacted;
        write_segment_file_metadata(&seg_path, &meta).await
    }

    /// Deletes the segment file and its sidecar for `segment_id`.
    ///
    /// Both files are removed independently; a missing sidecar is silently
    /// ignored so that partially-written compaction states are handled cleanly.
    ///
    /// # Errors
    ///
    /// Returns an error if the main `.seg` file cannot be removed.
    pub async fn delete_segment(&self, segment_id: u64) -> Result<()> {
        let seg_path = self.segment_path(segment_id);
        let meta_path = segment_sidecar_metadata_path(&seg_path);

        remove_file(&seg_path)
            .await
            .map_err(|e| eyre!("failed to delete segment {segment_id}: {e}"))?;

        // Sidecar removal is best-effort; ignore NotFound.
        if let Err(e) = remove_file(&meta_path).await {
            if e.kind() != std::io::ErrorKind::NotFound {
                warn!("Failed to delete sidecar for segment {segment_id}: {e}");
            }
        }

        Ok(())
    }

    /// Scans the segment directory for segments whose state is `Compacted` and
    /// deletes them.
    ///
    /// Call this at startup to clean up any segments that were marked as
    /// `Compacted` but not yet deleted before the process was interrupted.
    ///
    /// # Errors
    ///
    /// Returns an error if the segment directory cannot be read.
    pub async fn recover_compacted_segments(&self) -> Result<()> {
        let metas = self.list_segments_metadata().await?;
        for meta in metas {
            if meta.state == SegmentFileState::Compacted {
                debug!(
                    segment_id = meta.segment_id,
                    "Recovering: deleting compacted segment"
                );
                self.delete_segment(meta.segment_id).await?;
            }
        }
        Ok(())
    }

    /// Updates the `dead_stored_bytes` field in the sidecar for `segment_id`.
    ///
    /// `dead_stored_bytes` is the total footprint of dead index entries (chunk
    /// header + compressed payload for each), as returned by
    /// [`IndexSweeper::sweep`].  The update is used by
    /// [`SegmentFillReport::fill_rate`] to decide whether a segment is a
    /// compaction candidate.
    ///
    /// The operation is **best-effort**: if the sidecar cannot be updated, the
    /// segment will appear more full than it actually is and might be skipped
    /// by the next compaction pass.  The following sweep run will recover the
    /// correct value.
    ///
    /// The update is performed atomically: the existing sidecar is read,
    /// `dead_stored_bytes` is replaced, and the sidecar is rewritten.
    ///
    /// # Errors
    ///
    /// Returns an error if the sidecar cannot be read or written.
    pub async fn update_dead_stored_bytes(&self, segment_id: u64, dead_bytes: u64) -> Result<()> {
        let seg_path = self.segment_path(segment_id);
        let mut meta = read_persisted_segment_file_metadata(&seg_path).await?;
        meta.dead_stored_bytes = dead_bytes;
        write_segment_file_metadata(&seg_path, &meta).await
    }
}

// ---------------------------------- tests -----------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::tempdir;

    fn make_config(backup_path: std::path::PathBuf) -> Arc<Configuration> {
        Arc::new(Configuration::from_backup_path(backup_path))
    }

    // ── helpers ──────────────────────────────────────────────────────────────

    /// Build a `Segments` manager whose root is an isolated temp directory.
    async fn setup() -> (tempfile::TempDir, Segments) {
        let dir = tempdir().expect("tempdir");
        let segments = Segments::new(make_config(dir.path().to_path_buf()));
        (dir, segments)
    }

    // ── get_writer on empty directory ─────────────────────────────────────────

    #[tokio::test]
    async fn get_writer_on_empty_dir_creates_first_segment() {
        let (_dir, segments) = setup().await;

        let writer = segments.get_writer().await.expect("get_writer");

        // The new segment must be `segment-0000000000000000.seg`.
        let expected = segments.segment_path(0);
        assert_eq!(writer.path(), expected.as_path());
    }

    // ── two consecutive get_writer calls reuse the same open segment ─────────

    #[tokio::test]
    async fn get_writer_reuses_open_segment() {
        let (_dir, segments) = setup().await;

        let mut w1 = segments.get_writer().await.expect("first get_writer");
        let id1 = w1.path().file_name().unwrap().to_string_lossy().to_string();
        // Shutdown the first writer so the sidecar is properly flushed.
        w1.shutdown().await.expect("shutdown w1");

        let mut w2 = segments.get_writer().await.expect("second get_writer");
        let id2 = w2.path().file_name().unwrap().to_string_lossy().to_string();
        w2.shutdown().await.expect("shutdown w2");

        assert_eq!(id1, id2, "both calls should return the same open segment");
    }

    // ── a segment that is already full triggers creation of a new one ─────────

    #[tokio::test]
    async fn get_writer_creates_new_segment_when_existing_is_full() {
        let (_dir, segments) = setup().await;

        // Create a segment with target_size = 1 byte so the header alone
        // causes SegmentReader::is_full() to return true.
        let path = segments.segment_path(0);
        create_dir_all(path.parent().unwrap())
            .await
            .expect("mkdir segments");
        let mut w = SegmentWriter::create(&path, 0, 1)
            .await
            .expect("create tiny segment");
        w.shutdown().await.expect("shutdown tiny segment");

        // Now the existing segment (ID 0) is full; get_writer must allocate ID 1.
        let mut new_writer = segments.get_writer().await.expect("get_writer after full");
        assert_eq!(
            new_writer.path(),
            segments.segment_path(1).as_path(),
            "should have created segment with id=1"
        );
        new_writer.shutdown().await.expect("shutdown new writer");
    }

    // ── get_reader returns a reader for an existing segment ───────────────────

    #[tokio::test]
    async fn get_reader_opens_existing_segment() {
        let (_dir, segments) = setup().await;

        // Create the segment first.
        let mut w = segments.get_writer().await.expect("get_writer");
        w.shutdown().await.expect("shutdown");

        let reader = segments.get_reader(0).await.expect("get_reader");
        assert_eq!(reader.path(), segments.segment_path(0).as_path());
    }

    // ── get_reader on a missing ID returns an error ───────────────────────────

    #[tokio::test]
    async fn get_reader_on_missing_segment_returns_error() {
        let (_dir, segments) = setup().await;
        let result = segments.get_reader(99).await;
        assert!(result.is_err(), "should fail for non-existent segment");
    }
}
