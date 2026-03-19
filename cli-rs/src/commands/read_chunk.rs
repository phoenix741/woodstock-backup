//! This module provides chunk reading and searching utilities for Woodstock backups.
//!
//! It includes functions to decode, decompress, print, and search for chunk contents in the backup pool for inspection or debugging purposes.
//!
//! # Errors
//!
//! Functions in this module may return errors if chunk files are missing, corrupted, or if decompression fails.
//!
//! # Panics
//!
//! Some functions may panic if system resources are unavailable or if I/O operations fail unexpectedly.

use console::Term;
use eyre::{Result, WrapErr};
use std::{
    io::{stdout, Write},
    path::Path,
};
use tokio::io::AsyncReadExt;
use tracing::info;

use futures::{pin_mut, StreamExt};
use woodstock::pool::{PoolChunkWrapper, PoolIndex};

use crate::commands::CliServiceState;

/// Reads and prints the content of a chunk from the backup pool, decompressing it if necessary.
///
/// # Arguments
///
/// * `pool_path` - The path to the backup pool directory.
/// * `chunk` - The hexadecimal string representing the chunk hash.
///
/// # Errors
///
/// Returns an error if the chunk does not exist, cannot be decoded, or if reading/decompression fails.
///
/// # Panics
///
/// This function may panic if writing to stdout fails.
pub async fn read_chunk(pool_path: &Path, chunk: &str) -> Result<()> {
    let chunk = hex::decode(chunk)?;
    let chunk = PoolChunkWrapper::new(pool_path, Some(&chunk));

    if !chunk.exists() {
        return Err(eyre::eyre!("Chunk doesn't exist"));
    }

    let mut file = chunk.open_chunk_reader().await?;
    let mut buffer = vec![0; woodstock::config::BUFFER_SIZE];
    loop {
        let read = file.read(&mut buffer).await?;
        if read == 0 {
            break;
        }

        stdout()
            .write_all(&buffer[..read])
            .wrap_err("Failed to write chunk to stdout")?;
    }

    Ok(())
}

/// Searches for a chunk in all backups and prints its location if found.
///
/// # Arguments
///
/// * `config` - The Woodstock configuration containing backup and pool paths.
/// * `chunk` - The hexadecimal string representing the chunk hash.
///
/// # Errors
///
/// Returns an error if the chunk cannot be decoded, if searching fails, or if I/O operations fail.
///
/// # Panics
///
/// This function does not explicitly panic.
pub async fn search_chunk(state: CliServiceState, chunk: &str) -> Result<()> {
    let term = Term::stdout();

    let chunk = hex::decode(chunk)?;
    let pool_index = PoolIndex::open_or_create(state.config.path.pool_path.join("index"))?;
    if pool_index.get_chunk(&chunk)?.is_none() {
        return Ok(());
    }

    let hosts = state.hosts.list_hosts().await.unwrap_or_default();
    for host in hosts {
        let backups = state.backups.get_backups(&host).await;
        for backup in backups {
            let manifests = state.backups.get_manifests(&host, backup.id).await;

            for manifest in manifests {
                info!(
                    "Process {}/{}/{}",
                    host, backup.number, manifest.manifest_name
                );
                let entries = manifest.read_all_message();
                pin_mut!(entries);

                while let Some(entry) = entries.next().await {
                    let found = entry.chunks.iter().filter(|c| chunk.eq(*c)).count();
                    if found > 0 {
                        term.write_line(&std::format!(
                            "{} chunk(s) found: {}/{}/{:?}",
                            found,
                            host,
                            backup.number,
                            entry.path()
                        ))?;
                    }
                }
            }
        }
    }

    Ok(())
}
