use std::{collections::HashMap, path::Path};

use eyre::Result;

use crate::{
    pool::ChunkDescriptor,
    proto::{ProtobufWriter, UnCompressedWriter},
};

use super::staging_reader::StagingReader;

/// Name of the staging file stored in each backup directory.
pub const STAGING_FILENAME: &str = "staging.idx";

/// Writer for the per-backup staging file.
///
/// During a backup, every chunk encountered is appended to this file via
/// [`write`](Self::write).  Duplicates (the same hash appearing multiple times
/// in the same backup) are recorded as-is; [`compact`](Self::compact) later
/// merges them by accumulating their `refcount`.
///
/// The staging file is a flat, **non-compressed** sequence of length-delimited
/// [`ChunkDescriptor`] protobuf messages.  A `flush` is issued after each
/// write so that the file is always in a readable state, even if the process
/// crashes mid-backup.
///
/// ## Typical lifecycle
///
/// ```text
/// let mut sw = StagingWriter::create(&backup_dir).await?;
///
/// // For every chunk transferred:
/// sw.write(&descriptor).await?;
///
/// // At the end of the backup, deduplicate in-place:
/// sw.compact().await?;
///
/// // Drop or store `sw` — the file is now ready for index integration.
/// ```
pub struct StagingWriter {
    writer: ProtobufWriter<UnCompressedWriter, ChunkDescriptor>,
    /// Full path to `staging.idx` inside the backup directory.
    path: std::path::PathBuf,
    /// In-memory descriptor cache keyed by hash.
    ///
    /// The descriptor keeps the first seen storage metadata and accumulates
    /// `refcount` across writes during the current process lifetime.
    cache: HashMap<Vec<u8>, ChunkDescriptor>,
}

impl StagingWriter {
    fn staging_path(backup_dir: &Path) -> std::path::PathBuf {
        backup_dir.join(STAGING_FILENAME)
    }

    /// Creates (or truncates) a staging file in `backup_dir`.
    ///
    /// Use this at the start of a new backup.
    ///
    /// # Errors
    /// Returns an error if the file cannot be created.
    pub async fn create(backup_dir: &Path) -> Result<Self> {
        let path = Self::staging_path(backup_dir);
        let writer = ProtobufWriter::new(&path, false).await?;
        Ok(Self {
            writer,
            path,
            cache: HashMap::new(),
        })
    }

    /// Opens an existing staging file in `backup_dir` for **appending**.
    ///
    /// Use this to resume a backup that was interrupted.
    ///
    /// # Errors
    /// Returns an error if the file cannot be opened.
    pub async fn open(backup_dir: &Path) -> Result<Self> {
        let path = Self::staging_path(backup_dir);
        let writer = ProtobufWriter::open(&path).await?;
        let mut cache: HashMap<Vec<u8>, ChunkDescriptor> = HashMap::new();

        if path.exists() {
            let mut raw: Vec<ChunkDescriptor> = Vec::new();
            let mut reader = StagingReader::open(&path).await?;
            reader.read_to_end(&mut raw).await?;

            for desc in raw {
                let entry = cache.entry(desc.hash.clone()).or_insert_with(|| {
                    let mut initial = desc.clone();
                    initial.refcount = 0;
                    initial
                });
                entry.refcount = entry.refcount.saturating_add(desc.refcount);
            }
        }

        Ok(Self {
            writer,
            path,
            cache,
        })
    }

    /// Appends a [`ChunkDescriptor`] to the staging file and flushes the buffer
    /// to disk immediately so the file is always in a consistent state.
    ///
    /// The same chunk hash may be written multiple times if it appears in
    /// multiple files of the same backup.
    ///
    /// # Errors
    /// Returns an error if writing or flushing fails.
    pub async fn write(&mut self, descriptor: &ChunkDescriptor) -> Result<()> {
        self.writer.write(descriptor).await?;
        self.writer.flush_buffer().await?;

        let entry = self
            .cache
            .entry(descriptor.hash.clone())
            .or_insert_with(|| {
                let mut initial = descriptor.clone();
                initial.refcount = 0;
                initial
            });
        entry.refcount = entry.refcount.saturating_add(descriptor.refcount);

        Ok(())
    }

    /// Returns `true` when `hash` has already been staged in this backup.
    #[must_use]
    pub fn contains_hash(&self, hash: &[u8]) -> bool {
        self.cache.contains_key(hash)
    }

    /// Returns the cached descriptor for `hash` when present.
    #[must_use]
    pub fn get_descriptor(&self, hash: &[u8]) -> Option<ChunkDescriptor> {
        self.cache.get(hash).cloned()
    }

