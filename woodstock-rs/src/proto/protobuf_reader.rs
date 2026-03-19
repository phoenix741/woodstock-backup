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

/// Reads one optional length-delimited protobuf frame into a caller-provided buffer.
///
/// The buffer is cleared and then filled with the complete length-delimited frame, including the
/// encoded length prefix and the message payload. The function returns the total number of bytes
/// written into `buf`.
///
/// A return value of `0` means EOF was reached before reading any byte of a new message.
///
/// # Errors
/// Returns an error if the length prefix is malformed, if the total size overflows, or if the
/// payload cannot be fully read.
async fn read_optional_length_delimited_buffer<R>(
    reader: &mut R,
    buf: &mut Vec<u8>,
) -> io::Result<usize>
where
    R: AsyncRead + Unpin + ?Sized,
{
    buf.clear();

    let mut encoded_length = Vec::with_capacity(10);

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

    let length = prost::decode_length_delimiter(&encoded_length[..]).map_err(io::Error::other)?;
    let real_length_size = prost::length_delimiter_len(length);
    let total_size = length
        .checked_add(real_length_size)
        .ok_or_else(|| io::Error::other("protobuf message length overflow"))?;

    buf.reserve(total_size);
    buf.extend_from_slice(&encoded_length);
    buf.resize(total_size, 0);

    reader.read_exact(&mut buf[real_length_size..]).await?;

    Ok(total_size)
}

/// Reads one optional length-delimited protobuf message and decodes it.
///
/// The provided buffer is reused to avoid repeated allocations in hot paths. On success, the
/// function returns both the decoded message and the total serialized size of the frame, including
/// the length prefix.
///
/// If EOF is reached before reading any byte of a new frame, `Ok(None)` is returned.
///
/// # Errors
/// Returns an error if the frame is malformed or truncated.
pub async fn read_optional_length_delimited_message<T, R>(
    reader: &mut R,
    buf: &mut Vec<u8>,
) -> io::Result<Option<(T, usize)>>
where
    T: Message + Default,
    R: AsyncRead + Unpin + ?Sized,
{
    let total_size = read_optional_length_delimited_buffer(reader, buf).await?;
    if total_size == 0 {
        return Ok(None);
    }

    let message = T::decode_length_delimited(&buf[..]).map_err(io::Error::other)?;
    Ok(Some((message, total_size)))
}

/// Reads one mandatory length-delimited protobuf message and decodes it.
///
/// This is the strict counterpart of [`read_optional_length_delimited_message`]. Reaching EOF
/// before a new frame starts is treated as an error.
///
/// # Errors
/// Returns an error if EOF is reached before a new frame begins, or if the frame is malformed or
/// truncated.
pub async fn read_length_delimited_message<T, R>(
    reader: &mut R,
    buf: &mut Vec<u8>,
) -> io::Result<(T, usize)>
where
    T: Message + Default,
    R: AsyncRead + Unpin + ?Sized,
{
    read_optional_length_delimited_message(reader, buf)
        .await?
        .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "missing protobuf message"))
}

/// A reader for protobuf files.
///
/// The file is expected to be a sequence of length-delimited protobuf messages.
///
/// A single internal buffer is reused across reads to minimize allocations while iterating over a
/// file of protobuf frames.
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

    /// Reads a single protobuf message from the file into the provided message value.
    ///
    /// # Arguments
    /// * `buf` - Message instance to populate with the decoded frame.
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
        read_length_delimited_message::<T, _>(&mut self.reader, &mut self.buffer).await?;
        buf.merge_length_delimited(&self.buffer[..])?;

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
    /// A stream of protobuf messages, where each item is a `Result` containing a message or an
    /// `io::Error`.
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
