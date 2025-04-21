//! # Woodstock Manifest Module
//!
//! This module provides the core logic for managing backup manifests in the Woodstock backup system.
//! It includes reading, writing, indexing, and compacting manifest files, as well as utilities for
//! file metadata, chunk management, and index operations.
//!
//! ## Usage
//!
//! Typical usage involves creating a [`Manifest`] for a backup, reading or writing manifest entries,
//! and using the index for efficient file lookup and deduplication.
//!
//! ## Error Handling & Panics
//!
//! - All public methods return `Result` and propagate I/O or serialization errors using the `eyre` crate.
//! - Panics are not expected under normal operation; errors are returned as `Result`.
//!
//! ## Generics
//!
//! - The index and compaction logic are generic over manifest entry types.

/// Module used to read manifest file
mod core;
/// Module with extension of the `FileManifest` and `FileManifestJournalEntry`
mod file_manifest;
/// Module used to read a file from pool from information in the FileManifest
mod file_manifest_reader;
/// Module used to read the index of the manifest
mod index_manifest_model;

pub use core::*;
pub use file_manifest::*;
pub use index_manifest_model::*;
