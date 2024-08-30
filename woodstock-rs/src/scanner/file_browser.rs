use eyre::Result;
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
use nix::NixPath;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

use crate::utils::path::path_to_vec;
use crate::woodstock::{FileManifest, FileManifestStat, FileManifestType};
use crate::{EntryState, EntryType, FileManifestAcl, FileManifestJournalEntry, FileManifestXAttr};

#[derive(Clone)]
pub struct CreateManifestOptions {
    pub with_acl: bool,
    pub with_xattr: bool,
}

struct PathEntryWithError {
    pub path: PathBuf,
    pub state: EntryState,
    pub state_messages: Vec<String>,
}

impl PathEntryWithError {
    pub fn with_path(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            state: EntryState::Metadata,
            state_messages: Vec::new(),
        }
    }
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

fn read_xattr(file: &Path) -> Result<Vec<FileManifestXAttr>> {
    #[cfg(all(unix, feature = "xattr"))]
    {
        let attrs = xattr::list(file).map(|attrs| {
            attrs
                .filter_map(|attr| {
                    xattr::get(file, &attr).ok()?.map(|value| {
                        let key = attr.as_encoded_bytes().to_vec();
                        FileManifestXAttr { key, value }
                    })
                })
                .collect()
        })?;

        Ok(attrs)
    }
    #[cfg(not(all(unix, feature = "xattr")))]
    {
        let _file = file;
        Ok(Vec::new())
    }
}

fn read_acl(file: &Path) -> Result<Vec<FileManifestAcl>> {
    #[cfg(all(unix, feature = "acl"))]
    {
        use crate::FileManifestAclQualifier;
        use posix_acl::{PosixACL, Qualifier};

        let acls: PosixACL = PosixACL::read_acl(file)?;
        let acls = acls.entries();

        let acl = acls
            .iter()
            .map(|entry| {
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

                FileManifestAcl {
                    qualifier: qualifier as i32,
                    id,
                    perm: entry.perm,
                }
            })
            .collect();

        Ok(acl)
    }

    #[cfg(not(all(unix, feature = "acl")))]
    {
        let _file = file;
        Ok(Vec::new())
    }
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
    entry: PathEntryWithError,
    options: &CreateManifestOptions,
) -> FileManifestJournalEntry {
    let file = share_path.join(&entry.path);
    let mut state = entry.state;
    let mut state_messages = entry.state_messages;

    // Check if user has access to the file
    let metadata = file.symlink_metadata();

    let (symlink, stats) = match metadata {
        Ok(metadata) => {
            let symlink = if metadata.is_symlink() {
                path_to_vec(file.read_link().unwrap_or_default().as_path())
            } else {
                Vec::new()
            };
            (symlink, Some(create_stats_from_metadata(&metadata)))
        }
        Err(e) => {
            state = EntryState::Error;
            state_messages.push(format!("{:#}", e));
            (Vec::new(), None)
        }
    };

    let xattr = if options.with_xattr {
        match read_xattr(&file) {
            Ok(xattr) => xattr,
            Err(e) => {
                state = EntryState::PartialMetadata;
                state_messages.push(format!("{:#}", e));
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    let acl = if options.with_acl {
        match read_acl(&file) {
            Ok(acl) => acl,
            Err(e) => {
                state = EntryState::PartialMetadata;
                state_messages.push(format!("{:#}", e));
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    FileManifestJournalEntry {
        r#type: EntryType::Add as i32,
        manifest: Some(FileManifest {
            path: path_to_vec(&entry.path),
            stats,

            xattr,
            acl,
            chunks: Vec::new(),
            hash: Vec::new(),
            symlink,

            metadata: HashMap::new(),
        }),

        state: state as i32,
        state_messages,
    }
}

async fn one_level(
    path: &PathBuf,
    to_visit: &mut Vec<PathBuf>,

    share_path: &Path,
    includes: &GlobSet,
    excludes: &GlobSet,
) -> Vec<PathEntryWithError> {
    if !path.is_empty() && !is_file_authorized(path, includes, excludes) {
        return Vec::new();
    }

    let mut entry = PathEntryWithError::with_path(path);
    let mut dir = match tokio::fs::read_dir(&share_path.join(path)).await {
        Ok(dir) => dir,
        Err(e) => {
            entry.state = EntryState::Error;
            entry.state_messages.push(format!("{:#}", e));
            return vec![entry];
        }
    };

    let mut files = Vec::new();

    loop {
        let child = dir.next_entry().await;

        // En cas d'erreur, on log
        let child = match child {
            Ok(child) => child,
            Err(e) => {
                entry.state = EntryState::Error;
                entry.state_messages.push(format!("{:#}", e));
                break;
            }
        };
        // Si vide on continue
        let Some(child) = child else {
            break;
        };

        let child_path = path.join(child.file_name());
        let mut child_entry = PathEntryWithError::with_path(&child_path);

        let metadata = match child.metadata().await {
            Ok(metadata) => metadata,
            Err(e) => {
                child_entry.state = EntryState::Error;
                child_entry.state_messages.push(format!("{:#}", e));
                files.push(child_entry);
                continue;
            }
        };

        // Si c'est un dossier, on ajoute à la liste des dossiers à visiter
        if metadata.is_dir() {
            to_visit.push(child_path);
        } else if is_file_authorized(&child_path, includes, excludes) {
            files.push(child_entry);
        }
    }

    files.push(entry);

    files
}

fn get_files_recursive(
    share_path: &Path,
    includes: &GlobSet,
    excludes: &GlobSet,
) -> impl Stream<Item = PathEntryWithError> + Send + 'static {
    let share_path = share_path.to_path_buf();
    let includes = includes.clone();
    let excludes = excludes.clone();

    futures::stream::unfold(
        (vec![PathBuf::from("")], share_path, includes, excludes),
        |(mut to_visit, share_path, includes, excludes)| async {
            let path: PathBuf = to_visit.pop()?;

            let file_stream =
                one_level(&path, &mut to_visit, &share_path, &includes, &excludes).await;
            let file_stream = futures::stream::iter(file_stream);

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
) -> impl Stream<Item = FileManifestJournalEntry> + 'a {
    stream!({
        let files = get_files_recursive(share_path, &includes, &excludes);
        pin_mut!(files);

        loop {
            match files.next().await {
                Some(entry) => {
                    yield create_manifest_from_file(share_path, entry, options);
                }
                None => break,
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::path::{list_to_globset, vec_to_path};

    #[tokio::test]
    async fn test_get_files() {
        let dir_path = Path::new("./data");

        let includes = list_to_globset(&["*.pem"]).unwrap();
        let excludes = list_to_globset(&[]).unwrap();

        let options = CreateManifestOptions {
            with_acl: false,
            with_xattr: false,
        };

        let stream = get_files(dir_path, &includes, &excludes, &options);
        pin_mut!(stream);

        let mut filenames = Vec::new();

        while let Some(entry) = stream.next().await {
            let path = vec_to_path(&entry.manifest.unwrap().path);
            println!("{:?} {:?} {:?}", path, entry.state, entry.state_messages);
            filenames.push(path);
        }

        assert!(filenames.len() >= 3);
        assert!(filenames.contains(&PathBuf::from("")));
        assert!(filenames.contains(&PathBuf::from("private_key.pem")));
        assert!(filenames.contains(&PathBuf::from("public_key.pem")));
    }
}
