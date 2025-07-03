use eyre::Result;
use std::fmt::{self};

/// # Pool Chunk Information Module
///
/// This module defines the data structures for chunk metadata in the Woodstock backup pool.
/// It provides serialization, display, and utility methods for chunk information and hash conversion.
///
/// ## Main Structures
///
/// - [`PoolChunkInformation`]: Metadata for a single chunk (hash, size, compressed size).
/// - [`ConvertHashLink`]: Represents a mapping from an old chunk hash to a new one.
///
/// ## Usage
///
/// These structures are used throughout the pool for chunk management, deduplication, and integrity checks.
///
/// ## Error Handling & Panics
///
/// - All methods return `Result` and propagate errors using the `eyre` crate.
/// - Panics are not expected under normal operation.
#[serde_with::serde_as]
#[derive(Clone, PartialEq, ::prost::Message, serde::Serialize)]
/// Metadata for a single chunk in the pool.
pub struct PoolChunkInformation {
    #[serde_as(as = "serde_with::hex::Hex")]
    #[prost(bytes = "vec", tag = "1")]
    pub sha256: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub size: u64,
    #[prost(uint64, tag = "3")]
    pub compressed_size: u64,
    #[prost(uint32, tag = "4")]
    pub format: u32,
}

impl PoolChunkInformation {
    /// Converts the chunk information to a YAML string.
    ///
    /// # Returns
    /// A `Result` containing the YAML string if successful, or an error if the conversion fails.
    ///
    /// # Errors
    /// This function will return an error if the serialization to YAML fails due to invalid data or other issues.
    pub fn to_yaml(&self) -> Result<String> {
        let object = vec![self];
        let str = serde_yaml::to_string(&object)?;
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

        // Write the formatted path to the Formatter
        write!(f, "{yaml}")
    }
}

/* ConvertHashLink */

#[derive(Clone, PartialEq, ::prost::Message)]
/// Represents a mapping from an old chunk hash to a new one (for deduplication or migration).
pub struct ConvertHashLink {
    #[prost(bytes = "vec", tag = "1")]
    pub old_hash: Vec<u8>,

    #[prost(bytes = "vec", tag = "2")]
    pub new_hash: Vec<u8>,
}
