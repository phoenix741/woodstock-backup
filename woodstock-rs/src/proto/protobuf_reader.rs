use std::{io, path::Path, pin::Pin};

use futures::stream::unfold;
use futures::Stream;
use prost::Message;
use tokio::io::AsyncReadExt;
use tokio::{
    fs::File,
    io::{AsyncRead, BufReader},
};

use crate::utils::compression::WoodstockCompressionReader;

async fn read_length_delimited_message_with_allocated_buffer<R, T>(
    reader: &mut R,
    cache: &mut Vec<u8>,
    buf: &mut T,
) -> io::Result<usize>
where
    T: Message + Default,
    R: AsyncRead + Unpin + ?Sized,
{
    let mut encoded_length = Vec::with_capacity(10);
    // Read the length of the message (varint), one byte at a time. Each byte in the varint has a continuation bit that indicates if the byte that follows it is part of the varint. This is the most significant bit (MSB) of the byte (sometimes also called the sign bit). The lower 7 bits are a payload; the resulting integer is built by appending together the 7-bit payloads of its constituent bytes.

    loop {
        match reader.read_u8().await {
            Ok(byte) => {
                encoded_length.push(byte);

                if byte & 0b1000_0000 == 0 || encoded_length.len() == 10 {
                    break;
                }
            }
            Err(error)
                if error.kind() == io::ErrorKind::UnexpectedEof && encoded_length.is_empty() =>
            {
                return Ok(0);
            }
            Err(error) => return Err(error),
        }
    }

    let length = prost::decode_length_delimiter(&encoded_length[..])?;
    let encoded_length_size = encoded_length.len();
    let buffer_length = encoded_length_size + length;

    cache.clear();
    cache.reserve(buffer_length);
    cache.extend_from_slice(&encoded_length);
    cache.resize(buffer_length, 0);
    reader.read_exact(&mut cache[encoded_length_size..]).await?;

    buf.merge_length_delimited(&cache[..])?;

    Ok(buffer_length)
}

/// Reads a length-delimited protobuf message from the given async reader into a new instance of the message type `T`.
///
/// # Arguments
/// * `reader` - The async reader to read from.
///
/// # Returns
/// * `Ok((T, usize))` - A tuple containing the decoded message and the total number of bytes read (including the length
/// delimiter).
/// * `Err(io::Error)` if reading from the reader or decoding the message fails.
///
/// # Errors
/// Returns an error if reading from the reader or decoding the message fails.
///
/// # Notes
/// This function reads the length of the message as a varint, then reads the message bytes
/// into a provided buffer, and finally decodes the message from the buffer.
pub async fn read_length_delimited_message<R, T>(reader: &mut R) -> io::Result<Option<(T, usize)>>
where
    T: Message + Default,
    R: AsyncRead + Unpin + ?Sized,
{
    let mut cache = Vec::with_capacity(T::encoded_len(&T::default()) + 10); // Initial buffer size, can be adjusted based on expected message sizes
    let mut message = T::default();
    let size =
        read_length_delimited_message_with_allocated_buffer(reader, &mut cache, &mut message)
            .await?;

    if size == 0 {
        Ok(None)
    } else {
        Ok(Some((message, size)))
    }
}

/// A reader for protobuf files.
///
/// The file is expected to be a sequence of length-delimited protobuf messages.
pub struct ProtobufReader<T: Message + Default> {
    /// The underlying async reader.
    reader: Pin<Box<dyn AsyncRead + Send + Sync>>,
    /// Marker for the message type.
    _marker: std::marker::PhantomData<T>,
    /// Reusable buffer for reading messages.
    buffer: Vec<u8>,
}

impl<T: Message + Default> ProtobufReader<T> {
    /// Creates a new `ProtobufReader` for the given file path.
    ///
    /// # Arguments
    /// * `path` - Path to the protobuf file.
    /// * `compress` - Whether the file is compressed with zlib.
    ///
    /// # Returns
    ///
    /// * `Ok(ProtobufReader<T>)` - A new instance of `ProtobufReader`.
    /// * `Err(io::Error)` if the file cannot be opened.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened.
    pub async fn new<P: AsRef<Path>>(path: P, compress: bool) -> io::Result<Self> {
        let file = File::open(path).await?;
        let reader: Pin<Box<dyn AsyncRead + Send + Sync>> = if compress {
            Box::pin(WoodstockCompressionReader::new(BufReader::new(file)))
        } else {
            Box::pin(BufReader::new(file))
        };

        Ok(Self {
            reader,
            _marker: std::marker::PhantomData,
            buffer: Vec::with_capacity(1024), // Taille initiale du buffer
        })
    }

    /// Reads a single protobuf message from the file into the provided buffer.
    ///
    /// # Arguments
    /// * `buf` - The message buffer to fill.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the message is successfully read.
    /// * `Err(io::Error)` if reading or decoding fails.
    ///
    /// # Errors
    ///
    /// Returns an error if reading or decoding fails.
    pub async fn read(&mut self, buf: &mut T) -> io::Result<()> {
        let size = read_length_delimited_message_with_allocated_buffer(
            &mut self.reader,
            &mut self.buffer,
            buf,
        )
        .await?;
        if size == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "End of file reached",
            ));
        }

        Ok(())
    }

    /// Reads all protobuf messages from the file into the provided vector.
    ///
    /// # Arguments
    /// * `messages` - The vector to fill with messages.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - The number of messages read.
    /// * `Err(io::Error)` if reading fails.
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails.
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

    /// Returns a stream over all protobuf messages in the file.
    ///
    /// # Returns
    ///
    /// A stream of protobuf messages, where each item is a `Result` containing a message or an `io::Error`.
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
            "../e2e-tests/data/home.filelist",
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
            "../e2e-tests/data/home.filelist",
            true,
        )
        .await
        .unwrap();
        let iter = iter.into_stream();

        let count = iter.count().await;

        assert_eq!(count, 76);
    }
}
