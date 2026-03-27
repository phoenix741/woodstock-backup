use eyre::Result;
use futures::StreamExt;
use futures::{pin_mut, Stream};
use prost::Message;
use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use tokio::io::AsyncSeekExt;
use tokio::{
    fs::{create_dir_all, remove_file, rename, File},
    io::{AsyncWrite, AsyncWriteExt, BufWriter},
};
use tracing::{info, warn};

use crate::utils::compression::{CompressionFormat, WoodstockCompressionWriter};

/// Serializes one protobuf message in length-delimited form and writes it to an async writer.
///
/// The returned value is the total number of bytes written, including the length prefix.
///
/// # Errors
/// Returns an error if protobuf serialization fails or if the writer cannot accept all bytes.
pub async fn write_length_delimited_message<T, W>(writer: &mut W, message: &T) -> Result<usize>
where
    T: Message,
    W: AsyncWrite + Unpin + Send + ?Sized,
{
    let buffer = message.encode_length_delimited_to_vec();
    writer.write_all(&buffer).await?;

    Ok(buffer.len())
}

/// Uncompressed protobuf writer type alias.
pub type UnCompressedWriter = BufWriter<File>;
/// Compressed protobuf writer type alias.
pub type CompressedWriter = WoodstockCompressionWriter<File>;

/// Creates a new file for writing, optionally using atomic writes.
///
/// # Arguments
/// * `path` - Path to the file to create.
/// * `is_atomic` - Whether to use atomic writes.
///
/// # Returns
///
/// * `Ok((BufWriter<File>, Option<PathBuf>))` - A tuple containing the buffered writer and an optional temporary path.
/// * `Err(eyre::Report)` if the file or directory cannot be created.
///
/// # Errors
///
/// Returns an error if the file or directory cannot be created.
async fn create_file<P: AsRef<Path>>(
    path: P,
    is_atomic: bool,
) -> Result<(BufWriter<File>, Option<PathBuf>)> {
    let path = path.as_ref();
    let parent = path.parent().unwrap_or(path);
    create_dir_all(parent).await?;

    let temp_path = if is_atomic {
        Some(path.with_extension(format!("tmp.{}", uuid::Uuid::new_v4())))
    } else {
        None
    };

    let save_path = temp_path.as_deref().unwrap_or(path);
    let file = File::create(save_path).await?;
    let writer = BufWriter::new(file);

    Ok((writer, temp_path))
}

/// Opens a file for appending protobuf messages.
///
/// This function opens a file in append mode, creating it if it does not exist, and wraps it in a buffered writer.
///
/// # Type Parameters
///
/// * `P` - A type that can be referenced as a `Path`. Typically a `&str` or `PathBuf`.
///
/// # Arguments
///
/// * `path` - Path to the file to open for appending.
///
/// # Returns
///
/// * `Ok((BufWriter<File>, Option<PathBuf>))` - A tuple containing the buffered writer and an optional temporary path (always `None` for appending).
/// * `Err(eyre::Report)` if the file cannot be opened for appending.
///
/// # Errors
///
/// Returns an error if the file cannot be opened for appending.
async fn open_file<P: AsRef<Path>>(path: P) -> Result<(BufWriter<File>, Option<PathBuf>)> {
    let path = path.as_ref();

    let file = File::options().create(true).append(true).open(path).await?;
    let writer = BufWriter::new(file);

    Ok((writer, None))
}

/// Generic structure for writing protobuf messages to a file.
///
/// # Generics
///
/// * `W` - The writer type. This can be a buffered file writer (`BufWriter<File>`) for uncompressed output, or a zstd encoder (`ZstdEncoder<BufWriter<File>>`) for compressed output.
/// * `T` - The protobuf message type to write. Must implement the `prost::Message` trait.
///
/// The writer supports both compressed and uncompressed formats:
/// - **Uncompressed**: Uses a standard buffered file writer. Data is written as a sequence of length-delimited protobuf messages.
/// - **Compressed**: Uses a zlib encoder over a buffered file writer. Data is compressed using the zlib format and written as a sequence of length-delimited protobuf messages.
///
/// Atomic writes are supported by writing to a temporary file and renaming it to the target path on success.
pub struct ProtobufWriter<W, T>
where
    W: AsyncWrite + Unpin + Send,
    T: Message,
{
    /// The underlying writer.
    writer: W,
    /// Marker for the message type.
    _marker: std::marker::PhantomData<T>,
    /// Whether the write is atomic.
    is_atomic: bool,
    /// Path to the original file.
    original_path: PathBuf,
    /// Path to the temporary file, if atomic.
    temp_path: Option<PathBuf>,
}

