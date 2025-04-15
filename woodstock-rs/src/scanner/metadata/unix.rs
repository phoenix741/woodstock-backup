use eyre::Result;
use nix::libc::{S_IFBLK, S_IFCHR, S_IFDIR, S_IFIFO, S_IFLNK, S_IFMT, S_IFREG, S_IFSOCK};
use std::fs::File;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::FileManifest;
use crate::FileManifestStat;
use crate::FileManifestType;

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

pub fn create_symlink<P: AsRef<Path>>(path: P, symlink: P) -> Result<()> {
    std::os::unix::fs::symlink(symlink, path)?;

    Ok(())
}

pub fn restore_permissions<P: AsRef<Path>>(path: P, file_manifest: &FileManifest) -> Result<()> {
    let file = File::open(&path)?;
    let mut permissions = file.metadata()?.permissions();
    permissions.set_mode(file_manifest.mode() & 0o777);
    file.set_permissions(permissions)?;

    Ok(())
}
