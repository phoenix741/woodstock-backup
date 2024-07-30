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

use super::file_browser::get_files;
use super::CreateManifestOptions;
use crate::config::BUFFER_SIZE;
use crate::config::CHUNK_SIZE;
use crate::config::CHUNK_SIZE_U64;
use crate::manifest::IndexManifest;
use crate::manifest::PathManifest;
use crate::utils::path::vec_to_path;
use crate::woodstock::ChunkInformation;
use crate::woodstock::FileChunk;
use crate::woodstock::{
    file_chunk, EntryType, FileChunkData, FileChunkEndOfFile, FileChunkFooter, FileChunkHeader,
    FileManifest, FileManifestJournalEntry,
};
use crate::ChunkHashReply;
use crate::ChunkHashRequest;

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
pub async fn calculate_chunk_hash_future(request: &ChunkHashRequest) -> ChunkHashReply {
    let request = request.clone();
    let manifest = tokio::task::spawn_blocking(move || {
        let path = vec_to_path(&request.filename);
        debug!("Calculating chunk hash for {}", &path.display());

        let manifest = caculate_chunk_hash(&request);

        match manifest {
            Ok(manifest) => manifest,
            Err(e) => {
                error!("Can't read the file file {}: {e}", path.display());
                ChunkHashReply {
                    hash: Vec::new(),
                    chunks: Vec::new(),
                }
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
fn caculate_chunk_hash(request: &ChunkHashRequest) -> Result<ChunkHashReply, Box<dyn Error>> {
    let mut file_hasher = Sha3_256::new();
    let mut chunk_hasher = Sha3_256::new();
    let mut chunks = Vec::<Vec<u8>>::new();
    let mut chunk_read = 0;

    let file = vec_to_path(&request.filename);
    let file = std::fs::File::open(file)?;
    let mut reader = std::io::BufReader::new(file);

    // Read small chunk
    let mut buffer = vec![0; BUFFER_SIZE];

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
            chunks.push(chunk_hash.to_vec());

            chunk_hasher = Sha3_256::new();
            chunk_read = overflow;
            if overflow > 0 {
                chunk_hasher.update(&buffer[(read - overflow)..read]);
            }
        } else {
            chunk_hasher.update(&buffer[..read]);
        }

        file_hasher.update(&buffer[..read]);
    }

    let chunk_hash = chunk_hasher.finalize();
    chunks.push(chunk_hash.to_vec());

    let hash = file_hasher.finalize().to_vec();

    Ok(ChunkHashReply { chunks, hash })
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
    let mut chunks = chunk.chunks_id.clone();
    chunks.sort_unstable();

    debug!("Reading file {}", path.display());

    try_stream!({
        // Calculate the chunk count depending on the file size
        let file_size = tokio::fs::metadata(&path).await?.len();
        let chunk_count = file_size / CHUNK_SIZE_U64;
        let chunk_count = if file_size % CHUNK_SIZE_U64 > 0 {
            chunk_count + 1
        } else {
            chunk_count
        };
        let chunks = if chunks.is_empty() {
            (0..chunk_count).collect()
        } else {
            chunks
        };

        // Open the file and read it
        let file = File::open(path).await?;
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0; BUFFER_SIZE];

        let mut file_hasher = Sha3_256::new();

        for chunk in &chunks {
            let position = chunk * CHUNK_SIZE_U64;
            let mut remaining = CHUNK_SIZE;

            reader.seek(SeekFrom::Start(position)).await?;

            yield FileChunk {
                field: Some(file_chunk::Field::Header(FileChunkHeader {
                    chunk_id: *chunk,
                })),
            };

            let mut chunk_hasher = Sha3_256::new();

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

                chunk_hasher.update(&buffer[..length_to_return]);
                file_hasher.update(&buffer[..length_to_return]);

                yield FileChunk {
                    field: Some(file_chunk::Field::Data(FileChunkData {
                        data: buffer[..length_to_return].to_vec(),
                    })),
                };
            }

            let chunk_hash = chunk_hasher.finalize().to_vec();

            yield FileChunk {
                field: Some(file_chunk::Field::Footer(FileChunkFooter { chunk_hash })),
            };
        }

        let hash = file_hasher.finalize().to_vec();

        if usize::try_from(chunk_count).unwrap_or_default() == chunks.len() {
            yield FileChunk {
                field: Some(file_chunk::Field::Eof(FileChunkEndOfFile { hash })),
            };
        }
    })
}
