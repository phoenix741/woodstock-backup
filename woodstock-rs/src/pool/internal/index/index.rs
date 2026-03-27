//! [`ChunkIndex`]: high-level manager for the pool chunk index.
//!
//! The index is sharded into 256 files named `shard_{xx}.idx` (where `xx` is the
//! first byte of the chunk hash in lowercase hex) stored under `{pool_path}/index/`.
//! Each shard is a sorted, binary-searchable file managed by [`ShardReader`] /
//! [`ShardWriter`].
//!
//! # Concurrency model
//!
//! `ChunkIndex` is designed for **single-worker** access: only one process may
//! hold the Redis lock (`LockOperation::Index(IndexLockTarget::Writer)`) at any
//! time.  Within a process, `async/await` cooperative scheduling ensures that all
//! index mutations are sequential — no intra-process synchronization primitives
//! are needed.
//!
//! As a consequence, all mutating methods take `&mut self`, and the in-process
//! shard cache (`HashMap<u8, ShardReader>`) requires no locking.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use eyre::Result;
use tracing::debug;

use crate::config::Configuration;

use super::{index_writer::IndexWriter, ChunkDescriptor, ShardReader};

/// High-level manager for the pool chunk index.
///
/// Shards are loaded lazily on first access and kept in memory until
/// [`invalidate_shard`](Self::invalidate_shard) is called (typically by
/// [`IndexWriter`] after a successful flush).
pub struct ChunkIndex {
    pub(super) config: Arc<Configuration>,
    /// Lazily loaded shard readers, keyed by the first byte of the chunk hash.
    cache: HashMap<u8, ShardReader>,
}

impl ChunkIndex {
    /// Creates a new [`ChunkIndex`] backed by the given configuration.
    ///
    /// No I/O is performed until [`get_chunk`](Self::get_chunk) or
    /// [`get_writer`](Self::get_writer) is called.
    #[must_use]
    pub fn new(config: Arc<Configuration>) -> Self {
        Self {
            config,
            cache: HashMap::new(),
        }
    }

    /// Returns the filesystem path for the shard file corresponding to `shard_id`.
    pub(super) fn shard_path(&self, shard_id: u8) -> PathBuf {
        self.config
            .path
            .pool_index_path
            .join(format!("shard_{shard_id:02x}.idx"))
    }

    /// Looks up a chunk by its SHA-256 `hash`.
    ///
    /// The shard corresponding to `hash[0]` is opened on first access and kept
    /// in the in-memory cache for subsequent lookups.  If the shard file does not
    /// exist on disk, `Ok(None)` is returned immediately.
    ///
    /// # Errors
    ///
    /// Returns an error if the shard file exists but cannot be read or if
    /// protobuf decoding fails.
    pub async fn get_chunk(&mut self, hash: &[u8]) -> Result<Option<ChunkDescriptor>> {
        if hash.is_empty() {
            return Err(eyre::eyre!("chunk hash must not be empty"));
        }

        let shard_id = hash[0];

        // Lazy-load the shard into the cache if not already present.
        if !self.cache.contains_key(&shard_id) {
            let path = self.shard_path(shard_id);

            match ShardReader::open(&path).await {
                Ok(reader) => {
                    debug!(shard_id, path = %path.display(), "loaded shard into cache");
                    self.cache.insert(shard_id, reader);
                }
                Err(e) => {
                    // Distinguish "file not found" (normal: shard not yet created)
                    // from genuine I/O errors.
                    let is_not_found = e.chain().any(|cause| {
                        cause
                            .downcast_ref::<std::io::Error>()
                            .is_some_and(|io| io.kind() == std::io::ErrorKind::NotFound)
                    });

                    if is_not_found {
                        debug!(shard_id, "shard file not found — no entry for this prefix");
                        return Ok(None);
                    }

                    return Err(e);
                }
            }
        }

        // SAFETY: inserted just above if absent.
        let reader = self.cache.get(&shard_id).expect("shard must be in cache");
        reader.get_chunk(hash)
    }

    /// Removes the cached [`ShardReader`] for `shard_id`, forcing the next
    /// [`get_chunk`](Self::get_chunk) call to reload it from disk.
    ///
    /// Called by [`IndexWriter::shutdown`] after atomically replacing a shard
    /// file to ensure stale mmap data is not served.
    pub fn invalidate_shard(&mut self, shard_id: u8) {
        if self.cache.remove(&shard_id).is_some() {
            debug!(shard_id, "shard evicted from cache");
        }
    }

    /// Returns an [`IndexWriter`] that holds an exclusive Redis lock on the
    /// index directory for the duration of the write session.
    ///
    /// **Only one writer may exist at any time** (enforced by the Redis lock).
    /// Attempting to open a second writer while the first is active will block
    /// until the lock is released.
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis lock cannot be acquired.
    pub async fn get_writer(&mut self) -> Result<IndexWriter<'_>> {
        IndexWriter::new(self).await
    }
}

