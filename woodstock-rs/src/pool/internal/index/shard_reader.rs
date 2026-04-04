use std::mem::size_of;
use std::path::Path;

use eyre::{eyre, Result};
use memmap2::Mmap;
use prost::Message;

use super::index_model::ChunkDescriptor;

/// Byte size of a single offset table entry (u64 little-endian).
const OFFSET_SIZE: usize = size_of::<u64>();
/// Byte size of a hash value (SHA-256).
const HASH_SIZE: usize = 32;
/// Size of the fixed footer (excluding the variable-length offset table):
/// `[minhash: 32][maxhash: 32][generation: 8][count: 8]` = 80 bytes.
const FIXED_FOOTER_SIZE: usize = 2 * HASH_SIZE + 2 * OFFSET_SIZE;

/// Read-only accessor for a pool index shard file backed by a memory-mapped file.
///
/// On [`open`](Self::open) the file is memory-mapped and the footer offset table is parsed
/// into a small `Vec<u64>` (N × 8 bytes). Chunk descriptors are **not** loaded into memory —
/// they are decoded from the mmap slice on demand.
///
/// [`get_chunk`](Self::get_chunk) performs a **synchronous** O(log N) binary search by
/// decoding entries at midpoint positions directly from the mmap, letting the OS page cache
/// service repeated lookups without any explicit seeking or async I/O.
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
/// [u64 LE: N]                              ← last 8 bytes
/// ```
pub struct ShardReader {
    mmap: Mmap,
    /// Byte offsets of each entry from the start of the mmap. Loaded from the footer.
    offsets: Vec<u64>,
    /// Hash of the first (smallest) entry. Zero-filled when the shard is empty.
    min_hash: [u8; HASH_SIZE],
    /// Hash of the last (largest) entry. Zero-filled when the shard is empty.
    max_hash: [u8; HASH_SIZE],
    /// Monotonic generation counter embedded in the footer.
    generation: u64,
}

/// Extracts the first 8 bytes of a hash slice as a big-endian `u64`.
///
/// Used by [`ShardReader::get_chunk`] to perform proportional position estimation
/// (interpolation search) on uniformly-distributed SHA-256 hashes.
#[inline]
fn hash_prefix_u64(h: &[u8]) -> u64 {
    let mut bytes = [0u8; 8];
    let len = h.len().min(8);
    bytes[..len].copy_from_slice(&h[..len]);
    u64::from_be_bytes(bytes)
}

impl ShardReader {
    /// Opens a shard file, memory-maps it, and reads the footer offset table.
    ///
    /// No chunk data is decoded until [`get_chunk`](Self::get_chunk) or
    /// [`entries`](Self::entries) is called.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened, the `mmap` cannot be created, or
    /// the footer is corrupt.
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        // std::fs::File is required by memmap2; this open is cheap (no data read).
        let file = std::fs::File::open(path)?;
        // SAFETY: we do not mutate the file while the mmap is alive.
        let mmap = unsafe { Mmap::map(&file)? };

        let file_len = mmap.len();

        // Minimum valid file: fixed footer only (empty shard has N=0).
        if file_len < FIXED_FOOTER_SIZE {
            return Err(eyre!(
                "shard file is too small ({file_len} bytes) to contain a valid footer"
            ));
        }

        // Parse the fixed footer from the end of the mmap:
        //   [minhash 32B][maxhash 32B][generation 8B][count 8B]
        let count_pos = file_len - OFFSET_SIZE;
        let gen_pos = count_pos - OFFSET_SIZE;
        let maxhash_end = gen_pos;
        let maxhash_pos = maxhash_end - HASH_SIZE;
        let minhash_end = maxhash_pos;
        let minhash_pos = minhash_end - HASH_SIZE;

        let count = u64::from_le_bytes(
            mmap[count_pos..count_pos + OFFSET_SIZE]
                .try_into()
                .expect("slice has 8 bytes"),
        );
        let generation = u64::from_le_bytes(
            mmap[gen_pos..gen_pos + OFFSET_SIZE]
                .try_into()
                .expect("slice has 8 bytes"),
        );
        let max_hash: [u8; HASH_SIZE] = mmap[maxhash_pos..maxhash_end]
            .try_into()
            .expect("slice has 32 bytes");
        let min_hash: [u8; HASH_SIZE] = mmap[minhash_pos..minhash_end]
            .try_into()
            .expect("slice has 32 bytes");

