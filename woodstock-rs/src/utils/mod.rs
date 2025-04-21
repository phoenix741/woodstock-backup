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

/// Provides functionality for hashing data chunks.
pub mod chunk_hasher;
/// Handles encryption and decryption operations.
pub mod encryption;
/// Contains file manipulation utilities.
pub mod files;
/// Manages resource locking mechanisms.
pub mod lock;
/// Provides utilities for path manipulation.
pub mod path;
