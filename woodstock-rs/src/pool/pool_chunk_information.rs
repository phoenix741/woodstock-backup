#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolChunkInformation {
    #[prost(bytes = "vec", tag = "1")]
    pub sha256: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub size: u64,
    #[prost(uint64, tag = "3")]
    pub compressed_size: u64,
}
