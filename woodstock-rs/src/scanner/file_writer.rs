use eyre::Result;
use std::fs::OpenOptions;

use crate::{
    utils::path::vec_to_path,
    woodstock::{FileManifest, FileManifestType},
};
#[cfg(any(all(unix, feature = "xattr"), all(unix, feature = "acl")))]
use std::path::Path;

#[cfg(all(unix, feature = "xattr"))]
use crate::woodstock::FileManifestXAttr;

#[cfg(all(unix, feature = "xattr"))]
fn restore_xattr(file: &Path, xattrs: &[FileManifestXAttr]) -> Result<()> {
    use std::ffi::OsString;
    use std::fs::File;
    use xattr::FileExt;

    let file = File::open(file)?;

    for xattr in xattrs {
        let key = unsafe { OsString::from_encoded_bytes_unchecked(xattr.key.clone()) };
        file.set_xattr(key, &xattr.value)?;
    }

    Ok(())
}

#[cfg(all(unix, feature = "acl"))]
use crate::woodstock::FileManifestAcl;

#[cfg(all(unix, feature = "acl"))]
fn restore_acl(file: &Path, acls: &[FileManifestAcl]) -> Result<()> {
    use crate::woodstock::FileManifestAclQualifier;
    use posix_acl::{PosixACL, Qualifier};

    let mut acls_writer: PosixACL = PosixACL::read_acl(file)?;

    for acl in acls {
        let qualifier = match acl.qualifier() {
            FileManifestAclQualifier::Undefined => Qualifier::Undefined,
            FileManifestAclQualifier::UserObj => Qualifier::UserObj,
            FileManifestAclQualifier::UserId => {
                let user = acl.id;
                Qualifier::User(user)
            }
            FileManifestAclQualifier::GroupObj => Qualifier::GroupObj,
            FileManifestAclQualifier::GroupId => {
                let group = acl.id;
                Qualifier::Group(group)
            }
            FileManifestAclQualifier::Mask => Qualifier::Mask,
            FileManifestAclQualifier::Other => Qualifier::Other,
        };

        acls_writer.set(qualifier, acl.perm);
    }

    acls_writer.write_acl(file)?;
    Ok(())
}

#[cfg(unix)]
fn mknode(file_manifest: &FileManifest) -> Result<()> {
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
    let result = unsafe { mknod(path.as_os_str().as_bytes().as_ptr() as *const i8, mode, dev) };
    if result != 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    Ok(())
}

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
            #[cfg(unix)]
            mknode(file_manifest)?;
        }

        crate::FileManifestType::Directory => {
            std::fs::create_dir_all(&path)?;
        }
        crate::FileManifestType::Symlink => {
            let symlink = vec_to_path(&file_manifest.symlink);
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(&symlink, &path)?;
            }
            #[cfg(windows)]
            {
                std::os::windows::fs::symlink_file(&symlink, &path)?;
            }
        }
        _ => {
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)?;
        }
    }

    #[cfg(unix)]
    {
        use std::fs::File;
        use std::os::unix::fs::PermissionsExt;

        let file = File::open(&path)?;
        let mut permissions = file.metadata()?.permissions();
        permissions.set_mode(file_manifest.mode() & 0o777);
        file.set_permissions(permissions)?;
    }

    #[cfg(all(unix, feature = "xattr"))]
    if let Err(err) = restore_xattr(&path, &file_manifest.xattr) {
        log::warn!("Failed to restore xattr: {}", err);
    }
    #[cfg(all(unix, feature = "acl"))]
    if let Err(err) = restore_acl(&path, &file_manifest.acl) {
        log::warn!("Failed to restore acl: {}", err);
    }

    Ok(())
}
