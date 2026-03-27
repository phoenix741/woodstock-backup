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
}
