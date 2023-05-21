use std::{
    io,
    path::{Path, PathBuf},
    rc::Rc,
};

use async_stream::stream;
use futures::{pin_mut, Stream};
use futures::{Future, StreamExt};
use tokio::fs::{remove_file, rename};

use crate::proto::ProtobufReader;
use crate::{proto::save_file, FileManifest, FileManifestJournalEntry, PoolRefCount};

use super::IndexManifest;

pub struct ManifestChunk {
    pub manifest: Rc<FileManifest>,
    pub sha256: Vec<u8>,
}

pub struct Manifest {
    pub manifest_path: PathBuf,
    pub file_list_path: PathBuf,
    pub journal_path: PathBuf,
    pub new_path: PathBuf,
}

impl Manifest {
    #[must_use]
    pub fn new(manifest_name: &str, path: &Path) -> Self {
        Self {
            manifest_path: path.join(format!("{manifest_name}.manifest")),
            file_list_path: path.join(format!("{manifest_name}.filelist")),
            journal_path: path.join(format!("{manifest_name}.journal")),
            new_path: path.join(format!("{manifest_name}.new",)),
        }
    }

    #[must_use]
    pub fn exists(&self) -> bool {
        self.manifest_path.exists() && !self.journal_path.exists()
    }

    pub async fn remove(&self) -> io::Result<()> {
        for path in &[
            &self.manifest_path,
            &self.file_list_path,
            &self.journal_path,
            &self.new_path,
        ] {
            let _ = remove_file(path).await;
        }

        Ok(())
    }

    // TODO: Add log when fail
    pub fn read_manifest_entries(&self) -> impl Stream<Item = FileManifest> + '_ {
        stream!({
            let mut manifests =
                ProtobufReader::<FileManifest>::new(&self.manifest_path, true).await;
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

    pub fn read_journal_entries(&self) -> impl Stream<Item = FileManifestJournalEntry> + '_ {
        stream!({
            let mut manifests =
                ProtobufReader::<FileManifestJournalEntry>::new(&self.journal_path, true).await;
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

    pub fn read_filelist_entries(&self) -> impl Stream<Item = FileManifestJournalEntry> + '_ {
        stream!({
            let mut manifests =
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

    pub async fn save_filelist_entries(
        &self,
        source: impl Stream<Item = FileManifestJournalEntry>,
    ) -> io::Result<()> {
        save_file(&self.file_list_path, source, true, true).await
    }

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

    pub fn read_all_message(&self) -> impl Stream<Item = FileManifest> + '_ {
        stream!({
            let index = self.load_index().await;

            let entries = index.walk();

            for entry in entries {
                yield entry.manifest.clone();
            }
        })
    }

    pub async fn compact<Fut, F>(&self, mapping_callback: &F) -> io::Result<()>
    where
        F: Fn(FileManifest) -> Fut,
        Fut: Future<Output = Option<FileManifest>>,
    {
        let all_messages = self
            .read_all_message()
            .filter_map(|message| async { mapping_callback(message).await });
        pin_mut!(all_messages);

        save_file(&self.new_path, all_messages, true, false).await?;

        let _ = remove_file(&self.journal_path).await;
        let _ = remove_file(&self.file_list_path).await;
        let _ = remove_file(&self.manifest_path).await;

        rename(&self.new_path, &self.manifest_path).await?;

        Ok(())
    }

    pub fn list_chunks(&self) -> impl Stream<Item = ManifestChunk> + '_ {
        stream!({
            let messages = self.read_manifest_entries();
            pin_mut!(messages);

            while let Some(message) = messages.next().await {
                let manifest = Rc::new(message);

                for sha256 in &manifest.chunks {
                    yield ManifestChunk {
                        manifest: Rc::clone(&manifest),
                        sha256: sha256.clone(),
                    };
                }
            }
        })
    }

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
