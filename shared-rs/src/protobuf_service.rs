use flate2::bufread::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use prost::{decode_length_delimiter, length_delimiter_len, Message};
use std::fs::{create_dir_all, File};
use std::io::Error;
use std::io::{self, BufReader, Read};
use std::path::Path;

///
///  Load a protobuf file from disk.
///  The file is expected to be a sequence of length-delimited protobuf messages.
///  The file may be compressed with zlib.
///
/// # Arguments
///
/// * `path` - The path to the file to load.
/// * `compress` - Whether the file is compressed with zlib.
///
/// # Generics
/// * `T` - The protobuf message type to load.
///
/// # Returns
/// * `Result<T, Error>` - An iterator over the messages in the file.
pub fn load_file<T: Message + Default>(
  path: &str,
  compress: bool,
) -> impl Iterator<Item = Result<T, Error>> {
  let file = File::open(path).unwrap();
  let mut reader: Box<dyn Read> = if compress {
    Box::new(ZlibDecoder::new(BufReader::new(file)))
  } else {
    Box::new(BufReader::new(file))
  };

  let message_iter = std::iter::from_fn(move || {
    let mut encoded_length: [u8; 10] = [0; 10];
    match reader.read_exact(&mut encoded_length) {
      Ok(()) => {
        let length = decode_length_delimiter(&encoded_length[..]).unwrap();
        let real_length_size = length_delimiter_len(length);

        let mut message_buf = Vec::with_capacity(length + real_length_size);
        message_buf.extend_from_slice(&encoded_length);

        let remaining_length = length + real_length_size - encoded_length.len();
        let mut remaining_bytes = vec![0u8; remaining_length];
        reader.read_exact(&mut remaining_bytes).unwrap();

        message_buf.extend_from_slice(&remaining_bytes);

        let message = T::decode_length_delimited(&message_buf[..]).unwrap();
        Some(Ok(message))
      }
      Err(error) => {
        if error.kind() == io::ErrorKind::UnexpectedEof {
          None
        } else {
          Some(Err(error))
        }
      }
    }
  });

  message_iter
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
///
/// # Generics
/// * `T` - The protobuf message type to load.
pub fn save_file<T: Message + Default>(
  path: &str,
  source: impl Iterator<Item = T>,
  compress: bool,
) -> Result<(), Error> {
  // Create the directory if it does not exist.
  let parent = Path::new(path).parent().unwrap();
  create_dir_all(parent)?;

  let file = File::create(path)?;

  let mut writer: Box<dyn io::Write> = if compress {
    Box::new(ZlibEncoder::new(file, Compression::default()))
  } else {
    Box::new(file)
  };

  for message in source {
    let mut buf = Vec::new();
    message.encode_length_delimited(&mut buf)?;
    writer.write_all(&buf)?;
  }

  Ok(())
}

///
/// Write a protobuf file to disk atomically.
/// The file is expected to be a sequence of length-delimited protobuf messages.
/// The file may be compressed with zlib.
/// The file will be written to a temporary file and then renamed to the target file.
///
/// The directory containing the file will be created if it does not exist.
///
/// # Arguments
/// * `path` - The path to the file to load.
/// * `source` - An iterator over the messages to write.
/// * `compress` - Whether the file is compressed with zlib.
///
/// # Generics
/// * `T` - The protobuf message type to load.
///
/// # Returns
/// * `Result<(), Error>` - An error if the file could not be written.
pub fn save_file_atomic<T: Message + Default>(
  path: &str,
  source: impl Iterator<Item = T>,
  compress: bool,
) -> Result<(), Error> {
  // Create a temporary file with a random name in the same path that path is in.
  let temp_path = format!("{}.tmp.{}", path, uuid::Uuid::new_v4());

  let result = save_file(&temp_path, source, compress);
  if result.is_ok() {
    std::fs::rename(&temp_path, path)?;
  } else {
    std::fs::remove_file(&temp_path)?;
  }

  result
}

///
/// Remove a protobuf file from the disk.
/// If the file doesn't exist, this function does nothing.
///
/// # Arguments
/// * `path` - The path to the file to remove.
///
/// # Returns
/// * `Result<(), Error>` - An error if the file could not be removed.
///
pub fn rm_file(path: &str) -> Result<(), Error> {
  let result = std::fs::remove_file(path);
  if result.is_err() {
    let error = result.unwrap_err();
    if error.kind() == io::ErrorKind::NotFound {
      return Ok(());
    } else {
      return Err(error);
    }
  }

  result
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::woodstock;

  #[test]
  fn test_load_file() {
    let messages = load_file::<woodstock::FileManifestJournalEntry>("./data/home.filelist", true);
    let count = messages.count();
    assert_eq!(count, 1000);
  }

  #[test]
  fn test_save_file() {
    let messages = load_file::<woodstock::FileManifestJournalEntry>("./data/home.filelist", true);
    save_file(
      "./data/home.filelist.copy",
      messages.filter_map(|m| m.ok()),
      true,
    )
    .unwrap();

    let messages =
      load_file::<woodstock::FileManifestJournalEntry>("./data/home.filelist.copy", true);
    let count = messages.count();
    assert_eq!(count, 1000);

    rm_file("./data/home.filelist.copy").unwrap();
  }

  #[test]
  fn test_save_file_atomic() {
    let messages = load_file::<woodstock::FileManifestJournalEntry>("./data/home.filelist", true);
    save_file_atomic(
      "./data/home.filelist.atomic_copy",
      messages.filter_map(|m| m.ok()),
      true,
    )
    .unwrap();

    let messages =
      load_file::<woodstock::FileManifestJournalEntry>("./data/home.filelist.atomic_copy", true);
    let count = messages.count();
    assert_eq!(count, 1000);

    rm_file("./data/home.filelist.atomic_copy").unwrap();
  }
}
