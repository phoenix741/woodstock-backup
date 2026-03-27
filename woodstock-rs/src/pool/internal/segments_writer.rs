//! [`SegmentsWriter`]: a multi-segment append-only writer with automatic rotation.
//!
//! This writer exposes the same append API as [`SegmentWriter`] but transparently
//! manages segment transitions: once the current segment becomes full after a
//! write, it is shut down and the next available (or freshly created) segment is
//! opened before the next call.

use std::path::Path;

use eyre::Result;
use tracing::debug;

use crate::utils::compression::CompressionFormat;
use crate::utils::lock_redis::PoolLockRedis;

use super::{
    segments::Segments, SegmentChunkEntry, SegmentFileHeader, SegmentFileMetadata,
    SegmentFileState, SegmentWriter,
};

/// A multi-segment, append-only writer that automatically rotates to the next
/// segment when the current one reaches its target size.
///
/// Segments are obtained from a shared [`Segments`] manager: the writer always
/// uses the oldest non-full segment, creating a new one when all existing
/// segments have reached their target size — exactly the same policy as
/// [`Segments::get_writer`].
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
/// // Open a rotating writer backed by the segment collection.
/// let mut writer = segments.get_writer().await?;
///
/// // Append chunks — rotation happens automatically when a segment is full.
/// // let entry = writer.append_chunk_from_path(hash, size, compressed_size, fmt, &path).await?;
/// # Ok(())
/// # }
/// ```
pub struct SegmentsWriter<'a> {
    segments: &'a Segments,
    current: SegmentWriter,
    /// Exclusive Redis lock held on the current segment file.
    /// Released when the writer rotates to a new segment or is shut down.
    lock: Option<PoolLockRedis>,
}

impl<'a> SegmentsWriter<'a> {
    /// Creates a new [`SegmentsWriter`] backed by the given [`Segments`] manager.
    ///
    /// The inner writer is obtained by calling [`Segments::get_segment_writer`],
    /// which uses the oldest non-full segment or creates a fresh one.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying segment cannot be opened or created.
    pub(super) async fn new(segments: &'a Segments) -> Result<Self> {
        let (current, lock) = segments.get_segment_writer().await?;
        Ok(Self {
            segments,
            current,
            lock: Some(lock),
        })
    }

    /// Returns the filesystem path of the **current** segment file.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.current.path()
    }

    /// Returns the immutable header of the **current** segment.
    #[must_use]
    pub fn header(&self) -> &SegmentFileHeader {
        self.current.header()
    }

    /// Returns the persisted sidecar metadata of the **current** segment.
    #[must_use]
    pub fn file_metadata(&self) -> &SegmentFileMetadata {
        self.current.file_metadata()
    }

    /// Returns the total physical size of the **current** segment.
    #[must_use]
    pub fn size_total(&self) -> u64 {
        self.current.size_total()
    }

    /// Returns whether the **current** segment has reached its target size.
    ///
    /// After a successful [`append_chunk_from_path`](Self::append_chunk_from_path)
    /// call, this may transiently return `true` before the internal rotation is
    /// triggered on the next append.  Following the rotation the value resets to
    /// `false` (unless the new segment is itself immediately full, which only
    /// happens with a target size of 0 or 1).
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.current.is_full()
    }

    /// Returns the derived state (`Open` / `Full`) of the **current** segment.
    #[must_use]
    pub fn state(&self) -> SegmentFileState {
        self.current.state()
    }

    /// Returns remaining capacity in the **current** segment before its target
    /// size.
    #[must_use]
    pub fn remaining_capacity(&self) -> u64 {
        self.current.remaining_capacity()
    }

    /// Flushes and closes the **current** segment writer.
    ///
    /// After calling this, the [`SegmentsWriter`] should not be used again.
    ///
    /// # Errors
    ///
    /// Returns an error if flushing or closing the underlying file fails.
    pub async fn shutdown(&mut self) -> Result<()> {
        self.current.shutdown().await?;
        if let Some(lock) = self.lock.take() {
            lock.unlock().await?;
        }
        Ok(())
    }

    /// Appends a chunk payload (stored at `source_path`) to the **current**
    /// segment, rotating to the next segment if the current one becomes full
    /// after the write.
    ///
    /// # Rotation semantics
    ///
    /// The returned [`SegmentChunkEntry`] always references the segment that
    /// *received* the chunk (i.e. the pre-rotation segment).  The rotation — if
    /// triggered — happens *after* the successful write, so the caller can rely
    /// on the entry's `header_offset` and `payload_offset` being valid in that
    /// segment.
    ///
    /// # Errors
    ///
    /// Returns an error if the append fails or if the segment rotation cannot
    /// open or create the next segment.
    pub async fn append_chunk_from_path(
        &mut self,
        hash: Vec<u8>,
        size: u64,
        compressed_size: u64,
        compression_format: CompressionFormat,
        source_path: &Path,
    ) -> Result<SegmentChunkEntry> {
        let entry = self
            .current
            .append_chunk_from_path(hash, size, compressed_size, compression_format, source_path)
            .await?;

        // If the segment reached its target size, rotate to the next one so
        // that subsequent appends don't have to wait.
        if self.current.is_full() {
            debug!(
                segment_id = self.current.header().segment_id,
                "Segment is full after append — rotating to next segment"
            );
            self.current.shutdown().await?;
            let (new_writer, new_lock) = self.segments.get_segment_writer().await?;
            // Release the old lock after acquiring the new one to avoid a gap.
            if let Some(old_lock) = self.lock.replace(new_lock) {
                old_lock.unlock().await?;
            }
            self.current = new_writer;
        }

        Ok(entry)
    }
}
