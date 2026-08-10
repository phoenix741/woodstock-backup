//! ACL (Access Control List) module for managing file system permissions.
//!
//! Platform-specific implementations for reading and restoring file access
//! control lists, selected at compile time via conditional compilation.
//!
//! On Unix systems with the `acl` feature enabled, this uses POSIX ACLs. On
//! Windows, or when the `acl` feature is disabled, it provides a stub
//! implementation.
//!
//! ACLs are currently only implemented on Linux (via `posix_acl`/libacl).
//! FreeBSD's default filesystem (ZFS) uses NFSv4 ACLs, a different model
//! this module does not represent yet, so FreeBSD builds fall back to the
//! stub like Windows does.

/// Whether the current build actually persists ACLs, as opposed to silently
/// discarding them via the stub implementation.
pub const SUPPORTED: bool = cfg!(all(target_os = "linux", feature = "acl"));

#[cfg(all(target_os = "linux", feature = "acl"))]
/// Unix-specific implementation for ACLs.
mod unix;
#[cfg(not(all(target_os = "linux", feature = "acl")))]
/// Stub implementation for ACLs.
mod windows;

#[cfg(all(target_os = "linux", feature = "acl"))]
pub use unix::*;

#[cfg(not(all(target_os = "linux", feature = "acl")))]
pub use windows::*;
