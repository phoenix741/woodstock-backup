use eyre::Result;
use std::path::Path;

use crate::FileManifest;
use crate::FileManifestStat;
use crate::FileManifestType;
use std::os::windows::fs::MetadataExt;

const FILE_ATTRIBUTE_DIRECTORY: u32 = 16u32;
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 1024u32;

// Constants for converting Windows time (100-nanoseconds since 1601-01-01) to Unix time
// 11644473600 is the number of seconds between January 1, 1601 and January 1, 1970
const WINDOWS_TICK: u64 = 10_000_000;
const SEC_TO_UNIX_EPOCH: u64 = 11_644_473_600;

// Function to convert Windows time to Unix time (seconds)
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
        r#type: if (metadata.file_attributes() & FILE_ATTRIBUTE_DIRECTORY)
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

pub fn mknode(_file_manifest: &FileManifest) -> Result<()> {
    Ok(())
}

pub fn create_symlink<P: AsRef<Path>>(path: P, symlink: P) -> Result<()> {
    std::os::windows::fs::symlink_file(symlink, path)?;

    Ok(())
}

pub fn restore_permissions<P: AsRef<Path>>(_path: P, _file_manifest: &FileManifest) -> Result<()> {
    Ok(())
}
