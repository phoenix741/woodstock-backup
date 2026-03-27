//! [`Segments`]: manager for the collection of append-only segment files.
//!
//! Each segment file is named `segment-{id:016x}.seg` and lives in
//! `{pool_path}/segments/`.  The manager can open any segment for reading by
//! its numeric ID, and always returns a writable handle to the oldest
//! non-full segment (creating a new one when all existing segments have
//! reached their target size).

use std::path::PathBuf;
use std::sync::Arc;

use eyre::{eyre, Result};
use tokio::fs::{create_dir_all, read_dir};
use tracing::{debug, warn};

use crate::config::Configuration;

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

    /// Returns a [`SegmentWriter`] for the oldest non-full segment, creating a
    /// fresh one if all existing segments have reached their target size.
    ///
    /// This is the low-level building block used by [`Segments::get_writer`] and
    /// [`SegmentsWriter`] for rotation.  Prefer [`get_writer`](Self::get_writer)
    /// for normal use.
    pub(super) async fn get_segment_writer(&self) -> Result<SegmentWriter> {
        create_dir_all(&self.config.path.pool_segments_path).await?;

        let segments = self.list_segments().await?;

        // Return the oldest (lowest ID) open segment.
        if let Some((id, path, _)) = segments.iter().find(|&&(_, _, full)| !full) {
            debug!(
                segment_id = id,
                "Reusing open segment {:?}",
                path.file_name().unwrap_or_default()
            );
            return SegmentWriter::open(path).await;
        }

        // All segments are full or none exist — allocate a new one.
        let new_id = segments.last().map_or(0, |&(id, _, _)| id + 1);
        let new_path = self.segment_path(new_id);
        debug!(
            segment_id = new_id,
            "Creating new segment {:?}",
            new_path.file_name().unwrap_or_default()
        );
        SegmentWriter::create(&new_path, new_id, DEFAULT_SEGMENT_TARGET_SIZE).await
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
