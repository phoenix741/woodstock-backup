/// Windows-specific metadata handling implementation.
///
/// This module provides implementation for reading file system metadata on
/// Windows platforms during scanning. It handles Windows-specific file
/// attributes and converts Windows time formats to Unix time for
/// compatibility. Restore-side operations live in
/// `woodstock::utils::restore_metadata` — shared with
/// `archiving::fs_materialize`, see that module's doc comment.
use std::os::windows::fs::MetadataExt;
use woodstock::FileManifestStat;
use woodstock::FileManifestType;

/// Windows file attribute constant for directory.
const FILE_ATTRIBUTE_DIRECTORY: u32 = 16u32;

/// Windows file attribute constant for reparse point (which includes symlinks).
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 1024u32;

/// Constants for converting Windows time (100-nanoseconds since 1601-01-01) to Unix time
/// 11644473600 is the number of seconds between January 1, 1601 and January 1, 1970
const WINDOWS_TICK: u64 = 10_000_000;
const SEC_TO_UNIX_EPOCH: u64 = 11_644_473_600;

/// Converts Windows file time to Unix timestamp.
///
/// Windows time is represented as the number of 100-nanosecond intervals
/// since January 1, 1601 UTC. This function converts it to Unix time
/// (seconds since January 1, 1970 UTC).
///
/// # Arguments
/// * `win_time` - The Windows timestamp to convert.
///
/// # Returns
/// The equivalent Unix timestamp as an i64, or 0 if the input is invalid.
///
/// # Safety
/// This function includes protection against:
/// * Zero input (returns 0)
/// * Timestamps that would be before the Unix epoch (returns 0)
fn windows_time_to_unix_seconds(win_time: u64) -> i64 {
    if win_time == 0 {
        return 0;
    }

    let seconds = win_time / WINDOWS_TICK;
    if seconds < SEC_TO_UNIX_EPOCH {
        return 0; // Protection against invalid dates
    }

    (seconds - SEC_TO_UNIX_EPOCH) as i64
}

/// Creates a FileManifestStat structure from a file's metadata on Windows systems.
///
/// This function extracts available metadata information from a Windows file and
/// constructs a FileManifestStat structure containing these details. It handles
/// Windows-specific attributes and file times, converting them to a format
/// compatible with the cross-platform manifest.
///
/// # Arguments
/// * `metadata` - Reference to the file's metadata obtained from the file system.
///
/// # Returns
/// A FileManifestStat structure containing all available metadata information.
///
/// # Platform-specific details
/// This function:
/// * Sets owner_id and group_id to 0 (not used on Windows)
/// * Converts Windows file times to Unix timestamps
/// * Uses Windows file attributes to determine the file type
/// * Sets Unix-specific fields (dev, rdev, ino, nlink) to 0
pub fn create_stats_from_metadata(metadata: &std::fs::Metadata) -> FileManifestStat {
    FileManifestStat {
        owner_id: 0,
        group_id: 0,
        size: metadata.file_size(),
        compressed_size: 0,
        last_read: windows_time_to_unix_seconds(metadata.last_access_time()),
        last_modified: windows_time_to_unix_seconds(metadata.last_write_time()),
        created: windows_time_to_unix_seconds(metadata.creation_time()),
        mode: metadata.file_attributes(),
        dev: 0,
        rdev: 0,
        ino: 0,
        nlink: 0,
        file_type: if (metadata.file_attributes() & FILE_ATTRIBUTE_DIRECTORY)
            == FILE_ATTRIBUTE_DIRECTORY
        {
            FileManifestType::Directory.into()
        } else if (metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT)
            == FILE_ATTRIBUTE_REPARSE_POINT
        {
            FileManifestType::Symlink.into()
        } else {
            FileManifestType::RegularFile.into()
        },
    }
}
