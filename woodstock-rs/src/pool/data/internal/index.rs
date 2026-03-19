//! Unified Pool V3 index backed by [heed](https://docs.rs/heed)/LMDB.
//!
//! The index lives under `pool/index/` as a LMDB environment and is the authoritative
//! registry for two kinds of entries:
//!
//! * **Chunks** — every deduplicated content block written to the storage pool, keyed by its
//!   raw SHA-256 hash bytes. Each entry carries the chunk's uncompressed/compressed sizes,
//!   compression codec, reference count, and its exact location inside a segment file.
//! * **Segments** — the physical `.seg` files that store chunk payloads, tracked with their
//!   current fill level and lifecycle state (`Open` / `Full`).
//!
//! # Transaction model
//!
//! Read-only look-ups open a short-lived [`heed::RoTxn`] that is committed immediately after
//! the closure returns. Write operations go through [`PoolIndex::with_write_transaction`]
//! which wraps the caller's closure in a single [`heed::RwTxn`] and commits atomically.
//!
//! Both paths participate in automatic error recovery: LMDB map-full, map-resized, and
//! readers-full conditions are detected and healed transparently up to
//! `HEED_RECOVERY_RETRY_LIMIT` attempts before an error is propagated.
//!
//! # Opening
//!
//! [`PoolIndex::open_or_create`] opens the LMDB environment at the given path (creating the
//! directory if needed) and returns a handle. `heed` manages the single-open-per-process
//! invariant internally, so it is safe to call `open_or_create` multiple times for the
//! same path within a process.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use eyre::{eyre, Result, WrapErr};
use heed::types::Bytes;
use heed::{Database, Env, EnvOpenOptions, Error as HeedError, MdbError, RoTxn, RwTxn, WithoutTls};
use prost::{Enumeration, Message};

use super::segment::SegmentFileState;
use crate::utils::compression::CompressionFormat;

const INDEX_FORMAT_VERSION: u32 = 3;
const HEED_MAX_DATABASES: u32 = 5;
const HEED_MAX_READERS: u32 = 512;
const DEFAULT_HEED_MAP_SIZE_BYTES: usize = 4 * 1024 * 1024 * 1024;
const MIN_HEED_MAP_SIZE_BYTES: usize = 256 * 1024 * 1024;
const HEED_MAP_SIZE_ALIGNMENT_BYTES: usize = 64 * 1024 * 1024;
const HEED_MAP_GROWTH_PADDING_BYTES: usize = 256 * 1024 * 1024;
const HEED_RECOVERY_RETRY_LIMIT: usize = 8;

const CHUNKS_DATABASE_NAME: &str = "chunks";
const SEGMENTS_DATABASE_NAME: &str = "segments";
const MERGED_BACKUPS_DATABASE_NAME: &str = "merged_backups";
const REMOVED_BACKUPS_DATABASE_NAME: &str = "removed_backups";
const METADATA_DATABASE_NAME: &str = "metadata";

const METADATA_NEXT_SEGMENT_ID: &[u8] = b"next_segment_id";
const METADATA_FORMAT_VERSION: &[u8] = b"format_version";

type PoolEnv = Env<WithoutTls>;

/// A chunk entry as stored in the pool index.
///
/// One `IndexedChunk` represents a single deduplicated content block. The `hash` field is
/// the canonical LMDB key (raw SHA-256 bytes). All other fields are metadata needed to
/// locate and interpret the chunk payload inside a segment file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedChunk {
    /// Raw SHA-256 hash bytes — uniquely identifies the chunk and serves as the LMDB key.
    pub hash: Vec<u8>,
    /// Uncompressed payload size in bytes.
    pub size: u64,
    /// Compressed payload size in bytes (what is actually stored on disk).
    pub compressed_size: u64,
    /// Compression algorithm applied to the payload.
    pub compression_format: CompressionFormat,
    /// Number of active references to this chunk across all backup manifests.
    /// A value of `0` means the chunk is orphaned and eligible for garbage collection.
    pub ref_count: u64,
    /// Identifier of the segment file that contains this chunk's payload.
    pub segment_id: u64,
    /// Byte offset from the beginning of the segment file to the start of the chunk header.
    pub offset: u64,
    /// Size of the serialised chunk header in bytes (needed to locate the payload start).
    pub chunk_header_size: u64,
}

/// A segment entry as stored in the pool index.
///
/// Each `IndexedSegment` maps a numeric identifier to the corresponding `.seg` file and
/// carries enough metadata to make scheduling decisions (e.g. whether to open a new segment
/// or continue appending to an existing one).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedSegment {
    /// Monotonically increasing identifier that deterministically maps to a file path under
    /// `pool/segments/<id>.seg`.
    pub segment_id: u64,
    /// Lifecycle state of the segment: `Open` accepts new chunks, `Full` is sealed.
    pub state: SegmentFileState,
    /// Physical file size in bytes (total bytes written including headers).
    pub size_total: u64,
    /// Sum of uncompressed chunk sizes for all chunks in this segment.
    pub size_effective: u64,
    /// Maximum allowed `size_total` before the segment is marked `Full`.
    pub size_limit: u64,
    /// Number of chunk entries recorded in this segment.
    pub chunk_count: u64,
}

