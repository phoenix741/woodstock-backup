/// File writing module for the backup system.
///
/// This module provides functionality for restoring files from backup manifests,
/// including recreation of all file attributes such as permissions, ownership,
/// extended attributes, and access control lists. It handles different file types
/// including regular files, directories, symlinks, and special files like devices.
use eyre::Result;

use woodstock::{
    utils::{
        files::open_for_write_retrying_on_eacces,
        path::vec_to_path,
        restore_metadata::{
            acl::restore_acl, create_symlink, mknode, restore_permissions, xattr::restore_xattr,
        },
    },
    {FileManifest, FileManifestType},
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
            mknode(&path, file_manifest)?;
        }

        woodstock::FileManifestType::Directory => {
            std::fs::create_dir_all(&path)?;
        }
        woodstock::FileManifestType::Symlink => {
            let symlink = vec_to_path(&file_manifest.symlink);
            create_symlink(&path, &symlink)?;
        }
        _ => {
            open_for_write_retrying_on_eacces(&path)?;
        }
    }

    restore_permissions(&path, file_manifest.mode(), file_manifest.source_os())?;

    let _ = restore_xattr(&path, &file_manifest.xattr).inspect_err(|err| {
        tracing::warn!("Failed to restore xattr: {}", err);
    });
    let _ = restore_acl(&path, &file_manifest.acl).inspect_err(|err| {
        tracing::warn!("Failed to restore acl: {}", err);
    });

    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use woodstock::utils::path::path_to_vec;
    use woodstock::{FileManifestStat, SourceOs};

    fn regular_file_entry(path: &std::path::Path, mode: u32, source_os: SourceOs) -> FileManifest {
        FileManifest {
            path: path_to_vec(path),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::RegularFile as i32,
                mode,
                source_os: source_os as i32,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn unix_mode_bits(path: &std::path::Path) -> u32 {
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    /// Same self-healing this fix gives `fs_materialize::materialize_entry`
    /// (the archiving path) — a real restore onto a destination stuck at a
    /// restrictive mode (git's real `0o444` loose objects, or historical
    /// Windows-mode garbage) must not fail `EACCES` forever.
    #[test]
    fn restoring_over_a_file_stuck_at_a_restrictive_mode_now_succeeds() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("regular");

        std::fs::write(&path, b"old content").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o444)).unwrap();

        let entry = regular_file_entry(&path, 0o444, SourceOs::Unix);
        create_file_from_manifest(&entry)
            .expect("a legitimate real-Unix restrictive mode must not block restoring");

        assert_eq!(std::fs::read(&path).unwrap(), b"");
        assert_eq!(unix_mode_bits(&path), 0o444);
    }
}
