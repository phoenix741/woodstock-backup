//! [`IndexWriter`]: exclusive, atomic writer for the pool chunk index.
//!
//! An `IndexWriter` is obtained via [`ChunkIndex::get_writer`].  It holds an
//! exclusive Redis lock on the index directory for the entire write session, so
//! **only one writer may exist at any time** across all processes.
//!
//! # Usage
//!
//! ```rust,no_run
//! # use std::sync::Arc;
//! # use woodstock::pool::ChunkIndex;
//! # use woodstock::config::Configuration;
//! # async fn example(config: Arc<Configuration>) -> eyre::Result<()> {
//! let mut index = ChunkIndex::new(config);
//! let mut writer = index.get_writer().await?;
//!
//! // Queue chunk descriptors — they accumulate in memory.
//! // writer.add(descriptor).await?;
//!
//! // Atomically flush each modified shard and release the lock.
//! writer.shutdown().await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Flush semantics
//!
//! Each call to [`shutdown`](IndexWriter::shutdown) processes every shard bucket
//! that received at least one [`add`](IndexWriter::add) call:
//!
//! 1. The existing shard (if any) is read entirely into memory.
//! 2. Existing entries are merged with the pending entries via a `HashMap` keyed
//!    by hash: when the same hash appears in both, the metadata comes from the
//!    **new** entry and the refcount is the **sum** of both refcounts.
//! 3. The merged list is sorted by hash ascending.
//! 4. A new shard file is written atomically (temp file → `rename`) by
//!    [`ShardWriter`] with an incremented generation counter.
//! 5. The corresponding shard is evicted from the [`ChunkIndex`] cache so that
//!    the next read reloads the fresh file.

use std::collections::HashMap;
use std::path::Path;

use eyre::Result;
use tokio::fs::create_dir_all;
use tracing::debug;

use crate::utils::lock_redis::{IndexLockTarget, LockOperation, PoolLockRedis};

use super::{index::ChunkIndex, ChunkDescriptor, ShardReader, ShardWriter};

/// Exclusive, atomic writer for the pool chunk index.
///
/// Obtain via [`ChunkIndex::get_writer`].
pub struct IndexWriter<'a> {
    index: &'a mut ChunkIndex,
    /// Exclusive Redis lock on the index directory, held for the duration of the
    /// write session.  Dropped (i.e. released) when `shutdown` is called.
    lock: PoolLockRedis,
    /// Accumulated entries, bucketed by shard id (= `hash[0]`).
    pending: HashMap<u8, Vec<ChunkDescriptor>>,
}

impl<'a> IndexWriter<'a> {
    /// Creates a new [`IndexWriter`] and acquires an exclusive Redis lock on the
    /// index directory.
    ///
    /// Blocks until the lock is available.
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis lock cannot be established.
    pub(super) async fn new(index: &'a mut ChunkIndex) -> Result<Self> {
        let redis_url = index.config.redis_url();
        let index_path = &index.config.path.pool_index_path;

        let lock = PoolLockRedis::new_with_path(
            &redis_url,
            index_path,
            LockOperation::Index(IndexLockTarget::Writer),
        )
        .await?
        .lock_exclusive()
        .await?;

        debug!(path = %index_path.display(), "acquired exclusive index writer lock");

        Ok(Self {
            index,
            lock,
            pending: HashMap::new(),
        })
    }

    /// Queues a [`ChunkDescriptor`] to be written on [`shutdown`](Self::shutdown).
    ///
    /// Entries are bucketed by `descriptor.hash[0]` (the shard id).  Multiple
    /// entries for the same hash are merged during flush (the refcounts are
    /// accumulated and the metadata from the last-seen entry wins).
    ///
    /// # Errors
    ///
    /// Returns an error if the hash is empty.
    pub async fn add(&mut self, descriptor: ChunkDescriptor) -> Result<()> {
        if descriptor.hash.is_empty() {
            return Err(eyre::eyre!("chunk descriptor hash must not be empty"));
        }

        let shard_id = descriptor.hash[0];
        self.pending.entry(shard_id).or_default().push(descriptor);
        Ok(())
    }

