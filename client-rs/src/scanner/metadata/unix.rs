/// Unix-specific metadata handling implementation.
///
/// This module provides implementation for reading file system metadata on
/// Unix platforms during scanning. It uses native Unix file system APIs to
/// extract file types, permissions, and attributes. Restore-side operations
/// (creating special nodes, symlinks, restoring permissions) live in
/// `woodstock::utils::restore_metadata` — shared with
/// `archiving::fs_materialize`, see that module's doc comment.
use nix::libc::{S_IFBLK, S_IFCHR, S_IFDIR, S_IFIFO, S_IFLNK, S_IFMT, S_IFREG, S_IFSOCK};
use std::os::unix::fs::MetadataExt;

use woodstock::FileManifestStat;
use woodstock::FileManifestType;

/// Creates a `FileManifestStat` structure from a file's metadata on Unix systems.
///
/// This function extracts all relevant metadata information from a file and
/// constructs a `FileManifestStat` structure containing these details. It handles
/// Unix-specific metadata like user/group IDs, file modes, and special file types.
///
/// # Arguments
/// * `metadata` - Reference to the file's metadata obtained from the file system.
///
/// # Returns
/// A `FileManifestStat` structure containing all metadata information.
///
/// # Platform-specific details
/// This function extracts Unix-specific file attributes:
/// * Owner and group IDs
/// * File mode bits (permissions)
/// * Device IDs (for special files)
/// * Inode number
/// * Number of hard links
/// * File type (regular, directory, symbolic link, device, etc.)
pub fn create_stats_from_metadata(metadata: &std::fs::Metadata) -> FileManifestStat {
    // WARNING: `libc::S_IF*` constants are not consistently typed across Unix targets.
    // On FreeBSD some values are `u16` while `metadata.mode()` is `u32`.
    // We normalize everything to `u32` to keep matching portable and avoid
    // `u32 & u16` type mismatches during cross-compilation.
    let file_type_bits = metadata.mode() & (S_IFMT as u32);

    FileManifestStat {
        owner_id: metadata.uid(),
        group_id: metadata.gid(),
        size: metadata.size(),
        compressed_size: 0,
        last_read: metadata.atime(),
        last_modified: metadata.mtime(),
        created: metadata.ctime(),
        mode: metadata.mode(),
        dev: metadata.dev(),
        rdev: metadata.rdev(),
        ino: metadata.ino(),
        nlink: metadata.nlink(),
        file_type: match file_type_bits {
            x if x == S_IFREG as u32 => FileManifestType::RegularFile,
            x if x == S_IFLNK as u32 => FileManifestType::Symlink,
            x if x == S_IFDIR as u32 => FileManifestType::Directory,
            x if x == S_IFBLK as u32 => FileManifestType::BlockDevice,
            x if x == S_IFCHR as u32 => FileManifestType::CharacterDevice,
            x if x == S_IFIFO as u32 => FileManifestType::Fifo,
            x if x == S_IFSOCK as u32 => FileManifestType::Socket,
            _ => FileManifestType::Unknown,
        } as i32,
    }
}