/// Protobuf-serialisable representation of a chunk entry stored as a LMDB value.
///
/// This is the on-disk wire format. `encode_chunk` / `decode_chunk` convert between
/// this type and the public [`IndexedChunk`].
#[derive(Clone, PartialEq, Message)]
struct IndexedChunkValue {
    #[prost(uint64, tag = "1")]
    size: u64,
    #[prost(uint64, tag = "2")]
    compressed_size: u64,
    #[prost(uint32, tag = "3")]
    compression_format: u32,
    #[prost(uint64, tag = "4")]
    ref_count: u64,
    #[prost(uint64, tag = "5")]
    segment_id: u64,
    #[prost(uint64, tag = "6")]
    offset: u64,
    #[prost(uint64, tag = "7")]
    chunk_header_size: u64,
}

/// Protobuf-serialisable representation of a segment entry stored as a LMDB value.
///
/// This is the on-disk wire format. `encode_segment` / `decode_segment` convert between
/// this type and the public [`IndexedSegment`].
#[derive(Clone, PartialEq, Message)]
struct IndexedSegmentValue {
    #[prost(enumeration = "IndexedSegmentStateValue", tag = "1")]
    state: i32,
    #[prost(uint64, tag = "2")]
    size_total: u64,
    #[prost(uint64, tag = "3")]
    size_effective: u64,
    #[prost(uint64, tag = "4")]
    size_limit: u64,
    #[prost(uint64, tag = "5")]
    chunk_count: u64,
}

/// Protobuf enum tag for the segment lifecycle state.
///
/// Maps 1-to-1 with [`SegmentFileState`]; exists separately so that the public API is
/// decoupled from the wire format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
enum IndexedSegmentStateValue {
    /// Segment is accepting new chunk appends.
    Open = 0,
    /// Segment has reached its size limit and is sealed.
    Full = 1,
}

/// A live LMDB write transaction together with handles to each sub-database.
///
/// Obtained exclusively through [`PoolIndex::with_write_transaction`]. Callers interact with
/// it via the `put_*` / `get_*_for_write` / `delete_*` / `is_backup_*` / `mark_backup_*`
/// associated functions, which take `&mut WriteTransaction<'_>` so that several index
/// mutations can be batched into one atomic commit.
///
/// The transaction is committed automatically when the enclosing closure returns `Ok`.
pub struct WriteTransaction<'env> {
    /// The underlying heed write transaction.
    txn: RwTxn<'env>,
    /// Handle to the chunks sub-database (key = raw SHA-256, value = protobuf).
    chunks_db: Database<Bytes, Bytes>,
    /// Handle to the segments sub-database (key = big-endian u64 id, value = protobuf).
    segments_db: Database<Bytes, Bytes>,
    /// Handle to the merged-backups tombstone database (key = backup id bytes, value = 1).
    merged_backups_db: Database<Bytes, Bytes>,
    /// Handle to the removed-backups tombstone database (key = backup id bytes, value = 1).
    removed_backups_db: Database<Bytes, Bytes>,
    /// Handle to the scalar metadata database (key = UTF-8 label, value = big-endian integer).
    metadata_db: Database<Bytes, Bytes>,
}

/// Handle to the Pool V3 LMDB index.
///
/// `PoolIndex` is cheaply cloneable — all clones share the same underlying [`heed::Env`]
/// and the same `txn_gate` synchronisation primitive. Obtain an instance through
/// [`PoolIndex::open_or_create`].
///
/// # Thread safety
///
/// `PoolIndex` is `Clone + Send + Sync`. Concurrent reads are handled by independent
/// short-lived [`heed::RoTxn`]s. Writes serialise through the LMDB write lock. The
/// `txn_gate` `RwLock` is used exclusively to coordinate [`Self::resize_map`] (which
/// requires a quiescent period) with ongoing transactions.
#[derive(Clone)]
pub struct PoolIndex {
    /// Canonicalised path to the LMDB environment directory.
    root_path: PathBuf,
    /// The heed/LMDB environment. Shared across all clones via internal `Arc`.
    env: PoolEnv,
    /// Sub-database storing chunk entries (key = SHA-256 bytes, value = protobuf).
    chunks_db: Database<Bytes, Bytes>,
    /// Sub-database storing segment entries (key = big-endian u64, value = protobuf).
    segments_db: Database<Bytes, Bytes>,
    /// Tombstone set tracking which backup IDs have already been merged.
    merged_backups_db: Database<Bytes, Bytes>,
    /// Tombstone set tracking which backup IDs have already been removed.
    removed_backups_db: Database<Bytes, Bytes>,
    /// Key-value store for scalar metadata (`format_version`, `next_segment_id`).
    metadata_db: Database<Bytes, Bytes>,
    /// Guards map-resize operations: held as write lock during resize, read lock during
    /// normal transactions so that resizes and transactions are mutually exclusive.
    txn_gate: Arc<RwLock<()>>,
}

impl PoolIndex {
    /// Opens an existing index or initialises a new one at `path`.
    ///
    /// The directory is created if it does not exist. `heed` manages the
    /// single-open-per-process invariant for the LMDB environment internally, so this
    /// function may be called freely for the same path within a process.
    ///
    /// Returns an error if the existing index has an incompatible format version or if the
    /// LMDB environment cannot be opened.
    pub fn open_or_create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let root_path = path.as_ref().to_path_buf();
        fs::create_dir_all(&root_path)?;

        let mut options = EnvOpenOptions::new().read_txn_without_tls();
        options.max_dbs(HEED_MAX_DATABASES);
        options.max_readers(HEED_MAX_READERS);
        options.map_size(configured_initial_map_size_bytes());
        let env = unsafe { options.open(&root_path) }
            .wrap_err_with(|| format!("failed to open heed env {}", root_path.display()))?;

