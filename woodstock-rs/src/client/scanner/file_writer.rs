/// File writing module for the backup system.
///
/// This module provides functionality for restoring files from backup manifests,
/// including recreation of all file attributes such as permissions, ownership,
/// extended attributes, and access control lists. It handles different file types
/// including regular files, directories, symlinks, and special files like devices.
use eyre::Result;
use std::fs::OpenOptions;

use crate::{
    utils::path::vec_to_path,
    woodstock::{FileManifest, FileManifestType},
};

use super::metadata::{
    acl::restore_acl, create_symlink, mknode, restore_permissions, xattr::restore_xattr,
};

/// Creates a file from a manifest.
///
/// This function restores a file based on the provided manifest, recreating all of its
/// attributes including file type, permissions, extended attributes, and access control lists.
/// It handles different file types including regular files, directories, symlinks, and 
/// special device files.
///
/// # Arguments
///
/// * `file_manifest` - The `FileManifest` containing the file information to restore.
///
/// # Returns
///
/// A `Result` indicating success or failure of the file creation operation.
///
/// # Errors
///
/// Returns an error if:
/// - Parent directories cannot be created
/// - The file cannot be created with the appropriate type
/// - Permissions cannot be restored
/// - Extended attributes or ACLs cannot be applied (warnings logged)
///
/// # Panics
///
/// Will not panic but will log warnings if non-critical operations like restoring
/// extended attributes or ACLs fail.
pub fn create_file_from_manifest(file_manifest: &FileManifest) -> Result<()> {
    // Create an empty file, restore owner, group, mode, link, socket, fifo, block, char, symlink, xattr, acl, and metadata
    let path = file_manifest.path();

    // Create parent directory if not exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    match file_manifest.file_mode() {
        FileManifestType::BlockDevice
        | FileManifestType::CharacterDevice
        | FileManifestType::Fifo
        | FileManifestType::Socket => {
            mknode(file_manifest)?;
        }

        crate::FileManifestType::Directory => {
            std::fs::create_dir_all(&path)?;
        }
        crate::FileManifestType::Symlink => {
            let symlink = vec_to_path(&file_manifest.symlink);
            create_symlink(&path, &symlink)?;
        }
        _ => {
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)?;
        }
    }

    restore_permissions(&path, file_manifest)?;

    let _ = restore_xattr(&path, &file_manifest.xattr).inspect_err(|err| {
        log::warn!("Failed to restore xattr: {}", err);
    });
    let _ = restore_acl(&path, &file_manifest.acl).inspect_err(|err| {
        log::warn!("Failed to restore acl: {}", err);
    });

    Ok(())
}
