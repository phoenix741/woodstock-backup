/// File reader module for the backup system.
///
/// This module provides functionality for reading file contents, calculating file
/// and chunk hashes, and streaming file chunks for backup operations. It works closely
/// with the file browser to discover files and then processes their content for backup.
use async_stream::stream;
use async_stream::try_stream;
use eyre::Result;
use futures::pin_mut;
use futures::Stream;
use futures::StreamExt;
use globset::GlobSet;
use log::{debug, info};
use std::cmp::min;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, BufReader, SeekFrom};

use super::file_browser::get_files;
use super::CreateManifestOptions;
use woodstock::config::BUFFER_SIZE;
use woodstock::config::CHUNK_SIZE;
use woodstock::config::CHUNK_SIZE_U64;
use woodstock::manifest::IndexManifest;
use woodstock::manifest::PathManifest;
use woodstock::utils::chunk_hasher::create_chunk_hasher;
use woodstock::utils::path::vec_to_path;
use woodstock::ChunkAlgorithm;
use woodstock::ChunkHashReply;
use woodstock::ChunkHashRequest;
use woodstock::ChunkInformation;
use woodstock::FileChunk;
use woodstock::{
    file_chunk, EntryType, FileChunkData, FileChunkEndOfFile, FileChunkFooter, FileChunkHeader,
    FileManifest, FileManifestJournalEntry,
};

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
pub fn get_files_with_hash<'a, P: Into<PathBuf>, T: PathManifest>(
    index: &'a mut IndexManifest<T>,
    share_path: P,
    includes: &'a GlobSet,
    excludes: &'a GlobSet,
    options: &'a CreateManifestOptions,
) -> impl Stream<Item = FileManifestJournalEntry> + 'a {
    let share_path = share_path.into();
    debug!("Scanning files in {}", share_path.display());

    let files = get_files(share_path, includes, excludes, options);
    stream!({
        pin_mut!(files);

        while let Some(mut journal_entry) = files.next().await {
            if let Some(ref manifest) = &journal_entry.manifest {
                // Start by mark the file as viewed
                index.mark(&manifest.path);

                // If the file isn't modified, skip it
                match is_modified(index, manifest) {
                    Some(false) => continue,
                    Some(true) => {
                        journal_entry.r#type = EntryType::Modify as i32;
                    }
                    None => {
                        journal_entry.r#type = EntryType::Add as i32;
                    }
                }

                yield journal_entry;
            }
        }
    })
}

/// Checks if a file is modified based on its entry in the index and its manifest.
///
/// This function takes an `IndexManifest` and a `FileManifest` and determines if the file
/// has been modified since it was last backed up by comparing timestamps and file sizes.
///
/// # Arguments
///
/// * `index` - An `IndexManifest` containing the previously backed-up file information.
/// * `manifest` - A `FileManifest` containing the current file information.
///
/// # Returns
///
/// * `Some(true)` - The file exists in the index and has been modified.
/// * `Some(false)` - The file exists in the index and has not been modified.
/// * `None` - The file does not exist in the index (it's a new file).
fn is_modified<T: PathManifest>(index: &IndexManifest<T>, manifest: &FileManifest) -> Option<bool> {
    let entry = index.get_entry(&manifest.path);
    match entry {
        Some(entry) => {
            let manifest_stats = manifest.stats;
            let manifest_stats = manifest_stats.unwrap_or_default();

            // The file is modified
            if entry.manifest.last_modified() != manifest_stats.last_modified {
                return Some(true);
            }

            // The size is different
            if entry.manifest.size() != manifest_stats.size {
                return Some(true);
            }

            Some(false)
        }
        // Not in the index, so it's a new file
        None => None,
    }
}

/// Calculates the chunk hash for a file asynchronously.
///
/// This function runs the chunk hash calculation in a blocking task to avoid blocking
/// the async runtime with CPU-intensive work. It creates hash information for both
/// the entire file and its individual chunks according to the specified algorithm.
///
/// # Arguments
///
/// * `request` - The `ChunkHashRequest` containing the file path and algorithm information.
///
/// # Returns
///
/// A `ChunkHashReply` containing the file hash and individual chunk hashes.
///
/// # Errors
///
/// If the file cannot be read or hashed, an empty reply will be returned and an error
/// will be logged.
///
/// # Panics
///
/// This function will panic if the `manifest` future resolves to an error. Ensure that the manifest is properly initialized and does not encounter runtime errors during execution.
pub async fn calculate_chunk_hash_future(request: &ChunkHashRequest) -> Result<ChunkHashReply> {
    let request = request.clone();
    let manifest = tokio::task::spawn_blocking(move || {
        let path = Path::new(&request.share_path);
        let path = path.join(vec_to_path(&request.filename));
        debug!("Calculating chunk hash for {}", &path.display());

        caculate_chunk_hash(&path, request.algorithm())
    });

    manifest.await.unwrap()
}

