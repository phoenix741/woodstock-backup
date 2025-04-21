// # Index Manifest Model Module
//
// This module defines the trait and data structures for index-based file manifest management in the Woodstock backup system.
// It provides generic support for mapping file paths to manifest entries, journal application, and index walking.
//
// ## Main Structures
//
// - [`PathManifest`]: Trait for types that can be used as a path-based manifest entry.
// - [`IndexFileEntry`]: Represents an entry in the index file.
// - [`IndexManifest`]: Represents the index manifest, mapping file paths to entries.
//
// ## Main Methods
//
// - [`IndexManifest::new`]: Create a new index manifest.
// - [`IndexManifest::apply`]: Apply a journal entry to the index.
// - [`IndexManifest::walk`]: Iterate over sorted index entries.
// - [`IndexManifest::add`], [`IndexManifest::remove`], [`IndexManifest::mark`]: Index management.
// - [`IndexManifest::get_entry`]: Retrieve an entry by file path.
//
// ## Error Handling & Panics
//
// - All public methods return `Result` or Option and do not panic under normal operation.
//
// ## Generics
//
// - The index is generic over any type implementing [`PathManifest`].
//
// ## See Also
//
// - [`FileManifest`], [`FileManifestJournalEntry`], [`FileManifestLight`]

use std::collections::HashMap;

use crate::{woodstock::EntryType, FileManifest, FileManifestJournalEntry};

/// Trait for types that can be used as a path-based manifest entry.
pub trait PathManifest: Clone + From<FileManifest> {
    /// Returns the array path for the manifest entry.
    #[must_use]
    fn array_path(&self) -> &Vec<u8>;
    /// Returns the last modified timestamp.
    #[must_use]
    fn last_modified(&self) -> i64;
    /// Returns the size of the entry.
    #[must_use]
    fn size(&self) -> u64;
}

/// Represents an entry in the index file.
#[derive(Clone, Debug)]
pub struct IndexFileEntry<T: PathManifest> {
    /// Whether the entry has been marked as viewed.
    pub mark_viewed: bool,
    /// The manifest data for this entry.
    pub manifest: T,
}

impl<T: PathManifest> IndexFileEntry<T> {
    /// Returns the path of the index file entry.
    #[must_use]
    pub fn path(&self) -> &Vec<u8> {
        self.manifest.array_path()
    }
}

/// Represents the index manifest, mapping file paths to entries.
#[derive(Debug)]
pub struct IndexManifest<T: PathManifest> {
    /// Map of file paths to index file entries.
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

    /// Returns a sorted iterator over the index file entries.
    pub fn walk(&self) -> impl Iterator<Item = &IndexFileEntry<T>> {
        let mut entries: Vec<_> = self.files.values().collect();
        entries.sort_by_key(|entry| entry.path());
        entries.into_iter()
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
    pub fn mark(&mut self, file_path: &Vec<u8>) -> Option<&IndexFileEntry<T>> {
        if let Some(entry) = self.files.get_mut(file_path) {
            entry.mark_viewed = true;
            Some(entry)
        } else {
            None
        }
    }

    /// Retrieves the index file entry for a given file path.
    #[must_use]
    pub fn get_entry(&self, file_path: &Vec<u8>) -> Option<&IndexFileEntry<T>> {
        self.files.get(file_path)
    }
}
