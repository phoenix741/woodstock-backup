use async_stream::stream;
use async_stream::try_stream;
use futures::pin_mut;
use futures::Stream;
use futures::StreamExt;
use globset::GlobSet;
use log::{debug, error};
use sha3::{Digest, Sha3_256};
use std::cmp::min;
use std::{error::Error, io::Read, path::Path};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, BufReader, SeekFrom};

use super::config::BUFFER_SIZE;
use super::config::CHUNK_SIZE;
use super::file_browser::get_files;
use super::CreateManifestOptions;
use crate::manifest::FileManifestMode;
use crate::manifest::IndexManifest;
use crate::manifest::PathManifest;
use crate::utils::path::vec_to_path;
use crate::woodstock::ChunkInformation;
use crate::woodstock::FileChunk;
use crate::woodstock::{EntryType, FileManifest, FileManifestJournalEntry};

/// Retrieves a stream of `FileManifestJournalEntry` for files with hash.
///
/// This function takes an `IndexManifest`, a share path, an array of includes, and an array of excludes.
/// It returns a stream of `FileManifestJournalEntry` for each file that matches the includes and excludes criteria.
///
/// # Arguments
///
/// * `index` - An `IndexManifest`.
/// * `share_path` - A share path.
/// * `includes` - An array of includes.
/// * `excludes` - An array of excludes.
///
/// # Returns
///
/// A stream of `FileManifestJournalEntry`.
///
pub fn get_files_with_hash<'a, T: PathManifest>(
    index: &'a mut IndexManifest<T>,
    share_path: &'a Path,
    includes: &'a GlobSet,
    excludes: &'a GlobSet,
    options: &'a CreateManifestOptions,
) -> impl Stream<Item = FileManifestJournalEntry> + 'a {
    debug!("Scanning files in {}", share_path.display());

    stream!({
        let files = get_files(share_path, includes, excludes, options);
        pin_mut!(files);

        while let Some(manifest) = files.next().await {
            let mut manifest = manifest;

            // Start by mark the file as viewed
            index.mark(&manifest.path);

            // If the file isn't modified, skip it
            if !is_modified(index, &manifest) {
                continue;
            }

            // If the file is in the entry, and the file is a regular file calculate chunk hash
            if index.get_entry(&manifest.path).is_some()
                && manifest.file_mode() == FileManifestMode::RegularFile
            {
                manifest = calculate_chunk_hash_future(share_path, manifest).await;
            }

            yield FileManifestJournalEntry {
                r#type: EntryType::Add as i32,
                manifest: Some(manifest),
            };
        }
    })
}

/// Checks if a file is modified based on its entry in the index and its manifest.
///
/// This function takes an `IndexManifest` and a `FileManifest` and returns a boolean indicating whether the file is modified or not.
///
/// # Arguments
///
/// * `index` - An `IndexManifest`.
/// * `manifest` - A `FileManifest`.
///
/// # Returns
///
/// A boolean indicating whether the file is modified or not.
///
fn is_modified<T: PathManifest>(index: &IndexManifest<T>, manifest: &FileManifest) -> bool {
    let entry = index.get_entry(&manifest.path);
    match entry {
        Some(entry) => {
            let manifest_stats = manifest.stats.clone();
            let manifest_stats = manifest_stats.unwrap_or_default();

            // The file is modified
            if entry.manifest.last_modified() != manifest_stats.last_modified {
                return true;
            }

            // The size is different
            if entry.manifest.size() != manifest_stats.size {
                return true;
            }

            false
        }
        // Not in the index, so it's a new file
        None => true,
    }
}

