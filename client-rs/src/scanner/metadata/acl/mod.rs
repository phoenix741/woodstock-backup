//! ACL (Access Control List) module for managing file system permissions.
//!
//! This module provides platform-specific implementations for reading and restoring
//! file access control lists. It handles the differences between Unix and Windows
//! platforms transparently, using conditional compilation to select the appropriate
//! implementation.
//!
//! On Unix systems with the 'acl' feature enabled, it uses POSIX ACLs.
//! On Windows or when the 'acl' feature is disabled, it provides a stub implementation.
//!
//! # Feature Flags
//! - `acl`: When enabled on Unix systems, provides full ACL support.
//!          When disabled or on non-Unix systems, uses a stub implementation.

#[cfg(all(target_os = "linux", feature = "acl"))]
/// Unix-specific implementation for ACLs.
mod unix;
#[cfg(not(all(target_os = "linux", feature = "acl")))]
/// Windows-specific implementation for ACLs.
mod windows;

#[cfg(all(target_os = "linux", feature = "acl"))]
pub use unix::*;

#[cfg(not(all(target_os = "linux", feature = "acl")))]
pub use windows::*;
