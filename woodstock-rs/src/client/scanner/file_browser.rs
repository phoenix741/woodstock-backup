/// File browsing and discovery module for the backup system.
///
/// This module provides functionality for traversing the file system, discovering files, and
/// creating file manifests for backup operations. It handles file pattern matching through
/// include/exclude rules and extracts file metadata including permissions, access control
/// lists (ACLs), and extended attributes (xattr).
use async_stream::stream;
use futures::pin_mut;
use futures::stream::StreamExt;
use futures::Stream;
use globset::GlobSet;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use crate::utils::path::path_to_vec;
use crate::woodstock::FileManifest;
use crate::{EntryState, EntryType, FileManifestJournalEntry};

use lazy_static::lazy_static;

use super::metadata::acl::read_acl;
use super::metadata::create_stats_from_metadata;
use super::metadata::xattr::read_xattr;

lazy_static! {
    /// An empty path used as a starting point for directory traversal.
    ///
    /// This is used to begin the file system traversal at the root of the share path.
    static ref EMPTY_PATH: PathBuf = PathBuf::from("");
}

/// Options for creating file manifests.
///
/// These options control what additional metadata is included in file manifests.
#[derive(Clone)]
pub struct CreateManifestOptions {
    /// Whether to include Access Control Lists (ACLs) in the file manifest.
    pub with_acl: bool,
    /// Whether to include extended attributes (xattr) in the file manifest.
    pub with_xattr: bool,
}

/// Path entry with error information.
///
/// Represents a file or directory path with its current state and any error messages
/// that occurred during processing.
#[derive(Debug)]
struct PathEntryWithError {
    /// Path to the file or directory.
    pub path: PathBuf,
    /// Current state of the entry.
    pub state: EntryState,
    /// Error messages associated with this entry, if any.
    pub state_messages: Vec<String>,
}

impl PathEntryWithError {
    /// Creates a new `PathEntryWithError` with the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file or directory.
    ///
    /// # Returns
    ///
    /// A new `PathEntryWithError` with initial state set to `EntryState::Metadata`.
    pub fn with_path(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            state: EntryState::Metadata,
            state_messages: Vec::new(),
        }
    }
}