    /// Compacts the staging file in place by merging entries with the same hash.
    ///
    /// ## Algorithm
    ///
    /// 1. Flush any buffered bytes to disk.
    /// 2. Read all existing entries from the staging file.
    /// 3. Accumulate `refcount` for duplicate hashes (metadata of the first
    ///    occurrence is kept for all other fields).
    /// 4. Sort the merged entries by `hash` (ascending) for compatibility with
    ///    the pool index format.
    /// 5. Atomically rewrite the staging file (write to a temp file, then
    ///    `rename`).
    /// 6. Reopen the file in append mode so further [`write`](Self::write)
    ///    calls remain valid.
    ///
    /// # Errors
    /// Returns an error if reading, writing, or renaming fails.
    pub async fn compact(&mut self) -> Result<()> {
        // 1. Flush the current BufWriter so all bytes are on disk.
        self.writer.flush_buffer().await?;

        // 2. `self.cache` is already merged by hash (loaded in `open` and updated in `write`).
        // 3. Sort by hash for index compatibility.
        let mut sorted: Vec<ChunkDescriptor> = self.cache.values().cloned().collect();
        sorted.sort_unstable_by(|a, b| a.hash.cmp(&b.hash));

        // 4. Atomically rewrite: write to temp file then rename.
        {
            let mut new_writer: ProtobufWriter<UnCompressedWriter, ChunkDescriptor> =
                ProtobufWriter::new(&self.path, true).await?;
            for desc in &sorted {
                new_writer.write(desc).await?;
            }
            new_writer.flush().await?;
        }

        // 5. Reopen in append mode so future writes are still valid.
        self.writer = ProtobufWriter::open(&self.path).await?;

        Ok(())
    }

    /// Flushes remaining buffered bytes to disk and closes the writer.
    ///
    /// After this call the staging file is safe to read by [`StagingReader`].
    ///
    /// # Errors
    /// Returns an error if flushing fails.
    pub async fn close(mut self) -> Result<()> {
        self.shutdown().await?;
        Ok(())
    }

    /// Flushes buffered bytes without consuming the writer.
    ///
    /// Prefer this method for long-lived writers managed by a session.
    pub async fn shutdown(&mut self) -> Result<()> {
        self.writer.flush_buffer().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::ChunkDescriptor;
    use std::path::PathBuf;
    use tokio::fs;

    fn make_descriptor(hash_byte: u8, refcount: u64) -> ChunkDescriptor {
        ChunkDescriptor {
            hash: vec![hash_byte; 32],
            segment_id: 1,
            offset: 0,
            size: 64,
            compressed_size: 48,
            header_size: 8,
            compression_format: 0,
            refcount,
        }
    }

    /// Returns the `./data/` directory (relative to the crate root) used by all
    /// protobuf tests in this workspace.
    fn data_dir() -> PathBuf {
        PathBuf::from("./data")
    }

    #[tokio::test]
    async fn test_write_and_read_back() {
        let base_dir = data_dir();
        let backup_dir = base_dir.join("staging_test_backup");
        fs::create_dir_all(&backup_dir).await.unwrap();
        let _ = fs::remove_file(&backup_dir.join(STAGING_FILENAME)).await;

        {
            let mut sw = StagingWriter::create(&backup_dir).await.unwrap();
            sw.write(&make_descriptor(0xAA, 1)).await.unwrap();
            sw.write(&make_descriptor(0xBB, 1)).await.unwrap();
            // Same hash as first — intentional duplicate.
            sw.write(&make_descriptor(0xAA, 1)).await.unwrap();
            sw.close().await.unwrap();
        }

        // Verify 3 raw entries (duplicates not yet merged).
        {
            use super::super::staging_reader::StagingReader;
            let path = backup_dir.join(STAGING_FILENAME);
            let mut reader = StagingReader::open(&path).await.unwrap();
            let mut entries = Vec::new();
            reader.read_to_end(&mut entries).await.unwrap();
            assert_eq!(entries.len(), 3);
        }

        // Clean up.
        let _ = fs::remove_file(&backup_dir.join(STAGING_FILENAME)).await;
        let _ = fs::remove_dir(&backup_dir).await;
    }

    #[tokio::test]
    async fn test_compact() {
        let dir = data_dir();
        fs::create_dir_all(&dir).await.unwrap();

        let compact_path = dir.join("staging_compact.idx");
        // Clean up from previous runs.
        let _ = fs::remove_file(&compact_path).await;

        // Use a subdirectory trick: staging file is always named "staging.idx".
        // Create a temp backup dir.
        let backup_dir = dir.join("staging_compact_backup");
        fs::create_dir_all(&backup_dir).await.unwrap();
        let _ = fs::remove_file(&backup_dir.join(STAGING_FILENAME)).await;

        {
            let mut sw = StagingWriter::create(&backup_dir).await.unwrap();
            sw.write(&make_descriptor(0xAA, 1)).await.unwrap();
            sw.write(&make_descriptor(0xBB, 1)).await.unwrap();
            // Second occurrence of 0xAA — refcount should be 2 after compact.
            sw.write(&make_descriptor(0xAA, 1)).await.unwrap();

            sw.compact().await.unwrap();
            sw.close().await.unwrap();
        }

        // After compact: 2 unique hashes.
        {
            use super::super::staging_reader::StagingReader;
            let staging_path = backup_dir.join(STAGING_FILENAME);
            let mut reader = StagingReader::open(&staging_path).await.unwrap();
            let mut entries = Vec::new();
            reader.read_to_end(&mut entries).await.unwrap();

            assert_eq!(entries.len(), 2, "compact should merge duplicates");

            // Find the 0xAA entry and check refcount == 2.
            let aa = entries
                .iter()
                .find(|e| e.hash == vec![0xAA_u8; 32])
                .expect("0xAA entry must exist");
            assert_eq!(aa.refcount, 2, "duplicate chunks must accumulate refcount");

            // Copy to the shared data dir so staging_reader tests can pick it up.
            fs::copy(&staging_path, &compact_path).await.unwrap();
        }

        // Clean up.
        let _ = fs::remove_file(backup_dir.join(STAGING_FILENAME)).await;
        let _ = fs::remove_dir(&backup_dir).await;
    }
}
