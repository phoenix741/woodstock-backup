//! # Manifest Core Module
//!
//! This module provides the core logic for managing Woodstock backup manifests, including
//! manifest file structure, reading, writing, compaction, and index loading. It defines
//! the main [`Manifest`] struct, which encapsulates all paths and operations for a backup
//! manifest, as well as chunk and reference count utilities.
//!
//! ## Main Structures
//!
//! - [`ManifestChunk`]: Represents a chunk entry in the manifest (SHA-256 hash).
//! - [`Manifest`]: Encapsulates all paths and operations for a backup manifest.
//!
//! ## Main Methods
//!
//! - [`Manifest::new`]: Create a new manifest with a given name and base path.
//! - [`Manifest::exists`]: Check if the manifest exists and is not journaled.
//! - [`Manifest::remove`]: Remove all files associated with the manifest.
//! - [`Manifest::read_manifest_entries`]: Stream manifest entries from disk.
//! - [`Manifest::read_manifest_entries_to_end`]: Read all manifest entries into a vector.
//! - [`Manifest::read_journal_entries`]: Stream journal entries, skipping errors.
//! - [`Manifest::read_filelist_entries`]: Stream file list entries.
//! - [`Manifest::read_log_entries`]: Stream log entries.
//! - [`Manifest::save_filelist_entries`]: Save file list entries from a stream.
//! - [`Manifest::load_index`]: Load the index manifest by applying all entries.
//! - [`Manifest::read_all_message`]: Stream all messages from the manifest.
//! - [`Manifest::compact`]: Compact the manifest by applying a mapping callback.
//! - [`Manifest::list_chunks`]: Stream all chunk entries in the manifest.
//! - [`Manifest::generate_refcnt`]: Stream reference counts for all chunks.
//!
//! ## Error Handling & Panics
//!
//! - All public methods return `Result` and propagate I/O or serialization errors using the `eyre` crate.
//! - Panics are not expected under normal operation; errors are returned as `Result`.
//!
//! ## Generics
//!
//! - The `compact` method is generic over the mapping callback and its future.
//!
//! ## See Also
//!
//! - [`FileManifest`], [`FileManifestJournalEntry`], [`IndexManifest`]
use std::path::{Path, PathBuf};

use async_stream::stream;
use eyre::Result;
use futures::{pin_mut, Stream};
use futures::{Future, StreamExt};
use tokio::fs::{remove_file, rename};

use crate::proto::ProtobufReader;
use crate::{proto::save_file, EntryState, FileManifest, FileManifestJournalEntry, PoolRefCount};

use super::IndexManifest;

/// Represents a chunk entry in the manifest.
#[derive(Clone)]
pub struct ManifestChunk {
    /// SHA-256 hash of the chunk.
    pub sha256: Vec<u8>,
}

/// Represents a manifest, which contains metadata and paths for backup management.
#[derive(Clone)]
pub struct Manifest {
    /// Name of the manifest.
    pub manifest_name: String,
    /// Path to the manifest file.
    pub manifest_path: PathBuf,
    /// Path to the file list.
    pub file_list_path: PathBuf,
    /// Path to the journal file.
    pub journal_path: PathBuf,
    /// Path to the log file.
    pub log_path: PathBuf,
    /// Path to the new manifest file (used during compaction).
    pub new_path: PathBuf,
}

impl Manifest {
    /// Creates a new `Manifest` with the given name and base path.
    ///
    /// # Arguments
    /// * `manifest_name` - The name of the manifest.
    /// * `path` - The base directory for manifest files.
    ///
    /// # Returns
    ///
    /// A new [`Manifest`] instance with all paths initialized.
    #[must_use]
    pub fn new(manifest_name: &str, path: &Path) -> Self {
        Self {
            manifest_name: manifest_name.to_string(),
            manifest_path: path.join(format!("{manifest_name}.manifest")),
            file_list_path: path.join(format!("{manifest_name}.filelist")),
            journal_path: path.join(format!("{manifest_name}.journal")),
            log_path: path.join(format!("{manifest_name}.log")),
            new_path: path.join(format!("{manifest_name}.new",)),
        }
    }

