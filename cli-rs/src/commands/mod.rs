//! This module contains all submodules for the Woodstock CLI commands.
//!
//! Each submodule implements a specific set of commands for backup management, chunk operations, file manifest handling, and more. These modules are re-exported here for use in the main CLI entry points.
//!
//! # Modules
//!
//! - `convertion`: Commands for data conversion and migration.
//! - `convert_compression`: Commands for compression format conversion.
//! - `file_manifest`: File manifest comparison and management.
//! - `pool`: Pool management commands.
//! - `read_chunk`: Chunk reading and searching utilities.
//! - `read_protobuf`: Protobuf log and data reading commands.
//! - `resolve`: Network and mDNS resolution commands.
//! - `mount`: (Unix only) FUSE-based mounting commands.

pub mod convertion;
pub mod file_manifest;
pub mod pool;
pub mod read_chunk;
pub mod read_protobuf;
pub mod resolve;

#[cfg(all(unix, feature = "fuse_unix"))]
pub mod mount;
