//! # Staging module
//!
//! Provides a lightweight, per-backup staging file that accumulates
//! [`ChunkDescriptor`] entries as chunks are written during a backup.
//!
//! ## Purpose
//!
//! The staging file acts as an intermediate journal between the live backup
//! process and the shared pool index:
//!
//! - [`StagingWriter`] appends one [`ChunkDescriptor`] per chunk encountered
//!   (duplicates allowed).  Each write is immediately flushed to disk so the
//!   file survives crashes.
//! - [`StagingWriter::compact`] merges duplicate hashes by accumulating their
//!   `refcount` and rewrites the file atomically.
//! - [`StagingReader`] streams or bulk-reads the resulting file so the pool
//!   index writer can integrate it.
//!
//! The file is named [`STAGING_FILENAME`] (`staging.idx`) and lives in the
//! backup directory.

mod staging_reader;
mod staging_writer;

pub use staging_reader::StagingReader;
pub use staging_writer::{StagingWriter, STAGING_FILENAME};
