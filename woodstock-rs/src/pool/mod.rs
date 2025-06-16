//! # Woodstock Pool Module
//!
//! This module provides the core logic for managing the deduplicated chunk pool in the Woodstock backup system.
//! It includes chunk file management, reference counting, unused chunk tracking, integrity checking (fsck),
//! and utilities for chunk path calculation and atomic operations.
//!
//! ## Main Structures
//!
//! - [`PoolChunkInformation`], [`PoolChunkWrapper`], [`PoolChunkWriter`], [`Refcnt`]
//!
//! ## Usage
//!
//! The pool module is used by backup, restore, and maintenance operations to manage chunk files,
//! track usage, and ensure data integrity.
//!
//! ## Error Handling & Panics
//!
//! - All async methods return `Result` and propagate errors using the `eyre` crate.
//! - Panics are not expected under normal operation; assertion failures indicate programming errors.
//!
//! # Pool Module
//!
//! This module manages the storage pool for backups, including file integrity checks and metadata management.
//!
//! ## Submodules
//!
//! * `fsck` - Provides functionality for checking and repairing the integrity of the storage pool.
//!
//! ## Features
//!
//! * Efficient storage management
//! * Integrity checks to ensure data consistency
//! * Support for repairing corrupted data
//!
//! ## Usage
//!
//! Use the `fsck` submodule to perform integrity checks and repairs on the storage pool. Refer to the submodule documentation for detailed usage examples.

/// Module with method to check the integrity of the pool
mod fsck;
/// Module with chunk information and metadata
mod pool_chunk_information;
/// Module with chunk file operations and metadata
mod pool_chunk_wrapper;
/// Module with async writer for compressed chunk files
mod pool_chunk_wrapper_writer;
/// Module with display and formatting for reference counts and unused chunks
mod pool_refcnt;
/// Module with reference count and unused chunk management
mod refcnt;
/// Module with utility functions for chunk path and temp file management
mod utils;

pub use fsck::*;
pub use pool_chunk_information::*;
pub use pool_chunk_wrapper::*;
pub use pool_chunk_wrapper_writer::*;
pub use refcnt::*;
use tokio::fs::{read_dir, remove_file};
pub use utils::*;

use crate::{
    config::Configuration,
    utils::{files::copy_file, lock::PoolLock},
};

use eyre::Result;
use log::{debug, error, info};
use std::{path::Path, time::SystemTime};

/// Adds reference counts from a specific backup to the pool.
///
/// # Arguments
///
/// * `config` - Reference to the Woodstock [`Configuration`] struct.
/// * `filename` - The original reference count to add
/// * `hostname` - The hostname associated with the backup.
/// * `backup_number` - The backup number to add.
///
/// # Returns
///
/// * `Ok(())` if the operation succeeds.
/// * `Err(eyre::Report)` if an error occurs.
///
/// # Errors
///
/// Returns an error if the backup cannot be loaded or the reference counts cannot be applied.
///
/// # Panics
///
/// This function does not panic under normal operation.
pub async fn add_refcnt_to_pool<P: AsRef<Path>>(
    config: &Configuration,
    filename: P,
    hostname: &str,
    backup_number: usize,
) -> Result<()> {
    let pool_refcnt_path = &config.path.pool_refcnt_path;

    let source_path = filename.as_ref().join("REFCNT");
    let destination_path = pool_refcnt_path.join(format!("{}_{}.add", hostname, backup_number));

    info!("Add refcnt to pool for {}", source_path.display());
    copy_file(source_path, destination_path).await?;
    info!("Refcnt applied to pool");

    Ok(())
}

