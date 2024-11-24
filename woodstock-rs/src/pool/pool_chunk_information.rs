use eyre::Result;
use std::fmt::{self};

#[serde_with::serde_as]
#[derive(Clone, PartialEq, ::prost::Message, serde::Serialize)]
pub struct PoolChunkInformation {
    #[serde_as(as = "serde_with::hex::Hex")]
    #[prost(bytes = "vec", tag = "1")]
    pub sha256: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub size: u64,
    #[prost(uint64, tag = "3")]
    pub compressed_size: u64,
}

impl PoolChunkInformation {
    pub fn to_yaml(&self) -> Result<String> {
        let object = vec![self];
        let str = serde_yaml::to_string(&object)?;
        Ok(str)
    }
}

impl fmt::Display for PoolChunkInformation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let yaml = self.to_yaml();
        let yaml = match yaml {
            Ok(yaml) => yaml,
            Err(err) => {
                return write!(f, "Failed to serialize FileManifest: {err}");
            }
        };

        // Écrivez le chemin formaté dans le Formatter
        write!(f, "{yaml}")
    }
}
