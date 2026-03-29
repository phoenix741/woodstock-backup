use serde_with::serde_as;

#[serde_as]
#[derive(Clone, PartialEq, ::prost::Message, serde::Serialize)]
pub struct SegmentHeader {
    #[prost(uint32, tag = "1")]
    pub format_version: u32,
    #[prost(uint64, tag = "2")]
    pub segment_id: u64,
    #[prost(uint64, tag = "3")]
    pub target_size: u64,
    #[prost(uint64, tag = "4")]
    pub created_at: u64,
}

#[serde_as]
#[derive(Clone, PartialEq, ::prost::Message, serde::Serialize)]
pub struct SegmentChunkHeader {
    #[serde_as(as = "serde_with::hex::Hex")]
    #[prost(bytes = "vec", tag = "1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub size: u64,
    #[prost(uint64, tag = "3")]
    pub compressed_size: u64,
    #[prost(uint32, tag = "4")]
    pub compression_format: u32,
}

#[serde_as]
#[derive(Clone, PartialEq, ::prost::Message, serde::Serialize)]
pub struct SegmentFileMetadataRecord {
    #[prost(uint64, tag = "1")]
    pub segment_id: u64,
    #[prost(uint32, tag = "2")]
    pub state: u32,
    #[prost(uint64, tag = "3")]
    pub size_total: u64,
    #[prost(uint64, tag = "4")]
    pub size_effective: u64,
    #[prost(uint64, tag = "5")]
    pub size_limit: u64,
    #[prost(uint64, tag = "6")]
    pub chunk_count: u64,
    /// Compressed bytes known to be dead (refcount == 0 in the index).
    /// Populated by [`IndexSweeper`] after a sweep pass.
    /// Bytes occupied by dead (refcount=0) chunks: header + compressed payload each.
    /// Updated by `IndexSweeper` after each sweep run.
    #[prost(uint64, tag = "7")]
    pub dead_stored_bytes: u64,
}

/// Metadata record persisted in `segments/segments.info`.
///
/// Acts as a low-cost cache for the segment directory: avoids scanning all `.seg`
/// files on every `get_segment_writer()` call.  Fields are hints — a stale value
/// causes a single extra file open at most before the index self-corrects.
#[serde_as]
#[derive(Clone, PartialEq, ::prost::Message, serde::Serialize)]
pub struct SegmentsInformation {
    /// ID of the oldest segment still open (hint; may lag behind if a segment
    /// was filled by another writer before this record was refreshed).
    #[prost(uint64, tag = "1")]
    pub first_open_segment_id: u64,
    /// ID to assign to the next newly created segment.
    #[prost(uint64, tag = "2")]
    pub next_segment_id: u64,
    /// Monotonically increasing counter incremented on every write to this file.
    /// Useful to detect stale cached copies.
    #[prost(uint64, tag = "3")]
    pub generation: u64,
}
