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
mod backup_resolver;
mod commands;
#[cfg(all(unix, feature = "fuse_unix"))]
mod filesystem;

use crate::backup_resolver::resolve_backup_id;
use crate::commands::convertion::convert_flat_to_segment;
use chrono::Local;
use clap::{Parser, Subcommand};
use commands::convertion::convert_hash_repo;
use commands::file_manifest::compare;
use commands::read_chunk::search_chunk;
use commands::read_protobuf::read_log;
use commands::resolve::resolve_mdns;
use eyre::{Result, WrapErr};

#[cfg(all(unix, feature = "fuse_unix"))]
use commands::mount::{mount, MountOption};

use crate::commands::pool::{check_compression, clean_unused_pool, verify_all};
use crate::commands::read_chunk::read_chunk;
use crate::commands::read_protobuf::{read_protobuf, ProtobufFormat};
use crate::commands::CliServiceState;
use woodstock::config::Context;
use woodstock::pool::PoolManager;
use woodstock::utils::lock_redis::{LockOperation, PoolLockOperation, PoolLockRedis};

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

    /// Read and display the backup log for a given host and backup.
    ReadLog {
        /// The hostname of the backup server.
        hostname: String,

        /// The backup identifier (UUID or sequential number).
        backup_id: String,

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

    /// Convert to Segment File
    ConvertFlatToSegment {},

    #[cfg(all(unix, feature = "fuse_unix"))]
    /// Mount a backup to a specified mount point.
    Mount {
        /// The hostname to mount.
        #[clap(long)]
        hostname: Option<String>,

        /// The backup identifier to mount (UUID or sequential number, optional).
        #[clap(long)]
        backup_id: Option<String>,

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

    tracing_subscriber::fmt::init();

    let context = Context::default();
    let state = CliServiceState::default();

    let args = Cli::parse();

    let subcommand = args
        .subcommand
        .ok_or_else(|| eyre::eyre!("No subcommand provided"))?;
    match subcommand {
        Commands::ReadProtobuf {
            path,
            format,
            filter_name,
            filter_chunks,
        } => read_protobuf(&path, &format, filter_name.as_ref(), filter_chunks.as_ref())
            .await
            .wrap_err("Failed to read protobuf file")?,

        Commands::ReadLog {
            hostname,
            backup_id,
            share_path,
        } => {
            let (uuid, _number) = resolve_backup_id(&backup_id, &hostname, &state.backups).await?;
            read_log(state, &hostname, uuid, &share_path)
                .await
                .wrap_err("Failed to read log")?;
        }

        Commands::GetChunk { chunk } => {
            read_chunk(&state.config.path.pool_path, &chunk)
                .await
                .wrap_err("Failed to read chunk")?;
        }
        Commands::SearchChunk { chunk } => {
            search_chunk(state, &chunk)
                .await
                .wrap_err("Failed to search chunk")?;
        }
        Commands::CompactRefcnt {} => {
            // Acquire EXCLUSIVE lock to prevent race conditions
            let redis_url = state.config.redis_url();
            let _lock = PoolLockRedis::new_with_path(
                &redis_url,
                &state.config.path.pool_path,
                LockOperation::Pool(PoolLockOperation::CompactRefcntManual),
            )
            .await
            .wrap_err("Failed to acquire lock")?
            .lock_exclusive()
            .await
            .wrap_err("Failed to acquire exclusive lock")?;

            PoolManager::new(state.config.clone())
                .apply_pending(&Local::now())
                .await
                .wrap_err("Failed to compact refcnt")?;
        }
        Commands::CleanUnused { target } => {
            clean_unused_pool(state, context.source, target)
                .await
                .wrap_err("Clean unused failed")?;
        }
        Commands::CheckCompression {} => check_compression(state)
            .await
            .wrap_err("Failed to check compression")?,
        Commands::Fsck {
            dry_run,
            chunks,
            skip_ref_unused,
        } => {
            verify_all(state, context.source, dry_run, chunks, skip_ref_unused)
                .await
                .wrap_err("Can't verify refcnt")?;
        }
        Commands::Compare {
            file_manifest_source,
            file_manifest_target,
        } => {
            compare(&file_manifest_source, &file_manifest_target)
                .await
                .wrap_err("Failed to compare file manifest")?;
        }
        Commands::ResolveHost { hostname } => {
            resolve_mdns(state, &hostname)
                .await
                .wrap_err("Failed to resolve mDNS")?;
        }
        Commands::ConvertHashRepo { backup_path, hash } => {
            convert_hash_repo(&backup_path, &hash)
                .await
                .wrap_err("Failed to convert hash repository")?;
        }
        Commands::ConvertFlatToSegment {} => {
            convert_flat_to_segment(state)
                .await
                .wrap_err("Failed to convert flat to segment")?;
        }
        #[cfg(all(unix, feature = "fuse_unix"))]
        Commands::Mount {
            hostname,
            backup_id,
            path,
            mount_point,
        } => {
            mount(
                state,
                &MountOption {
                    hostname,
                    backup_id,
                    path,
                    mount_point,
                },
            )
            .await
            .wrap_err("Failed to mount")?;
        }
    }

    Ok(())
}
