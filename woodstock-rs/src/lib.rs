#![recursion_limit = "512"]

//! # Woodstock Backup
//!
//! The Woodstock Backup crate provides a comprehensive solution for managing backups, including client-server communication, file scanning, and metadata management. It is designed to be modular and extensible, supporting various backup and restoration scenarios.
//!
//! ## Features
//!
//! * Secure authentication and authorization
//! * Efficient file scanning and metadata extraction
//! * Incremental and full backup capabilities
//! * Cross-platform support for Windows and Unix-like systems
//! * Integration with gRPC for client-server communication
//!
//! ## Modules
//!
//! The crate is organized into several modules, each responsible for a specific aspect of the backup system:
//!
//! * `server` - Implements server-side functionality for managing backups
//! * `proto` - Contains protocol buffer definitions for gRPC communication
//! * `utils` - Provides utility functions and helpers
//! * `view` - Manages user interface components for the backup system
//!
//! ## Usage
//!
//! To use this crate, include it in your `Cargo.toml` and import the necessary modules in your Rust code. Refer to the module-level documentation for detailed usage examples.

pub mod pool;
pub mod view;

pub mod server;

pub mod config;
pub mod events;
pub mod manifest;
pub mod proto;
pub mod statistics;
pub mod utils;

mod woodstock {
    #![allow(
        clippy::all,
        clippy::pedantic,
        clippy::missing_docs_in_private_items,
        clippy::missing_errors_doc,
        clippy::missing_panics_doc
    )]
    tonic::include_proto!("woodstock");
}

pub use woodstock::*;
