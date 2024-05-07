#[cfg(unix)]
use nix::libc::{S_IFBLK, S_IFCHR, S_IFDIR, S_IFIFO, S_IFLNK, S_IFMT, S_IFREG, S_IFSOCK};

#[cfg(windows)]
const FILE_ATTRIBUTE_DIRECTORY: u32 = 16u32;
#[cfg(windows)]
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 1024u32;

use async_stream::stream;
use futures::pin_mut;
use futures::stream::StreamExt;
use futures::Stream;
use globset::GlobSet;
use log::error;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{io, path::Path};
use tokio::fs::DirEntry;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

use crate::utils::path::path_to_vec;
use crate::woodstock::{FileManifest, FileManifestStat, FileManifestType};
use crate::{FileManifestAcl, FileManifestXAttr};

#[derive(Clone)]
pub struct CreateManifestOptions {
    pub with_acl: bool,
    pub with_xattr: bool,
}

/// Checks if a file is authorized based on the include and exclude patterns.
///
/// # Arguments
///
/// * `file` - The file to check.
/// * `includes` - The include patterns.
/// * `excludes` - The exclude patterns.
///
/// # Returns
///
/// `true` if the file is authorized, `false` otherwise.
///
fn is_file_authorized(file: &Path, includes: &GlobSet, excludes: &GlobSet) -> bool {
    if !includes.is_empty() && !includes.is_match(file) {
        return false;
    }

    if !excludes.is_empty() && excludes.is_match(file) {
        return false;
    }

    true
}