impl<T> ProtobufWriter<UnCompressedWriter, T>
where
    T: Message,
{
    /// Creates a new uncompressed protobuf writer.
    ///
    /// # Arguments
    /// * `path` - Path to the file to write.
    /// * `is_atomic` - Whether to use atomic writes.
    ///
    /// # Errors
    /// Returns an error if the file cannot be created.
    pub async fn new<P: AsRef<Path>>(path: P, is_atomic: bool) -> Result<Self> {
        let (writer, temp_path) = create_file(&path, is_atomic).await?;
        let path = path.as_ref();

        Ok(Self {
            writer,
            _marker: std::marker::PhantomData,
            is_atomic,
            original_path: path.to_path_buf(),
            temp_path,
        })
    }

    /// Opens an existing file for appending protobuf messages.
    ///
    /// # Arguments
    /// * `path` - Path to the file to open.
    ///
    /// # Errors
    /// Returns an error if the file cannot be opened.
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let (writer, temp_path) = open_file(&path).await?;
        let path = path.as_ref();

        Ok(Self {
            writer,
            _marker: std::marker::PhantomData,
            is_atomic: false,
            original_path: path.to_path_buf(),
            temp_path,
        })
    }

    /// Seeks to a position in the file.
    ///
    /// # Arguments
    /// * `pos` - The position to seek to.
    ///
    /// # Errors
    /// Returns an error if seeking fails.
    pub async fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let count = self.writer.get_mut().seek(pos).await?;

        Ok(count)
    }
}

impl<T> ProtobufWriter<CompressedWriter, T>
where
    T: Message,
{
    /// Creates a new compressed protobuf writer.
    ///
    /// # Arguments
    /// * `path` - Path to the file to write.
    /// * `is_atomic` - Whether to use atomic writes.
    ///
    /// # Errors
    /// Returns an error if the file cannot be created.
    pub async fn new_compressed<P: AsRef<Path>>(
        path: P,
        is_atomic: bool,
        compression_format: CompressionFormat,
    ) -> Result<Self> {
        let (writer, temp_path) = create_file(&path, is_atomic).await?;
        let path = path.as_ref();

        let writer = WoodstockCompressionWriter::new(writer, compression_format); // Compression activée

        Ok(Self {
            writer,
            _marker: std::marker::PhantomData,
            is_atomic,
            original_path: path.to_path_buf(),
            temp_path,
        })
    }
}

impl<W, T> ProtobufWriter<W, T>
where
    W: AsyncWrite + Unpin + Send,
    T: Message,
{
    /// Writes a single protobuf message to the file.
    ///
    /// # Arguments
    /// * `message` - The message to write.
    ///
    /// # Errors
    /// Returns an error if writing fails.
    pub async fn write(&mut self, message: &T) -> Result<()> {
        write_length_delimited_message(&mut self.writer, message).await?;

        Ok(())
    }

    /// Writes a single protobuf message and returns the total number of bytes written
    /// (including the length-delimiter prefix).
    ///
    /// This is identical to [`write`](Self::write) but exposes the byte count, which is
    /// useful for callers that track byte offsets (e.g., shard index writers).
    ///
    /// # Errors
    /// Returns an error if writing fails.
    pub async fn write_size(&mut self, message: &T) -> Result<usize> {
        write_length_delimited_message(&mut self.writer, message).await
    }

    /// Writes raw bytes directly to the underlying writer.
    ///
    /// This bypasses protobuf framing and is intended for appending non-protobuf
    /// data such as an offset table footer.
    ///
    /// # Errors
    /// Returns an error if writing fails.
    pub async fn write_raw(&mut self, bytes: &[u8]) -> Result<()> {
        use tokio::io::AsyncWriteExt;
        self.writer.write_all(bytes).await?;
        Ok(())
    }

    /// Writes all protobuf messages from an iterator to the file.
    ///
    /// # Arguments
    /// * `messages` - The messages to write.
    ///
    /// # Errors
    /// Returns an error if writing fails.
    pub async fn write_all(&mut self, messages: impl IntoIterator<Item = T>) -> Result<()> {
        for message in messages {
            self.write(&message).await?;
        }

        Ok(())
    }

    /// Cancels the write operation, deleting the temporary file if atomic.
    ///
    /// # Errors
    /// Returns an error if the temporary file cannot be deleted.
    pub async fn cancel(&mut self) -> Result<()> {
        // If the file is atomic, the temporary file will be deleted.
        if self.is_atomic {
            if let Some(ref temp_path) = self.temp_path {
                remove_file(temp_path).await?;

                self.temp_path = None;
            }
        }

        Ok(())
    }

    /// Flushes the writer and finalizes the file.
    ///
    /// # Errors
    /// Returns an error if flushing or renaming fails.
    pub async fn flush(&mut self) -> Result<()> {
        // If ZstdEncoder
        self.writer.shutdown().await?;

        // If the file is atomic, the temporary file will be renamed to the target file.
        if self.is_atomic {
            if let Some(ref temp_path) = self.temp_path {
                rename(temp_path, &self.original_path).await?;
            }
        }

        Ok(())
    }
}