        let (chunks_db, segments_db, merged_backups_db, removed_backups_db, metadata_db) = {
            let mut txn = env.write_txn()?;
            let chunks_db = env.create_database(&mut txn, Some(CHUNKS_DATABASE_NAME))?;
            let segments_db = env.create_database(&mut txn, Some(SEGMENTS_DATABASE_NAME))?;
            let merged_backups_db =
                env.create_database(&mut txn, Some(MERGED_BACKUPS_DATABASE_NAME))?;
            let removed_backups_db =
                env.create_database(&mut txn, Some(REMOVED_BACKUPS_DATABASE_NAME))?;
            let metadata_db = env.create_database(&mut txn, Some(METADATA_DATABASE_NAME))?;

            match metadata_db.get(&txn, METADATA_FORMAT_VERSION)? {
                Some(encoded_format_version) => {
                    let format_version = decode_u32(encoded_format_version)?;
                    if format_version != INDEX_FORMAT_VERSION {
                        return Err(eyre!(
                            "unsupported pool index format version {format_version}, expected {}. Rebuild the index with fsck.",
                            INDEX_FORMAT_VERSION
                        ));
                    }
                }
                None => {
                    let format_version = u32::to_be_bytes(INDEX_FORMAT_VERSION);
                    metadata_db.put(
                        &mut txn,
                        METADATA_FORMAT_VERSION,
                        format_version.as_slice(),
                    )?;
                }
            }
            if metadata_db.get(&txn, METADATA_NEXT_SEGMENT_ID)?.is_none() {
                let next_segment_id = u64::to_be_bytes(1);
                metadata_db.put(
                    &mut txn,
                    METADATA_NEXT_SEGMENT_ID,
                    next_segment_id.as_slice(),
                )?;
            }

            txn.commit()?;

            (
                chunks_db,
                segments_db,
                merged_backups_db,
                removed_backups_db,
                metadata_db,
            )
        };