impl PartialEq for PathEntryWithError {
    /// Compares two `PathEntryWithError` instances for equality.
    ///
    /// Two entries are considered equal if they have the same path.
    ///
    /// # Arguments
    ///
    /// * `other` - The other `PathEntryWithError` to compare with.
    ///
    /// # Returns
    ///
    /// `true` if the paths are equal, `false` otherwise.
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl PartialEq<PathBuf> for PathEntryWithError {
    /// Compares a `PathEntryWithError` with a `PathBuf` for equality.
    ///
    /// A `PathEntryWithError` is considered equal to a `PathBuf` if they have the same path.
    ///
    /// # Arguments
    ///
    /// * `other` - The `PathBuf` to compare with.
    ///
    /// # Returns
    ///
    /// `true` if the paths are equal, `false` otherwise.
    fn eq(&self, other: &PathBuf) -> bool {
        self.path == *other
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

/// Creates a file manifest from a file or directory entry.
///
/// This function builds a complete `FileManifestJournalEntry` for a given path, including
/// its metadata, extended attributes, and access control lists as specified in the options.
///
/// # Arguments
///
/// * `share_path` - The base path of the share being processed.
/// * `entry` - The path entry with error state to process.
/// * `options` - Options controlling what metadata to include.
///
/// # Returns
///
/// A `FileManifestJournalEntry` containing all the file's information and state.
///
/// # Errors
///
/// If metadata, extended attributes, or ACLs cannot be read, the function will still
/// return a valid entry but with the appropriate error state and messages set.
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

    let (symlink, metadata_stats) = match metadata {
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
            state_messages.push(format!("{e:#}"));
            (Vec::new(), None)
        }
    };

    let xattr = if options.with_xattr {
        match read_xattr(&file) {
            Ok(xattr) => xattr,
            Err(e) => {
                state = EntryState::PartialMetadata;
                state_messages.push(format!("{e:#}"));
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
                state_messages.push(format!("{e:#}"));
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
            stats: metadata_stats,

            xattr,
            acl,
            chunks: Vec::new(),
            hash: Vec::new(),
            symlink,

            metadata: HashMap::new(),
        }),

        state: state as i32,
        state_messages,

        xfer_start: 0,
        xfer_calculation: 0,
        xfer_duration: 0,
        xfer_check: 0,
    }
}

/// Recursively scans a directory for files, creating one-level-at-a-time traversal.
///
/// This function scans one level of the directory and adds subdirectories to the `to_visit` list
/// for later processing. It also filters files based on the include and exclude patterns.
///
/// # Arguments
///
/// * `path` - The current path being scanned.
/// * `to_visit` - A list of paths to visit in future iterations.
/// * `share_path` - The base path of the share being scanned.
/// * `includes` - Include glob patterns to filter files.
/// * `excludes` - Exclude glob patterns to filter files.
///
/// # Returns
///
/// A vector of `PathEntryWithError` representing the files and directories found at this level.
///
/// # Errors
///
/// If directory listing fails, an error entry will be returned.
async fn one_level(
    path: &PathBuf,
    to_visit: &mut Vec<PathBuf>,

    share_path: &Path,
    includes: &GlobSet,
    excludes: &GlobSet,
) -> Vec<PathEntryWithError> {
    if !path.eq(&*EMPTY_PATH) && !is_file_authorized(path, includes, excludes) {
        return Vec::new();
    }

    let mut entry = PathEntryWithError::with_path(path);
    let mut dir = match tokio::fs::read_dir(&share_path.join(path)).await {
        Ok(dir) => dir,
        Err(e) => {
            entry.state = EntryState::Error;
            entry.state_messages.push(format!("{e:#}"));
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
                entry.state_messages.push(format!("{e:#}"));
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
                child_entry.state_messages.push(format!("{e:#}"));
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

    if entry != *EMPTY_PATH {
        files.push(entry);
    }

    files
}

/// Creates a stream that recursively traverses a directory tree, filtering files based on patterns.
///
/// This function creates a stream that visits directories breadth-first and yields file entries
/// that match the given include and exclude patterns.
///
/// # Arguments
///
/// * `share_path` - The base path to scan.
/// * `includes` - Include glob patterns to filter files.
/// * `excludes` - Exclude glob patterns to filter files.
///
/// # Returns
///
/// A stream that yields `PathEntryWithError` instances for each file and directory found.
fn get_files_recursive(
    share_path: &Path,
    includes: &GlobSet,
    excludes: &GlobSet,
) -> impl Stream<Item = PathEntryWithError> + Send + 'static {
    let share_path = share_path.to_path_buf();
    let includes = includes.clone();
    let excludes = excludes.clone();

    futures::stream::unfold(
        (vec![EMPTY_PATH.clone()], share_path, includes, excludes),
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
        let files = get_files_recursive(share_path, includes, excludes);
        pin_mut!(files);

        while let Some(entry) = files.next().await {
            yield create_manifest_from_file(share_path, entry, options);
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

        assert!(filenames.len() >= 2);
        assert!(filenames.contains(&PathBuf::from("private_key.pem")));
        assert!(filenames.contains(&PathBuf::from("public_key.pem")));
    }

    #[tokio::test]
    async fn test_authorized_files() {
        let includes = list_to_globset(&["/Jeux/*", "/rsyncd/*", "/Users/*"]).unwrap();
        let excludes = list_to_globset(&[
            "/Users/Public/Documents/Embarcadero/*",
            "/Users/alexandre/.cache/*",
            "/Users/alexandre/.mcreator/gradle/*",
            "*$RECYCLE.BIN",
            "*.vmdk",
            "*.vdi",
            "*.iso",
            "*node_modules",
        ])
        .unwrap();

        let ok_files = ["/Jeux/SC2000/Disk2/scenario._", "/rsyncd/doc/rsync.html"];
        let ko_files = [
            "/Windows/System32/drivers/etc/hosts",
            "/Users/alexandre/.cache/kdeconnect/kdeconnectd/kdeconnectd.log",
            "/Users/alexandre/.mcreator/gradle/wrapper/dists/gradle-6.8.3-all/b0k2r0v3t4v1x1/gradle-6.8.3/docs/javadoc/org/gradle/api",
        ];

        for file in &ok_files {
            println!("Testing ok {file}");
            assert!(is_file_authorized(
                &PathBuf::from(file),
                &includes,
                &excludes
            ));
        }
        for file in &ko_files {
            println!("Testing ko {file}");
            assert!(!is_file_authorized(
                &PathBuf::from(file),
                &includes,
                &excludes
            ));
        }
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn test_authorized_files_windows() {
        let includes = list_to_globset(&["\\Jeux*", "\\rsyncd*", "\\Users*"]).unwrap();
        let excludes = list_to_globset(&[
            "\\Users\\Public\\Documents\\Embarcadero*",
            "\\Users\\alexandre\\.cache*",
            "\\Users\\alexandre\\.mcreator\\gradle*",
            "*$RECYCLE.BIN",
            "*.vmdk",
            "*.vdi",
            "*.iso",
            "*node_modules",
        ])
        .unwrap();

        let ok_files = [
            "\\Jeux",
            "\\Jeux\\SC2000",
            "\\Jeux\\SC2000\\Disk2",
            "\\Jeux\\SC2000\\Disk2\\scenario._",
            "\\rsyncd\\doc\\rsync.html",
        ];
        let ko_files = [
            "\\Windows\\System32\\drivers\\etc\\hosts",
            "\\Users\\alexandre\\.cache\\kdeconnect\\kdeconnectd\\kdeconnectd.log",
            "\\Users\\alexandre\\.mcreator\\gradle\\wrapper\\dists\\gradle-6.8.3-all\\b0k2r0v3t4v1x1\\gradle-6.8.3\\docs\\javadoc\\org\\gradle\\api",
        ];

        for file in &ok_files {
            println!("Testing ok {}", file);
            assert!(is_file_authorized(
                &PathBuf::from(file),
                &includes,
                &excludes
            ));
        }
        for file in &ko_files {
            println!("Testing ko {}", file);
            assert!(!is_file_authorized(
                &PathBuf::from(file),
                &includes,
                &excludes
            ));
        }
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn test_authorized_files_windows_2() {
        let includes = list_to_globset(&[]).unwrap();
        let excludes = list_to_globset(&[
            "\\VMWare*",
            "\\VirtualBox VMs*",
            "\\VirtualBox VMs Win*",
            "\\VMs*",
            "\\System Volume Information*",
            "\\Servers_abandonnees*",
            "*$RECYCLE.BIN*",
            "*.vmdk",
            "*.vdi",
            "*.iso",
            "*node_modules*",
        ])
        .unwrap();

        let ok_files = ["\\autres\\dosiers\\app"];
        let ko_files = [
            "\\VMWare",
            "\\VMWare\\Virtual Machines",
            "\\VMWare\\Virtual Machines\\Windows XP",
            "\\VirtualBox VMs",
            "\\VirtualBox VMs\\test_",
            "\\VirtualBox VMs\\test_\\Logs",
            "\\VirtualBox VMs Win",
        ];

        for file in &ok_files {
            println!("Testing ok {}", file);
            assert!(is_file_authorized(
                &PathBuf::from(file),
                &includes,
                &excludes
            ));
        }
        for file in &ko_files {
            println!("Testing ko {}", file);
            assert!(!is_file_authorized(
                &PathBuf::from(file),
                &includes,
                &excludes
            ));
        }
    }
}
