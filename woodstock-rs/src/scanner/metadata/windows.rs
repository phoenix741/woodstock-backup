use eyre::Result;
use std::path::Path;

use crate::FileManifest;
use crate::FileManifestStat;
use crate::FileManifestType;
use std::os::windows::fs::MetadataExt;

const FILE_ATTRIBUTE_DIRECTORY: u32 = 16u32;
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 1024u32;

pub fn create_stats_from_metadata(metadata: &std::fs::Metadata) -> FileManifestStat {
    FileManifestStat {
        owner_id: 0,
        group_id: 0,
        size: metadata.file_size(),
        compressed_size: 0,
        last_read: metadata.last_access_time() as i64,
        last_modified: metadata.last_write_time() as i64,
        created: metadata.creation_time() as i64,
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
