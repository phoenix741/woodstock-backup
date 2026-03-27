//! This module provides commands for data conversion and migration in Woodstock backups.
//!
//! It includes functions for converting hash algorithms and reporting conversion progress, supporting pool upgrades and format changes.
//!
//! # Errors
//!
//! Functions in this module may return errors if conversion fails, if files are missing or corrupted, or if I/O operations fail.
//!
//! # Panics
//!
//! Some functions may panic if system resources are unavailable or if I/O operations fail unexpectedly.

use eyre::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::{cell::RefCell, path::PathBuf, sync::Arc};
use tokio::sync::mpsc;

use woodstock::{
    config::{Configuration, ConfigurationPath, DEFAULT_CHANNEL_BUFFER_SIZE},
    pool::{ChunkDescriptor, ChunkIndex, PoolChunkWrapper, Refcnt, Segments},
    server::pool::{
        convert_state::{ConvertExecutionState, ConvertState},
        hash_converter_machine::HashConverterMachine,
    },
    utils::compression::CompressionFormat,
    ChunkAlgorithm, EventSource,
};

use crate::commands::CliServiceState;

/// Returns a human-readable message describing the current conversion execution state.
///
/// # Arguments
///
/// * `state` - The current execution state of the conversion process.
///
/// # Returns
///
/// A string describing the current step in the conversion process.
fn message_from_state(state: &ConvertExecutionState) -> String {
    match state {
        ConvertExecutionState::Waiting => "Waiting to start conversion".to_string(),
        ConvertExecutionState::Initialization => "Analyzing pool size".to_string(),
        ConvertExecutionState::Converting => "Converting hash algorithm".to_string(),
        ConvertExecutionState::Completed => "Conversion completed".to_string(),
    }
}

/// Converts the hash algorithm used in a Woodstock backup repository.
///
/// # Arguments
///
/// * `backup_path` - The path to the backup repository to convert.
/// * `hash` - The name of the new chunk hash algorithm to use.
///
/// # Errors
///
/// Returns an error if the hash algorithm is invalid, the configuration cannot be loaded, or the conversion process fails.
///
/// # Panics
///
/// This function does not explicitly panic.
pub async fn convert_hash_repo(backup_path: &str, hash: &str) -> Result<()> {
    let hash = ChunkAlgorithm::from_str_name(hash)
        .ok_or_else(|| eyre::eyre!("Invalid chunk algorithm: {}", hash))?;

    let backup_path = PathBuf::from(backup_path);
    let configuration = Configuration::from_backup_path(backup_path);

    let hosts_path = configuration
        .path
        .hosts_path
        .clone()
        .with_extension(hash.as_str_name());

    let pool_path = configuration
        .path
        .pool_path
        .clone()
        .with_extension(hash.as_str_name());

    let new_configuration = configuration.clone();
    let new_configuration = Configuration {
        path: ConfigurationPath {
            hosts_path,
            pool_path,
            ..new_configuration.path
        },
        chunk_algorithm: hash,
        ..new_configuration
    };

    let configuration = Arc::new(configuration);
    let new_configuration = Arc::new(new_configuration);
    let scheduler = Arc::new(woodstock::config::Scheduler::new(configuration.clone()));
    let hosts = Arc::new(woodstock::config::Hosts::new(
        configuration.clone(),
        scheduler,
    ));
    let backups = Arc::new(woodstock::config::Backups::new(configuration.clone()));

    let previous_execution_state = RefCell::new(ConvertExecutionState::Waiting);
    let (tx, mut rx) = mpsc::channel::<ConvertState>(DEFAULT_CHANNEL_BUFFER_SIZE);

    let bar = Arc::new(ProgressBar::new(0));
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {percent_precise}% {pos}/{len} ETA: {eta}",
        )
        // SAFETY: hardcoded template string is always valid
        .unwrap(),
    );

    let bar_clone = Arc::clone(&bar);
    // Create a task to process conversion states
    let progress_task = tokio::spawn(async move {
        while let Some(state) = rx.recv().await {
            let current_execution_state = state.execution_state.clone();
            if current_execution_state != *previous_execution_state.borrow() {
                println!("{}", message_from_state(&current_execution_state));
                *previous_execution_state.borrow_mut() = current_execution_state;
            }

            bar_clone.set_length(state.global_progression.progress_max as u64);
            bar_clone.set_position(state.global_progression.progress_current as u64);
        }
    });

    // Create and run the conversion machine
    let converter = HashConverterMachine::new(
        EventSource::Cli,
        Some(tx),
        configuration,
        hosts,
        backups,
        new_configuration,
    );

    // Execute the conversion
    converter.execute().await?;

    drop(converter);

    // Wait for the progress task to finish
    let _ = progress_task.await;

    bar.finish();

    Ok(())
}

pub async fn convert_flat_to_segment(state: CliServiceState) -> Result<()> {
    // Read all existing chunk (and metadata) and write them to a segment file with the new format
    let mut refcnt = Refcnt::new(&state.config.path.pool_path);
    refcnt.load_refcnt(false).await;

    let segments = Segments::new(state.config.clone());
    let mut segment = segments.get_writer().await?;

    let mut chunk_index = ChunkIndex::new(state.config.clone());
    let mut index_writer = chunk_index.get_writer().await?;

    println!("[memory] avant conversion : RSS = {} KiB", read_rss_kib());

    let list = refcnt.list_refcnt();
    for chunk in list {
        let wrapper = PoolChunkWrapper::new(&state.config.path.pool_path, Some(&chunk.sha256));
        let Ok(metadata) = wrapper.chunk_information().await else {
            eprintln!(
                "Failed to read chunk information for chunk with hash {} - refcount {}",
                hex::encode(&chunk.sha256),
                chunk.ref_count
            );
            continue;
        };

        // Capture the segment_id BEFORE the append — the writer may rotate
        // internally after the write, changing `segment.header()`.
        let segment_id = segment.header().segment_id;

        let Ok(entry) = segment
            .append_chunk_from_path(
                chunk.sha256.clone(),
                metadata.size,
                metadata.compressed_size,
                CompressionFormat::try_from(metadata.format)?,
                &wrapper.chunk_path(),
            )
            .await
        else {
            eprintln!(
                "Failed to append chunk with hash {} to segment file - refcount {}",
                hex::encode(&chunk.sha256),
                chunk.ref_count
            );
            continue;
        };

        let descriptor = ChunkDescriptor {
            hash: entry.hash,
            segment_id,
            offset: entry.header_offset,
            size: entry.size,
            compressed_size: entry.compressed_size,
            header_size: u32::try_from(entry.chunk_header_size).unwrap_or(u32::MAX),
            compression_format: entry.compression_format.as_u32(),
            refcount: u64::from(chunk.ref_count),
        };

        if let Err(e) = index_writer.add(descriptor).await {
            eprintln!(
                "Failed to queue index entry for chunk {} : {e}",
                hex::encode(&chunk.sha256)
            );
        }
    }

    println!(
        "[memory] avant flush de l'index : RSS = {} KiB",
        read_rss_kib()
    );

    index_writer.shutdown().await?;
    segment.shutdown().await?;

    Ok(())
}

/// Reads the current resident set size (RSS) of the process from `/proc/self/status`.
///
/// Returns the value in KiB, or 0 if the file cannot be read or parsed.
fn read_rss_kib() -> u64 {
    let Ok(status) = std::fs::read_to_string("/proc/self/status") else {
        return 0;
    };
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            // Format: "VmRSS:  12345 kB"
            if let Some(kib) = rest.split_whitespace().next().and_then(|s| s.parse().ok()) {
                return kib;
            }
        }
    }
    0
}
