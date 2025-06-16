/// Unix-specific metadata handling implementation.
///
/// This module provides implementation for reading, creating, and restoring file system
/// metadata on Unix platforms. It uses native Unix file system APIs to handle various
/// file types, permissions, and attributes.
use eyre::Result;
use nix::libc::{S_IFBLK, S_IFCHR, S_IFDIR, S_IFIFO, S_IFLNK, S_IFMT, S_IFREG, S_IFSOCK};
use std::fs::File;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::FileManifest;
use crate::FileManifestStat;
use crate::FileManifestType;

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
        r#type: match metadata.mode() & S_IFMT {
            S_IFREG => FileManifestType::RegularFile,
            S_IFLNK => FileManifestType::Symlink,
            S_IFDIR => FileManifestType::Directory,
            S_IFBLK => FileManifestType::BlockDevice,
            S_IFCHR => FileManifestType::CharacterDevice,
            S_IFIFO => FileManifestType::Fifo,
            S_IFSOCK => FileManifestType::Socket,
            _ => FileManifestType::Unknown,
        } as i32,
    }
}

/// Creates a special node (device file, FIFO, socket) in the file system.
///
/// This function creates special file system nodes based on the type specified
/// in the file manifest. It uses the Unix `mknod()` syscall to create these special files.
///
/// # Arguments
/// * `file_manifest` - The file manifest containing the type of node to create,
///                     its path, and other metadata.
///
/// # Returns
/// * `Result<()>` - Success or error result.
///
/// # Errors
/// Returns an error if:
/// * The mknod syscall fails (due to permissions, invalid path, etc.)
/// * The requested file type is not supported
///
/// # Platform-specific details
/// This function can create the following special file types on Unix:
/// * Block devices
/// * Character devices
/// * FIFOs (named pipes)
/// * Unix domain sockets
pub fn mknode(file_manifest: &FileManifest) -> Result<()> {
    use libc::{dev_t, mknod, mode_t, S_IFBLK, S_IFCHR, S_IFIFO, S_IFSOCK};
    use std::os::unix::ffi::OsStrExt;

    let path = file_manifest.path();

    let mode_filter = match file_manifest.file_mode() {
        FileManifestType::BlockDevice => S_IFBLK,
        FileManifestType::CharacterDevice => S_IFCHR,
        FileManifestType::Fifo => S_IFIFO,
        FileManifestType::Socket => S_IFSOCK,
        _ => 0,
    };

    let dev = file_manifest.stats.map(|s| s.dev).unwrap_or_default() as dev_t;
    let mode = file_manifest.mode() as mode_t | mode_filter;
    let result = unsafe { mknod(path.as_os_str().as_bytes().as_ptr().cast::<i8>(), mode, dev) };
    if result != 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    Ok(())
}

/// Creates a symbolic link in the file system.
///
/// This function creates a symbolic link at the specified path, pointing to the target.
///
/// # Arguments
/// * `path` - The path where the symbolic link should be created.
/// * `symlink` - The path that the symbolic link should point to (the target).
///
/// # Returns
/// * `Result<()>` - Success or error result.
///
/// # Errors
/// Returns an error if:
/// * The symlink creation fails (due to permissions, invalid path, etc.)
///
/// # Type Parameters
/// * `P: AsRef<Path>` - A type that can be referenced as a path, typically a string or Path.
pub fn create_symlink<P: AsRef<Path>>(path: P, symlink: P) -> Result<()> {
    std::os::unix::fs::symlink(symlink, path)?;

    Ok(())
}

/// Restores permissions to a file based on the information in a file manifest.
///
/// This function sets file permissions (mode bits) on a file based on the metadata
/// stored in the file manifest.
///
/// # Arguments
/// * `path` - The path to the file to modify.
/// * `file_manifest` - The file manifest containing the permission information.
///
/// # Returns
/// * `Result<()>` - Success or error result.
///
/// # Errors
/// Returns an error if:
/// * The file cannot be opened
/// * The permissions cannot be set
///
/// # Type Parameters
/// * `P: AsRef<Path>` - A type that can be referenced as a path, typically a string or Path.
///
/// # Implementation Details
/// This function only restores the permission bits (0o777) from the file mode,
/// ignoring other mode bits like the file type.
pub fn restore_permissions<P: AsRef<Path>>(path: P, file_manifest: &FileManifest) -> Result<()> {
    let file = File::open(&path)?;
    let mut permissions = file.metadata()?.permissions();
    permissions.set_mode(file_manifest.mode() & 0o777);
    file.set_permissions(permissions)?;

    Ok(())
}
