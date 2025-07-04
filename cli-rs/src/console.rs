#![recursion_limit = "512"]

//! This module provides the main command-line interface for managing the Woodstock backup pool and related operations.
//!
//! It exposes commands for chunk management, file manifest comparison, chunk search, log reading, mDNS resolution, and more. The module acts as the entry point for the CLI, delegating subcommands to their respective handlers.
//!
//! # Errors
//!
//! Functions in this module may return errors if:
//! - The provided file paths or chunk identifiers are invalid or not found.
//! - The pool operations fail due to I/O or data corruption.
//! - The system dependencies (such as FUSE) are missing or misconfigured.
//! - Any command-specific error occurs during execution.
//!
//! The goal of this module is to permit to manage the pool.
//!
//! The command can be used to
//!
//! * remove unused chunks
//! * check all chunks
//! * recalculate all the chunks
//!
mod commands;
#[cfg(all(unix, feature = "fuse_unix"))]
mod filesystem;

use std::time::SystemTime;

use clap::{Parser, Subcommand};
use commands::convertion::convert_hash_repo;
use commands::file_manifest::compare;
use commands::read_chunk::search_chunk;
use commands::read_protobuf::read_log;
use commands::resolve::resolve_mdns;
use eyre::Result;

#[cfg(all(unix, feature = "fuse_unix"))]
use commands::mount::{mount, MountOption};

use crate::commands::pool::{check_compression, clean_unused_pool, verify_all};
use crate::commands::read_chunk::read_chunk;
use crate::commands::read_protobuf::{read_protobuf, ProtobufFormat};
use woodstock::config::{Context, GlobalConfiguration};
use woodstock::pool::apply_pending_refcnt_operations;

/// Command-line interface options for the Woodstock CLI tool.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The subcommand to execute, if any.
    #[command(subcommand)]
    subcommand: Option<Commands>,
}

/// Available subcommands for the Woodstock CLI tool.
#[derive(Subcommand)]
enum Commands {
    /// Read and display protobuf-encoded data from a file.
    ReadProtobuf {
        /// The path to the file to read.
        path: String,
        /// The type of the file to read.
        format: ProtobufFormat,
        /// Filter the output by filename.
        #[clap(long)]
        filter_name: Option<String>,
        /// Filter the output by file chunks.
        #[clap(long)]
        filter_chunks: Option<String>,
    },

    /// Read and display the backup log for a given host and backup number.
    ReadLog {
        /// The hostname of the backup server.
        hostname: String,

        /// The backup number to read the log for.
        backup_number: usize,

        /// The share path for the log.
        share_path: String,
    },

    /// Retrieve a specific chunk from the pool.
    GetChunk {
        /// The chunk to get.
        chunk: String,
    },

    /// Search for a manifest that contains the specified chunk.
    SearchChunk {
        /// The chunk to search for.
        chunk: String,
    },

    /// Add reference count to the pool for a specific backup.
    CompactRefcnt {},

    /// Clean unused chunks from the pool.
    CleanUnused {
        /// The target backup for cleaning unused chunks (optional).
        target: Option<String>,
    },

    /// Check the compression of the pool.
    CheckCompression {},

    /// Verify the integrity of the pool.
    Fsck {
        /// If true, perform a dry run without making changes.
        #[clap(short)]
        dry_run: bool,
        /// If true, include chunk information in the output.
        #[clap(short)]
        chunks: bool,
        /// If true, skip reference count and unused file verification.
        #[clap(long)]
        skip_ref_unused: bool,
    },

    /// Compare two file manifests and generate a journal file.
    Compare {
        /// The source file manifest for comparison.
        file_manifest_source: String,

        /// The target file manifest for comparison.
        file_manifest_target: String,
    },

    /// Resolve the hostname using the cache.
    ///
    /// Work only on a redis on the localhost.
    ///
    /// For DEBUG purpose only.
    ResolveHost {
        /// The hostname to resolve.
        hostname: String,
    },

    /// Convert hash from a repository to another hash.
    ConvertHashRepo {
        /// The path of the backup.
        backup_path: String,

        /// The hash to convert.
        hash: String,
    },

    #[cfg(all(unix, feature = "fuse_unix"))]
    /// Mount a backup to a specified mount point.
    Mount {
        /// The hostname to mount.
        #[clap(long)]
        hostname: Option<String>,

        /// The backup number to mount.
        #[clap(long)]
        backup_number: Option<usize>,

        /// The path to mount.
        #[clap(long)]
        path: Option<String>,

        /// The mount point.
        mount_point: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    env_logger::init();

    let context = Context::default();

    let args = Cli::parse();

    let subcommand = args.subcommand.expect("No subcommand provided");
    match subcommand {
        Commands::ReadProtobuf {
            path,
            format,
            filter_name,
            filter_chunks,
        } => read_protobuf(&path, &format, filter_name.as_ref(), filter_chunks.as_ref())
            .await
            .expect("Failed to read protobuf file"),

        Commands::ReadLog {
            hostname,
            backup_number,
            share_path,
        } => {
            read_log(&GlobalConfiguration, &hostname, backup_number, &share_path)
                .await
                .expect("Failed to read log");
        }

        Commands::GetChunk { chunk } => {
            read_chunk(&GlobalConfiguration.path.pool_path, &chunk)
                .await
                .expect("Failed to read chunk");
        }
        Commands::SearchChunk { chunk } => {
            search_chunk(&GlobalConfiguration, &chunk)
                .await
                .expect("Failed to search chunk");
        }
        Commands::CompactRefcnt {} => {
            apply_pending_refcnt_operations(&GlobalConfiguration, &SystemTime::now())
                .await
                .expect("Failed to compact refcnt");
        }
        Commands::CleanUnused { target } => {
            clean_unused_pool(&GlobalConfiguration, context.source, target)
                .await
                .expect("Clean unused failed");
        }
        Commands::CheckCompression {} => check_compression(&GlobalConfiguration)
            .await
            .expect("Failed to check compression"),
        Commands::Fsck {
            dry_run,
            chunks,
            skip_ref_unused,
        } => {
            verify_all(
                &GlobalConfiguration,
                context.source,
                dry_run,
                chunks,
                skip_ref_unused,
            )
            .await
            .expect("Can't verify refcnt");
        }
        Commands::Compare {
            file_manifest_source,
            file_manifest_target,
        } => {
            compare(&file_manifest_source, &file_manifest_target)
                .await
                .expect("Failed to compare file manifest");
        }
        Commands::ResolveHost { hostname } => {
            resolve_mdns(&GlobalConfiguration, &hostname)
                .await
                .expect("Failed to resolve mDNS");
        }
        Commands::ConvertHashRepo { backup_path, hash } => {
            convert_hash_repo(&backup_path, &hash)
                .await
                .expect("Failed to convert hash repository");
        }
        #[cfg(all(unix, feature = "fuse_unix"))]
        Commands::Mount {
            hostname,
            backup_number,
            path,
            mount_point,
        } => {
            mount(
                &GlobalConfiguration,
                &MountOption {
                    hostname,
                    backup_number,
                    path,
                    mount_point,
                },
            )
            .await
            .expect("Failed to mount");
        }
    }

    Ok(())
}
