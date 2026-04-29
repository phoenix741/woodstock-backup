/// # File Manifest Module
///
/// This module defines the main data structures and logic for file manifests in the Woodstock backup system.
/// It provides lightweight and full-featured manifest types, trait implementations for index operations,
/// and utility methods for file metadata, serialization, and display.
///
/// ## Main Structures
///
/// - [`FileManifestLight`]: Lightweight file manifest for index operations.
/// - [`FileManifest`]: Full-featured file manifest with metadata and chunk information.
/// - [`FileManifestJournalEntry`]: Journal entry for file manifest changes.
///
/// ## Main Methods
///
/// - [`FileManifest::path`], [`FileManifest::set_path`]: Get/set the file path.
/// - [`FileManifest::mode`], [`FileManifest::file_mode`]: Get file mode and type.
/// - [`FileManifest::size`], [`FileManifest::compressed_size`]: Get file sizes.
/// - [`FileManifest::chunk_count`]: Get the number of chunks for the file.
/// - [`FileManifest::is_special_file`], [`FileManifest::is_regular_file`]: File type checks.
/// - [`FileManifest::to_yaml`], [`FileManifest::to_log`]: Serialization and logging.
///
/// ## Error Handling & Panics
///
/// - All public methods return `Result` and propagate I/O or serialization errors using the `eyre` crate.
/// - Panics are not expected under normal operation; errors are returned as `Result`.
///
/// ## Generics
///
/// - The `PathManifest` trait is implemented for both lightweight and full manifests.
///
/// ## See Also
///
/// - [`PathManifest`], [`FileManifestJournalEntry`], [`FileManifestLight`]
use std::{
    fmt::{self},
    path::{Path, PathBuf},
};

use eyre::Result;

use crate::{
    config::CHUNK_SIZE_U64,
    utils::path::{path_to_vec, vec_to_path},
    woodstock::{FileManifest, FileManifestJournalEntry, FileManifestType},
    EntryState, EntryType, FileManifestStat,
};

use super::PathManifest;

#[derive(Clone)]
/// Lightweight file manifest for index operations.
pub struct FileManifestLight {
    /// Path of the file as a byte vector.
    pub path: Vec<u8>,
    /// Last modification timestamp (UNIX epoch).
    last_modified: i64,
    /// File size in bytes.
    size: u64,
}

impl PathManifest for FileManifestLight {
    /// Returns the file path as a byte vector.
    fn array_path(&self) -> &Vec<u8> {
        &self.path
    }

    /// Returns the last modification timestamp.
    fn last_modified(&self) -> i64 {
        self.last_modified
    }

    /// Returns the file size in bytes.
    fn size(&self) -> u64 {
        self.size
    }
}

impl PathManifest for FileManifest {
    /// Returns the file path as a byte vector.
    fn array_path(&self) -> &Vec<u8> {
        &self.path
    }

    /// Returns the last modification timestamp.
    fn last_modified(&self) -> i64 {
        self.stats
            .as_ref()
            .map(|stats| stats.last_modified)
            .unwrap_or_default()
    }

    /// Returns the file size in bytes.
    fn size(&self) -> u64 {
        self.stats
            .as_ref()
            .map(|stats| stats.size)
            .unwrap_or_default()
    }
}

impl From<FileManifest> for FileManifestLight {
    fn from(manifest: FileManifest) -> Self {
        Self {
            last_modified: manifest.last_modified(),
            size: manifest.size(),
            path: manifest.path,
        }
    }
}

impl FileManifest {
    #[must_use]
    pub fn path(&self) -> PathBuf {
        vec_to_path(&self.path)
    }

    pub fn set_path(&mut self, path: &Path) {
        self.path = path_to_vec(path);
    }

    /// Returns the mode of the file.
    ///
    /// # Returns
    ///
    /// The mode of the file as a `u32` value.
    ///
    #[must_use]
    pub fn mode(&self) -> u32 {
        self.stats
            .as_ref()
            .map(|stats| stats.mode)
            .unwrap_or_default()
    }

