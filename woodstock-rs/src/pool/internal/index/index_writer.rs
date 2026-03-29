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
//! // writer.remove(descriptor).await?;
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
//! that received at least one [`add`](IndexWriter::add) or
//! [`remove`](IndexWriter::remove) call:
//!
//! 1. The existing shard (if any) is read entirely into memory.
//! 2. Pending entries are pre-merged into a `(descriptor, delta: i64)` map keyed
//!    by hash: deltas are cumulated; metadata comes from the last **add** entry
//!    (positive delta).
//! 3. The pre-merged map is combined with the existing shard entries:
//!    - `final_refcount = (existing.refcount as i64 + total_delta).max(0) as u64`
//!    - Metadata wins from the pending add entry when present; otherwise the
//!      existing shard metadata is kept unchanged.
//!    - New hashes (not yet in the shard) are inserted only when `delta > 0`.
//!    - Existing entries whose refcount drops to 0 are **kept** (GC delegated to
//!      `ws_console clean-unused`).
//! 4. The merged list is sorted by hash ascending.
//! 5. A new shard file is written atomically (temp file → `rename`) by
//!    [`ShardWriter`] with an incremented generation counter.
//! 6. The corresponding shard is evicted from the [`ChunkIndex`] cache so that
//!    the next read reloads the fresh file.

use std::collections::HashMap;
use std::path::Path;

use eyre::Result;
use futures::StreamExt;
use tokio::fs::create_dir_all;
use tracing::debug;

use crate::pool::staging::StagingReader;
use crate::utils::lock_redis::{IndexLockTarget, LockOperation, PoolLockRedis};

use super::{index::ChunkIndex, ChunkDescriptor, ShardReader, ShardWriter, SignedChunkDescriptor};

