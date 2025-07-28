//! This module provides pool management commands for Woodstock backups.
//!
//! It includes utilities for checking compression, verifying pool integrity, and cleaning unused data, ensuring the health and efficiency of the backup storage pool.
//!
//! # Errors
//!
//! Functions in this module may return errors if pool files are missing, corrupted, or if locking or I/O operations fail.
//!
//! # Panics
//!
//! Some functions may panic if internal invariants are violated or if system resources are unavailable.

use std::{cell::RefCell, path::PathBuf};

use console::Term;
use eyre::Result;
use indicatif::{HumanBytes, HumanCount, ProgressBar, ProgressStyle};
use tracing::error;

use woodstock::{
    pool::Refcnt,
    server::pool::{
        fsck_machine::FsckMachine, fsck_state::FsckExecutionState,
        pool_cleaner_machine::PoolCleanerMachine, pool_cleaner_state::CleanerExecutionState,
    },
    utils::lock_redis::PoolLockRedis,
    EventSource,
};

use crate::commands::CliServiceState;

/// Checks the compression ratios of all chunks in the pool and reports anomalies.
///
/// # Arguments
///
/// * `config` - The Woodstock configuration containing the pool path.
///
/// # Errors
///
/// Returns an error if the pool cannot be locked or if I/O operations fail.
///
/// # Panics
///
/// This function does not explicitly panic.
pub async fn check_compression(state: CliServiceState) -> Result<()> {
    let redis_url = state.config.redis_url();
    let _lock = PoolLockRedis::new_with_path(
        &redis_url,
        &state.config.path.pool_path,
        "check_compression",
    )
    .await?
    .lock_exclusive()
    .await?;

    let mut pool_refcnt = Refcnt::new(&state.config.path.pool_path);
    pool_refcnt.load_refcnt(false).await;

    let mut compressed_size: u64 = 0;
    let mut uncompressed_size: u64 = 0;
    let mut error_count: u64 = 0;
    let mut total_count: u64 = 0;

    let term = Term::stdout();

    for refcnt in pool_refcnt.list_refcnt() {
        total_count += 1;
        compressed_size += refcnt.compressed_size;
        uncompressed_size += refcnt.size;

        if refcnt.compressed_size > refcnt.size {
            error_count += 1;
            let hash_str = hex::encode(&refcnt.sha256);
            error!(
                "{}: compressed size {} is greater than uncompressed size {}",
                hash_str,
                HumanBytes(refcnt.compressed_size),
                HumanBytes(refcnt.size)
            );
        }
    }

    term.write_line(&std::format!(
        "Total errors: {}/{}",
        HumanCount(error_count),
        HumanCount(total_count)
    ))?;
    term.write_line(&std::format!(
        "Total compressed size: {}",
        HumanBytes(compressed_size)
    ))?;
    term.write_line(&std::format!(
        "Total uncompressed size: {}",
        HumanBytes(uncompressed_size)
    ))?;

    Ok(())
}

/// Returns a human-readable message for the current fsck execution state.
///
/// # Arguments
///
/// * `state` - The current fsck execution state.
///
/// # Returns
///
/// A string describing the current step in the fsck process.
fn fsck_message_from_state(state: &FsckExecutionState) -> String {
    match state {
        FsckExecutionState::Initialization => "Initializing verification".to_string(),
        FsckExecutionState::ApplyingRefcnt => "Applying reference counts".to_string(),
        FsckExecutionState::VerifyRefcnt => "Verifying references".to_string(),
        FsckExecutionState::VerifyUnused => "Verifying unused files".to_string(),
        FsckExecutionState::VerifyChunk => "Verifying chunks".to_string(),
        FsckExecutionState::Completed => "Verification completed".to_string(),
        FsckExecutionState::Waiting => "Waiting for verification".to_string(),
    }
}

