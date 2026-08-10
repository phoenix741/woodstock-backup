//! Utilities module.
//!
//! This module provides various utility functions and structures used throughout the project.
//!
//! # Submodules
//!
//! * `chunk_hasher` - Provides functionality for hashing data chunks.
//! * `encryption` - Handles encryption and decryption operations.
//! * `files` - Contains file manipulation utilities.
//! * `lock` - Manages resource locking mechanisms.
//! * `path` - Provides utilities for path manipulation.

/// Generic Redis read-through cache helpers (cache_wrap / cache_invalidate).
pub mod cache;
/// Provides functionality for hashing data chunks.
pub mod chunk_hasher;
/// Provides utililties for compress / uncompress
pub mod compression;
/// Handles encryption and decryption operations.
pub mod encryption;
/// Contains file manipulation utilities.
pub mod files;
/// Manages resource locking mechanisms using Redis.
pub mod lock_redis;
/// Provides utilities for path manipulation.
pub mod path;
/// Restore-side filesystem metadata operations (special nodes, symlinks,
/// permissions, xattrs, ACLs) — shared between `client-rs`'s restore path
/// and `archiving::fs_materialize`.
pub mod restore_metadata;
/// Serde deserializers: date/time helpers and flexible numeric/string deserializers
pub mod serde;
/// HashMap with automatic expiration of entries based on time.
pub mod timed_hashmap;
