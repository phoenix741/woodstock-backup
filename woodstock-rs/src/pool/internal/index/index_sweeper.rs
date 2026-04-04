//! [`IndexSweeper`]: removes zero-refcount entries from the chunk index shards.
//!
//! An `IndexSweeper` is obtained via [`IndexSweeper::new`].  It holds the same
//! exclusive Redis lock as [`super::index_writer::IndexWriter`] for the
//! duration of the sweep so that no concurrent writer can race with the shard
//! rewrite.
//!
//! # Purpose
//!
//! The [`super::index_writer::IndexWriter`] intentionally keeps entries whose
//! `refcount` drops to 0 in the shard (GC is delegated).  `IndexSweeper` is
//! the GC pass: it removes those tombstone entries and returns a list of
//! [`DeadChunkRecord`]s that the caller uses to update segment sidecar
//! accounting (`dead_compressed_bytes`).
//!
//! # Atomicity / crash safety
//!
//! Each shard is processed independently.  For each shard that contains dead
//! entries:
//!
//! 1. The shard is read entirely into memory.
//! 2. Live entries (`refcount > 0`) are separated from dead ones.
//! 3. The shard is rewritten atomically (temp file → `rename`) containing
//!    only the live entries, with an incremented generation counter.
//! 4. The [`ChunkIndex`] cache entry for the shard is invalidated.
//!
//! If the process crashes mid-sweep, at most one shard may be partially
//! processed on disk (the rename is atomic on POSIX).  On the next sweep run
//! the already-cleaned shards will simply have no dead entries to remove, and
//! the untouched shards will be cleaned normally.  No corruption is possible.
//!
//! # Usage
//!
//! ```rust,no_run
//! # use std::sync::Arc;
//! # use woodstock::pool::{ChunkIndex, Segments};
//! # use woodstock::config::Configuration;
//! # use woodstock::pool::internal::IndexSweeper;
//! # async fn example(config: Arc<Configuration>) -> eyre::Result<()> {
//! let mut index = ChunkIndex::new(Arc::clone(&config));
//! let segments = Segments::new(Arc::clone(&config));
//!
//! // Segment sidecar `dead_stored_bytes` fields are updated automatically.
//! let result = IndexSweeper::new(&mut index).await?.sweep(&segments, None).await?;
//! println!("Removed {} dead entries", result.dead_records.len());
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;

use eyre::Result;
use tokio::fs::create_dir_all;
use tokio::sync::mpsc;
use tracing::debug;

use crate::utils::lock_redis::{IndexLockTarget, LockOperation, PoolLockRedis};

use super::super::segment::{Segments, SweepProgression};
use super::index::ChunkIndex;
use super::{ChunkDescriptor, ShardReader, ShardWriter};

/// A single entry removed from the index during a sweep pass.
///
/// Contains enough information for the caller to attribute the freed bytes back
/// to the correct segment sidecar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeadChunkRecord {
    /// SHA-256 hash of the chunk that was removed.
    pub hash: Vec<u8>,
    /// ID of the segment that physically stores the chunk payload.
    pub segment_id: u64,
    /// Total bytes freed in the segment file: chunk header + compressed payload.
    /// Corresponds to [`SegmentChunkEntry::stored_len`] and is what
    /// [`Segments::update_dead_stored_bytes`] expects.
    pub stored_size: u64,
}

/// Aggregated outcome of a completed sweep pass.
#[derive(Debug, Clone)]
pub struct SweepResult {
    /// All index entries that were found with `refcount == 0` and removed.
    pub dead_records: Vec<DeadChunkRecord>,
    /// Number of shards that were inspected (always 256 unless the index
    /// directory is empty, in which case it may be 0).
    pub shards_processed: usize,
}

impl SweepResult {
    /// Groups dead records by `segment_id` and sums `stored_size` for each.
    ///
    /// Returned as a `HashMap<segment_id, total_dead_stored_bytes>`.  Pass
    /// the values to [`Segments::update_dead_stored_bytes`] to keep sidecar
    /// accounting accurate.
    #[must_use]
    pub fn dead_bytes_per_segment(&self) -> HashMap<u64, u64> {
        let mut map: HashMap<u64, u64> = HashMap::new();
        for record in &self.dead_records {
            *map.entry(record.segment_id).or_default() += record.stored_size;
        }
        map
    }
}

/// Exclusive sweep pass over all 256 index shards.
///
/// Holds the same Redis write-lock as [`super::index_writer::IndexWriter`];
/// only one writer or sweeper may run at a time.
pub struct IndexSweeper<'a> {
    index: &'a mut ChunkIndex,
    lock: PoolLockRedis,
}

impl<'a> IndexSweeper<'a> {
    /// Acquires the exclusive index writer lock and returns a ready sweeper.
    ///
    /// Blocks until the lock is available (up to the global `MAX_WAIT_TIME`).
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis lock cannot be established.
    pub async fn new(index: &'a mut ChunkIndex) -> Result<Self> {
        let redis_url = index.config.redis_url();
        let index_path = &index.config.path.pool_index_path;

        let lock = PoolLockRedis::new_with_path(
            &redis_url,
            index_path,
            LockOperation::Index(IndexLockTarget::Sweeper),
        )
        .await?
        .lock_exclusive()
        .await?;

        debug!(path = %index_path.display(), "acquired exclusive index writer lock for sweep");

        Ok(Self { index, lock })
    }

