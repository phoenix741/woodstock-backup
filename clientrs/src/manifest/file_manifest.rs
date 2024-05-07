use std::{
    fmt,
    path::{Path, PathBuf},
};

use crate::{
    config::CHUNK_SIZE_U64,
    utils::path::{path_to_vec, vec_to_path},
    woodstock::{FileManifest, FileManifestJournalEntry, FileManifestType},
    FileManifestStat,
};

use super::PathManifest;

#[derive(Clone)]
pub struct FileManifestLight {
    pub path: Vec<u8>,
    last_modified: i64,
    size: u64,
}

impl PathManifest for FileManifestLight {
    fn array_path(&self) -> &Vec<u8> {
        &self.path
    }

    fn last_modified(&self) -> i64 {
        self.last_modified
    }

    fn size(&self) -> u64 {
        self.size
    }
}

impl PathManifest for FileManifest {
    fn array_path(&self) -> &Vec<u8> {
        &self.path
    }

    fn last_modified(&self) -> i64 {
        self.stats
            .as_ref()
            .map(|stats| stats.last_modified)
            .unwrap_or_default()
    }

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
            .map(FileManifestStat::r#type)
            .unwrap_or_default()
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
}

impl fmt::Display for FileManifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let object = vec![self];
        let yaml = serde_yaml::to_string(&object);
        let yaml = match yaml {
            Ok(yaml) => yaml,
            Err(err) => {
                return write!(f, "Failed to serialize FileManifest: {err}");
            }
        };

        // Écrivez le chemin formaté dans le Formatter
        write!(f, "{yaml}")
    }
}

impl fmt::Display for FileManifestJournalEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let object = vec![self];
        let yaml = serde_yaml::to_string(&object);
        let yaml = match yaml {
            Ok(yaml) => yaml,
            Err(err) => {
                return write!(f, "Failed to serialize FileManifest: {err}");
            }
        };

        // Écrivez le chemin formaté dans le Formatter
        write!(f, "{yaml}")
    }
}
