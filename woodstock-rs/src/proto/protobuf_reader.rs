use std::{io, path::Path, pin::Pin};

use async_compression::tokio::bufread::ZlibDecoder;
use futures::stream::unfold;
use futures::Stream;
use prost::Message;
use tokio::io::AsyncReadExt;
use tokio::{
    fs::File,
    io::{AsyncRead, BufReader},
};

/// A reader for protobuf files.
///
/// The file is expected to be a sequence of length-delimited protobuf messages.
pub struct ProtobufReader<T: Message + Default> {
    reader: Pin<Box<dyn AsyncRead + Send + Sync>>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Message + Default> ProtobufReader<T> {
    pub async fn new<P: AsRef<Path>>(path: P, compress: bool) -> io::Result<Self> {
        let file = File::open(path).await?;
        let reader: Pin<Box<dyn AsyncRead + Send + Sync>> = if compress {
            Box::pin(ZlibDecoder::new(BufReader::new(file)))
        } else {
            Box::pin(BufReader::new(file))
        };

        Ok(Self {
            reader,
            _marker: std::marker::PhantomData,
        })
    }

    pub async fn read(&mut self, buf: &mut T) -> io::Result<()> {
        let mut encoded_length = Vec::with_capacity(10);
        // Read the length of the message (varint), one byte at a time. Each byte in the varint has a continuation bit that indicates if the byte that follows it is part of the varint. This is the most significant bit (MSB) of the byte (sometimes also called the sign bit). The lower 7 bits are a payload; the resulting integer is built by appending together the 7-bit payloads of its constituent bytes.

        loop {
            let byte = self.reader.read_u8().await?;
            encoded_length.push(byte);

            if byte & 0b1000_0000 == 0 || encoded_length.len() == 10 {
                break;
            }
        }

        let length = prost::decode_length_delimiter(&encoded_length[..])?;
        let real_length_size = prost::length_delimiter_len(length);

        let mut message_buf = Vec::with_capacity(length + real_length_size);
        message_buf.extend_from_slice(&encoded_length);

        let mut messages_bytes = vec![0u8; length];
        self.reader.read_exact(&mut messages_bytes).await?;

        message_buf.extend_from_slice(&messages_bytes);

        buf.merge_length_delimited(&message_buf[..])?;

        Ok(())
    }

    pub async fn read_to_end(&mut self, messages: &mut Vec<T>) -> io::Result<usize> {
        let mut count = 0;
        loop {
            let mut message = T::default();
            match self.read(&mut message).await {
                Ok(()) => {
                    messages.push(message);
                    count += 1;
                }
                Err(e) => {
                    if e.kind() == io::ErrorKind::UnexpectedEof {
                        break;
                    }

                    return Err(e);
                }
            }
        }

        Ok(count)
    }

    pub fn into_stream(&mut self) -> Pin<Box<dyn Stream<Item = io::Result<T>> + Send + Sync + '_>> {
        Box::pin(unfold(self, |reader| async move {
            let mut message = T::default();

            match reader.read(&mut message).await {
                Ok(()) => Some((Ok(message), reader)),
                Err(e) => {
                    if e.kind() == io::ErrorKind::UnexpectedEof {
                        None
                    } else {
                        Some((Err(e), reader))
                    }
                }
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;

    use super::*;
    use crate::woodstock;

    #[tokio::test]
    async fn test_load_file() {
        let mut reader = ProtobufReader::<woodstock::FileManifestJournalEntry>::new(
            "./data/home.filelist",
            true,
        )
        .await
        .unwrap();

        let mut messages = Vec::<woodstock::FileManifestJournalEntry>::new();
        reader.read_to_end(&mut messages).await.unwrap();

        let count = messages.len();
        assert_eq!(count, 76);
    }

    #[tokio::test]
    async fn test_iterator() {
        let mut iter = ProtobufReader::<woodstock::FileManifestJournalEntry>::new(
            "./data/home.filelist",
            true,
        )
        .await
        .unwrap();
        let iter = iter.into_stream();

        let count = iter.count().await;

        assert_eq!(count, 76);
    }
}