    /// Scans all 256 shards, removes `refcount == 0` entries, and returns the
    /// aggregated [`SweepResult`].
    ///
    /// After the sweep, each segment sidecar's `dead_stored_bytes` field is
    /// updated via [`Segments::update_dead_stored_bytes`].  Failed updates are
    /// logged as warnings but do not abort the sweep: the sidecar accounting
    /// is a best-effort hint; the canonical source of truth is the index itself.
    ///
    /// An optional `progress_tx` channel receives one [`SweepProgression`]
    /// message after each shard is processed.  Send errors are logged as
    /// warnings but do not abort the sweep.
    ///
    /// Releases the Redis lock when it returns (or on panic via Drop).
    ///
    /// # Errors
    ///
    /// Returns an error if any shard cannot be read or written.
    pub async fn sweep(
        mut self,
        segments: &Segments,
        progress_tx: Option<mpsc::Sender<SweepProgression>>,
    ) -> Result<SweepResult> {
        const TOTAL_SHARDS: usize = 256;

        let mut dead_records: Vec<DeadChunkRecord> = Vec::new();
        let mut shards_done: usize = 0;
        let mut dead_entries_found: usize = 0;
        let mut dead_bytes_accounted: u64 = 0;

        for shard_id in 0u8..=255 {
            let shard_dead = self.sweep_shard(shard_id).await?;

            for record in &shard_dead {
                dead_entries_found += 1;
                dead_bytes_accounted += record.stored_size;
            }
            dead_records.extend(shard_dead);
            shards_done += 1;

            if let Some(tx) = &progress_tx {
                let progression = SweepProgression {
                    shards_total: TOTAL_SHARDS,
                    shards_done,
                    dead_entries_found,
                    dead_bytes_accounted,
                };
                if let Err(e) = tx.send(progression).await {
                    tracing::warn!("Failed to send sweep progression: {e}");
                }
            }
        }

        self.lock.unlock().await?;

        // ── Apply results to segment sidecars (best-effort) ───────────────────
        // We update dead_stored_bytes *after* releasing the index lock so that
        // the sidecar writes don't block index writers unnecessarily.
        let result = SweepResult {
            dead_records,
            shards_processed: shards_done,
        };

        for (segment_id, dead_bytes) in result.dead_bytes_per_segment() {
            if let Err(e) = segments
                .update_dead_stored_bytes(segment_id, dead_bytes)
                .await
            {
                tracing::warn!(
                    segment_id,
                    "Failed to update dead_stored_bytes for segment after sweep: {e}.  \
                     Fill-rate will be overestimated until the next sweep run."
                );
            }
        }

        Ok(result)
    }

    /// Sweeps one shard, returning the dead records found (and rewriting the
    /// shard on disk without them if any were present).
    async fn sweep_shard(&mut self, shard_id: u8) -> Result<Vec<DeadChunkRecord>> {
        let shard_path = self.index.shard_path(shard_id);

        // Load the existing shard — skip silently if it does not exist yet.
        let (entries, next_generation) = match ShardReader::open(&shard_path).await {
            Ok(reader) => {
                let generation = reader.generation().saturating_add(1);
                let entries = reader.entries()?;
                (entries, generation)
            }
            Err(e) => {
                let is_not_found = e.chain().any(|cause| {
                    cause
                        .downcast_ref::<std::io::Error>()
                        .is_some_and(|io| io.kind() == std::io::ErrorKind::NotFound)
                });
                if is_not_found {
                    return Ok(Vec::new());
                }
                return Err(e);
            }
        };

        // Partition into live and dead.
        let mut live: Vec<ChunkDescriptor> = Vec::with_capacity(entries.len());
        let mut dead: Vec<DeadChunkRecord> = Vec::new();

        for entry in entries {
            if entry.refcount == 0 {
                dead.push(DeadChunkRecord {
                    hash: entry.hash.clone(),
                    segment_id: entry.segment_id,
                    // Full footprint in the segment file: header bytes + compressed payload.
                    stored_size: entry.compressed_size + u64::from(entry.header_size),
                });
            } else {
                live.push(entry);
            }
        }

        // If nothing dead: no rewrite needed.
        if dead.is_empty() {
            return Ok(Vec::new());
        }

        debug!(
            shard_id,
            live = live.len(),
            dead = dead.len(),
            generation = next_generation,
            "sweeping shard"
        );

        // Live entries are already sorted (the shard format requires it), but
        // re-sort defensively to guarantee the ShardWriter invariant.
        live.sort_unstable_by(|a, b| a.hash.cmp(&b.hash));

        // Ensure the index directory exists.
        if let Some(parent) = shard_path.parent() {
            create_dir_all(parent).await?;
        }

        // Rewrite atomically.
        let mut writer = ShardWriter::create(&shard_path, next_generation).await?;
        for entry in &live {
            writer.write(entry).await?;
        }
        writer.flush().await?;

        // Invalidate the in-process cache for this shard.
        self.index.invalidate_shard(shard_id);

        Ok(dead)
    }
}