        Ok(Self {
            root_path: root_path.to_path_buf(),
            env,
            chunks_db,
            segments_db,
            merged_backups_db,
            removed_backups_db,
            metadata_db,
            txn_gate: Arc::new(RwLock::new(())),
        })
    }

    /// Executes `callback` inside a single atomic LMDB write transaction.
    ///
    /// The transaction is committed when `callback` returns `Ok`. If `callback` returns
    /// `Err`, the transaction is rolled back automatically by `heed` on drop.
    ///
    /// Recoverable LMDB errors (`MapFull`, `MapResized`, `ReadersFull`) cause the map to be
    /// resized or stale readers to be cleared before transparently retrying. Unrecoverable
    /// errors, or exhaustion of the retry budget, are propagated as-is.
    pub fn with_write_transaction<T>(
        &self,
        callback: impl Fn(&mut WriteTransaction<'_>) -> Result<T>,
    ) -> Result<T> {
        for attempt in 0..HEED_RECOVERY_RETRY_LIMIT {
            let guard = self.txn_gate.read().expect("pool index txn gate poisoned");
            let result = self.try_with_write_transaction(&callback);
            drop(guard);

            match result {
                Ok(value) => return Ok(value),
                Err(error) if self.try_recover_report(&error, attempt)? => continue,
                Err(error) => return Err(error),
            }
        }

        Err(eyre!(
            "pool index write transaction recovery retry budget exhausted"
        ))
    }

    /// Inserts or replaces a single chunk entry in its own write transaction.
    pub fn add_chunk(&self, chunk: &IndexedChunk) -> Result<()> {
        self.with_write_transaction(|write_txn| Self::put_chunk(write_txn, chunk))
    }

    /// Inserts or replaces multiple chunk entries in a single atomic write transaction.
    ///
    /// Prefer this over repeated [`Self::add_chunk`] calls when batching is possible, as it
    /// issues only one LMDB commit.
    pub fn add_chunks(&self, chunks: &[IndexedChunk]) -> Result<()> {
        self.with_write_transaction(|write_txn| {
            for chunk in chunks {
                Self::put_chunk(write_txn, chunk)?;
            }
            Ok(())
        })
    }

    /// Looks up a chunk by its raw SHA-256 hash bytes.
    ///
    /// Returns `Ok(None)` when no entry exists for the given hash. Entries with `ref_count == 0`
    /// (orphaned/hidden chunks) are still returned — callers must filter if needed.
    pub fn get_chunk(&self, hash: &[u8]) -> Result<Option<IndexedChunk>> {
        self.with_read_transaction(|txn| {
            self.chunks_db
                .get(txn, hash)?
                .map(|bytes| decode_chunk(hash, bytes))
                .transpose()
        })
    }

    /// Lists all chunks with a non-zero reference count (active/visible chunks).
    ///
    /// Orphaned chunks (`ref_count == 0`) are excluded. Use [`Self::list_all_chunks`] to
    /// include them.
    pub fn list_chunks(&self) -> Result<Vec<IndexedChunk>> {
        Ok(self
            .list_all_chunks()?
            .into_iter()
            .filter(|chunk| chunk.ref_count > 0)
            .collect())
    }

    /// Lists every chunk entry regardless of reference count.
    ///
    /// Includes orphaned chunks (`ref_count == 0`). Primarily used by `fsck` and
    /// garbage-collection passes.
    pub fn list_all_chunks(&self) -> Result<Vec<IndexedChunk>> {
        self.with_read_transaction(|txn| {
            let mut chunks = Vec::new();
            for entry in self.chunks_db.iter(txn)? {
                let (hash, bytes) = entry?;
                chunks.push(decode_chunk(hash, bytes)?);
            }
            Ok(chunks)
        })
    }

    /// Removes the chunk entry for the given hash.
    ///
    /// Returns `true` if an entry was deleted, `false` if no entry existed.
    pub fn remove_chunk(&self, hash: &[u8]) -> Result<bool> {
        self.with_write_transaction(|write_txn| Self::delete_chunk(write_txn, hash))
    }

    /// Inserts or replaces a segment entry in its own write transaction.
    pub fn add_segment(&self, segment: &IndexedSegment) -> Result<()> {
        self.with_write_transaction(|write_txn| Self::put_segment(write_txn, segment))
    }

    /// Looks up a segment by its numeric identifier.
    ///
    /// Returns `Ok(None)` when no entry exists for the given `segment_id`.
    pub fn get_segment(&self, segment_id: u64) -> Result<Option<IndexedSegment>> {
        self.with_read_transaction(|txn| {
            let key = encode_u64(segment_id);
            self.segments_db
                .get(txn, &key)?
                .map(|bytes| decode_segment(segment_id, bytes))
                .transpose()
        })
    }

    /// Returns whether the given backup publication was already merged into the logical index.
    pub fn backup_is_merged(&self, backup_id: &[u8]) -> Result<bool> {
        self.with_read_transaction(|txn| Ok(self.merged_backups_db.get(txn, backup_id)?.is_some()))
    }

    /// Lists all segment entries ordered by segment identifier (ascending).
    pub fn list_segments(&self) -> Result<Vec<IndexedSegment>> {
        self.with_read_transaction(|txn| {
            let mut segments = Vec::new();
            for entry in self.segments_db.iter(txn)? {
                let (key, bytes) = entry?;
                segments.push(decode_segment(decode_u64(key)?, bytes)?);
            }
            Ok(segments)
        })
    }

    /// Lists only segments in the `Open` state (i.e. that accept new chunk appends).
    pub fn list_open_segments(&self) -> Result<Vec<IndexedSegment>> {
        Ok(self
            .list_segments()?
            .into_iter()
            .filter(|segment| segment.state == SegmentFileState::Open)
            .collect())
    }

    /// Atomically allocates and returns the next available segment identifier.
    ///
    /// The counter stored under `METADATA_NEXT_SEGMENT_ID` is incremented as part of the
    /// same write transaction. The allocated ID is guaranteed to be strictly greater than any
    /// segment ID already present in the segments database, so it is safe to use even after a
    /// crash-recovery that left gaps in the identifier sequence.
    pub fn allocate_next_segment_id(&self) -> Result<u64> {
        self.with_write_transaction(|write_txn| {
            let highest_existing_id = highest_segment_id(&write_txn.txn, write_txn.segments_db)?;
            let current_next_segment_id = read_u64_metadata(
                &write_txn.txn,
                write_txn.metadata_db,
                METADATA_NEXT_SEGMENT_ID,
            )?
            .unwrap_or(1);
            let next_segment_id =
                current_next_segment_id.max(highest_existing_id.saturating_add(1));
            let following_segment_id = next_segment_id
                .checked_add(1)
                .ok_or_else(|| eyre!("segment identifier overflow"))?;
            write_u64_metadata(
                &mut write_txn.txn,
                write_txn.metadata_db,
                METADATA_NEXT_SEGMENT_ID,
                following_segment_id,
            )?;
            Ok(next_segment_id)
        })
    }

    /// Removes the segment entry for the given identifier.
    ///
    /// Returns `true` if an entry was deleted, `false` if no entry existed.
    pub fn remove_segment(&self, segment_id: u64) -> Result<bool> {
        self.with_write_transaction(|write_txn| Self::delete_segment(write_txn, segment_id))
    }

    /// Serialises `chunk` and writes it into the chunks sub-database within `write_txn`.
    ///
    /// Intended for use inside [`Self::with_write_transaction`] closures when multiple index
    /// mutations must be batched into one commit.
    pub(crate) fn put_chunk(
        write_txn: &mut WriteTransaction<'_>,
        chunk: &IndexedChunk,
    ) -> Result<()> {
        write_txn
            .chunks_db
            .put(&mut write_txn.txn, &chunk.hash, &encode_chunk(chunk))?;
        Ok(())
    }

    /// Reads a chunk entry using the read view of an active write transaction.
    ///
    /// This allows read-modify-write operations (e.g. incrementing `ref_count`) within a
    /// single atomic commit without opening a separate read transaction.
    pub(crate) fn get_chunk_for_write(
        write_txn: &mut WriteTransaction<'_>,
        hash: &[u8],
    ) -> Result<Option<IndexedChunk>> {
        write_txn
            .chunks_db
            .get(&write_txn.txn, hash)?
            .map(|bytes| decode_chunk(hash, bytes))
            .transpose()
    }

    /// Deletes the chunk entry for `hash` within `write_txn`.
    ///
    /// Returns `true` if an entry was present and deleted.
    pub(crate) fn delete_chunk(write_txn: &mut WriteTransaction<'_>, hash: &[u8]) -> Result<bool> {
        Ok(write_txn.chunks_db.delete(&mut write_txn.txn, hash)?)
    }

    /// Serialises `segment` and writes it into the segments sub-database within `write_txn`.
    pub(crate) fn put_segment(
        write_txn: &mut WriteTransaction<'_>,
        segment: &IndexedSegment,
    ) -> Result<()> {
        let key = encode_u64(segment.segment_id);
        write_txn
            .segments_db
            .put(&mut write_txn.txn, &key, &encode_segment(segment))?;
        Ok(())
    }

    /// Reads a segment entry using the read view of an active write transaction.
    ///
    /// Mirrors `get_chunk_for_write` for segment read-modify-write patterns.
    pub(crate) fn get_segment_for_write(
        write_txn: &mut WriteTransaction<'_>,
        segment_id: u64,
    ) -> Result<Option<IndexedSegment>> {
        let key = encode_u64(segment_id);
        write_txn
            .segments_db
            .get(&write_txn.txn, &key)?
            .map(|bytes| decode_segment(segment_id, bytes))
            .transpose()
    }

    /// Deletes the segment entry for `segment_id` within `write_txn`.
    ///
    /// Returns `true` if an entry was present and deleted.
    pub(crate) fn delete_segment(
        write_txn: &mut WriteTransaction<'_>,
        segment_id: u64,
    ) -> Result<bool> {
        let key = encode_u64(segment_id);
        Ok(write_txn.segments_db.delete(&mut write_txn.txn, &key)?)
    }

    /// Returns `true` if `backup_id` has been recorded as fully merged.
    ///
    /// Used to implement idempotent merge operations: the caller checks this flag before
    /// reprocessing a backup, ensuring that a partial merge is never applied twice.
    pub(crate) fn is_backup_merged(
        write_txn: &mut WriteTransaction<'_>,
        backup_id: &[u8],
    ) -> Result<bool> {
        Ok(write_txn
            .merged_backups_db
            .get(&write_txn.txn, backup_id)?
            .is_some())
    }

    /// Records `backup_id` as fully merged in the tombstone database.
    ///
    /// Subsequent calls to [`Self::is_backup_merged`] for the same id will return `true`.
    pub(crate) fn mark_backup_merged(
        write_txn: &mut WriteTransaction<'_>,
        backup_id: &[u8],
    ) -> Result<()> {
        write_txn
            .merged_backups_db
            .put(&mut write_txn.txn, backup_id, &[1])?;
        Ok(())
    }

    /// Returns `true` if `backup_id` has been recorded as removed.
    ///
    /// Mirrors [`Self::is_backup_merged`] for the removal tombstone database.
    pub(crate) fn is_backup_removed(
        write_txn: &mut WriteTransaction<'_>,
        backup_id: &[u8],
    ) -> Result<bool> {
        Ok(write_txn
            .removed_backups_db
            .get(&write_txn.txn, backup_id)?
            .is_some())
    }

    /// Records `backup_id` as removed in the tombstone database.
    ///
    /// Subsequent calls to [`Self::is_backup_removed`] for the same id will return `true`.
    pub(crate) fn mark_backup_removed(
        write_txn: &mut WriteTransaction<'_>,
        backup_id: &[u8],
    ) -> Result<()> {
        write_txn
            .removed_backups_db
            .put(&mut write_txn.txn, backup_id, &[1])?;
        Ok(())
    }

    /// Returns the path to the LMDB environment directory.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.root_path
    }

    /// Attempts a single write transaction without retry logic.
    ///
    /// Opens a write transaction, runs `callback`, and commits on success. Called by
    /// [`Self::with_write_transaction`] which adds the retry/recovery loop on top.
    fn try_with_write_transaction<T>(
        &self,
        callback: impl Fn(&mut WriteTransaction<'_>) -> Result<T>,
    ) -> Result<T> {
        let txn = self.env.write_txn()?;
        let mut write_txn = WriteTransaction {
            txn,
            chunks_db: self.chunks_db,
            segments_db: self.segments_db,
            merged_backups_db: self.merged_backups_db,
            removed_backups_db: self.removed_backups_db,
            metadata_db: self.metadata_db,
        };

        let result = callback(&mut write_txn)?;
        write_txn.txn.commit()?;
        Ok(result)
    }

    /// Executes `callback` inside a short-lived LMDB read transaction with automatic recovery.
    ///
    /// Mirrors [`Self::with_write_transaction`] for read-only use cases. Retries up to
    /// `HEED_RECOVERY_RETRY_LIMIT` times on recoverable LMDB errors.
    fn with_read_transaction<T>(
        &self,
        callback: impl Fn(&RoTxn<'_, WithoutTls>) -> Result<T>,
    ) -> Result<T> {
        for attempt in 0..HEED_RECOVERY_RETRY_LIMIT {
            let guard = self.txn_gate.read().expect("pool index txn gate poisoned");
            let result = self.try_with_read_transaction(&callback);
            drop(guard);

            match result {
                Ok(value) => return Ok(value),
                Err(error) if self.try_recover_report(&error, attempt)? => continue,
                Err(error) => return Err(error),
            }
        }

        Err(eyre!(
            "pool index read transaction recovery retry budget exhausted"
        ))
    }

    /// Attempts a single read transaction without retry logic.
    ///
    /// Called by [`Self::with_read_transaction`] which adds the retry/recovery loop on top.
    fn try_with_read_transaction<T>(
        &self,
        callback: impl Fn(&RoTxn<'_, WithoutTls>) -> Result<T>,
    ) -> Result<T> {
        let txn = self.env.read_txn()?;
        let result = callback(&txn);
        txn.commit()?;
        result
    }

    /// Inspects `error` and attempts to recover from a known transient LMDB failure.
    ///
    /// Returns `Ok(true)` if recovery succeeded and the caller should retry, `Ok(false)` if
    /// the error is not recoverable or the retry budget is exhausted, or `Err` if the
    /// recovery itself failed.
    fn try_recover_report(&self, error: &eyre::Report, attempt: usize) -> Result<bool> {
        if attempt + 1 >= HEED_RECOVERY_RETRY_LIMIT {
            return Ok(false);
        }

        let Some(heed_error) = find_heed_error(error) else {
            return Ok(false);
        };

        self.try_recover_heed_error(heed_error)
    }

    /// Dispatches recovery based on the concrete LMDB error variant.
    ///
    /// * `MapFull` → [`Self::resize_map`] (grow by 1.5× or to current disk usage + padding)
    /// * `MapResized` → [`Self::refresh_map_size`] (sync the in-process view with the file)
    /// * `ReadersFull` → [`Self::clear_stale_readers`] (reap dead reader slots)
    fn try_recover_heed_error(&self, error: &HeedError) -> Result<bool> {
        match error {
            HeedError::Mdb(MdbError::MapFull) => {
                self.resize_map(None)?;
                Ok(true)
            }
            HeedError::Mdb(MdbError::MapResized) => {
                self.refresh_map_size()?;
                Ok(true)
            }
            HeedError::Mdb(MdbError::ReadersFull) => Ok(self.clear_stale_readers()? > 0),
            _ => Ok(false),
        }
    }

    /// Grows the LMDB memory-map to accommodate additional data.
    ///
    /// The new size is the maximum of:
    /// * `requested_minimum_size` (caller hint)
    /// * 1.5× the current map size (to amortise frequent resizes)
    /// * current on-disk usage + `HEED_MAP_GROWTH_PADDING_BYTES`
    /// * the configured initial map size
    ///
    /// The result is aligned to `HEED_MAP_SIZE_ALIGNMENT_BYTES`. Acquires `txn_gate` as a
    /// write lock to ensure no transactions are in flight during the `unsafe` resize call.
    fn resize_map(&self, requested_minimum_size: Option<usize>) -> Result<()> {
        let _guard = self.txn_gate.write().expect("pool index txn gate poisoned");
        let env_info = self.env.info();
        let current_map_size = env_info.map_size;
        let disk_size = usize::try_from(self.env.real_disk_size()?)
            .wrap_err("failed to convert heed disk size to usize")?;
        let growth_floor = current_map_size
            .checked_add(current_map_size / 2)
            .ok_or_else(|| eyre!("heed map size growth overflow"))?;
        let based_on_disk = disk_size
            .checked_add(HEED_MAP_GROWTH_PADDING_BYTES)
            .ok_or_else(|| eyre!("heed map size disk growth overflow"))?;
        let target_size = requested_minimum_size
            .unwrap_or(0)
            .max(growth_floor)
            .max(based_on_disk)
            .max(configured_initial_map_size_bytes());

        unsafe { self.env.resize(align_map_size(target_size)) }?;
        Ok(())
    }

    /// Re-synchronises the in-process mmap view after an external resize.
    ///
    /// Called when LMDB reports `MapResized`, which means another process (e.g. a parallel
    /// `fsck` run) has grown the environment file since this process last opened it.
    /// Acquires `txn_gate` as a write lock before the `unsafe` resize call.
    fn refresh_map_size(&self) -> Result<()> {
        let _guard = self.txn_gate.write().expect("pool index txn gate poisoned");
        let env_info = self.env.info();
        let disk_size = usize::try_from(self.env.real_disk_size()?)
            .wrap_err("failed to convert heed disk size to usize")?;
        let target_size = disk_size
            .checked_add(HEED_MAP_GROWTH_PADDING_BYTES)
            .ok_or_else(|| eyre!("heed map size refresh overflow"))?
            .max(env_info.map_size)
            .max(configured_initial_map_size_bytes());

        unsafe { self.env.resize(align_map_size(target_size)) }?;
        Ok(())
    }

    /// Reaps LMDB reader slots left behind by dead processes or threads.
    ///
    /// Returns the number of stale slots cleared. Called when LMDB reports `ReadersFull`.
    /// Acquires `txn_gate` as a write lock before calling into LMDB.
    fn clear_stale_readers(&self) -> Result<usize> {
        let _guard = self.txn_gate.write().expect("pool index txn gate poisoned");
        Ok(self.env.clear_stale_readers()?)
    }
}

/// Returns the initial LMDB map size in bytes.
///
/// Reads the `WOODSTOCK_POOL_INDEX_MAP_SIZE_BYTES` environment variable and parses it as a
/// `usize`. If the variable is absent the default `DEFAULT_HEED_MAP_SIZE_BYTES` (4 GiB) is
/// used. In either case the result is clamped to `MIN_HEED_MAP_SIZE_BYTES` and aligned to
/// `HEED_MAP_SIZE_ALIGNMENT_BYTES`.
///
/// # Panics
///
/// Panics if the environment variable is set but cannot be parsed as a valid integer.
fn configured_initial_map_size_bytes() -> usize {
    match std::env::var("WOODSTOCK_POOL_INDEX_MAP_SIZE_BYTES") {
        Ok(value) => {
            let parsed = value.parse::<usize>().unwrap_or_else(|error| {
                panic!("invalid WOODSTOCK_POOL_INDEX_MAP_SIZE_BYTES value {value}: {error}")
            });
            align_map_size(parsed.max(MIN_HEED_MAP_SIZE_BYTES))
        }
        Err(_) => DEFAULT_HEED_MAP_SIZE_BYTES,
    }
}

/// Rounds `size` up to the next multiple of `HEED_MAP_SIZE_ALIGNMENT_BYTES`, clamped to
/// at least `MIN_HEED_MAP_SIZE_BYTES`.
///
/// LMDB requires the map size to be a multiple of the OS page size (typically 4 KiB), but
/// we use a larger alignment to limit the frequency of resize operations.
fn align_map_size(size: usize) -> usize {
    let clamped = size.max(MIN_HEED_MAP_SIZE_BYTES);
    let remainder = clamped % HEED_MAP_SIZE_ALIGNMENT_BYTES;
    if remainder == 0 {
        clamped
    } else {
        clamped + (HEED_MAP_SIZE_ALIGNMENT_BYTES - remainder)
    }
}

/// Walks the `eyre` error chain and returns the first [`HeedError`] found, if any.
///
/// Used by the recovery logic to distinguish LMDB-specific errors (which can be healed)
/// from unrelated failures.
fn find_heed_error(error: &eyre::Report) -> Option<&HeedError> {
    error
        .chain()
        .find_map(|cause| cause.downcast_ref::<HeedError>())
}

/// Returns the highest segment identifier currently stored in `segments_db`.
///
/// Iterates the entire segments database (ordered by big-endian key) and returns the last
/// key decoded as a `u64`. Returns `0` when the database is empty. Used by
/// [`PoolIndex::allocate_next_segment_id`] to guarantee that the next ID is strictly greater
/// than any existing one even after a crash left gaps in the sequence.
fn highest_segment_id(txn: &RoTxn<'_>, segments_db: Database<Bytes, Bytes>) -> Result<u64> {
    let mut highest_segment_id = 0;
    for entry in segments_db.iter(txn)? {
        let (key, _) = entry?;
        highest_segment_id = decode_u64(key)?;
    }
    Ok(highest_segment_id)
}

/// Reads a `u64` value from the metadata sub-database.
///
/// Returns `Ok(None)` when the key does not exist.
fn read_u64_metadata(
    txn: &RoTxn<'_>,
    metadata_db: Database<Bytes, Bytes>,
    key: &[u8],
) -> Result<Option<u64>> {
    metadata_db.get(txn, key)?.map(decode_u64).transpose()
}

/// Writes a `u64` value into the metadata sub-database.
///
/// The value is encoded as 8 big-endian bytes. Overwrites any existing entry for `key`.
fn write_u64_metadata(
    txn: &mut RwTxn<'_>,
    metadata_db: Database<Bytes, Bytes>,
    key: &[u8],
    value: u64,
) -> Result<()> {
    let encoded = encode_u64(value);
    metadata_db.put(txn, key, &encoded)?;
    Ok(())
}

/// Serialises an [`IndexedChunk`] to its protobuf wire format.
fn encode_chunk(chunk: &IndexedChunk) -> Vec<u8> {
    IndexedChunkValue {
        size: chunk.size,
        compressed_size: chunk.compressed_size,
        compression_format: chunk.compression_format.as_u32(),
        ref_count: chunk.ref_count,
        segment_id: chunk.segment_id,
        offset: chunk.offset,
        chunk_header_size: chunk.chunk_header_size,
    }
    .encode_to_vec()
}

/// Deserialises a chunk entry from its protobuf wire format.
///
/// `hash` is the LMDB key (raw SHA-256 bytes) and is embedded directly into the returned
/// [`IndexedChunk`] since it is not part of the protobuf value.
fn decode_chunk(hash: &[u8], bytes: &[u8]) -> Result<IndexedChunk> {
    let chunk = IndexedChunkValue::decode(bytes)?;

    Ok(IndexedChunk {
        hash: hash.to_vec(),
        size: chunk.size,
        compressed_size: chunk.compressed_size,
        compression_format: CompressionFormat::try_from(chunk.compression_format)?,
        ref_count: chunk.ref_count,
        segment_id: chunk.segment_id,
        offset: chunk.offset,
        chunk_header_size: chunk.chunk_header_size,
    })
}

/// Serialises an [`IndexedSegment`] to its protobuf wire format.
fn encode_segment(segment: &IndexedSegment) -> Vec<u8> {
    IndexedSegmentValue {
        state: encode_segment_state(segment.state) as i32,
        size_total: segment.size_total,
        size_effective: segment.size_effective,
        size_limit: segment.size_limit,
        chunk_count: segment.chunk_count,
    }
    .encode_to_vec()
}

/// Deserialises a segment entry from its protobuf wire format.
///
/// `segment_id` is the LMDB key decoded as a `u64` and is embedded directly into the
/// returned [`IndexedSegment`] since it is not part of the protobuf value.
fn decode_segment(segment_id: u64, bytes: &[u8]) -> Result<IndexedSegment> {
    let segment = IndexedSegmentValue::decode(bytes)?;

    Ok(IndexedSegment {
        segment_id,
        state: decode_segment_state(segment.state)?,
        size_total: segment.size_total,
        size_effective: segment.size_effective,
        size_limit: segment.size_limit,
        chunk_count: segment.chunk_count,
    })
}

/// Encodes a `u64` as 8 big-endian bytes for use as a LMDB key or metadata value.
///
/// Big-endian encoding preserves numerical ordering under LMDB's lexicographic key sort,
/// which allows `iter` to visit segment IDs in ascending order without an explicit comparator.
fn encode_u64(value: u64) -> [u8; 8] {
    value.to_be_bytes()
}

/// Decodes a big-endian `u64` from a byte slice.
///
/// Returns an error if `bytes` is not exactly 8 bytes long.
fn decode_u64(bytes: &[u8]) -> Result<u64> {
    let array: [u8; 8] = bytes
        .try_into()
        .map_err(|_| eyre!("invalid u64 byte length: {}", bytes.len()))?;
    Ok(u64::from_be_bytes(array))
}

/// Decodes a big-endian `u32` from a byte slice.
///
/// Used to read the format version stored in the metadata database.
/// Returns an error if `bytes` is not exactly 4 bytes long.
fn decode_u32(bytes: &[u8]) -> Result<u32> {
    let array: [u8; 4] = bytes
        .try_into()
        .map_err(|_| eyre!("invalid u32 byte length: {}", bytes.len()))?;
    Ok(u32::from_be_bytes(array))
}

/// Converts the public [`SegmentFileState`] into the protobuf enum tag.
fn encode_segment_state(state: SegmentFileState) -> IndexedSegmentStateValue {
    match state {
        SegmentFileState::Open => IndexedSegmentStateValue::Open,
        SegmentFileState::Full => IndexedSegmentStateValue::Full,
    }
}

/// Converts a raw protobuf enum discriminant back into the public [`SegmentFileState`].
///
/// Returns an error if `value` does not correspond to a known [`IndexedSegmentStateValue`].
fn decode_segment_state(value: i32) -> Result<SegmentFileState> {
    match IndexedSegmentStateValue::try_from(value)
        .map_err(|_| eyre!("invalid segment state in index: {value}"))?
    {
        IndexedSegmentStateValue::Open => Ok(SegmentFileState::Open),
        IndexedSegmentStateValue::Full => Ok(SegmentFileState::Full),
    }
}

#[cfg(test)]
mod tests {
    use super::{IndexedChunk, IndexedSegment, PoolIndex};

    use eyre::Result;
    use tempfile::tempdir;

    use crate::pool::data::SegmentFileState;
    use crate::utils::compression::CompressionFormat;

    fn sample_chunk(hash_seed: u8) -> IndexedChunk {
        IndexedChunk {
            hash: vec![hash_seed; 32],
            size: 4096,
            compressed_size: 1024,
            compression_format: CompressionFormat::Zstd,
            ref_count: 1,
            segment_id: 7,
            offset: 128,
            chunk_header_size: 24,
        }
    }

    fn sample_segment(segment_id: u64) -> IndexedSegment {
        IndexedSegment {
            segment_id,
            state: SegmentFileState::Open,
            size_total: 1024,
            size_effective: 768,
            size_limit: 4096,
            chunk_count: 3,
        }
    }

    #[test]
    fn create_and_reopen_index() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("index");

        let index = PoolIndex::open_or_create(&path)?;
        drop(index);

        let reopened = PoolIndex::open_or_create(&path)?;
        assert!(reopened.get_chunk(&[1; 32])?.is_none());
        assert!(reopened.get_segment(1)?.is_none());
        Ok(())
    }

    #[test]
    fn add_and_get_chunk() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("index");
        let index = PoolIndex::open_or_create(&path)?;
        let chunk = sample_chunk(0x11);

        index.add_chunk(&chunk)?;
        assert_eq!(index.get_chunk(&chunk.hash)?, Some(chunk));
        Ok(())
    }

    #[test]
    fn add_chunks_persists_all_entries_in_single_transaction() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("index");
        let index = PoolIndex::open_or_create(&path)?;
        let chunks = vec![sample_chunk(0x11), sample_chunk(0x22), sample_chunk(0x33)];

        index.add_chunks(&chunks)?;

        assert_eq!(index.list_all_chunks()?, chunks);
        Ok(())
    }

    #[test]
    fn get_chunk_returns_hidden_chunk() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("index");
        let index = PoolIndex::open_or_create(&path)?;
        let mut chunk = sample_chunk(0x21);
        chunk.ref_count = 0;

        index.add_chunk(&chunk)?;

        assert_eq!(index.get_chunk(&chunk.hash)?, Some(chunk));
        Ok(())
    }

    #[test]
    fn add_and_get_segment() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("index");
        let index = PoolIndex::open_or_create(&path)?;
        let segment = sample_segment(7);

        index.add_segment(&segment)?;
        assert_eq!(index.get_segment(segment.segment_id)?, Some(segment));
        Ok(())
    }

    #[test]
    fn allocate_next_segment_id_skips_existing_ids() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("index");
        let index = PoolIndex::open_or_create(&path)?;
        index.add_segment(&sample_segment(7))?;

        assert_eq!(index.allocate_next_segment_id()?, 8);
        assert_eq!(index.allocate_next_segment_id()?, 9);
        Ok(())
    }

    #[test]
    fn list_open_segments_filters_full_entries() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("index");
        let index = PoolIndex::open_or_create(&path)?;
        let first = sample_segment(1);
        let mut second = sample_segment(2);
        second.state = SegmentFileState::Full;
        let third = sample_segment(3);

        index.add_segment(&second)?;
        index.add_segment(&third)?;
        index.add_segment(&first)?;

        let stored_ids: Vec<u64> = index
            .list_open_segments()?
            .into_iter()
            .map(|segment| segment.segment_id)
            .collect();

        assert_eq!(stored_ids, vec![1, 3]);
        Ok(())
    }

    #[test]
    fn list_all_chunks_keeps_hidden_entries() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("index");
        let index = PoolIndex::open_or_create(&path)?;
        let visible = sample_chunk(0x31);
        let mut hidden = sample_chunk(0x32);
        hidden.ref_count = 0;

        index.add_chunk(&hidden)?;
        index.add_chunk(&visible)?;

        let visible_hashes = index
            .list_chunks()?
            .into_iter()
            .map(|chunk| chunk.hash)
            .collect::<Vec<_>>();
        let all_hashes = index
            .list_all_chunks()?
            .into_iter()
            .map(|chunk| chunk.hash)
            .collect::<Vec<_>>();

        assert_eq!(visible_hashes, vec![visible.hash.clone()]);
        assert_eq!(all_hashes, vec![visible.hash, hidden.hash]);
        Ok(())
    }

    #[test]
    fn open_or_create_returns_index_at_path() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("index");

        let index = PoolIndex::open_or_create(&path)?;

        assert_eq!(index.path(), path.as_path());
        Ok(())
    }
}
