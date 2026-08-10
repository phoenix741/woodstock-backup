//! Restore-side filesystem metadata operations — recreating special nodes,
//! symlinks, permission bits, extended attributes and ACLs at a destination
//! path from a [`crate::FileManifest`].
//!
//! Shared between `client-rs`'s real restore path
//! (`client-rs::scanner::file_writer::create_file_from_manifest`) and
//! [`crate::archiving::fs_materialize`]'s `dir`-mode archive materializer —
//! previously duplicated between the two crates, which is how they
//! diverged: the old `client-rs` copy of `mknode` used the wrong device
//! field (`stats.dev`, the containing filesystem's device, instead of
//! `stats.rdev`, the node's own major/minor), and `tar_writer` disagreed
//! with `fs_materialize` on whether `mode` needed masking to its permission
//! bits before being written out.
//!
//! Ownership (uid/gid) and timestamps are restored by neither side today and
//! remain out of scope here.

/// Access Control Lists (ACL) handling submodule.
pub mod acl;

/// Extended Attributes (xattr) handling submodule.
pub mod xattr;

/// Unix-specific implementation.
#[cfg(unix)]
mod unix;

/// Windows-specific implementation.
#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub use unix::*;

#[cfg(windows)]
pub use windows::*;