    #[must_use]
    pub fn compressed_size(&self) -> u64 {
        self.stats
            .as_ref()
            .map(|stats| stats.compressed_size)
            .unwrap_or_default()
    }

    #[must_use]
    pub fn size(&self) -> u64 {
        self.stats
            .as_ref()
            .map(|stats| stats.size)
            .unwrap_or_default()
    }

    #[must_use]
    pub fn chunk_count(&self) -> usize {
        let size = self.size();
        let chunk_count = size / CHUNK_SIZE_U64;

        if size % CHUNK_SIZE_U64 > 0 {
            usize::try_from(chunk_count + 1).unwrap_or_default()
        } else {
            usize::try_from(chunk_count).unwrap_or_default()
        }
    }

    /// Returns the file mode as a `FileManifestMode` enum.
    ///
    /// # Returns
    ///
    /// The file mode as a `FileManifestMode` enum.
    ///
    #[must_use]
    pub fn file_mode(&self) -> FileManifestType {
        self.stats
            .as_ref()
            .map(FileManifestStat::file_type)
            .unwrap_or_default()
    }

    #[must_use]
    pub fn file_mode_char(&self) -> char {
        match self.file_mode() {
            FileManifestType::Directory => 'd',
            FileManifestType::RegularFile => '-',
            FileManifestType::Symlink => 'l',
            FileManifestType::BlockDevice => 'b',
            FileManifestType::CharacterDevice => 'c',
            FileManifestType::Fifo => 'p',
            FileManifestType::Socket => 's',
            FileManifestType::Unknown => '?',
        }
    }

    #[must_use]
    pub fn permissions_string(&self) -> String {
        let mode = self.mode();
        let mode = mode & 0o777;

        let mode = format!(
            "{}{}{}{}{}{}{}{}{}",
            if mode & 0o400 != 0 { "r" } else { "-" },
            if mode & 0o200 != 0 { "w" } else { "-" },
            if mode & 0o100 != 0 { "x" } else { "-" },
            if mode & 0o040 != 0 { "r" } else { "-" },
            if mode & 0o020 != 0 { "w" } else { "-" },
            if mode & 0o010 != 0 { "x" } else { "-" },
            if mode & 0o004 != 0 { "r" } else { "-" },
            if mode & 0o002 != 0 { "w" } else { "-" },
            if mode & 0o001 != 0 { "x" } else { "-" }
        );

        mode
    }

    /// Checks if the file is a special file.
    ///
    /// # Returns
    ///
    /// - `true` if the file is not a regular file.
    /// - `false` if the file is a regular file.
    ///
    #[must_use]
    pub fn is_special_file(&self) -> bool {
        self.file_mode() != FileManifestType::RegularFile
    }

    #[must_use]
    pub fn is_regular_file(&self) -> bool {
        self.file_mode() == FileManifestType::RegularFile
            || self.file_mode() == FileManifestType::Unknown
    }

    /// Serializes the file manifest to YAML format.
    ///
    /// # Errors
    /// Returns an error if serialization fails.
    pub fn to_yaml(&self) -> Result<String> {
        let object = vec![self];
        let str = serde_yaml_ng::to_string(&object)?;
        Ok(str)
    }

    #[must_use]
    pub fn to_log(&self) -> String {
        // Log should have the form
        // drwxr-xr-x     1000,    1000      5156 phoenix/README.md

        let file_mode = self.file_mode_char();
        let path = self.path();
        let mode = self.permissions_string();
        let size = self.size();
        let owner = self
            .stats
            .as_ref()
            .map(|stats| stats.owner_id)
            .unwrap_or_default();
        let group = self
            .stats
            .as_ref()
            .map(|stats| stats.group_id)
            .unwrap_or_default();

        let symlink = if self.file_mode() == FileManifestType::Symlink {
            let symlink = vec_to_path(&self.symlink);
            format!(" -> {}", symlink.display())
        } else {
            String::new()
        };

        let str = format!(
            "{}{} {:>8} {:>8} {:>12} {}{}",
            file_mode,
            mode,
            owner,
            group,
            size,
            path.display(),
            symlink,
        );

        str
    }
}

