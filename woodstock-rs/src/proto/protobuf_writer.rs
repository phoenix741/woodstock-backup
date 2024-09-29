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

pub type UnCompressedWriter = BufWriter<File>;
pub type CompressedWriter = ZlibEncoder<BufWriter<File>>;

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

async fn open_file<P: AsRef<Path>>(path: P) -> Result<(BufWriter<File>, Option<PathBuf>)> {
    let path = path.as_ref();

    let file = File::options().create(true).append(true).open(path).await?;
    let writer = BufWriter::new(file);

    Ok((writer, None))
}

// Structure générique
pub struct ProtobufWriter<W, T>
where
    W: AsyncWrite + Unpin + Send,
    T: Message,
{
    writer: W,
    _marker: std::marker::PhantomData<T>,
    is_atomic: bool,
    original_path: PathBuf,
    temp_path: Option<PathBuf>,
}

impl<T> ProtobufWriter<UnCompressedWriter, T>
where
    T: Message,
{
    // Constructeur pour écriture sans compression
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

    // Constructeur pour écriture sans compression
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

    pub async fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let count = self.writer.get_mut().seek(pos).await?;

        Ok(count)
    }
}

impl<T> ProtobufWriter<CompressedWriter, T>
where
    T: Message,
{
    // Constructeur pour écriture avec compression
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
    // Méthode d'écriture d'un message protobuf
    pub async fn write(&mut self, message: &T) -> Result<()> {
        let mut buf = Vec::new();
        message.encode_length_delimited(&mut buf)?;
        self.writer.write_all(&buf).await?;

        Ok(())
    }

    pub async fn write_all(&mut self, messages: impl IntoIterator<Item = T>) -> Result<()> {
        for message in messages {
            self.write(&message).await?;
        }

        Ok(())
    }

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

///
///  Write a protobuf file to disk.
///  The file is expected to be a sequence of length-delimited protobuf messages.
///  The file may be compressed with zlib.
///
/// The directory containing the file will be created if it does not exist.
///
/// # Arguments
///
/// * `path` - The path to the file to load.
/// * `source` - An iterator over the messages to write.
/// * `compress` - Whether the file is compressed with zlib.
/// * `is_atomic` - Whether the file is written atomically.
///
/// # Generics
/// * `T` - The protobuf message type to load.
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
        proto::ProtobufReader,
        scanner::{get_files, CreateManifestOptions},
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
