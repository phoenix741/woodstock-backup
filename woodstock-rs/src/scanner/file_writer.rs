use eyre::Result;
use std::fs::OpenOptions;

use crate::{
    utils::path::vec_to_path,
    woodstock::{FileManifest, FileManifestType},
};

use super::metadata::{
    acl::restore_acl, create_symlink, mknode, restore_permissions, xattr::restore_xattr,
};

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

    if let Err(err) = restore_xattr(&path, &file_manifest.xattr) {
        log::warn!("Failed to restore xattr: {}", err);
    }
    if let Err(err) = restore_acl(&path, &file_manifest.acl) {
        log::warn!("Failed to restore acl: {}", err);
    }

    Ok(())
}
