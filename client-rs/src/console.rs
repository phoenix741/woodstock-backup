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

use clap::{Parser, Subcommand};
use eyre::Result;
use woodstock::ChunkAlgorithm;

use crate::commands::client::list_client_files;
use commands::client::read_chunk_from_file;
use woodstock::config::GlobalConfiguration;

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
    /// List directory like the client will do on the share directory.
    ///
    /// The scan is made on the computer where the command is run but the config
    /// will be take in the `CONFIG_DIRECTORY` (like on server).
    ///
    /// This command can be used for debugging purpose.
    ///
    /// For DEBUG purpose only.
    ListDirectory {
        /// Config path.
        config_path: String,

        /// The share path to scan.
        share_path: String,
    },

    /// Read the file and return the list of hash of the file.
    ///
    /// For DEBUG purpose only.
    ReadFileChunk {
        /// The path to the file to read.
        file_name: String,

        /// The algorithm to use.
        algorithm: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    env_logger::init();

    let args = Cli::parse();

    let subcommand = args.subcommand.expect("No subcommand provided");
    match subcommand {
        Commands::ListDirectory {
            share_path,
            config_path,
        } => list_client_files(&share_path, &config_path, &GlobalConfiguration)
            .await
            .expect("Failed to list files"),
        Commands::ReadFileChunk {
            file_name,
            algorithm,
        } => read_chunk_from_file(
            &file_name,
            algorithm
                .as_deref()
                .unwrap_or(ChunkAlgorithm::Blake3.as_str_name()),
        )
        .await
        .expect("Failed to read chunk from file"),
    }

    Ok(())
}
