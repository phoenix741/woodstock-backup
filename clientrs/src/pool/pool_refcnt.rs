use core::fmt;

use crate::woodstock::{PoolRefCount, PoolUnused};

impl fmt::Display for PoolRefCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let object = vec![self];
        let yaml = serde_yaml::to_string(&object);
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

impl fmt::Display for PoolUnused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let object = vec![self];
        let yaml = serde_yaml::to_string(&object);
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