/// Removes reference counts from a specific backup from the pool.
///
/// # Arguments
///
/// * `config` - Reference to the Woodstock [`Configuration`] struct.
/// * `filename` - The original reference count to remove
/// * `hostname` - The hostname associated with the backup.
/// * `backup_number` - The backup number to remove.
///
/// # Returns
///
/// * `Ok(())` if the operation succeeds.
/// * `Err(eyre::Report)` if an error occurs.
///
/// # Errors
///
/// Returns an error if the backup cannot be loaded or the reference counts cannot be removed.
///
/// # Panics
///
/// This function does not panic under normal operation.
pub async fn remove_refcnt_to_pool<P: AsRef<Path>>(
    config: &Configuration,
    filename: P,
    hostname: &str,
    backup_number: usize,
) -> Result<()> {
    let pool_refcnt_path = &config.path.pool_refcnt_path;

    let source_path = filename.as_ref().join("REFCNT");
    let destination_path = pool_refcnt_path.join(format!("{}_{}.remove", hostname, backup_number));

    info!("Remove refcnt from pool for {}", source_path.display());
    copy_file(source_path, destination_path).await?;
    info!("Refcnt removed from pool");

    Ok(())
}

/// Applies all pending reference count operations from the refcnt directory to the pool.
///
/// This function locks the pool, lists all .add and .remove files in the refcnt directory,
/// applies each operation to the global pool REFCNT, and removes the processed files.
///
/// # Arguments
///
/// * `config` - Reference to the Woodstock [`Configuration`] struct.
/// * `date` - The timestamp to use when saving the updated reference counts.
///
/// # Returns
///
/// * `Ok(())` if all operations are processed successfully.
/// * `Err(eyre::Report)` if an error occurs.
///
/// # Errors
///
/// Returns an error if:
/// * The pool cannot be locked
/// * The refcnt directory cannot be read
/// * Reference counts cannot be loaded or saved
/// * The pool finishing operation fails
///
/// # Panics
///
/// This function does not panic under normal operation.
pub async fn apply_pending_refcnt_operations(
    config: &Configuration,
    date: &SystemTime,
) -> Result<()> {
    info!("Applying pending refcnt operations...");

    let pool_directory = &config.path.pool_path;
    let pool_refcnt_path = &config.path.pool_refcnt_path;
    let _lock = PoolLock::new_with_name(&pool_directory, "apply_pending_refcnt_operations")
        .lock_shared()
        .await?;

    // Load global REFCNT and UNUSED once
    let mut pool_refcnt = Refcnt::load_refcnt_from_path(pool_directory).await?;

    let Ok(mut dir) = read_dir(pool_refcnt_path).await else {
        info!("No pending refcnt operations found.");
        return Ok(());
    };

    let mut add_files = Vec::new();
    let mut remove_files = Vec::new();

    // Collect all .add and .remove files separately
    while let Some(entry) = dir.next_entry().await? {
        let path = entry.path();
        let Some(extension) = path.extension().and_then(|s| s.to_str()) else {
            continue;
        };

        if extension == "add" {
            add_files.push(path);
        } else if extension == "remove" {
            remove_files.push(path);
        }
    }

    if add_files.is_empty() && remove_files.is_empty() {
        debug!("No pending refcnt operations found.");
        return Ok(());
    }

    let mut files_to_remove = Vec::new();

    // Process .add files first
    for path in add_files {
        info!("Applying add refcnt operation from file: {path:?}");

        let backup_refcnt = Refcnt::load_refcnt_from_file(&path).await?;
        pool_refcnt.apply_all(&backup_refcnt, &RefcntApplySens::Increase);

        files_to_remove.push(path);
    }

    // Process .remove files second
    for path in remove_files {
        info!("Applying remove refcnt operation from file: {path:?}");

        let backup_refcnt = Refcnt::load_refcnt_from_file(&path).await?;
        pool_refcnt.apply_all(&backup_refcnt, &RefcntApplySens::Decrease);

        files_to_remove.push(path);
    }

    pool_refcnt.repair(&pool_directory).await?;
    pool_refcnt.save_refcnt(date, true).await?;

    // Remove all processed files
    for file in files_to_remove {
        if let Err(e) = remove_file(&file).await {
            error!("Failed to remove file {file:?}: {e}");
        } else {
            debug!("Removed file: {file:?}");
        }
    }

    info!("All pending refcnt operations applied successfully.");
    Ok(())
}