#[cfg(unix)]
fn create_stats_from_metadata(metadata: &std::fs::Metadata) -> FileManifestStat {
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

#[cfg(windows)]
fn create_stats_from_metadata(metadata: &std::fs::Metadata) -> FileManifestStat {
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

#[must_use]
pub fn read_xattr(file: &Path) -> Vec<FileManifestXAttr> {
    let mut xattr = Vec::new();

    #[cfg(unix)]
    #[cfg(feature = "xattr")]
    {
        if let Ok(attrs) = xattr::list(file) {
            for attr in attrs {
                let value = if let Ok(value) = xattr::get(file, &attr) {
                    value.unwrap_or_else(Vec::new)
                } else {
                    Vec::new()
                };

                let key = attr.as_encoded_bytes().to_vec();
                xattr.push(FileManifestXAttr { key, value });
            }
        }
    }

    xattr
}

#[must_use]
pub fn read_acl(file: &Path) -> Vec<FileManifestAcl> {
    let mut results = Vec::new();

    #[cfg(unix)]
    #[cfg(feature = "acl")]
    {
        use crate::FileManifestAclQualifier;
        use posix_acl::{PosixACL, Qualifier};

        let acls: Result<PosixACL, posix_acl::ACLError> = PosixACL::read_acl(file);

        if let Ok(acl) = acls {
            for entry in acl.entries() {
                let mut id = 0;
                let qualifier = match entry.qual {
                    Qualifier::Undefined => FileManifestAclQualifier::Undefined,
                    Qualifier::UserObj => FileManifestAclQualifier::UserObj,
                    Qualifier::User(user) => {
                        id = user;
                        FileManifestAclQualifier::UserId
                    }
                    Qualifier::GroupObj => FileManifestAclQualifier::GroupObj,
                    Qualifier::Group(group) => {
                        id = group;
                        FileManifestAclQualifier::GroupId
                    }
                    Qualifier::Mask => FileManifestAclQualifier::Mask,
                    Qualifier::Other => FileManifestAclQualifier::Other,
                };

                results.push(FileManifestAcl {
                    qualifier: qualifier as i32,
                    id,
                    perm: entry.perm,
                });
            }
        }
    }

    results
}

/// Creates a `FileManifest` from a file.
///
/// # Arguments
///
/// * `share_path` - The path to the share.
/// * `path` - The path to the file.
///
/// # Returns
///
/// A `FileManifest` representing the file.
///
fn create_manifest_from_file(
    share_path: &Path,
    path: &Path,
    options: &CreateManifestOptions,
) -> Result<FileManifest, io::Error> {
    let file = share_path.join(path);

    // Check if user has access to the file
    let metadata = file.symlink_metadata()?;

    let symlink = if metadata.is_symlink() {
        path_to_vec(file.read_link().unwrap_or_default().as_path())
    } else {
        Vec::new()
    };

    let xattr = if options.with_xattr {
        read_xattr(&file)
    } else {
        Vec::new()
    };

    let acl = if options.with_acl {
        read_acl(&file)
    } else {
        Vec::new()
    };

    Ok(FileManifest {
        path: path_to_vec(path),
        stats: Some(create_stats_from_metadata(&metadata)),

        xattr,
        acl,
        chunks: Vec::new(),
        symlink,

        metadata: HashMap::new(),
    })
}

fn get_files_recursive(
    share_path: &Path,
    includes: &GlobSet,
    excludes: &GlobSet,
) -> impl Stream<Item = DirEntry> + Send + 'static {
    async fn one_level(
        path: PathBuf,
        to_visit: &mut Vec<PathBuf>,
        share_path: &PathBuf,
        includes: &GlobSet,
        excludes: &GlobSet,
    ) -> io::Result<Vec<DirEntry>> {
        let reduced_path = path.strip_prefix(share_path).unwrap();
        if !is_file_authorized(reduced_path, includes, excludes) {
            return Ok(Vec::new());
        }

        let mut dir = tokio::fs::read_dir(path).await?;
        let mut files = Vec::new();

        while let Some(child) = dir.next_entry().await? {
            if child.metadata().await?.is_dir() {
                to_visit.push(child.path());
            }

            files.push(child);
        }

        Ok(files)
    }

    let share_path = share_path.to_path_buf();
    let includes = includes.clone();
    let excludes = excludes.clone();

    futures::stream::unfold(
        (vec![share_path.clone()], share_path, includes, excludes),
        |(mut to_visit, share_path, includes, excludes)| async {
            let path: PathBuf = to_visit.pop()?;

            let file_stream =
                match one_level(path, &mut to_visit, &share_path, &includes, &excludes).await {
                    Ok(files) => futures::stream::iter(files).left_stream(),
                    Err(e) => futures::stream::iter({
                        error!("Can't read the file in directory: {}", e);
                        Vec::<DirEntry>::new()
                    })
                    .right_stream(),
                };

            Some((file_stream, (to_visit, share_path, includes, excludes)))
        },
    )
    .flatten()
}

/// Returns a stream of `FileManifest` for all authorized files in a directory and its subdirectories.
///
/// # Arguments
///
/// * `share_path` - The path to the share.
/// * `includes` - The include patterns.
/// * `excludes` - The exclude patterns.
///
/// # Returns
///
/// A stream of `FileManifest`.
///
pub fn get_files<'a>(
    share_path: &'a Path,
    includes: &'a GlobSet,
    excludes: &'a GlobSet,
    options: &'a CreateManifestOptions,
) -> impl Stream<Item = FileManifest> + 'a {
    stream!({
        let files = get_files_recursive(share_path, &includes, &excludes);
        pin_mut!(files);

        loop {
            match files.next().await {
                Some(entry) => {
                    // Transform entry.path() to be relative to share_path
                    let entry_path = entry.path();
                    let path = entry_path.strip_prefix(share_path).unwrap();

                    let manifest = create_manifest_from_file(share_path, &path, options);
                    match manifest {
                        Ok(manifest) => yield manifest,
                        Err(e) => {
                            error!(
                                "Can't read the file {}: {}",
                                path.to_str().unwrap_or_default(),
                                e
                            );
                        }
                    }
                }
                None => break,
            }
        }
    })
}
