use async_compression::tokio::write::ZlibEncoder;
use eyre::Result;
use futures::StreamExt;
use futures::{pin_mut, Stream};
use log::{info, warn};
use prost::Message;
use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use tokio::io::AsyncSeekExt;
use tokio::{
    fs::{create_dir_all, remove_file, rename, File},
    io::{AsyncWrite, AsyncWriteExt, BufWriter},
};

/// Uncompressed protobuf writer type alias.
pub type UnCompressedWriter = BufWriter<File>;
/// Compressed protobuf writer type alias.
pub type CompressedWriter = ZlibEncoder<BufWriter<File>>;

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
/// * `W` - The writer type. This can be a buffered file writer (`BufWriter<File>`) for uncompressed output, or a zlib encoder (`ZlibEncoder<BufWriter<File>>`) for compressed output.
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
    pub async fn new_compressed<P: AsRef<Path>>(path: P, is_atomic: bool) -> Result<Self> {
        let (writer, temp_path) = create_file(&path, is_atomic).await?;
        let path = path.as_ref();

        let writer = ZlibEncoder::new(writer); // Compression activée

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
        let mut buf = Vec::new();
        message.encode_length_delimited(&mut buf)?;
        self.writer.write_all(&buf).await?;

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
        // If ZLibEncoder
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
) -> Result<()> {
    info!(
        "Saving file to {:?} (is_atomic = {is_atomic})",
        path.as_ref()
    );

    let mut writer =
        ProtobufWriter::<CompressedWriter, T>::new_compressed(&path, is_atomic).await?;
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
    use crate::{
        client::scanner::{get_files, CreateManifestOptions},
        proto::ProtobufReader,
        utils::path::list_to_globset,
        FileManifestJournalEntry,
    };
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

        {
            let share_path = std::env::current_dir()?;
            let includes = list_to_globset(&[])?;
            let excludes = list_to_globset(&["target"])?;
            let options = CreateManifestOptions {
                with_acl: cfg!(unix),
                with_xattr: cfg!(unix),
            };

            let files = get_files(&share_path, &includes, &excludes, &options);
            save_file("./data/home.filelist.test", files, false).await?;
        };

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

            assert!(count > 50);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_save_file() {
        let _clean_up = CleanUp {
            filename: "./data/home.filelist.copy",
        };

        {
            let mut messages =
                ProtobufReader::<FileManifestJournalEntry>::new("./data/home.filelist", true)
                    .await
                    .unwrap();
            let messages = messages.into_stream().filter_map(|x| async move {
                match x {
                    Ok(x) => Some(x),
                    Err(_) => None,
                }
            });

            save_file("./data/home.filelist.copy", messages, true)
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
