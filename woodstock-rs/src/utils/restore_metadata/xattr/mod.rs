//! # Extended Attributes (xattr) Module
//!
//! Platform-specific implementations for reading and restoring extended file
//! attributes, selected at compile time via conditional compilation.
//!
//! On Unix (Linux/FreeBSD) with the `xattr` feature enabled, this provides
//! full support for extended attributes. On other platforms, or with the
//! feature disabled, a stub implementation is used.

/// Whether the current build actually persists extended attributes, as
/// opposed to silently discarding them via the stub implementation.
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
