//! This module contains all submodules for the Woodstock CLI commands.
//!
//! Each submodule implements a specific set of commands for backup management, chunk operations, file manifest handling, and more. These modules are re-exported here for use in the main CLI entry points.
//!
//! # Modules
//!
//! - `convertion`: Commands for data conversion and migration.
//! - `convert_compression`: Commands for compression format conversion.
//! - `file_manifest`: File manifest comparison and management.
//! - `pool`: Pool management commands.
//! - `read_chunk`: Chunk reading and searching utilities.
//! - `read_protobuf`: Protobuf log and data reading commands.
//! - `resolve`: Network and mDNS resolution commands.
//! - `mount`: (Unix only) FUSE-based mounting commands.

use std::sync::Arc;

use tracing::info;
use woodstock::{
    config::{Backups, Configuration, Hosts, Scheduler, GLOBAL_CONFIGURATION},
    server::resolve::SocketAddrResolver,
};

pub mod archive;
pub mod backups;
pub mod convertion;
pub mod file_manifest;
pub mod pool;
pub mod read_chunk;
pub mod read_protobuf;
pub mod resolve;

#[cfg(all(unix, feature = "fuse_unix"))]
pub mod mount;

/// Shared state for the `BackupPC` importer application.
pub struct CliServiceState {
    pub config: Arc<Configuration>,
    pub hosts: Arc<Hosts>,
    pub backups: Arc<Backups>,
    pub resolver: Option<Arc<SocketAddrResolver>>,
}

impl Default for CliServiceState {
    fn default() -> Self {
        let config = Arc::new(GLOBAL_CONFIGURATION.clone());

        let scheduler = Arc::new(Scheduler::new(config.clone()));
        let hosts = Arc::new(Hosts::new(config.clone(), scheduler));
        let backups = Arc::new(Backups::new(config.clone()));

        let resolver = {
            let redis_url = config.redis_url();
            info!("Connect to Redis URL for DNS resolution: {}", redis_url);
            match redis::Client::open(redis_url) {
                Ok(client) => {
                    let resolver = SocketAddrResolver::new(client)
                        .map(|arc| Arc::new(arc))
                        .ok();
                    resolver
                }
                Err(_) => {
                    info!("No Redis URL configured, skipping DNS resolution");
                    None
                }
            }
        };

        CliServiceState {
            config,
            hosts,
            backups,
            resolver,
        }
    }
}
