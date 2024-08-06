#![recursion_limit = "512"]

//! The goal of this module is to permit to manage the pool.
//!
//! The command can be used to
//!
//! * remove unused chunks
//! * check all chunks
//! * recalculate all the chunks
//!

use clap::{Parser, Subcommand};
use commands::file_manifest::compare;
use commands::read_chunk::search_chunk;
use eyre::Result;

mod commands;

use crate::commands::client::list_client_files;
use crate::commands::pool::{
    check_compression, clean_unused_pool, verify_chunk, verify_refcnt, verify_unused,
};
use crate::commands::read_chunk::read_chunk;
use crate::commands::read_protobuf::{read_protobuf, ProtobufFormat};
use woodstock::config::Context;
use woodstock::pool::{add_refcnt_to_pool, remove_refcnt_to_pool};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    subcommand: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    ReadProtobuf {
        /// The path to the file to read
        path: String,

        /// The type of the file to read
        format: ProtobufFormat,

        /// Filter the output by filename
        #[clap(long)]
        filter_name: Option<String>,

        /// Filter the output by file chunks
        #[clap(long)]
        filter_chunks: Option<String>,
    },

    GetChunk {
        /// The chunk to get
        chunk: String,
    },

    /// Searcg manifest that contains the chunk
    SearchChunk {
        /// The chunk to get
        chunk: String,
    },

    AddRefCntToPool {
        /// The hostname of the backup to add to pool
        hostname: String,

        /// The backup number of the backup to add to pool
        backup_number: usize,
    },

    RemoveRefCntToPool {
        /// The hostname of the backup to add to pool
        hostname: String,

        /// The backup number of the backup to add to pool
        backup_number: usize,
    },

    CleanUnused {
        #[clap(short, long)]
        target: Option<String>,
    },

    CheckCompression {},

    VerifyChunk {},

    VerifyRefcnt {
        #[clap(short, long, default_value_t = false)]
        dry_run: bool,
    },

    VerifyUnused {
        #[clap(short, long, default_value_t = false)]
        dry_run: bool,
    },

    /// Can be used to compare two file manifest (and generate a journal file)
    Compare {
        file_manifest_source: String,

        file_manifest_target: String,
    },

    /// List directory like the client will do on the share directory
    /// The scan is made on the computer where the command is run but the config
    /// will be take in the CONFIG_DIRECTORY (like on server)
    ///
    /// This command can be used for debugging purpose
    ListDirectory {
        /// The hostname to scan
        hostname: String,

        /// The share path to scan
        share_path: String,
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
        } => read_protobuf(&path, &format, &filter_name, &filter_chunks)
            .await
            .expect("Failed to read protobuf file"),

        Commands::GetChunk { chunk } => {
            read_chunk(&context.config.path.pool_path, &chunk).expect("Failed to read chunk");
        }
        Commands::SearchChunk { chunk } => {
            search_chunk(&context, &chunk)
                .await
                .expect("Failed to search chunk");
        }
        Commands::AddRefCntToPool {
            hostname,
            backup_number,
        } => {
            add_refcnt_to_pool(&context, &hostname, backup_number)
                .await
                .expect("Failed to add refcnt to pool");
        }
        Commands::RemoveRefCntToPool {
            hostname,
            backup_number,
        } => {
            remove_refcnt_to_pool(&context, &hostname, backup_number)
                .await
                .expect("Failed to remove refcnt to pool");
        }
        Commands::CleanUnused { target } => {
            clean_unused_pool(&context, target)
                .await
                .expect("Clean unused failed");
        }
        Commands::CheckCompression {} => check_compression(&context)
            .await
            .expect("Failed to check compression"),
        Commands::VerifyChunk {} => verify_chunk(&context)
            .await
            .expect("Can't verify integrity"),
        Commands::VerifyRefcnt { dry_run } => {
            verify_refcnt(&context, dry_run)
                .await
                .expect("Can't verify refcnt");
        }
        Commands::VerifyUnused { dry_run } => {
            verify_unused(&context, dry_run)
                .await
                .expect("Can't verify unused");
        }
        Commands::Compare {
            file_manifest_source,
            file_manifest_target,
        } => {
            compare(&file_manifest_source, &file_manifest_target)
                .await
                .expect("Failed to compare file manifest");
        }
        Commands::ListDirectory {
            hostname,
            share_path,
        } => list_client_files(&hostname, &share_path, &context)
            .await
            .expect("Failed to list files"),
    }

    Ok(())
}