/// Calculates the chunk hash for a file.
///
/// This function reads a file in chunks, calculates a hash for each chunk using the
/// specified algorithm, and also calculates a hash for the entire file. It's the
/// core implementation used by `calculate_chunk_hash_future`.
///
/// # Arguments
///
/// * `file` - The path to the file to hash.
/// * `algorithm` - The algorithm to use for chunk hashing.
///
/// # Returns
///
/// A `Result` containing either a `ChunkHashReply` with the file hash and chunk hashes,
/// or an error if the file could not be read or processed.
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be opened
/// - Reading from the file fails
/// - Hashing operations fail
fn caculate_chunk_hash<P: AsRef<Path>>(
    file: P,
    algorithm: ChunkAlgorithm,
) -> Result<ChunkHashReply> {
    info!("Calculating chunk hash for {}", file.as_ref().display());

    let mut file_hasher = create_chunk_hasher(algorithm);

    let file = std::fs::File::open(file)?;
    let mut reader = std::io::BufReader::new(file);
    let mut buffer = vec![0; CHUNK_SIZE];
    let mut chunks = Vec::new();

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let mut chunk_hasher = create_chunk_hasher(algorithm);
        if bytes_read == CHUNK_SIZE {
            chunk_hasher.update(&buffer);
            file_hasher.update(&buffer);
        } else {
            chunk_hasher.update(&buffer[..bytes_read]);
            file_hasher.update(&buffer[..bytes_read]);
        }

        let chunk = chunk_hasher.finalize();
        chunks.push(chunk);

        if bytes_read < CHUNK_SIZE {
            break;
        }
    }
    let hash = file_hasher.finalize();

    Ok(ChunkHashReply { chunks, hash })
}

/// Reads a chunk of a file.
///
/// This function streams chunks of a file, creating a sequence of `FileChunk` objects
/// that represent the requested portions of the file. It can either stream specific
/// chunks identified by their IDs or all chunks if no specific IDs are provided.
///
/// # Arguments
///
/// * `chunk` - The `ChunkInformation` containing file path and chunk identifiers.
///
/// # Returns
///
/// A stream of `Result<FileChunk, std::io::Error>` representing the requested file chunks.
/// The stream includes chunk headers, data, footers, and an end-of-file marker.
///
/// # Errors
///
/// Returns errors in the stream if:
/// - The file cannot be opened or accessed
/// - Reading operations fail
/// - Seeking to positions in the file fails
///
/// # Panics
///
/// May panic if chunk index calculations overflow, though this is highly unlikely
/// with normal file sizes.
pub fn read_chunk(
    chunk: ChunkInformation,
) -> impl Stream<Item = Result<FileChunk, std::io::Error>> {
    let path = Path::new(&chunk.share_path);
    let path = path.join(vec_to_path(&chunk.filename));
    let mut chunks = chunk.chunks_id.clone();
    chunks.sort_unstable();

    let algorithm = chunk.algorithm();

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

        let mut file_hasher = create_chunk_hasher(algorithm);

        for chunk in &chunks {
            let position = chunk * CHUNK_SIZE_U64;
            let mut remaining = CHUNK_SIZE;

            reader.seek(SeekFrom::Start(position)).await?;

            yield FileChunk {
                field: Some(file_chunk::Field::Header(FileChunkHeader {
                    chunk_id: *chunk,
                })),
            };

            let mut chunk_hasher = create_chunk_hasher(algorithm);

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

            let chunk_hash = chunk_hasher.finalize();

            yield FileChunk {
                field: Some(file_chunk::Field::Footer(FileChunkFooter { chunk_hash })),
            };
        }

        let hash = file_hasher.finalize();

        if usize::try_from(chunk_count).unwrap_or_default() == chunks.len() {
            yield FileChunk {
                field: Some(file_chunk::Field::Eof(FileChunkEndOfFile { hash })),
            };
        }
    })
}
