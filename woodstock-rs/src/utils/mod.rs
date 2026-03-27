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
/// Copy-on-Write file copy using `copy_file_range(2)` on Linux, with a
/// standard tokio I/O fallback on other platforms.
pub mod cow_copy;
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
/// Serde deserializers: date/time helpers and flexible numeric/string deserializers
pub mod serde;
/// HashMap with automatic expiration of entries based on time.
pub mod timed_hashmap;