    /// Flushes all pending entries to disk and releases the Redis lock.
    ///
    /// For each modified shard:
    ///
    /// 1. The existing shard file (if any) is read into memory.
    /// 2. Existing and new entries are merged (refcount additive, new metadata
    ///    wins for duplicate hashes).
    /// 3. The merged, sorted entries are written to a temp file and atomically
    ///    renamed to the final shard path (`ShardWriter::flush`).
    /// 4. The shard is evicted from the [`ChunkIndex`] cache.
    ///
    /// The Redis lock is released when this method returns (or on drop if it
    /// panics).
    ///
    /// # Errors
    ///
    /// Returns an error if any shard cannot be read or written.
    pub async fn shutdown(mut self) -> Result<()> {
        // Collect first to avoid a double-mutable-borrow: `drain()` holds a
        // borrow on `self.pending` while `flush_shard` borrows `self` again.
        let buckets: Vec<(u8, Vec<ChunkDescriptor>)> = self.pending.drain().collect();
        for (shard_id, new_entries) in buckets {
            self.flush_shard(shard_id, new_entries).await?;
        }

        // Explicitly release the lock — the Drop impl also handles panics.
        self.lock.unlock().await?;
        Ok(())
    }

    /// Merges `new_entries` into the shard identified by `shard_id` and writes
    /// the result atomically.
    async fn flush_shard(&mut self, shard_id: u8, new_entries: Vec<ChunkDescriptor>) -> Result<()> {
        let shard_path = self.index.shard_path(shard_id);

        // ── Step 1: read the existing shard (if any) ─────────────────────────
        let (existing_entries, next_generation) = load_existing_shard(&shard_path).await?;

        debug!(
            shard_id,
            existing = existing_entries.len(),
            new = new_entries.len(),
            generation = next_generation,
            "flushing shard"
        );

        // ── Step 2: merge by hash ─────────────────────────────────────────────
        // Build a map from the existing entries first; new entries will overwrite
        // metadata but their refcounts are *added* to the existing refcount.
        let mut merged: HashMap<Vec<u8>, ChunkDescriptor> = existing_entries
            .into_iter()
            .map(|d| (d.hash.clone(), d))
            .collect();

        for mut new in new_entries {
            let existing_refcount = merged.get(&new.hash).map_or(0, |e| e.refcount);
            new.refcount = new.refcount.saturating_add(existing_refcount);
            merged.insert(new.hash.clone(), new);
        }

        // ── Step 3: sort by hash ascending ───────────────────────────────────
        let mut sorted: Vec<ChunkDescriptor> = merged.into_values().collect();
        sorted.sort_unstable_by(|a, b| a.hash.cmp(&b.hash));

        // ── Step 4: ensure directory exists ──────────────────────────────────
        if let Some(parent) = shard_path.parent() {
            create_dir_all(parent).await?;
        }

        // ── Step 5: write atomically via ShardWriter ──────────────────────────
        let mut writer = ShardWriter::create(&shard_path, next_generation).await?;
        for entry in &sorted {
            writer.write(entry).await?;
        }
        writer.flush().await?;

        // ── Step 6: invalidate the cache entry ────────────────────────────────
        self.index.invalidate_shard(shard_id);

        Ok(())
    }
}

/// Reads all entries and the current generation from an existing shard file.
///
/// Returns `(entries, next_generation)` where `next_generation = generation + 1`.
/// If the file does not exist, returns an empty `Vec` and generation `0`.
///
/// # Errors
///
/// Propagates I/O and protobuf decode errors for files that exist but are corrupt.
async fn load_existing_shard(path: &Path) -> Result<(Vec<ChunkDescriptor>, u64)> {
    match ShardReader::open(path).await {
        Ok(reader) => {
            let generation = reader.generation().saturating_add(1);
            let entries = reader.entries()?;
            Ok((entries, generation))
        }
        Err(e) => {
            let is_not_found = e.chain().any(|cause| {
                cause
                    .downcast_ref::<std::io::Error>()
                    .is_some_and(|io| io.kind() == std::io::ErrorKind::NotFound)
            });

            if is_not_found {
                Ok((Vec::new(), 0))
            } else {
                Err(e)
            }
        }
    }
}
