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
use log::info;
use std::{
    error::Error,
    io::{stdout, Write},
    path::Path,
};
use tokio::io::AsyncReadExt;

use futures::{pin_mut, StreamExt};
use woodstock::{
    config::{Backups, Configuration, Hosts, BUFFER_SIZE},
    pool::{PoolChunkWrapper, Refcnt},
    utils::compression::WoodstockCompressionReader,
};

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
pub async fn read_chunk(pool_path: &Path, chunk: &str) -> Result<(), Box<dyn Error>> {
    let chunk = hex::decode(chunk)?;
    let chunk = PoolChunkWrapper::new(pool_path, Some(&chunk));

    if !chunk.exists() {
        return Err("Chunk doesn't exist".into());
    }

    let chunk_path = chunk.chunk_path();

    // Read all the chunk content
    let file = tokio::fs::File::open(chunk_path).await?;
    let file = tokio::io::BufReader::new(file);
    let mut file = WoodstockCompressionReader::new(file);

    let mut buffer = vec![0; BUFFER_SIZE];
    loop {
        let read = file.read(&mut buffer).await?;
        if read == 0 {
            break;
        }

        stdout().write_all(&buffer[..read]).unwrap();
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
pub async fn search_chunk(config: &Configuration, chunk: &str) -> Result<(), Box<dyn Error>> {
    let term = Term::stdout();

    let chunk = hex::decode(chunk)?;

    let hosts_config = Hosts::new(config);
    let backups_config = Backups::new(config);

    let hosts = hosts_config.list_hosts().await.unwrap_or_default();
    for host in hosts {
        {
            // Heuristic, check if the chunk is in the host's refcnt
            let refcnt_path = backups_config.get_host_path(&host);
            let refcnt = Refcnt::load_refcnt_from_path(refcnt_path).await?;
            if refcnt.get_refcnt(&chunk).is_none() {
                continue;
            }
        }

        let backups = backups_config.get_backups(&host).await;
        for backup in backups {
            {
                // Heuristic, check if the chunk is in the backup's refcnt
                let refcnt_path =
                    backups_config.get_backup_destination_directory(&host, backup.number);
                let refcnt = Refcnt::load_refcnt_from_path(refcnt_path).await?;
                if refcnt.get_refcnt(&chunk).is_none() {
                    continue;
                }
            }

            let manifests = backups_config.get_manifests(&host, backup.number).await;

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
