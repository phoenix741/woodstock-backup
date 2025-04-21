//! This module provides FUSE-based mounting commands for Woodstock backups (Unix only).
//!
//! It includes utilities for mounting backup data as a filesystem, allowing users to browse and access backup contents using standard file operations.
//!
//! # Errors
//!
//! Functions in this module may return errors if mounting fails, if system dependencies are missing, or if I/O operations fail.
//!
//! # Panics
//!
//! Some functions may panic if system resources are unavailable or if FUSE is misconfigured.

use std::path::PathBuf;

use eyre::Result;
use woodstock::config::Configuration;

use crate::filesystem::WoodstockFileSystem;

/// Options for mounting a Woodstock backup as a FUSE filesystem.
///
/// - `hostname`: The hostname of the backup server (optional).
/// - `backup_number`: The backup number to mount (optional).
/// - `path`: The path within the backup to mount (optional).
/// - `mount_point`: The local mount point for the FUSE filesystem.
pub struct MountOption {
    /// The hostname of the backup server.
    pub hostname: Option<String>,
    /// The backup number to mount.
    pub backup_number: Option<usize>,
    /// The path within the backup to mount.
    pub path: Option<String>,
    /// The local mount point for the FUSE filesystem.
    pub mount_point: String,
}

/// Mounts a Woodstock backup as a FUSE filesystem at the specified mount point.
///
/// # Arguments
///
/// * `config` - The Woodstock configuration containing backup paths.
/// * `options` - The mount options specifying hostname, backup number, path, and mount point.
///
/// # Errors
///
/// Returns an error if mounting fails, if system dependencies are missing, or if I/O operations fail.
///
/// # Panics
///
/// This function does not explicitly panic.
pub async fn mount(config: &Configuration, options: &MountOption) -> Result<()> {
    let path = vec![
        options.hostname.clone(),
        options.backup_number.map(|x| x.to_string()),
        options.path.clone(),
    ];
    let path = path
        .into_iter()
        .flatten()
        .collect::<Vec<String>>()
        .join("/");
    let path = format!("/{path}");

    // Mount path
    println!("Mounting path: {}", options.mount_point);
    if !path.is_empty() {
        println!("Prefix Path: {path}");
    }

    let path = PathBuf::from(&path);

    let fs = WoodstockFileSystem::new(config, &path);
    let mount_options = [];
    fuser::mount2(fs, &options.mount_point, &mount_options).unwrap();
    Ok(())
}
