//! # Server-rs - Rust migration of NestJS components
//!
//! This crate provides the Rust implementation of the Woodstock Backup server
//! components, replacing the NestJS applications:
//! - clientApi (mTLS authenticated API for backup clients)
//! - api (public REST/GraphQL API)
//! - job processing (background tasks and cron jobs)

pub mod api;
pub mod client_api;
pub mod error;
pub mod graphql;
pub mod jobs;
pub mod logger;
pub mod shared_state;

// Re-export the canonical error types at the crate root for convenience
pub use error::*;
