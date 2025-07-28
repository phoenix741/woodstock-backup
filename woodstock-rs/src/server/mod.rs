//! This module contains the server-side logic for the Woodstock backup system.
//!
//! # Modules
//!
//! * `backup` - Handles backup operations.
//! * `client` - Manages client-server communication.
//! * `pool` - Provides functionality for managing the backup pool.
//! * `progression` - Tracks the progression of backup and file operations.
//! * `resolve` - Resolves hostnames and checks connectivity.
//! * `tools` - Utility functions for server operations.

pub mod backup;
pub mod client;
pub mod job;
pub mod pool;
pub mod progression;
pub mod resolve;
pub mod tools;