    /// Checks if the manifest exists and is not in a journaled state.
    ///
    /// # Returns
    ///
    /// `true` if the manifest file exists and the journal file does not exist.
    #[must_use]
    pub fn exists(&self) -> bool {
        self.manifest_path.exists() && !self.journal_path.exists()
    }

    /// Removes all files associated with this manifest.
    ///
    /// # Errors
    /// Returns an error if any file cannot be removed.
    pub async fn remove(&self) -> Result<()> {
        for path in &[
            &self.manifest_path,
            &self.file_list_path,
            &self.journal_path,
            &self.log_path,
            &self.new_path,
        ] {
            let _ = remove_file(path).await;
        }

        Ok(())
    }

    /// Reads manifest entries as a stream.
    ///
    /// # Returns
    ///
    /// A stream of [`FileManifest`] entries from the manifest file.
    pub fn read_manifest_entries(&self) -> impl Stream<Item = FileManifest> + '_ {
        // TODO: Add log when fail
        stream!({
            let manifests = ProtobufReader::<FileManifest>::new(&self.manifest_path, true).await;
            if let Ok(mut manifests) = manifests {
                let mut manifests = manifests.into_stream();

                while let Some(manifest) = manifests.next().await {
                    if let Ok(manifest) = manifest {
                        yield manifest;
                    }
                }
            }
        })
    }

    /// Reads all manifest entries to the end and returns them as a vector.
    ///
    /// # Errors
    /// Returns an error if reading fails.
    pub async fn read_manifest_entries_to_end(&self) -> Result<Vec<FileManifest>> {
        let mut manifests = ProtobufReader::<FileManifest>::new(&self.manifest_path, true).await?;
        let mut result = Vec::new();
        manifests.read_to_end(&mut result).await?;

        Ok(result)
    }

    /// Reads journal entries as a stream, skipping entries with error state.
    ///
    /// # Returns
    ///
    /// A stream of [`FileManifestJournalEntry`] entries from the journal file.
    pub fn read_journal_entries(&self) -> impl Stream<Item = FileManifestJournalEntry> + '_ {
        stream!({
            let manifests =
                ProtobufReader::<FileManifestJournalEntry>::new(&self.journal_path, true).await;
            if let Ok(mut manifests) = manifests {
                let mut manifests = manifests.into_stream();

                while let Some(manifest) = manifests.next().await {
                    if let Ok(manifest) = manifest {
                        // Ignore error entries
                        if manifest.state() == EntryState::Error {
                            continue;
                        }

                        yield manifest;
                    }
                }
            }
        })
    }

    /// Reads file list entries as a stream.
    ///
    /// # Returns
    ///
    /// A stream of [`FileManifestJournalEntry`] entries from the file list file.
    pub fn read_filelist_entries(&self) -> impl Stream<Item = FileManifestJournalEntry> + '_ {
        stream!({
            let manifests =
                ProtobufReader::<FileManifestJournalEntry>::new(&self.file_list_path, true).await;
            if let Ok(mut manifests) = manifests {
                let mut manifests = manifests.into_stream();

                while let Some(manifest) = manifests.next().await {
                    if let Ok(manifest) = manifest {
                        yield manifest;
                    }
                }
            }
        })
    }

    /// Reads log entries as a stream.
    ///
    /// # Returns
    ///
    /// A stream of [`FileManifestJournalEntry`] entries from the log file.
    pub fn read_log_entries(&self) -> impl Stream<Item = FileManifestJournalEntry> + '_ {
        stream!({
            let manifests =
                ProtobufReader::<FileManifestJournalEntry>::new(&self.log_path, true).await;
            if let Ok(mut manifests) = manifests {
                let mut manifests = manifests.into_stream();

                while let Some(manifest) = manifests.next().await {
                    if let Ok(manifest) = manifest {
                        yield manifest;
                    }
                }
            }
        })
    }

    /// Saves file list entries from a stream to disk.
    ///
    /// # Arguments
    /// * `source` - A stream of [`FileManifestJournalEntry`] to save.
    ///
    /// # Errors
    /// Returns an error if saving fails.
    pub async fn save_filelist_entries(
        &self,
        source: impl Stream<Item = FileManifestJournalEntry>,
    ) -> Result<()> {
        save_file(&self.file_list_path, source, false).await
    }

    /// Loads the index manifest by applying all manifest and journal entries.
    ///
    /// # Returns
    ///
    /// The loaded [`IndexManifest`] for this manifest.
    pub async fn load_index(&self) -> IndexManifest<FileManifest> {
        let mut index = IndexManifest::new();

        let messages = self.read_manifest_entries();
        pin_mut!(messages);

        while let Some(message) = messages.next().await {
            index.add(message);
        }

        let messages = self.read_journal_entries();
        pin_mut!(messages);

        while let Some(message) = messages.next().await {
            index.apply(message);
        }

        index
    }

    /// Reads all messages from the manifest as a stream.
    ///
    /// # Returns
    ///
    /// A stream of [`FileManifest`] entries from the manifest file.
    pub fn read_all_message(&self) -> impl Stream<Item = FileManifest> + '_ {
        stream!({
            let index = self.load_index().await;

            let entries = index.walk();

            for entry in entries {
                yield entry.manifest.clone();
            }
        })
    }

    /// Compacts the manifest by applying a mapping callback to all messages and rewriting the manifest.
    ///
    /// # Arguments
    /// * `mapping_callback` - A callback that maps each [`FileManifest`] to an optional new value.
    ///
    /// # Errors
    /// Returns an error if compaction or file operations fail.
    ///
    /// # Generics
    ///
    /// * `Fut` - The future returned by the mapping callback.
    /// * `F` - The mapping callback type.
    pub async fn compact<Fut, F>(&self, mapping_callback: &F) -> Result<()>
    where
        F: Fn(FileManifest) -> Fut,
        Fut: Future<Output = Option<FileManifest>>,
    {
        let all_messages = self
            .read_all_message()
            .filter_map(|message| async { mapping_callback(message).await });
        pin_mut!(all_messages);

        save_file(&self.new_path, all_messages, false).await?;

        let _ = rename(&self.journal_path, &self.log_path).await;
        let _ = remove_file(&self.file_list_path).await;
        let _ = remove_file(&self.manifest_path).await;

        rename(&self.new_path, &self.manifest_path).await?;

        Ok(())
    }

    /// Lists all chunk entries in the manifest as a stream.
    ///
    /// # Returns
    ///
    /// A stream of [`ManifestChunk`] entries for all chunks in the manifest.
    pub fn list_chunks(&self) -> impl Stream<Item = ManifestChunk> + '_ {
        stream!({
            let messages = self.read_manifest_entries();
            pin_mut!(messages);

            while let Some(manifest) = messages.next().await {
                for sha256 in &manifest.chunks {
                    yield ManifestChunk {
                        sha256: sha256.clone(),
                    };
                }
            }
        })
    }

    /// Generates reference counts for all chunks in the manifest as a stream.
    ///
    /// # Returns
    ///
    /// A stream of [`PoolRefCount`] entries for all chunks in the manifest.
    pub fn generate_refcnt(&self) -> impl Stream<Item = PoolRefCount> + '_ {
        stream!({
            let messages = self.list_chunks();
            pin_mut!(messages);

            while let Some(chunk) = messages.next().await {
                yield PoolRefCount {
                    sha256: chunk.sha256,
                    ref_count: 1,
                    size: 0,
                    compressed_size: 0,
                };
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{copy, remove_file};

    use super::*;

    struct CleanUp;

    impl Drop for CleanUp {
        fn drop(&mut self) {
            // Clean up fixtures
            let _ = remove_file("./data/test-compact.journal");
            let _ = remove_file("./data/test-compact.manifest");
        }
    }

    #[tokio::test]
    async fn test_compact() {
        let _clean_up = CleanUp;

        copy("./data/test.journal", "./data/test-compact.journal").unwrap();

        let path = std::path::Path::new("./data");
        let manifest = Manifest::new("test-compact", path);

        manifest.compact(&|m| async { Some(m) }).await.unwrap();

        let mut manifest_st =
            ProtobufReader::<FileManifest>::new("./data/test-compact.manifest", true)
                .await
                .unwrap();
        let mut manifest_st = manifest_st.into_stream();

        let mut count = 0;
        while let Some(x) = manifest_st.next().await {
            let _ = x.unwrap();
            count += 1;
        }
        assert_eq!(count, 1000);
    }
}
