use serde_with::serde_as;

/// A descriptor for a single chunk stored in the pool index shard.
///
/// Each `ChunkDescriptor` entry is written in sorted order by `hash` within a shard file,
/// enabling O(log N) binary search via disk seeks on the offset table in the footer.
///
/// # On-disk layout
///
/// Entries are stored as length-delimited protobuf messages. A footer is appended after the
/// last entry:
/// ```text
/// [length-delimited ChunkDescriptor 0]     ← at offsets[0]
/// [length-delimited ChunkDescriptor 1]     ← at offsets[1]
/// ...
/// [length-delimited ChunkDescriptor N-1]   ← at offsets[N-1]
/// [u64 LE: offsets[0]] ... [u64 LE: offsets[N-1]]
/// [u64 LE: N]                              ← last 8 bytes of the file
/// ```
#[serde_as]
#[derive(Clone, PartialEq, ::prost::Message, serde::Serialize)]
pub struct ChunkDescriptor {
    /// SHA-256 hash of the chunk payload (32 bytes).
    #[serde_as(as = "serde_with::hex::Hex")]
    #[prost(bytes = "vec", tag = "1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,

    /// Identifier of the segment file that contains this chunk.
    #[prost(uint64, tag = "2")]
    pub segment_id: u64,

    /// Byte offset of the encoded chunk header (length-delimited `SegmentChunkHeader`)
    /// from the start of the segment file.
    #[prost(uint64, tag = "3")]
    pub offset: u64,

    /// Uncompressed size of the chunk payload in bytes.
    #[prost(uint64, tag = "4")]
    pub size: u64,

    /// Compressed size of the chunk payload in bytes (as stored in the segment).
    #[prost(uint64, tag = "5")]
    pub compressed_size: u64,

    /// Size in bytes of the encoded chunk header record (length-delimiter + protobuf bytes).
    /// The actual payload starts at `offset + header_size` in the segment file.
    #[prost(uint32, tag = "6")]
    pub header_size: u32,

    /// Compression algorithm applied to the chunk payload.
    /// Encoded as a `u32` matching `CompressionFormat::as_u32()`.
    #[prost(uint32, tag = "7")]
    pub compression_format: u32,

    /// Number of active references to this chunk across all manifests.
    #[prost(uint64, tag = "8")]
    pub refcount: u64,
}

/// A [`ChunkDescriptor`] paired with a signed delta to apply to the index.
///
/// The `delta` encodes the direction and magnitude of the reference-count
/// change:
///
/// - `delta > 0` — add references (typically `descriptor.refcount` reference
///   units); treated as an *add* operation during flush.
/// - `delta < 0` — remove references; treated as a *remove* operation during
///   flush.
/// - `delta == 0` — no-op (allowed but has no effect on the shard).
///
/// Create instances via [`SignedChunkDescriptor::for_add`] or
/// [`SignedChunkDescriptor::for_remove`] rather than constructing directly.
#[derive(Debug, Clone)]
pub struct SignedChunkDescriptor {
    /// The chunk descriptor carrying metadata and the base refcount.
    pub descriptor: ChunkDescriptor,
    /// Signed delta to apply to the stored refcount.
    pub delta: i64,
}

impl SignedChunkDescriptor {
    /// Wraps `descriptor` as an *add* operation.
    ///
    /// The delta is set to `+descriptor.refcount` (clamped to [`i64::MAX`] on
    /// overflow, which is practically impossible for real refcounts).
    #[must_use]
    pub fn for_add(descriptor: ChunkDescriptor) -> Self {
        let delta = i64::try_from(descriptor.refcount).unwrap_or(i64::MAX);
        Self { descriptor, delta }
    }

    /// Wraps `descriptor` as a *remove* operation.
    ///
    /// The delta is set to `−descriptor.refcount` (clamped to [`i64::MIN`] on
    /// overflow, which is practically impossible for real refcounts).
    #[must_use]
    pub fn for_remove(descriptor: ChunkDescriptor) -> Self {
        let delta = -i64::try_from(descriptor.refcount).unwrap_or(i64::MAX);
        Self { descriptor, delta }
    }
}
