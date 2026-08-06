//! # Scanner Module
//!
//! The scanner module is responsible for scanning files and directories to generate metadata for backup operations.
//! It ensures efficient and accurate data collection to support incremental and full backups.
//!
//! ## Submodules
//!
//! * `file_reader` - Handles low-level file reading and metadata extraction
//! * `file_writer` - Manages writing file data and metadata during restoration
//! * `metadata` - Provides utilities for handling file system metadata, including ACLs and extended attributes
//!
//! ## Features
//!
//! * Recursive directory scanning
//! * Metadata extraction for files, including size, timestamps, and permissions
//! * Support for platform-specific metadata, such as ACLs and xattrs
//!
//! ## Usage
//!
//! Use the `file_reader` submodule to read files and extract metadata. The `file_writer` submodule is used for
//! restoring files, and the `metadata` submodule provides additional utilities for handling file system attributes.

/// Handles file system traversal and discovery.
mod file_browser;
/// Reads file contents and attributes.
mod file_reader;
/// Writes and restores file data.
mod file_writer;
/// Manages file metadata like permissions and timestamps.
pub(crate) mod metadata;

pub use file_browser::*;
pub use file_reader::*;
pub use file_writer::*;
