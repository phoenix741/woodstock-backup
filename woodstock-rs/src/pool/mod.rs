//! # Woodstock Pool Module
//!
//! This module provides the core logic for managing the deduplicated chunk pool in the Woodstock backup system.
//! It includes chunk file management, reference counting, unused chunk tracking, integrity checking (fsck),
//! and utilities for chunk path calculation and atomic operations.
//!
//! ## Main Structures
//!
//! - [`PoolChunkInformation`], [`PoolChunkWrapper`], [`PoolChunkWriter`], [`Refcnt`]
//!
//! ## Usage
//!
//! The pool module is used by backup, restore, and maintenance operations to manage chunk files,
//! track usage, and ensure data integrity.
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

/// Module with method to check the integrity of the pool
mod fsck;
/// Internal module on pool
mod internal;
/// Module with chunk information and metadata
mod pool_chunk_information;
/// Module with chunk file operations and metadata
mod pool_chunk_wrapper;
/// Module with async writer for compressed chunk files
mod pool_chunk_wrapper_writer;
/// High-level pool manager grouping all pool operations under one struct
pub mod pool_manager;
/// Module with display and formatting for reference counts and unused chunks
mod pool_refcnt;
/// Module with reference count and unused chunk management
mod refcnt;
/// Per-backup staging file: accumulates ChunkDescriptors before index integration
pub mod staging;
/// Module with utility functions for chunk path and temp file management
mod utils;

pub use fsck::{
    check_backup_integrity, check_host_integrity, check_pool_integrity, check_unused, FsckCount,
    FsckUnusedCount,
};
pub use internal::{
    ChunkDescriptor, ChunkIndex, DeadChunkRecord, IndexSweeper, IndexWriter, ShardReader,
    ShardWriter, SignedChunkDescriptor, SweepResult,
};
pub use internal::{
    CompactionProgression, CompactionReport, SegmentChunkEntry, SegmentCompactor, SegmentFileHeader,
    SegmentFileMetadata, SegmentFileState, SegmentFillReport, SegmentReader, SegmentWriter,
    SweepProgression,
};
pub use internal::{Segments, SegmentsWriter};
pub use pool_chunk_information::{ConvertHashLink, PoolChunkInformation};
pub use pool_chunk_wrapper::PoolChunkWrapper;
pub use pool_chunk_wrapper_writer::PoolChunkWriter;
pub use pool_manager::PoolManager;
pub use refcnt::{Refcnt, RefcntApplySens};
pub use staging::{StagingReader, StagingWriter, STAGING_FILENAME};
pub use utils::{calculate_chunk_path, get_temp_chunk_path};