// ─────────────────────────────── tests ───────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::internal::index::index_writer::IndexWriter;
    use tempfile::tempdir;

    fn make_config(backup_path: std::path::PathBuf) -> Arc<Configuration> {
        Arc::new(Configuration::from_backup_path(backup_path))
    }

    fn make_descriptor(hash_byte: u8, segment_id: u64, offset: u64) -> ChunkDescriptor {
        ChunkDescriptor {
            hash: vec![hash_byte; 32],
            segment_id,
            offset,
            size: 1024,
            compressed_size: 512,
            header_size: 10,
            compression_format: 2,
            refcount: 1,
        }
    }

    // ── get_chunk on an empty index ───────────────────────────────────────────

    #[tokio::test]
    async fn get_chunk_on_empty_index() {
        let dir = tempdir().expect("tempdir");
        let mut index = ChunkIndex::new(make_config(dir.path().to_path_buf()));

        let hash = vec![0xABu8; 32];
        let result = index.get_chunk(&hash).await.expect("get_chunk");
        assert!(result.is_none(), "should return None for missing shard");
    }

    // ── get_chunk with empty hash returns an error ────────────────────────────

    #[tokio::test]
    async fn get_chunk_empty_hash_returns_error() {
        let dir = tempdir().expect("tempdir");
        let mut index = ChunkIndex::new(make_config(dir.path().to_path_buf()));

        let result = index.get_chunk(&[]).await;
        assert!(result.is_err(), "empty hash should return an error");
    }

    // ── add via writer then get_chunk ─────────────────────────────────────────

    #[tokio::test]
    async fn get_chunk_after_writer_add() {
        let dir = tempdir().expect("tempdir");
        let mut index = ChunkIndex::new(make_config(dir.path().to_path_buf()));
        let descriptor = make_descriptor(0x42, 7, 9999);

        {
            let mut writer = IndexWriter::new(&mut index).await.expect("get_writer");
            writer.add(descriptor.clone()).await.expect("add");
            writer.shutdown().await.expect("shutdown");
        }

        let found = index
            .get_chunk(&descriptor.hash)
            .await
            .expect("get_chunk")
            .expect("should find the descriptor");

        assert_eq!(found.segment_id, 7);
        assert_eq!(found.offset, 9999);
        assert_eq!(found.refcount, 1);
    }

    // ── invalidate_shard evicts the cache entry ───────────────────────────────

    #[tokio::test]
    async fn invalidate_shard_clears_cache() {
        let dir = tempdir().expect("tempdir");
        let mut index = ChunkIndex::new(make_config(dir.path().to_path_buf()));
        let descriptor = make_descriptor(0x10, 0, 0);

        // Populate shard 0x10 via the writer.
        {
            let mut writer = IndexWriter::new(&mut index).await.expect("get_writer");
            writer.add(descriptor.clone()).await.expect("add");
            writer.shutdown().await.expect("shutdown");
        }

        // Prime the cache.
        let _ = index.get_chunk(&descriptor.hash).await.expect("get_chunk");
        assert!(index.cache.contains_key(&0x10), "cache should be populated");

        // Invalidate.
        index.invalidate_shard(0x10);
        assert!(
            !index.cache.contains_key(&0x10),
            "cache should be empty after invalidation"
        );

        // Re-reading must still succeed (reload from disk).
        let found = index
            .get_chunk(&descriptor.hash)
            .await
            .expect("get_chunk after invalidate")
            .expect("entry should still be on disk");
        assert_eq!(found.segment_id, 0);
    }

    // ── writer merges two writes: refcount is additive ────────────────────────

    #[tokio::test]
    async fn writer_merges_refcount() {
        let dir = tempdir().expect("tempdir");
        let mut index = ChunkIndex::new(make_config(dir.path().to_path_buf()));

        let mut d1 = make_descriptor(0x55, 3, 100);
        d1.refcount = 2;

        // First write: refcount = 2.
        {
            let mut writer = IndexWriter::new(&mut index).await.expect("get_writer 1");
            writer.add(d1.clone()).await.expect("add 1");
            writer.shutdown().await.expect("shutdown 1");
        }

        // Second write: same hash, different segment/offset, refcount = 5.
        let mut d2 = make_descriptor(0x55, 9, 500);
        d2.refcount = 5;
        {
            let mut writer = IndexWriter::new(&mut index).await.expect("get_writer 2");
            writer.add(d2.clone()).await.expect("add 2");
            writer.shutdown().await.expect("shutdown 2");
        }

        let found = index
            .get_chunk(&d1.hash)
            .await
            .expect("get_chunk")
            .expect("merged entry should exist");

        // Metadata from the latest entry, refcount = 2 + 5 = 7.
        assert_eq!(found.refcount, 7, "refcounts should be summed");
        assert_eq!(found.segment_id, d2.segment_id, "metadata from new entry");
        assert_eq!(found.offset, d2.offset, "metadata from new entry");
    }
}
