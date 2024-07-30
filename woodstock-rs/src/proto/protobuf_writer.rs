use std::{
    io,
    path::{Path, PathBuf},
    pin::Pin,
};

use async_compression::tokio::write::ZlibEncoder;
use futures::StreamExt;
use futures::{pin_mut, Stream};
use prost::Message;
use tokio::{
    fs::{create_dir_all, remove_file, rename, File},
    io::{AsyncWrite, AsyncWriteExt, BufWriter},
};

pub struct ProtobufWriter<T: Message> {
    writer: Pin<Box<dyn AsyncWrite + Send>>,
    _marker: std::marker::PhantomData<T>,

    is_atomic: bool,
    original_path: PathBuf,
    temp_path: Option<PathBuf>,
}

impl<T: Message> ProtobufWriter<T> {
    pub async fn new<P: AsRef<Path>>(path: P, compress: bool, is_atomic: bool) -> io::Result<Self> {
        // Create the directory if it does not exist.
        let path: &Path = path.as_ref();
        let parent = path.parent().unwrap_or(path);
        create_dir_all(parent).await?;

        // If is atomic the filename will be modified
        let temp_path = if is_atomic {
            Some(path.with_extension(format!("tmp.{}", uuid::Uuid::new_v4())))
        } else {
            None
        };

        let save_path = match &temp_path {
            Some(path) => path.as_ref(),
            None => path,
        };

        // Create the buffer
        let file = File::create(save_path).await?;
        let writer: Pin<Box<dyn AsyncWrite + Send>> = if compress {
            Box::pin(ZlibEncoder::new(BufWriter::new(file)))
        } else {
            Box::pin(BufWriter::new(file))
        };

        Ok(Self {
            writer,
            _marker: std::marker::PhantomData,
            is_atomic,

            original_path: path.to_path_buf(),
            temp_path,
        })
    }

    pub async fn write(&mut self, message: &T) -> io::Result<()> {
        let mut buf = Vec::new();
        message.encode_length_delimited(&mut buf)?;
        self.writer.write_all(&buf).await?;

        Ok(())
    }

    pub async fn write_all(&mut self, messages: impl IntoIterator<Item = T>) -> io::Result<()> {
        for message in messages {
            self.write(&message).await?;
        }

        Ok(())
    }

    pub async fn cancel(&mut self) -> io::Result<()> {
        // If the file is atomic, the temporary file will be deleted.
        if self.is_atomic {
            if let Some(ref temp_path) = self.temp_path {
                remove_file(temp_path).await?;

                self.temp_path = None;
            }
        }

        Ok(())
    }

    pub async fn flush(&mut self) -> io::Result<()> {
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
    compress: bool,
    is_atomic: bool,
) -> io::Result<()> {
    let mut writer = ProtobufWriter::<T>::new(path, compress, is_atomic).await?;
    pin_mut!(source);

    while let Some(message) = source.next().await {
        let result = writer.write(&message).await;
        if let Err(e) = result {
            writer.cancel().await?;
            return Err(e);
        }
    }

    writer.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        proto::ProtobufReader,
        scanner::{get_files, CreateManifestOptions},
        utils::path::list_to_globset,
        EntryType, FileManifestJournalEntry,
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

            let files = get_files(&share_path, &includes, &excludes, &options).map(|m| {
                FileManifestJournalEntry {
                    r#type: EntryType::Add as i32,
                    manifest: Some(m),
                }
            });
            save_file("./data/home.filelist.test", files, true, false).await?;
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

            save_file("./data/home.filelist.copy", messages, true, true)
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
