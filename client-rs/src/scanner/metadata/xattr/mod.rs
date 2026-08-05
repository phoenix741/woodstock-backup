//! # Extended Attributes (xattr) Module
//!
//! This module provides platform-specific implementations for reading and restoring extended file attributes. It handles the differences between Unix and Windows platforms transparently, using conditional compilation to select the appropriate implementation.
//!
//! ## Features
//!
//! * Read and write extended attributes on supported platforms (Unix with `xattr` feature enabled)
//! * Stub implementation for non-Unix platforms or when the `xattr` feature is disabled
//!
//! ## Platform Support
//!
//! On Unix systems with the `xattr` feature enabled, this module provides full support for extended attributes. On other platforms, a stub implementation is used to ensure compatibility.

/// Whether the current build actually persists extended attributes, as opposed to
/// silently discarding them via the stub implementation.
pub const SUPPORTED: bool = cfg!(all(
    any(target_os = "linux", target_os = "freebsd"),
    feature = "xattr"
));

/// Unix-specific implementation for extended attributes.
#[cfg(all(any(target_os = "linux", target_os = "freebsd"), feature = "xattr"))]
mod unix;
/// Stub implementation for platforms without extended attribute support.
#[cfg(not(all(any(target_os = "linux", target_os = "freebsd"), feature = "xattr")))]
mod windows;

#[cfg(all(any(target_os = "linux", target_os = "freebsd"), feature = "xattr"))]
pub use unix::*;

#[cfg(not(all(any(target_os = "linux", target_os = "freebsd"), feature = "xattr")))]
pub use windows::*;
