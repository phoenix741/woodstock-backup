//! [`SegmentCompactor`]: rewrites under-utilised segments into new, denser ones.
//!
//! # Overview
//!
//! After an [`IndexSweeper`](super::super::index::index_sweeper::IndexSweeper)
//! run has removed zero-refcount entries from the index shards and updated the
//! segment sidecar `dead_compressed_bytes` fields, some `Full` segments may
//! have a fill rate well below the configured threshold.  `SegmentCompactor`
//! reclaims that wasted space by:
//!
//! 1. Reading every chunk entry from the source segment.
//! 2. Consulting the index to decide which entries are still live (triple
//!    condition: `refcount > 0`, `segment_id` matches, `offset` matches).
//! 3. Copying each live chunk into the current open segment via
//!    [`SegmentsWriter::append_chunk_from_reader`](super::segments_writer::SegmentsWriter::append_chunk_from_reader).
//! 4. Atomically updating the index for all moved chunks (old location removed,
//!    new location added) via [`IndexWriter`](super::super::index::IndexWriter).
//! 5. Marking the source segment as `Compacted` in its sidecar and deleting it.
//!
//! # Atomicity / crash safety
//!
//! | Crash window                              | State at restart              | Action needed |
//! |-------------------------------------------|-------------------------------|---------------|
//! | During copy (before `IndexWriter::shutdown`) | Source still `Full`, orphans in target | No action — orphans are not indexed |
//! | After index update, before `Compacted` mark | Source still `Full`, 0 live chunks | Next compaction fast-path: fill=0%, mark+delete |
//! | After `Compacted` mark, before delete    | Sidecar says `Compacted`      | `Segments::recover_compacted_segments` at startup |
//!
//! The triple-condition live check prevents a moved chunk from being "live"
//! in the source segment after the index has been updated to point at the
//! target segment.
//!
//! # Usage
//!
//! ```rust,no_run
//! # use std::sync::Arc;
//! # use woodstock::pool::{ChunkIndex, Segments};
//! # use woodstock::pool::internal::SegmentCompactor;
//! # use woodstock::config::Configuration;
//! # async fn example(config: Arc<Configuration>) -> eyre::Result<()> {
//! let mut index = ChunkIndex::new(Arc::clone(&config));
//! let segments = Segments::new(Arc::clone(&config));
//!
//! let compactor = SegmentCompactor::new(Arc::clone(&config));
//! let candidates = segments.find_compaction_candidates(0.8).await?;
//!
//! for candidate in candidates {
//!     compactor
//!         .compact_segment(candidate.segment_id, &mut index, &segments, None)
//!         .await?;
//! }
//! # Ok(())
//! # }
//! ```

use eyre::Result;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use super::super::index::ChunkDescriptor;
use super::super::index::ChunkIndex;
use super::segment_model::{CompactionProgression, SegmentChunkEntry};
use super::segments::Segments;

/// Outcome of successfully compacting one segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionReport {
    /// ID of the segment that was compacted and deleted.
    pub source_segment_id: u64,
    /// Number of live chunk entries copied to a new segment.
    pub chunks_moved: u64,
    /// Physical size in bytes of the source segment file that was freed.
    pub bytes_freed: u64,
}

/// Compacts under-utilised pool segments by rewriting their live chunks.
///
/// # Thread safety
///
/// `SegmentCompactor` is `Clone + Send + Sync`.  Concurrent calls to
/// `compact_segment` on **different** source segments are safe because each
/// call sequences the critical operations under the index writer Redis lock.
/// Do not run two instances concurrently on the **same** source segment.
#[derive(Clone, Default)]
pub struct SegmentCompactor;

