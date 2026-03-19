use serde_with::serde_as;

#[serde_as]
#[derive(Clone, PartialEq, ::prost::Message, serde::Serialize)]
pub struct PoolV3SegmentHeader {
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
pub struct PoolV3ChunkHeader {
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
pub struct PoolV3StagingHeader {
    #[prost(uint32, tag = "1")]
    pub format_version: u32,
    #[prost(string, tag = "2")]
    pub hostname: ::prost::alloc::string::String,
    #[serde_as(as = "serde_with::hex::Hex")]
    #[prost(bytes = "vec", tag = "3")]
    pub backup_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "4")]
    pub created_at: u64,
}

#[derive(Clone, PartialEq, ::prost::Message, serde::Serialize)]
pub struct PoolV3StagingEntry {
    #[prost(message, optional, tag = "1")]
    pub chunk: ::core::option::Option<PoolV3StagingChunkEntry>,
}

#[derive(Clone, PartialEq, ::prost::Message, serde::Serialize)]
pub struct PoolV3StagingEnvelope {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<PoolV3StagingHeader>,
    #[prost(message, optional, tag = "2")]
    pub entry: ::core::option::Option<PoolV3StagingEntry>,
}

#[serde_as]
#[derive(Clone, PartialEq, ::prost::Message, serde::Serialize)]
pub struct PoolV3StagingChunkEntry {
    #[serde_as(as = "serde_with::hex::Hex")]
    #[prost(bytes = "vec", tag = "1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub size: u64,
    #[prost(uint64, tag = "3")]
    pub compressed_size: u64,
    #[prost(uint64, tag = "4")]
    pub chunk_header_size: u64,
    #[prost(uint32, tag = "5")]
    pub compression_format: u32,
    #[prost(uint64, tag = "6")]
    pub ref_count_delta: u64,
    #[prost(bool, tag = "7")]
    pub publishes_new_chunk: bool,
    #[prost(uint64, tag = "8")]
    pub segment_id: u64,
    #[prost(uint64, tag = "9")]
    pub offset: u64,
}

pub type PoolV3StagingChunkRecord = PoolV3StagingChunkEntry;

#[serde_as]
#[derive(Clone, PartialEq, ::prost::Message, serde::Serialize)]
pub struct PoolV3PublicationHeader {
    #[prost(uint32, tag = "1")]
    pub format_version: u32,
    #[prost(string, tag = "2")]
    pub hostname: ::prost::alloc::string::String,
    #[serde_as(as = "serde_with::hex::Hex")]
    #[prost(bytes = "vec", tag = "3")]
    pub backup_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "4")]
    pub created_at: u64,
}

#[derive(Clone, PartialEq, ::prost::Message, serde::Serialize)]
pub struct PoolV3PublicationEntry {
    #[prost(message, optional, tag = "1")]
    pub chunk: ::core::option::Option<PoolV3PublicationChunkEntry>,
}

#[derive(Clone, PartialEq, ::prost::Message, serde::Serialize)]
pub struct PoolV3PublicationEnvelope {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<PoolV3PublicationHeader>,
    #[prost(message, optional, tag = "2")]
    pub entry: ::core::option::Option<PoolV3PublicationEntry>,
}

#[serde_as]
#[derive(Clone, PartialEq, ::prost::Message, serde::Serialize)]
pub struct PoolV3PublicationChunkEntry {
    #[serde_as(as = "serde_with::hex::Hex")]
    #[prost(bytes = "vec", tag = "1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub ref_count_delta: u64,
    #[prost(bool, tag = "3")]
    pub publishes_new_chunk: bool,
    #[prost(uint64, tag = "4")]
    pub segment_id: u64,
    #[prost(uint64, tag = "5")]
    pub offset: u64,
    #[prost(uint64, tag = "6")]
    pub size: u64,
    #[prost(uint64, tag = "7")]
    pub compressed_size: u64,
    #[prost(uint64, tag = "8")]
    pub chunk_header_size: u64,
    #[prost(uint32, tag = "9")]
    pub compression_format: u32,
}

#[serde_as]
#[derive(Clone, PartialEq, ::prost::Message, serde::Serialize)]
pub struct PoolV3RemovalHeader {
    #[prost(uint32, tag = "1")]
    pub format_version: u32,
    #[prost(string, tag = "2")]
    pub hostname: ::prost::alloc::string::String,
    #[serde_as(as = "serde_with::hex::Hex")]
    #[prost(bytes = "vec", tag = "3")]
    pub backup_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "4")]
    pub created_at: u64,
}

#[derive(Clone, PartialEq, ::prost::Message, serde::Serialize)]
pub struct PoolV3RemovalEntry {
    #[prost(message, optional, tag = "1")]
    pub chunk: ::core::option::Option<PoolV3RemovalChunkEntry>,
}

#[serde_as]
#[derive(Clone, PartialEq, ::prost::Message, serde::Serialize)]
pub struct PoolV3RemovalChunkEntry {
    #[serde_as(as = "serde_with::hex::Hex")]
    #[prost(bytes = "vec", tag = "1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub size: u64,
    #[prost(uint64, tag = "3")]
    pub compressed_size: u64,
    #[prost(uint64, tag = "4")]
    pub chunk_header_size: u64,
    #[prost(uint64, tag = "5")]
    pub ref_count_delta: u64,
}

pub type PoolV3RemovalChunkRecord = PoolV3RemovalChunkEntry;

#[serde_as]
#[derive(Clone, PartialEq, ::prost::Message, serde::Serialize)]
pub struct PoolV3PendingHeader {
    #[prost(uint32, tag = "1")]
    pub format_version: u32,
    #[prost(string, tag = "2")]
    pub operation_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub operation_type: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub hostname: ::prost::alloc::string::String,
    #[serde_as(as = "serde_with::hex::Hex")]
    #[prost(bytes = "vec", tag = "5")]
    pub backup_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "6")]
    pub journal_path: ::prost::alloc::string::String,
    #[prost(uint64, tag = "7")]
    pub created_at: u64,
}
