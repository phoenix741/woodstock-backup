use flate2::bufread::ZlibDecoder;
use prost::{decode_length_delimiter, length_delimiter_len, Message};
use std::fs::File;
use std::io::{self, BufReader, Read};

/// Lit un fichier et émet des messages Protobuf.
fn load_file<T: Message + Default>(path: &str, compress: bool) -> impl Iterator<Item = T> {
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
        Some(message)
      }
      Err(error) => {
        if error.kind() == io::ErrorKind::UnexpectedEof {
          // Fin du fichier atteinte
          None
        } else {
          // Erreur lors de la lecture
          println!("Erreur de lecture : {}", error);
          None
        }
      }
    }
  });

  message_iter
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
}
