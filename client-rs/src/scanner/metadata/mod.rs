//! # File System Metadata Module
//!
//! Platform-specific implementations for reading file system metadata during
//! scanning — basic file attributes like permissions, ownership, timestamps,
//! and file types. Handles the differences between Unix and Windows
//! platforms transparently through conditional compilation.
//!
//! Restore-side operations (creating special nodes/symlinks, restoring
//! permissions/xattrs/ACLs) live in `woodstock::utils::restore_metadata` —
//! shared with `archiving::fs_materialize` in `woodstock-rs`, since both
//! need the exact same logic to restore a `FileManifest` onto disk.

/// Unix-specific metadata implementation.
#[cfg(unix)]
mod unix;

/// Windows-specific metadata implementation.
#[cfg(windows)]
mod windows;

/// Re-export platform-specific implementations based on the target platform.
#[cfg(unix)]
pub use unix::*;

/// Re-export platform-specific implementations based on the target platform.
#[cfg(windows)]
pub use windows::*;
