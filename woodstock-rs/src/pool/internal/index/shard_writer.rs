use std::mem::size_of;
use std::path::Path;

use eyre::{eyre, Result};

use crate::proto::{ProtobufWriter, UnCompressedWriter};

use super::index_protobuf::ChunkDescriptor;

/// Byte size of a single offset table entry (u64 little-endian).
const OFFSET_SIZE: usize = size_of::<u64>();
/// Byte size of a hash value (SHA-256).
const HASH_SIZE: usize = 32;

/// Stream writer for a pool index shard file.
///
/// The caller is responsible for providing `ChunkDescriptor` entries in **sorted order by hash**
/// (ascending). A `debug_assert!` enforces this invariant in debug builds.
///
/// Atomicity is delegated to [`ProtobufWriter`]: all bytes go to a UUID-named temporary file
/// in the same directory; [`flush`](Self::flush) atomically renames it to the final path.
///
/// # On-disk layout
///
/// ```text
/// [length-delimited ChunkDescriptor 0]     ← at offsets[0]
/// [length-delimited ChunkDescriptor 1]     ← at offsets[1]
/// ...
/// [length-delimited ChunkDescriptor N-1]   ← at offsets[N-1]
/// [u64 LE: offsets[0]] ... [u64 LE: offsets[N-1]]
/// [minhash: 32 bytes]                      ← hash of the first (smallest) entry
/// [maxhash: 32 bytes]                      ← hash of the last (largest) entry
/// [u64 LE: generation]                     ← monotonic generation counter
/// [u64 LE: N]                              ← always the last 8 bytes of the file
/// ```
///
/// The fixed footer (minhash + maxhash + generation + count) is 80 bytes. The reader
/// can locate it by seeking from the end of the file, then find the offset table
/// immediately before it, and perform O(log N) binary search via mmap.
pub struct ShardWriter {
    proto_writer: ProtobufWriter<UnCompressedWriter, ChunkDescriptor>,
    /// Byte offsets of each entry from file start, accumulated during writes.
    offsets: Vec<u64>,
    /// Running byte position (total bytes written so far).
    current_offset: u64,
    /// Hash of the first entry written (minimum hash, since entries are sorted ascending).
    min_hash: Vec<u8>,
    /// Hash of the last entry written (maximum hash). Also used for the sort-order assertion.
    max_hash: Vec<u8>,
    /// Monotonic generation counter passed through to the footer unchanged.
    generation: u64,
}

impl ShardWriter {
    /// Creates a new shard file writer targeting `path`.
    ///
    /// The actual bytes are written to a temporary file adjacent to `path` until
    /// [`flush`](Self::flush) is called.
    ///
    /// # Errors
    ///
    /// Returns an error if the parent directory cannot be created or the temporary file
    /// cannot be opened.
    pub async fn create<P: AsRef<Path>>(path: P, generation: u64) -> Result<Self> {
        let proto_writer = ProtobufWriter::new(path, true).await?;

        Ok(Self {
            proto_writer,
            offsets: Vec::new(),
            current_offset: 0,
            min_hash: Vec::new(),
            max_hash: Vec::new(),
            generation,
        })
    }

    /// Writes a single `ChunkDescriptor` to the shard file.
    ///
    /// Entries **must** be provided in ascending hash order.
    ///
    /// # Errors
    ///
    /// Returns an error if the write fails.
    pub async fn write(&mut self, descriptor: &ChunkDescriptor) -> Result<()> {
        debug_assert!(
            descriptor.hash >= self.max_hash,
            "ChunkDescriptor entries must be written in ascending hash order; \
             previous hash: {:?}, current hash: {:?}",
            hex::encode(&self.max_hash),
            hex::encode(&descriptor.hash),
        );

        self.offsets.push(self.current_offset);

        let bytes_written = self.proto_writer.write_size(descriptor).await?;

        self.current_offset = self
            .current_offset
            .checked_add(u64::try_from(bytes_written)?)
            .ok_or_else(|| eyre!("shard file offset overflow"))?;

        if self.min_hash.is_empty() {
            self.min_hash.clone_from(&descriptor.hash);
        }
        self.max_hash.clone_from(&descriptor.hash);

        Ok(())
    }

    /// Finalizes the shard file.
    ///
    /// Appends the offset table (N × u64 LE) followed by the entry count (u64 LE), then
    /// delegates to [`ProtobufWriter::flush`] which flushes all buffers and performs the
    /// atomic rename to the final path.
    ///
    /// # Errors
    ///
    /// Returns an error if any I/O operation fails.
    pub async fn flush(mut self) -> Result<()> {
        let count = u64::try_from(self.offsets.len())?;

        // Footer layout:
        //   [offset_0 u64 LE] ... [offset_{N-1} u64 LE]
        //   [minhash: HASH_SIZE bytes] [maxhash: HASH_SIZE bytes]
        //   [generation: u64 LE] [N: u64 LE]
        let footer_len = self.offsets.len() * OFFSET_SIZE + 2 * HASH_SIZE + 2 * OFFSET_SIZE;
        let mut footer = Vec::with_capacity(footer_len);

        for offset in &self.offsets {
            footer.extend_from_slice(&offset.to_le_bytes());
        }

        // minhash and maxhash: always HASH_SIZE bytes, zero-padded when shard is empty.
        let mut minhash_bytes = [0u8; HASH_SIZE];
        let copy_len = self.min_hash.len().min(HASH_SIZE);
        minhash_bytes[..copy_len].copy_from_slice(&self.min_hash[..copy_len]);
        footer.extend_from_slice(&minhash_bytes);

        let mut maxhash_bytes = [0u8; HASH_SIZE];
        let copy_len = self.max_hash.len().min(HASH_SIZE);
        maxhash_bytes[..copy_len].copy_from_slice(&self.max_hash[..copy_len]);
        footer.extend_from_slice(&maxhash_bytes);

        footer.extend_from_slice(&self.generation.to_le_bytes());
        footer.extend_from_slice(&count.to_le_bytes());

        self.proto_writer.write_raw(&footer).await?;
        self.proto_writer.flush().await?;

        Ok(())
    }

    /// Cancels the write and deletes the temporary file.
    ///
    /// # Errors
    ///
    /// Returns an error if the temporary file cannot be removed.
    pub async fn cancel(mut self) -> Result<()> {
        self.proto_writer.cancel().await?;
        Ok(())
    }

    /// Returns the number of entries written so far.
    #[must_use]
    pub fn len(&self) -> usize {
        self.offsets.len()
    }

    /// Returns `true` if no entries have been written yet.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }
}