        let count_usize = usize::try_from(count)
            .map_err(|_| eyre!("shard entry count {count} exceeds platform usize"))?;

        // The offset table sits immediately before the fixed footer.
        let table_bytes = count_usize
            .checked_mul(OFFSET_SIZE)
            .ok_or_else(|| eyre!("shard offset table byte count overflows usize"))?;
        let table_start = minhash_pos
            .checked_sub(table_bytes)
            .ok_or_else(|| eyre!("shard offset table extends beyond the start of the file"))?;

        let mut offsets = Vec::with_capacity(count_usize);
        for i in 0..count_usize {
            let o = table_start + i * OFFSET_SIZE;
            let offset = u64::from_le_bytes(
                mmap[o..o + OFFSET_SIZE]
                    .try_into()
                    .expect("slice has 8 bytes"),
            );
            offsets.push(offset);
        }

        Ok(Self {
            mmap,
            offsets,
            min_hash,
            max_hash,
            generation,
        })
    }

    /// Returns the number of entries in this shard.
    #[must_use]
    pub fn len(&self) -> usize {
        self.offsets.len()
    }

    /// Returns `true` if the shard contains no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }

    /// Returns the hash of the smallest entry in this shard.
    ///
    /// The returned slice is zero-filled when the shard is empty.
    #[must_use]
    pub fn min_hash(&self) -> &[u8] {
        &self.min_hash
    }

    /// Returns the hash of the largest entry in this shard.
    ///
    /// The returned slice is zero-filled when the shard is empty.
    #[must_use]
    pub fn max_hash(&self) -> &[u8] {
        &self.max_hash
    }

    /// Returns the generation counter embedded in the footer.
    #[must_use]
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Decodes the `ChunkDescriptor` at the given index position.
    fn decode_at(&self, idx: usize) -> Result<ChunkDescriptor> {
        let offset = usize::try_from(self.offsets[idx])?;
        Ok(ChunkDescriptor::decode_length_delimited(
            &self.mmap[offset..],
        )?)
    }

    /// Searches for a chunk by its hash using **interpolation search** over the mmap.
    ///
    /// SHA-256 hashes are uniformly distributed, so rather than always probing the midpoint
    /// (binary search, O(log N)), each step estimates the target's position proportionally
    /// using 8-byte big-endian prefixes of the current boundary hashes:
    ///
    /// ```text
    /// mid ≈ lo + (target_prefix − lo_prefix) × (hi − lo) / (hi_prefix − lo_prefix)
    /// ```
    ///
    /// This yields **O(log log N)** average comparisons. The algorithm falls back to bisection
    /// when `lo_prefix == hi_prefix` to avoid division by zero, and remains correct for any
    /// hash distribution.
    ///
    /// No `await` is required; the OS page cache handles data locality via the mmap.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(descriptor))` if an entry with the given hash is found.
    /// - `Ok(None)` if no matching entry exists.
    ///
    /// # Errors
    ///
    /// Returns an error if protobuf decoding fails.
    pub fn get_chunk(&self, hash: &[u8]) -> Result<Option<ChunkDescriptor>> {
        if self.offsets.is_empty() {
            return Ok(None);
        }

        // O(1) fast path: hash is outside the [minhash, maxhash] range of this shard.
        if hash < self.min_hash.as_slice() || hash > self.max_hash.as_slice() {
            return Ok(None);
        }

        let mut lo = 0usize;
        let mut hi = self.offsets.len() - 1;

        // Boundary prefixes for interpolation. Initialised from `min_hash`/`max_hash`
        // (already in memory — no mmap decode needed for the first estimate).
        let mut lo_prefix = hash_prefix_u64(&self.min_hash);
        let mut hi_prefix = hash_prefix_u64(&self.max_hash);
        let target_prefix = hash_prefix_u64(hash);

        loop {
            // Interpolate: estimate the target position proportionally within [lo, hi].
            // Falls back to bisection when the prefix range collapses to avoid div-by-zero.
            let mid = if hi_prefix > lo_prefix {
                let range = hi_prefix - lo_prefix;
                let distance = target_prefix.saturating_sub(lo_prefix);
                let span = hi - lo;
                let offset = (distance as u128 * span as u128 / range as u128) as usize;
                lo + offset.min(span)
            } else {
                lo + (hi - lo) / 2
            };

            let entry = self.decode_at(mid)?;

            match entry.hash.as_slice().cmp(hash) {
                std::cmp::Ordering::Equal => return Ok(Some(entry)),
                std::cmp::Ordering::Less => {
                    if mid == hi {
                        return Ok(None);
                    }
                    lo_prefix = hash_prefix_u64(&entry.hash);
                    lo = mid + 1;
                }
                std::cmp::Ordering::Greater => {
                    if mid == lo {
                        return Ok(None);
                    }
                    hi_prefix = hash_prefix_u64(&entry.hash);
                    hi = mid - 1;
                }
            }
        }
    }

    /// Reads all chunk descriptors from the shard in sorted order.
    ///
    /// Decodes each entry from the mmap sequentially by following the offset table.
    ///
    /// # Errors
    ///
    /// Returns an error if protobuf decoding fails.
    pub fn entries(&self) -> Result<Vec<ChunkDescriptor>> {
        (0..self.offsets.len()).map(|i| self.decode_at(i)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::internal::index::shard_writer::ShardWriter;
    use eyre::Result;
    use tempfile::NamedTempFile;

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

    #[tokio::test]
    async fn test_write_and_read_shard() -> Result<()> {
        let tmp = NamedTempFile::new()?;
        let path = tmp.path().to_path_buf();
        // NamedTempFile holds the file open; we need the path only.
        drop(tmp);

        let d0 = make_descriptor(0x10, 0, 0);
        let d1 = make_descriptor(0x20, 1, 1000);
        let d2 = make_descriptor(0x30, 2, 2000);

        // Write.
        let mut writer = ShardWriter::create(&path, 42).await?;
        writer.write(&d0).await?;
        writer.write(&d1).await?;
        writer.write(&d2).await?;
        assert_eq!(writer.len(), 3);
        writer.flush().await?;

        // Open reader.
        let reader = ShardReader::open(&path).await?;
        assert_eq!(reader.len(), 3);
        assert_eq!(reader.generation(), 42);
        assert_eq!(reader.min_hash(), d0.hash.as_slice());
        assert_eq!(reader.max_hash(), d2.hash.as_slice());

        // get_chunk — found.
        let found = reader.get_chunk(&d1.hash)?;
        assert_eq!(found.as_ref().map(|e| e.segment_id), Some(1));
        assert_eq!(found.as_ref().map(|e| e.offset), Some(1000));

        // get_chunk — not found (above maxhash, early exit).
        let absent = reader.get_chunk(&[0xFFu8; 32])?;
        assert!(absent.is_none());

        // get_chunk — not found (below minhash, early exit).
        let below = reader.get_chunk(&[0x00u8; 32])?;
        assert!(below.is_none());

        // get_chunk — first entry.
        let first = reader.get_chunk(&d0.hash)?;
        assert_eq!(first.as_ref().map(|e| e.segment_id), Some(0));

        // get_chunk — last entry.
        let last = reader.get_chunk(&d2.hash)?;
        assert_eq!(last.as_ref().map(|e| e.segment_id), Some(2));

        // entries.
        let all = reader.entries()?;
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].hash, d0.hash);
        assert_eq!(all[1].hash, d1.hash);
        assert_eq!(all[2].hash, d2.hash);

        Ok(())
    }

    #[tokio::test]
    async fn test_empty_shard() -> Result<()> {
        let tmp = NamedTempFile::new()?;
        let path = tmp.path().to_path_buf();
        drop(tmp);

        let writer = ShardWriter::create(&path, 0).await?;
        assert!(writer.is_empty());
        writer.flush().await?;

        let reader = ShardReader::open(&path).await?;
        assert!(reader.is_empty());
        assert_eq!(reader.len(), 0);
        assert_eq!(reader.generation(), 0);
        assert_eq!(reader.min_hash(), &[0u8; 32]);
        assert_eq!(reader.max_hash(), &[0u8; 32]);

        let not_found = reader.get_chunk(&[0u8; 32])?;
        assert!(not_found.is_none());

        let all = reader.entries()?;
        assert!(all.is_empty());

        Ok(())
    }
}