/// Writes a protobuf file to disk from a stream of messages.
///
/// The file is expected to be a sequence of length-delimited protobuf messages, optionally compressed.
///
/// # Arguments
/// * `path` - The path to the file to write.
/// * `source` - An async stream of messages to write.
/// * `is_atomic` - Whether to use atomic writes.
///
/// # Errors
/// Returns an error if writing fails.
pub async fn save_file<T: Message + Default, P: AsRef<Path>>(
    path: P,
    source: impl Stream<Item = T>,
    is_atomic: bool,
    compression_format: CompressionFormat,
) -> Result<()> {
    info!(
        "Saving file to {:?} (is_atomic = {is_atomic})",
        path.as_ref()
    );

    let mut writer =
        ProtobufWriter::<CompressedWriter, T>::new_compressed(&path, is_atomic, compression_format)
            .await?;
    pin_mut!(source);

    while let Some(message) = source.next().await {
        let result = writer.write(&message).await;
        if let Err(e) = result {
            warn!("Cancel saving file {:?}", path.as_ref());
            writer.cancel().await?;
            return Err(e);
        }
    }

    writer.flush().await?;

    info!("Saved file to {:?}", path.as_ref());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{proto::ProtobufReader, FileManifestJournalEntry};
    use eyre::Result;
    use futures::StreamExt;
    use std::fs::remove_file;

    struct CleanUp {
        filename: &'static str,
    }

    impl Drop for CleanUp {
        fn drop(&mut self) {
            remove_file(self.filename).unwrap();
        }
    }

    #[tokio::test]
    // Generate the file ./data/home.filelist that will take all file of clientrs to generate a filelist
    async fn generate_test_file() -> Result<()> {
        let _clean_up = CleanUp {
            filename: "./data/home.filelist.test",
        };

        use crate::EntryState;
        use crate::EntryType;
        use crate::FileManifestJournalEntry;
        use futures::stream;

        // Génère un stream de 100 FileManifestJournalEntry factices (avec valeurs minimales)
        let fake_entries = (0..100).map(|_| FileManifestJournalEntry {
            entry_type: EntryType::Add as i32,
            manifest: None,
            state: EntryState::Metadata as i32,
            state_messages: vec![],
            xfer_start: 0,
            xfer_calculation: 0,
            xfer_duration: 0,
            xfer_check: 0,
            chunk_sizes: vec![],
            chunk_compressed_sizes: vec![],
            snapshot_result: None,
        });
        let fake_stream = stream::iter(fake_entries);

        save_file(
            "./data/home.filelist.test",
            fake_stream,
            false,
            CompressionFormat::Zstd,
        )
        .await?;

        {
            let mut messages =
                ProtobufReader::<FileManifestJournalEntry>::new("./data/home.filelist.test", true)
                    .await?;
            let mut messages = messages.into_stream();

            let mut count = 0;
            while let Some(x) = messages.next().await {
                let _ = x?;
                count += 1;
            }

            assert_eq!(count, 100);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_save_file() {
        let _clean_up = CleanUp {
            filename: "./data/home.filelist.copy",
        };

        {
            let mut messages = ProtobufReader::<FileManifestJournalEntry>::new(
                "../e2e-tests/data/home.filelist",
                true,
            )
            .await
            .unwrap();
            let messages = messages.into_stream().filter_map(|x| async move {
                match x {
                    Ok(x) => Some(x),
                    Err(_) => None,
                }
            });

            save_file(
                "./data/home.filelist.copy",
                messages,
                true,
                CompressionFormat::Zstd,
            )
            .await
            .unwrap();
        }

        {
            let mut messages =
                ProtobufReader::<FileManifestJournalEntry>::new("./data/home.filelist.copy", true)
                    .await
                    .unwrap();
            let mut messages = messages.into_stream();

            let mut count = 0;
            while let Some(x) = messages.next().await {
                let _ = x.unwrap();
                count += 1;
            }

            assert_eq!(count, 76);
        }
    }
}
