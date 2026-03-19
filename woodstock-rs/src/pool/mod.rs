//! # Woodstock Pool Module
//!
//! This module provides the core logic for managing the deduplicated chunk pool in the Woodstock backup system.
//! It includes Pool V3 chunk storage, materialized index maintenance, integrity checking (fsck),
//! and utilities for chunk path calculation and atomic operations.
//!
//! ## Main Structures
//!
//! - [`PoolChunkInformation`], [`PoolChunkWrapper`], [`PoolChunkWriter`], [`PoolManager`]
//!
//! ## Usage
//!
//! The pool module is used by backup, restore, and maintenance operations to manage chunk files,
//! materialize logical visibility, and ensure data integrity.
//!
//! ## Error Handling & Panics
//!
//! - All async methods return `Result` and propagate errors using the `eyre` crate.
//! - Panics are not expected under normal operation; assertion failures indicate programming errors.
//!
//! # Pool Module
//!
//! This module manages the storage pool for backups, including file integrity checks and metadata management.
//!
//! ## Submodules
//!
//! * `fsck` - Provides functionality for checking and repairing the integrity of the storage pool.
//!
//! ## Features
//!
//! * Efficient storage management
//! * Integrity checks to ensure data consistency
//! * Support for repairing corrupted data
//!
//! ## Usage
//!
//! Use the `fsck` submodule to perform integrity checks and repairs on the storage pool. Refer to the submodule documentation for detailed usage examples.

/// Grouped Pool V3 workflow artifacts: staging, publication, removal and pending
mod artifacts;
/// Grouped chunk facade modules: metadata, wrapper and writer
mod chunk;
/// Module with Pool V3 low-level data access and storage primitives
pub mod data;
/// Module with method to check the integrity of the pool
mod fsck;
/// High-level pool manager grouping all pool operations under one struct
pub mod pool_manager;
/// Module with utility functions for chunk path and temp file management
mod utils;
/// Module with Pool V3 local prost records used on disk only
pub mod v3;

pub use artifacts::{
    PoolV3PendingFile, PoolV3PublicationFile, PoolV3RemovalFile, PoolV3StagingFile,
    PoolV3StagingWriter,
};
pub use chunk::{ConvertHashLink, PoolChunkInformation, PoolChunkWrapper, PoolChunkWriter};
pub use data::{IndexedChunk, IndexedSegment, PoolIndex};
pub use fsck::{
    check_backup_integrity, check_host_integrity, check_pool_integrity, check_unused, FsckCount,
    FsckUnusedCount,
};
pub use pool_manager::PoolManager;
pub use utils::{calculate_chunk_path, get_temp_chunk_path};
pub use v3::{
    PoolV3ChunkHeader, PoolV3PendingHeader, PoolV3PublicationChunkEntry, PoolV3PublicationEntry,
    PoolV3PublicationEnvelope, PoolV3PublicationHeader, PoolV3RemovalChunkRecord,
    PoolV3RemovalEntry, PoolV3RemovalHeader, PoolV3SegmentHeader,
    PoolV3StagingChunkRecord, PoolV3StagingEntry, PoolV3StagingEnvelope, PoolV3StagingHeader,
};
