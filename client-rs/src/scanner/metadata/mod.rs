//! # File System Metadata Module
//!
//! This module provides platform-specific implementations for reading, creating, and restoring various types of
//! file system metadata, such as file permissions, ownership, timestamps, and special file types. It handles the
//! differences between Unix and Windows platforms transparently through conditional compilation.
//!
//! ## Module Structure
//!
//! The module is organized into three main parts:
//!
//! 1. Core metadata handling (this module) - deals with basic file attributes like permissions, ownership, timestamps, and file types.
//! 2. Access Control Lists (ACL) submodule - handles file system access control lists.
//! 3. Extended Attributes (xattr) submodule - manages additional file metadata attributes.
//!
//! Each part provides platform-specific implementations that are selected at compile time based on the target platform (Unix/Windows) and enabled features.

/// Access Control Lists (ACL) handling submodule.
pub mod acl;

/// Extended Attributes (xattr) handling submodule.
pub mod xattr;

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