impl SegmentCompactor {
    /// Creates a new [`SegmentCompactor`].
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Compacts the segment identified by `source_segment_id`.
    ///
    /// Performs the full compaction sequence described in the [module
    /// documentation](self): read → filter live → copy → update index →
    /// mark Compacted → delete.
    ///
    /// `progress_tx` receives one [`CompactionProgression`] message after each
    /// live chunk is copied.  Send errors are logged as warnings and do not
    /// abort the operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the source segment cannot be read, any chunk copy or
    /// index update fails, or the sidecar cannot be updated.
    pub async fn compact_segment(
        &self,
        source_segment_id: u64,
        index: &mut ChunkIndex,
        segments: &Segments,
        progress_tx: Option<mpsc::Sender<CompactionProgression>>,
    ) -> Result<CompactionReport> {
        // ── Step 1: read all chunk entries from the source segment ────────────
        let source_reader = segments.get_reader(source_segment_id).await?;
        let all_entries = source_reader.chunks().await?;
        let source_size = source_reader.size_total();

        debug!(
            segment_id = source_segment_id,
            total_chunks = all_entries.len(),
            size = source_size,
            "read source segment"
        );

        // ── Step 2: identify live chunks via the triple condition ─────────────
        // live ⇔  refcount > 0  ∧  segment_id == source  ∧  offset == entry.header_offset
        let mut live: Vec<(SegmentChunkEntry, ChunkDescriptor)> = Vec::new();

        for entry in &all_entries {
            match index.get_chunk(&entry.hash).await? {
                Some(desc)
                    if desc.refcount > 0
                        && desc.segment_id == source_segment_id
                        && desc.offset == entry.header_offset =>
                {
                    live.push((entry.clone(), desc));
                }
                _ => {
                    // dead, relocated, or hash collision — skip
                }
            }
        }

        debug!(
            segment_id = source_segment_id,
            live = live.len(),
            dead = all_entries.len() - live.len(),
            "identified live chunks"
        );

        // ── Step 3: fast-path when nothing is live ────────────────────────────
        if live.is_empty() {
            info!(
                segment_id = source_segment_id,
                "no live chunks — fast-path compaction (mark + delete)"
            );
            segments.mark_segment_compacted(source_segment_id).await?;
            segments.delete_segment(source_segment_id).await?;
            return Ok(CompactionReport {
                source_segment_id,
                chunks_moved: 0,
                bytes_freed: source_size,
            });
        }

        // ── Step 4: copy live chunks into new segment(s) ──────────────────────
        // Collect (old_descriptor, new_entry, new_segment_id) for the index update.
        let mut moved: Vec<(ChunkDescriptor, SegmentChunkEntry, u64)> =
            Vec::with_capacity(live.len());

        let total_to_move = live.len();
        let mut chunks_moved_count: u64 = 0;

        {
            let mut writer = segments.get_writer().await?;

            for (entry, old_desc) in &live {
                let chunk_reader = source_reader.chunk_reader(entry).await?;
                let (new_entry, new_seg_id) = writer
                    .append_chunk_from_reader(
                        entry.hash.clone(),
                        entry.size,
                        entry.compressed_size,
                        entry.compression_format,
                        chunk_reader,
                    )
                    .await?;

                chunks_moved_count += 1;

                if let Some(tx) = &progress_tx {
                    let update = CompactionProgression {
                        segments_total: 1,
                        segments_done: 0,
                        chunks_moved: chunks_moved_count,
                        bytes_freed: 0,
                    };
                    if let Err(e) = tx.send(update).await {
                        warn!("Failed to send compaction progression: {e}");
                    }
                }

                moved.push((old_desc.clone(), new_entry, new_seg_id));
            }

            writer.shutdown().await?;
        }

        debug!(
            segment_id = source_segment_id,
            moved = total_to_move,
            "copied live chunks to new segment(s)"
        );

        // ── Step 5: atomically update the index ───────────────────────────────
        // remove(old_location) + add(new_location, same refcount) per chunk.
        // `IndexWriter::shutdown` is the atomic commit point.
        {
            let mut writer = index.get_writer().await?;

            for (old_desc, new_entry, new_seg_id) in &moved {
                let header_size = u32::try_from(new_entry.chunk_header_size)
                    .map_err(|_| eyre::eyre!("chunk_header_size does not fit in u32"))?;

                let new_desc = ChunkDescriptor {
                    hash: new_entry.hash.clone(),
                    segment_id: *new_seg_id,
                    offset: new_entry.header_offset,
                    size: new_entry.size,
                    compressed_size: new_entry.compressed_size,
                    header_size,
                    compression_format: new_entry.compression_format.as_u32(),
                    refcount: old_desc.refcount,
                };

                writer.remove(old_desc.clone()).await?;
                writer.add(new_desc).await?;
            }

            writer.shutdown().await?;
        }

        debug!(
            segment_id = source_segment_id,
            "index updated for moved chunks"
        );

        // ── Step 6: mark source as Compacted then delete ──────────────────────
        segments.mark_segment_compacted(source_segment_id).await?;
        segments.delete_segment(source_segment_id).await?;

        info!(
            segment_id = source_segment_id,
            chunks_moved = total_to_move,
            bytes_freed = source_size,
            "segment compacted and deleted"
        );

        Ok(CompactionReport {
            source_segment_id,
            chunks_moved: u64::try_from(total_to_move)?,
            bytes_freed: source_size,
        })
    }
}