impl FileManifestJournalEntry {
    #[must_use]
    pub fn path(&self) -> PathBuf {
        if let Some(ref manifest) = self.manifest {
            manifest.path()
        } else {
            PathBuf::new()
        }
    }

    /// Checks if the file is a special file.
    ///
    /// # Returns
    ///
    /// - `true` if the file is not a regular file.
    /// - `false` if the file is a regular file.
    ///
    #[must_use]
    pub fn is_special_file(&self) -> bool {
        if let Some(ref manifest) = self.manifest {
            manifest.is_special_file()
        } else {
            false
        }
    }

    #[must_use]
    pub fn size(&self) -> u64 {
        if let Some(ref manifest) = self.manifest {
            manifest.size()
        } else {
            0
        }
    }

    #[must_use]
    pub fn compressed_size(&self) -> u64 {
        if let Some(ref manifest) = self.manifest {
            manifest.compressed_size()
        } else {
            0
        }
    }

    /// Serializes the journal entry to YAML format.
    ///
    /// # Errors
    /// Returns an error if serialization fails.
    pub fn to_yaml(&self) -> Result<String> {
        let object = vec![self];
        let str = serde_yaml_ng::to_string(&object)?;
        Ok(str)
    }

    #[must_use]
    pub fn to_log(&self) -> String {
        let manifest_str = self
            .manifest
            .as_ref()
            .map(super::super::woodstock::FileManifest::to_log)
            .unwrap_or_default();

        let timestamp = if self.xfer_start > 0 {
            chrono::DateTime::<chrono::Utc>::from_timestamp(
                i64::try_from(self.xfer_start).unwrap_or_default(),
                0,
            )
            .map_or_else(
                || "invalid timestamp".to_string(),
                |dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string(),
            )
        } else {
            String::new()
        };

        let type_str = match self.entry_type() {
            EntryType::Add => "create",
            EntryType::Modify => "change",
            EntryType::Remove => "remove",
            EntryType::SnapshotInfo => "snapshot_info",
        };

        let state = match self.state() {
            EntryState::Metadata => "metadata",
            EntryState::PartialMetadata => "partial metadata",
            EntryState::Chunks => "data",
            EntryState::ChunksPartialMetadata => "data/partial metadata",
            EntryState::Error => "error",
        };

        let state_messages = if self.state_messages.is_empty() {
            String::new()
        } else {
            "\n".to_string() + &self.state_messages.join("\n")
        };

        let xfer_calculation = if self.xfer_calculation > 0 {
            format!("calc {}s", self.xfer_calculation)
        } else {
            String::new()
        };
        let xfer_duration = if self.xfer_duration > 0 {
            format!("xfer {}s", self.xfer_duration)
        } else {
            String::new()
        };
        let xfer_check = if self.xfer_check > 0 {
            format!("chk {}s", self.xfer_check)
        } else {
            String::new()
        };
        let timings = if !xfer_calculation.is_empty()
            || !xfer_duration.is_empty()
            || !xfer_check.is_empty()
        {
            format!(
                "(timings: {})",
                [xfer_calculation, xfer_duration, xfer_check]
                    .iter()
                    .filter(|s| !s.is_empty())
                    .cloned()
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        } else {
            String::new()
        };

        format!("{timestamp} {type_str} {state:>22} {manifest_str} {state_messages} {timings}")
    }
}

impl fmt::Display for FileManifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let yaml = self.to_yaml();
        let yaml = match yaml {
            Ok(yaml) => yaml,
            Err(err) => {
                return write!(f, "Failed to serialize FileManifest: {err}");
            }
        };

        // Write the formatted path to the Formatter
        write!(f, "{yaml}")
    }
}

impl fmt::Display for FileManifestJournalEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let yaml = self.to_yaml();
        let yaml = match yaml {
            Ok(yaml) => yaml,
            Err(err) => {
                return write!(f, "Failed to serialize FileManifest: {err}");
            }
        };

        // Write the formatted path to the Formatter
        write!(f, "{yaml}")
    }
}