/// Calculates the chunk hash for a file asynchronously.
///
/// This function takes a share path and a `FileManifest` and returns the updated `FileManifest` with chunk hash information.
///
/// # Arguments
///
/// * `share_path` - The path to the share.
/// * `file` - The `FileManifest` to calculate the chunk hash for.
///
/// # Returns
///
/// The updated `FileManifest` with chunk hash information.
///
async fn calculate_chunk_hash_future(share_path: &Path, file: FileManifest) -> FileManifest {
    let share_path = share_path.to_path_buf();
    let original = file.clone();
    let manifest = tokio::task::spawn_blocking(move || {
        let path = vec_to_path(&file.path);
        debug!(
            "Calculating chunk hash for {}/{}",
            share_path.display(),
            &path.display()
        );

        let manifest = caculate_chunk_hash(&share_path, file);

        match manifest {
            Ok(manifest) => manifest,
            Err(e) => {
                error!("Can't read the file file {}: {e}", path.display());
                original
            }
        }
    });

    manifest.await.unwrap()
}

/// Calculates the chunk hash for a file.
///
/// This function takes a share path and a `FileManifest` and returns the updated `FileManifest` with chunk hash information.
///
/// # Arguments
///
/// * `share_path` - The path to the share.
/// * `manifest` - The `FileManifest` to calculate the chunk hash for.
///
/// # Returns
///
/// The updated `FileManifest` with chunk hash information.
///
fn caculate_chunk_hash(
    share_path: &Path,
    mut manifest: FileManifest,
) -> Result<FileManifest, Box<dyn Error>> {
    let mut chunk_hasher = Sha3_256::new();
    let mut chunks = Vec::<[u8; 32]>::new();
    let mut chunk_read = 0;

    let path = vec_to_path(&manifest.path);

    let file = share_path.join(path);
    let file = std::fs::File::open(file)?;
    let mut reader = std::io::BufReader::new(file);

    // Read small chunk
    let mut buffer = [0; BUFFER_SIZE];

    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }

        chunk_read += read;

        if chunk_read >= CHUNK_SIZE {
            let overflow = chunk_read - CHUNK_SIZE;
            chunk_hasher.update(&buffer[..(read - (overflow))]);

            let chunk_hash = chunk_hasher.finalize();
            chunks.push(chunk_hash.into());

            chunk_hasher = Sha3_256::new();
            chunk_read = overflow;
            if overflow > 0 {
                chunk_hasher.update(&buffer[(read - overflow)..read]);
            }
        } else {
            chunk_hasher.update(&buffer[..read]);
        }
    }

    let chunk_hash = chunk_hasher.finalize();
    chunks.push(chunk_hash.into());

    manifest.chunks = chunks.into_iter().map(|chunk| chunk.to_vec()).collect();

    // Ensure the number of chunk is equals to the size divid by CHUNK_SIZE
    // File size can be readed in Some(manifest.size).size
    let stats = manifest.stats.clone();
    assert_eq!(
        manifest.chunks.len(),
        usize::try_from(stats.unwrap().size / CHUNK_SIZE as u64)? + 1
    );

    Ok(manifest)
}

/// Reads a chunk of a file.
///
/// This function takes a `ChunkInformation` and returns a stream of `Result<FileChunk, std::io::Error>`.
///
/// # Arguments
///
/// * `chunk` - The `ChunkInformation` to read.
///
/// # Returns
///
/// A stream of `Result<FileChunk, std::io::Error>`.
///
pub fn read_chunk(
    chunk: &ChunkInformation,
) -> impl Stream<Item = Result<FileChunk, std::io::Error>> {
    let path = vec_to_path(&chunk.filename);
    let position = chunk.position;
    let size = usize::try_from(chunk.size).unwrap_or(0);

    debug!(
        "Reading chunk for path {}:{position}:{size}",
        path.display()
    );

    let mut remaining = size;

    try_stream!({
        let file = File::open(path).await?;
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0; BUFFER_SIZE];

        reader.seek(SeekFrom::Start(position)).await?;

        loop {
            if remaining == 0 {
                break;
            }

            let read = reader.read(&mut buffer).await?;
            if read == 0 {
                break;
            }

            let length_to_return = min(read, remaining);
            remaining -= length_to_return;

            yield FileChunk {
                data: buffer[..length_to_return].to_vec(),
            };
        }
    })
}