/// Verifies the integrity of all chunks in the pool, optionally as a dry run.
///
/// # Arguments
///
/// * `config` - The Woodstock configuration containing the pool path.
/// * `source` - The event source for reporting progress.
/// * `dry_run` - If true, performs a dry run without making changes.
/// * `verify_chunks` - If true, verifies the chunks in the pool.
/// * `skip_ref_unused` - If true, skips the verification of refcnt and unused files.
///
/// # Errors
///
/// Returns an error if verification fails or if I/O operations fail.
///
/// # Panics
///
/// This function does not explicitly panic.
pub async fn verify_all(
    state: CliServiceState,
    source: EventSource,
    dry_run: bool,
    verify_chunks: bool,
    skip_ref_unused: bool,
) -> Result<()> {
    let term = Term::stdout();

    // Create a channel to receive states
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    // Create the fsck machine
    let machine = FsckMachine::new(
        source,
        dry_run,
        verify_chunks,
        skip_ref_unused,
        Some(tx),
        state.config.clone(),
        state.hosts.clone(),
        state.backups.clone(),
    );

    // Create a single progress bar for all processes
    let progress_bar = ProgressBar::no_length();
    progress_bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {percent_precise}% {human_pos}/{human_len} ETA: {eta} {msg}",
        )
        // SAFETY: hardcoded template string is always valid
        .unwrap(),
    );
    progress_bar.set_message("Initializing...");

    let previous_execution_state = RefCell::new(FsckExecutionState::Waiting);

    let term_clone = term.clone();
    // Launch a task to update the display
    let display_task = tokio::spawn(async move {
        while let Some(state) = rx.recv().await {
            let current_execution_state = state.execution_state.clone();
            if current_execution_state != *previous_execution_state.borrow() {
                progress_bar.set_message(fsck_message_from_state(&current_execution_state));
                *previous_execution_state.borrow_mut() = current_execution_state;
            }

            // Calculate overall progress percentage
            match state.execution_state {
                FsckExecutionState::Initialization
                | FsckExecutionState::ApplyingRefcnt
                | FsckExecutionState::VerifyRefcnt
                | FsckExecutionState::VerifyUnused
                | FsckExecutionState::VerifyChunk => {
                    let length = state.chunk_progression.progress_max
                        + state.refcnt_progression.progress_max
                        + state.unused_progression.progress_max;

                    if progress_bar.length().is_none() && length > 0 {
                        progress_bar.set_position(0);
                        progress_bar.set_length(length as u64);
                    }

                    let position = state.refcnt_progression.progress_current
                        + state.unused_progression.progress_current
                        + state.chunk_progression.progress_current;
                    progress_bar.set_position(position as u64);
                }
                FsckExecutionState::Completed => {
                    progress_bar.finish_with_message("Verification completed");

                    // Display summary — terminal writes are best-effort in a display task
                    let _ = term.write_line("");
                    let _ = term.write_line(&format!(
                        "Reference count verification results: {} errors/{}",
                        state.refcnt_progression.error_count, state.refcnt_progression.total_count
                    ));

                    let _ = term.write_line("Unused files verification results:");
                    let _ = term.write_line(&format!(
                        "- In refcnt: {}",
                        HumanCount(state.unused_progression.in_refcnt as u64)
                    ));
                    let _ = term.write_line(&format!(
                        "- In unused: {}",
                        HumanCount(state.unused_progression.in_unused as u64)
                    ));
                    let _ = term.write_line(&format!(
                        "- In nothing: {}",
                        HumanCount(state.unused_progression.in_nothing as u64)
                    ));
                    let _ = term.write_line(&format!(
                        "- Missing: {}",
                        HumanCount(state.unused_progression.missing as u64)
                    ));

                    if verify_chunks {
                        let _ = term.write_line(&format!(
                            "Chunks verification results: {} errors/{}",
                            state.chunk_progression.error_count,
                            state.chunk_progression.total_count
                        ));
                    }
                }
                FsckExecutionState::Waiting => {
                    // Do nothing
                }
            }
        }
    });

    // Execute verification
    let result = machine.execute().await;

    // Make sure the display task ends
    drop(machine);
    display_task.await?;

    if let Err(e) = result {
        term_clone.write_line(&format!("Error during verification: {e}"))?;
    }

    Ok(())
}

/// Cleans unused chunks from the pool, optionally targeting a specific backup.
///
/// # Arguments
///
/// * `config` - The Woodstock configuration containing the pool path.
/// * `source` - The event source for reporting progress.
/// * `target` - An optional target backup to clean unused chunks for.
///
/// # Errors
///
/// Returns an error if cleaning fails or if I/O operations fail.
///
/// # Panics
///
/// This function does not explicitly panic.
pub async fn clean_unused_pool(
    state: CliServiceState,
    source: EventSource,
    target: Option<String>,
) -> Result<()> {
    let target = target.map(PathBuf::from);

    let term = Term::stdout();

    // Create a channel to receive states
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    // Create the pool cleaner machine
    let machine = PoolCleanerMachine::new(target, source, Some(tx), state.config.clone());

    // Create a progress bar
    let bar = ProgressBar::new(0);
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {percent_precise}% {human_pos}/{human_len} ETA: {eta} {msg}",
        )
        // SAFETY: hardcoded template string is always valid
        .unwrap(),
    );

    // Launch a task to update the display
    let display_task = tokio::spawn(async move {
        while let Some(state) = rx.recv().await {
            // Update the progress bar according to the state
            match state.execution_state {
                CleanerExecutionState::Initialization => {
                    bar.set_message("Initializing pool cleaner...");
                }
                CleanerExecutionState::ApplyingRefcnt => {
                    bar.set_message("Applying reference counts...");
                }
                CleanerExecutionState::Cleaning => {
                    let total = state.progression.progress_max as u64;
                    let current = state.progression.progress_current as u64;

                    if bar
                        .length()
                        .is_some_and(|length| length != total && total > 0)
                    {
                        bar.set_length(total);
                    }

                    bar.set_position(current);
                }
                CleanerExecutionState::Completed => {
                    bar.finish();
                }
                CleanerExecutionState::Waiting => {
                    // Do nothing
                }
            }
        }
    });

    // Execute the cleaning
    let result = machine.execute().await?;

    // Make sure the display task ends
    display_task.abort();

    term.write_line(&std::format!(
        "Total removed: {}",
        HumanBytes(result.progression.compressed_file_size)
    ))?;

    Ok(())
}
