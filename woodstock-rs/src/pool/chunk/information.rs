use eyre::Result;
use std::fmt::{self};

/// Data structures describing chunk metadata stored in the Woodstock pool.
///
/// This module contains the protobuf-backed types used to exchange and persist the logical and
/// physical metadata of chunks inside the pool.
///
/// [`PoolChunkInformation`] is the main record used by read, save, compaction, and integrity
/// checking code paths. It carries both logical information, such as the chunk hash and original
/// size, and Pool V3 physical location information, such as the segment identifier and offset.
///
/// [`ConvertHashLink`] is a lightweight helper used when one hash must be remapped to another,
/// for example during migration or deduplication-related tooling.
#[serde_with::serde_as]
#[derive(Clone, PartialEq, ::prost::Message, serde::Serialize)]
/// Metadata describing one chunk stored in the pool.
///
/// In Pool V3 this structure contains enough information to locate the chunk payload inside a
/// segment without rescanning the full pool.
pub struct PoolChunkInformation {
    /// Logical chunk hash, serialized as hex in YAML output.
    #[serde_as(as = "serde_with::hex::Hex")]
    #[prost(bytes = "vec", tag = "1")]
    pub chunk_hash: ::prost::alloc::vec::Vec<u8>,
    /// Original uncompressed size of the chunk.
    #[prost(uint64, tag = "2")]
    pub size: u64,
    /// Compressed size of the payload as stored in the pool.
    #[prost(uint64, tag = "3")]
    pub compressed_size: u64,
    /// Numeric compression format identifier used by the stored payload.
    #[prost(uint32, tag = "4")]
    pub format: u32,
    /// Identifier of the segment containing the chunk in Pool V3.
    #[prost(uint64, tag = "5")]
    pub segment_id: u64,
    /// Byte offset of the chunk header inside the segment file.
    #[prost(uint64, tag = "6")]
    pub offset: u64,
    /// Serialized size of the length-delimited chunk header stored before the payload.
    #[prost(uint64, tag = "7")]
    pub chunk_header_size: u64,
}

impl PoolChunkInformation {
    /// Serializes the chunk information as a YAML list containing this entry.
    ///
    /// # Errors
    /// Returns an error if YAML serialization fails.
    pub fn to_yaml(&self) -> Result<String> {
        let object = vec![self];
        let str = serde_yaml_ng::to_string(&object)?;
        Ok(str)
    }
}

impl fmt::Display for PoolChunkInformation {
    /// Formats the chunk information as YAML for display.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let yaml = self.to_yaml();
        let yaml = match yaml {
            Ok(yaml) => yaml,
            Err(err) => {
                return write!(f, "Failed to serialize FileManifest: {err}");
            }
        };

        write!(f, "{yaml}")
    }
}

#[derive(Clone, PartialEq, ::prost::Message)]
/// Mapping from one chunk hash to another.
///
/// This is typically used by migration or conversion routines that need to preserve a link
/// between an old logical identifier and its replacement.
pub struct ConvertHashLink {
    /// Previous chunk hash.
    #[prost(bytes = "vec", tag = "1")]
    pub old_hash: Vec<u8>,

    /// Replacement chunk hash.
    #[prost(bytes = "vec", tag = "2")]
    pub new_hash: Vec<u8>,
}
