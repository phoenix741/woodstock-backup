use core::fmt;

use crate::woodstock::{PoolRefCount, PoolUnused};

/// # Pool Reference Count Display Module
///
/// This module provides display implementations for pool reference count and unused chunk structures.
/// It allows pretty-printing of these structures as YAML for debugging and reporting.
///
/// ## Main Implementations
///
/// - [`fmt::Display` for `PoolRefCount`]: Formats a reference count as YAML.
/// - [`fmt::Display` for `PoolUnused`]: Formats an unused chunk as YAML.
///
/// ## Usage
///
/// These implementations are used for logging, debugging, and reporting pool state.
///
/// ## Error Handling & Panics
///
/// - Panics are not expected under normal operation; errors are handled gracefully in formatting.
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

        // Write the formatted path to the Formatter
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

        // Write the formatted path to the Formatter
        write!(f, "{yaml}")
    }
}