/// Exclusive, atomic writer for the pool chunk index.
///
/// Obtain via [`ChunkIndex::get_writer`].
pub struct IndexWriter<'a> {
    index: &'a mut ChunkIndex,
    /// Exclusive Redis lock on the index directory, held for the duration of the
    /// write session.  Dropped (i.e. released) when `shutdown` is called.
    lock: PoolLockRedis,
    /// Accumulated signed entries, bucketed by shard id (= `hash[0]`).
    pending: HashMap<u8, Vec<SignedChunkDescriptor>>,
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

    /// Queues a [`ChunkDescriptor`] for *addition* on [`shutdown`](Self::shutdown).
    ///
    /// The descriptor's `refcount` is used as the positive delta applied to the
    /// stored refcount during flush.  Entries are bucketed by `descriptor.hash[0]`
    /// (the shard id).
    ///
    /// # Errors
    ///
    /// Returns an error if the hash is empty.
    pub async fn add(&mut self, descriptor: ChunkDescriptor) -> Result<()> {
        if descriptor.hash.is_empty() {
            return Err(eyre::eyre!("chunk descriptor hash must not be empty"));
        }

        let shard_id = descriptor.hash[0];
        self.pending
            .entry(shard_id)
            .or_default()
            .push(SignedChunkDescriptor::for_add(descriptor));
        Ok(())
    }

    /// Queues a [`ChunkDescriptor`] for *removal* on [`shutdown`](Self::shutdown).
    ///
    /// The descriptor's `refcount` is used as the magnitude of the negative delta
    /// applied to the stored refcount during flush.  If the stored refcount would
    /// drop below 0 it is clamped to 0; the entry is **not** deleted from the
    /// shard (GC is handled separately by `ws_console clean-unused`).
    ///
    /// Entries are bucketed by `descriptor.hash[0]` (the shard id).
    ///
    /// # Errors
    ///
    /// Returns an error if the hash is empty.
    pub async fn remove(&mut self, descriptor: ChunkDescriptor) -> Result<()> {
        if descriptor.hash.is_empty() {
            return Err(eyre::eyre!("chunk descriptor hash must not be empty"));
        }

        let shard_id = descriptor.hash[0];
        self.pending
            .entry(shard_id)
            .or_default()
            .push(SignedChunkDescriptor::for_remove(descriptor));
        Ok(())
    }

    /// Reads every entry from the staging file at `path` and queues each one as
    /// an *add* operation.
    ///
    /// This is equivalent to calling [`add`](Self::add) for every
    /// [`ChunkDescriptor`] yielded by a [`StagingReader`] opened on `path`.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened, read, or if any descriptor
    /// has an empty hash.
    pub async fn add_staging(&mut self, path: &Path) -> Result<()> {
        let mut reader = StagingReader::open(path)
            .await
            .map_err(|e| eyre::eyre!("failed to open staging file {}: {}", path.display(), e))?;

        let stream = reader.into_stream();
        futures::pin_mut!(stream);

        while let Some(item) = stream.next().await {
            let descriptor = item.map_err(|e| {
                eyre::eyre!(
                    "failed to read staging entry from {}: {}",
                    path.display(),
                    e
                )
            })?;
            self.add(descriptor).await?;
        }
        Ok(())
    }

    /// Reads every entry from the staging file at `path` and queues each one as
    /// a *remove* operation.
    ///
    /// This is equivalent to calling [`remove`](Self::remove) for every
    /// [`ChunkDescriptor`] yielded by a [`StagingReader`] opened on `path`.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened, read, or if any descriptor
    /// has an empty hash.
    pub async fn remove_staging(&mut self, path: &Path) -> Result<()> {
        let mut reader = StagingReader::open(path)
            .await
            .map_err(|e| eyre::eyre!("failed to open staging file {}: {}", path.display(), e))?;

        let stream = reader.into_stream();
        futures::pin_mut!(stream);

        while let Some(item) = stream.next().await {
            let descriptor = item.map_err(|e| {
                eyre::eyre!(
                    "failed to read staging entry from {}: {}",
                    path.display(),
                    e
                )
            })?;
            self.remove(descriptor).await?;
        }
        Ok(())
    }

    /// Flushes all pending entries to disk and releases the Redis lock.
    ///
    /// For each modified shard the signed deltas are merged with the on-disk
    /// state (see [module docs](self) for the detailed algorithm), the result is
    /// sorted by hash, and written atomically via [`ShardWriter`].
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
        let buckets: Vec<(u8, Vec<SignedChunkDescriptor>)> = self.pending.drain().collect();
        for (shard_id, new_entries) in buckets {
            self.flush_shard(shard_id, new_entries).await?;
        }

        // Explicitly release the lock — the Drop impl also handles panics.
        self.lock.unlock().await?;
        Ok(())
    }

    /// Merges `new_entries` (signed) into the shard identified by `shard_id`
    /// and writes the result atomically.
    async fn flush_shard(
        &mut self,
        shard_id: u8,
        new_entries: Vec<SignedChunkDescriptor>,
    ) -> Result<()> {
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

        // ── Step 2: pre-merge pending entries into (descriptor, delta) map ───
        // For each hash we accumulate the total signed delta.  Metadata is taken
        // from the last entry that carries a positive (add) delta; if no add
        // entry exists for that hash, we fall back to the existing shard entry
        // later.
        let mut pre_merged: HashMap<Vec<u8>, (ChunkDescriptor, i64)> = HashMap::new();
        for signed in new_entries {
            let hash = signed.descriptor.hash.clone();
            let entry = pre_merged.entry(hash).or_insert_with(|| {
                // Seed with a zeroed-refcount clone of the descriptor as the
                // initial metadata placeholder.
                let mut seed = signed.descriptor.clone();
                seed.refcount = 0;
                (seed, 0_i64)
            });
            entry.1 = entry.1.saturating_add(signed.delta);
            // Last positive-delta entry wins for metadata.
            if signed.delta > 0 {
                entry.0 = signed.descriptor;
                entry.0.refcount = 0; // refcount is managed via delta; reset here
            }
        }

        // ── Step 3: merge pre_merged with existing entries ───────────────────
        let mut merged: HashMap<Vec<u8>, ChunkDescriptor> = existing_entries
            .into_iter()
            .map(|d| (d.hash.clone(), d))
            .collect();

        for (hash, (pending_meta, total_delta)) in pre_merged {
            if let Some(existing) = merged.get_mut(&hash) {
                // Hash exists in the shard: adjust refcount and update metadata
                // when we have a positive delta (i.e. an add operation).
                existing.refcount = existing.refcount.saturating_add_signed(total_delta);
                if total_delta > 0 {
                    // Update storage metadata from the pending add entry,
                    // preserving the freshly computed refcount.
                    let refcount = existing.refcount;
                    *existing = pending_meta;
                    existing.refcount = refcount;
                }
            } else if total_delta > 0 {
                // New hash: insert only when we are adding references.
                #[allow(clippy::cast_sign_loss)]
                let mut new_entry = pending_meta;
                new_entry.refcount = total_delta as u64;
                merged.insert(hash, new_entry);
            }
            // total_delta <= 0 and hash absent → nothing to remove; ignore.
        }

        // ── Step 4: sort by hash ascending ───────────────────────────────────
        let mut sorted: Vec<ChunkDescriptor> = merged.into_values().collect();
        sorted.sort_unstable_by(|a, b| a.hash.cmp(&b.hash));

        // ── Step 5: ensure directory exists ──────────────────────────────────
        if let Some(parent) = shard_path.parent() {
            create_dir_all(parent).await?;
        }

        // ── Step 6: write atomically via ShardWriter ──────────────────────────
        let mut writer = ShardWriter::create(&shard_path, next_generation).await?;
        for entry in &sorted {
            writer.write(entry).await?;
        }
        writer.flush().await?;

        // ── Step 7: invalidate the cache entry ────────────────────────────────
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
