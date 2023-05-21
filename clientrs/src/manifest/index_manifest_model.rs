use std::collections::HashMap;

use crate::{woodstock::EntryType, FileManifest, FileManifestJournalEntry};

pub trait PathManifest: Clone + From<FileManifest> {
    #[must_use]
    fn array_path(&self) -> &Vec<u8>;
    #[must_use]
    fn last_modified(&self) -> i64;
    #[must_use]
    fn size(&self) -> u64;
}

/// Represents an entry in the index file.
#[derive(Clone, Debug)]
pub struct IndexFileEntry<T: PathManifest> {
    pub mark_viewed: bool,
    pub manifest: T,
}

impl<T: PathManifest> IndexFileEntry<T> {
    /// Returns the path of the index file entry.
    #[must_use]
    pub fn path(&self) -> &Vec<u8> {
        self.manifest.array_path()
    }
}

/// Represents the index manifest.
#[derive(Debug)]
pub struct IndexManifest<T: PathManifest> {
    files: HashMap<Vec<u8>, IndexFileEntry<T>>,
}

impl<T: PathManifest> Default for IndexManifest<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: PathManifest> IndexManifest<T> {
    /// Creates a new instance of `IndexManifest`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    /// Applies a journal entry to the index manifest.
    pub fn apply(&mut self, journal_entry: FileManifestJournalEntry) {
        match journal_entry.r#type() {
            EntryType::Remove => {
                if let Some(manifest) = journal_entry.manifest {
                    self.remove(&manifest.path);
                }
            }
            _ => {
                if let Some(manifest) = journal_entry.manifest {
                    self.add(T::from(manifest));
                }
            }
        }
    }

    /// Returns an iterator over the index file entries.
    pub fn walk(&self) -> impl Iterator<Item = &IndexFileEntry<T>> {
        self.files.values()
    }

    /// Returns the size of the index.
    #[must_use]
    pub fn index_size(&self) -> usize {
        self.files.len()
    }

    /// Adds a file manifest to the index.
    pub fn add(&mut self, manifest: T) {
        let key = manifest.array_path().clone();
        self.files.insert(
            key,
            IndexFileEntry {
                mark_viewed: false,
                manifest,
            },
        );
    }

    /// Removes a file from the index.
    pub fn remove(&mut self, file_path: &Vec<u8>) {
        self.files.remove(file_path);
    }

    /// Marks a file as viewed in the index.
    pub fn mark(&mut self, file_path: &Vec<u8>) {
        if let Some(entry) = self.files.get_mut(file_path) {
            entry.mark_viewed = true;
        }
    }

    /// Retrieves the index file entry for a given file path.
    #[must_use]
    pub fn get_entry(&self, file_path: &Vec<u8>) -> Option<&IndexFileEntry<T>> {
        self.files.get(file_path)
    }
}
