//! This module provides utilities for reading and displaying protobuf-encoded data from Woodstock backup files.
//!
//! It supports multiple protobuf formats, including file manifests, journal entries, reference counts, unused chunks, events, and chunk information. The module allows filtering and pretty-printing of protobuf data for inspection and debugging.
//!
//! # Errors
//!
//! Functions in this module may return errors if protobuf files are missing, corrupted, or if decoding fails.
//!
//! # Panics
//!
//! Some functions may panic if system resources are unavailable or if I/O operations fail unexpectedly.

use eyre::Result;
use std::ffi::OsString;

use clap::ValueEnum;
use futures::{pin_mut, StreamExt};

use woodstock::{
    pool::PoolChunkInformation, proto::ProtobufReader, Event, FileManifest,
    FileManifestJournalEntry, PoolRefCount, PoolUnused,
};

use crate::commands::CliServiceState;

/// Supported protobuf formats for reading Woodstock backup data.
///
/// - `FileManifest`: Represents a file manifest.
/// - `FileManifestJournalEntry`: Represents a journal entry for file manifests.
/// - `RefCount`: Represents reference count information for pool chunks.
/// - `Unused`: Represents unused chunk information.
/// - `Event`: Represents backup or restore events.
/// - `ChunkInformation`: Represents detailed chunk information.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ProtobufFormat {
    /// File manifest protobuf format.
    FileManifest,
    /// File manifest journal entry protobuf format.
    FileManifestJournalEntry,
    /// Reference count protobuf format for pool chunks.
    RefCount,
    /// Unused chunk protobuf format.
    Unused,
    /// Event protobuf format for backup/restore events.
    Event,
    /// Chunk information protobuf format.
    ChunkInformation,
}

/// Reads and prints protobuf-encoded data from a file, supporting multiple formats and optional filtering.
///
/// # Arguments
///
/// * `path` - The path to the protobuf file to read.
/// * `format` - The protobuf format to decode.
/// * `filter_name` - Optional filter for file names.
/// * `filter_chunks` - Optional filter for chunk identifiers.
///
/// # Errors
///
/// Returns an error if the file cannot be read, decoded, or if filtering fails.
///
/// # Panics
///
/// This function does not explicitly panic.
pub async fn read_protobuf(
    path: &str,
    format: &ProtobufFormat,
    filter_name: Option<&String>,
    filter_chunks: Option<&String>,
) -> Result<()> {
    match format {
        ProtobufFormat::FileManifest => {
            let mut messages = ProtobufReader::<FileManifest>::new(path, true).await?;
            let mut messages = messages.into_stream();

            while let Some(message) = messages.next().await {
                let message = message?;

                // Filter the output by file name
                if let Some(filter_name) = filter_name {
                    let filter_name: OsString = filter_name.into();

                    let path = message.path();
                    let path = path.file_name();
                    if let Some(path) = path {
                        if path != filter_name {
                            continue;
                        }
                    }
                }

                // Filter the output by chunks
                if let Some(filter_chunks) = filter_chunks {
                    let filter_chunks = hex::decode(filter_chunks)?;
                    // Check if on of the chunks of the file is in the message
                    if !message.chunks.iter().any(|chunk| chunk == &filter_chunks) {
                        continue;
                    }
                }

                let message_str = match message.to_yaml() {
                    Ok(message_str) => message_str,
                    Err(err) => format!("Error: {err}"),
                };

                print!("{message_str}");
            }
        }
        ProtobufFormat::FileManifestJournalEntry => {
            let mut messages = ProtobufReader::<FileManifestJournalEntry>::new(path, true).await?;
            let mut messages = messages.into_stream();

            while let Some(message) = messages.next().await {
                let message = message?;
                // Filter the output by file name
                if let Some(filter_name) = filter_name {
                    let filter_name: OsString = filter_name.into();

                    if let Some(manifest) = &message.manifest {
                        let path = manifest.path();
                        let path = path.file_name();
                        if let Some(path) = path {
                            if path != filter_name {
                                continue;
                            }
                        }
                    }
                }

                // Filter the output by chunks
                if let Some(filter_chunks) = filter_chunks {
                    let filter_chunks = hex::decode(filter_chunks)?;
                    // Check if on of the chunks of the file is in the message
                    if let Some(manifest) = &message.manifest {
                        if !manifest.chunks.iter().any(|chunk| chunk == &filter_chunks) {
                            continue;
                        }
                    }
                }

                let message_str = match message.to_yaml() {
                    Ok(message_str) => message_str,
                    Err(err) => format!("Error: {err}"),
                };

                print!("{message_str}");
            }
        }
        ProtobufFormat::RefCount => {
            let mut messages = ProtobufReader::<PoolRefCount>::new(path, true).await?;
            let mut messages = messages.into_stream();

            while let Some(message) = messages.next().await {
                let message = message?;
                // Filter the output by chunks
                if let Some(filter_chunks) = filter_chunks {
                    let filter_chunks = hex::decode(filter_chunks)?;

                    if message.sha256 != filter_chunks {
                        continue;
                    }
                }

                print!("{message}");
            }
        }
        ProtobufFormat::Unused => {
            let mut messages = ProtobufReader::<PoolUnused>::new(path, true).await?;
            let mut messages = messages.into_stream();

            while let Some(message) = messages.next().await {
                let message = message?;
                // Filter the output by chunks
                if let Some(filter_chunks) = filter_chunks {
                    let filter_chunks = hex::decode(filter_chunks)?;

                    if message.sha256 != filter_chunks {
                        continue;
                    }
                }

                print!("{message}");
            }
        }
        ProtobufFormat::Event => {
            let mut messages = ProtobufReader::<Event>::new(path, false).await?;
            let mut messages = messages.into_stream();

            while let Some(message) = messages.next().await {
                let message = message?;

                print!("{message}");
            }
        }
        ProtobufFormat::ChunkInformation => {
            let mut messages = ProtobufReader::<PoolChunkInformation>::new(path, false).await?;
            let mut messages = messages.into_stream();

            while let Some(message) = messages.next().await {
                let message = message?;
                // Filter the output by chunks
                if let Some(filter_chunks) = filter_chunks {
                    let filter_chunks = hex::decode(filter_chunks)?;

                    if message.sha256 != filter_chunks {
                        continue;
                    }
                }

                print!("{message}");
            }
        }
    };

    Ok(())
}

/// Reads and prints the backup log for a given host and backup number.
///
/// # Arguments
///
/// * `config` - The Woodstock configuration containing backup paths.
/// * `hostname` - The hostname of the backup server.
/// * `backup_number` - The backup UUID to read the log for.
///
/// # Errors
///
/// Returns an error if the log file cannot be read or decoded.
///
/// # Panics
///
/// This function does not explicitly panic.
pub async fn read_log(
    state: CliServiceState,
    hostname: &str,
    backup_id: uuid::Uuid,
    share_path: &str,
) -> Result<()> {
    let manifest = state.backups.get_manifest(hostname, backup_id, share_path);

    let messages = manifest.read_log_entries();
    pin_mut!(messages);

    while let Some(message) = messages.next().await {
        let log_line = message.to_log();

        println!("{log_line}");
    }

    Ok(())
}
