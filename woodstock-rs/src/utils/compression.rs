use std::fmt::{Display, Formatter};
use std::io::{ErrorKind, Result as IoResult};
use std::pin::Pin;
use std::task::{Context, Poll};

use async_compression::tokio::bufread::{
    BrotliDecoder, LzmaDecoder, XzDecoder, ZlibDecoder, ZstdDecoder,
};
use async_compression::tokio::write::{
    BrotliEncoder, LzmaEncoder, XzEncoder, ZlibEncoder, ZstdEncoder,
};
use futures::ready;
use std::io::Cursor;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, BufWriter, ReadBuf};

/// Compression format magic numbers used in Woodstock files
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompressionHeader {
    /// No compression
    None = 0x57534300, // WSC00
    /// Zlib compression format.
    Zlib = 0x57534301, // WSC01
    /// Zstd compression format.
    Zstd = 0x57534302, // WSC02
    /// Brotli
    Brotli = 0x57534303, // WSC03
    /// LZMA compression format.
    Lzma = 0x57534304, // WSC04
    /// XZ compression format.
    Xz = 0x57534305, // WSC05
}

impl CompressionHeader {
    const HEADER_SIZE: usize = 4;
}

impl TryFrom<u32> for CompressionHeader {
    type Error = ();

    /// Convert from u32 to compression header
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            x if x == CompressionHeader::None as u32 => Ok(Self::None),
            x if x == CompressionHeader::Zlib as u32 => Ok(Self::Zlib),
            x if x == CompressionHeader::Zstd as u32 => Ok(Self::Zstd),
            x if x == CompressionHeader::Brotli as u32 => Ok(Self::Brotli),
            x if x == CompressionHeader::Lzma as u32 => Ok(Self::Lzma),
            x if x == CompressionHeader::Xz as u32 => Ok(Self::Xz),
            _ => Err(()),
        }
    }
}

/// Internal reader state after header detection
enum ReaderState<R> {
    /// Raw data without compression
    None(BufReader<R>),
    /// Zlib compressed data
    Zlib(ZlibDecoder<BufReader<R>>),
    /// Zstd compressed data  
    Zstd(ZstdDecoder<BufReader<R>>),
    /// Brotli compressed data
    Brotli(BrotliDecoder<BufReader<R>>),
    /// LZMA compressed data
    Lzma(LzmaDecoder<BufReader<R>>),
    /// XZ compressed data
    Xz(XzDecoder<BufReader<R>>),
    /// Legacy zlib with header bytes prepended
    LegacyZlib(ZlibDecoder<BufReader<ChainReader<BufReader<R>>>>),
}

/// A reader that chains header bytes with the original reader for legacy format support
struct ChainReader<R> {
    header_cursor: Option<Cursor<Vec<u8>>>,
    original_reader: R,
}

impl<R> ChainReader<R> {
    fn new(header_bytes: Vec<u8>, reader: R) -> Self {
        Self {
            header_cursor: Some(Cursor::new(header_bytes)),
            original_reader: reader,
        }
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for ChainReader<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        // First try to read from header cursor
        if let Some(ref mut cursor) = self.header_cursor {
            match Pin::new(cursor).poll_read(cx, buf) {
                Poll::Ready(Ok(())) => {
                    if buf.filled().is_empty() {
                        // Header exhausted, remove cursor and continue with original reader
                        self.header_cursor = None;
                    } else {
                        // Still have header data to read
                        return Poll::Ready(Ok(()));
                    }
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }

        // Read from original reader
        Pin::new(&mut self.original_reader).poll_read(cx, buf)
    }
}

/// The goal of WoodstockCompressionReader is to provide a way to read compressed data
/// in a Woodstock backup transparently.
///
/// The file to read may contain a header with the compression type (None, Zlib or Zstd),
/// followed by the (possibly compressed) data. The reader will automatically detect the
/// compression type and decompress the data accordingly.
///
/// If no header is found, we assume legacy ZLib format, and we will use
/// `ZlibDecoder` to decompress the data.
///
/// If the header indicates "no compression", the reader will simply return the data
/// without any decompression.
///
/// This reader implements `AsyncRead` for seamless integration with tokio-based code.
pub struct WoodstockCompressionReader<R: AsyncRead + Unpin> {
    /// Current state of the reader after format detection
    state: Option<ReaderState<R>>,
    /// Buffer for header detection
    header_buffer: [u8; CompressionHeader::HEADER_SIZE],
    /// Number of bytes read for header detection
    header_bytes_read: usize,
    /// Whether format detection is complete
    format_detected: bool,
}

impl<R: AsyncRead + Unpin> WoodstockCompressionReader<R> {
    /// Create a new WoodstockCompressionReader
    ///
    /// Format detection will happen on the first read operation.
    pub fn new(reader: R) -> Self {
        WoodstockCompressionReader {
            state: Some(ReaderState::None(BufReader::new(reader))),
            header_buffer: [0u8; CompressionHeader::HEADER_SIZE],
            header_bytes_read: 0,
            format_detected: false,
        }
    }

    /// Create a new WoodstockCompressionReader
    pub fn with_none(reader: R) -> Self {
        WoodstockCompressionReader {
            state: Some(ReaderState::None(BufReader::new(reader))),
            header_buffer: [0u8; CompressionHeader::HEADER_SIZE],
            header_bytes_read: 0,
            format_detected: true,
        }
    }

    /// Detect the compression format by reading the header
    fn detect_format(&mut self, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        if self.format_detected {
            return Poll::Ready(Ok(()));
        }

        // Extract reader from state to read header
        let mut reader = match self.state.take() {
            Some(ReaderState::None(r)) => r,
            _ => {
                return Poll::Ready(Err(std::io::Error::new(
                    ErrorKind::InvalidData,
                    "Invalid state during format detection",
                )))
            }
        };

        // Try to read header bytes
        while self.header_bytes_read < CompressionHeader::HEADER_SIZE {
            let mut read_buf = ReadBuf::new(&mut self.header_buffer[self.header_bytes_read..]);
            match Pin::new(&mut reader).poll_read(cx, &mut read_buf) {
                Poll::Ready(Ok(())) => {
                    let bytes_read = read_buf.filled().len();
                    if bytes_read == 0 {
                        // EOF reached before complete header - assume legacy zlib
                        self.state = Some(ReaderState::Zlib(ZlibDecoder::new(reader)));
                        self.format_detected = true;
                        return Poll::Ready(Ok(()));
                    }
                    self.header_bytes_read += bytes_read;
                }
                Poll::Ready(Err(e)) => {
                    self.state = Some(ReaderState::None(reader));
                    return Poll::Ready(Err(e));
                }
                Poll::Pending => {
                    self.state = Some(ReaderState::None(reader));
                    return Poll::Pending;
                }
            }
        }

        // We have a complete header, parse it
        let header_value = u32::from_be_bytes(self.header_buffer);

        match CompressionHeader::try_from(header_value) {
            Ok(CompressionHeader::None) => {
                // No compression, use reader directly
                self.state = Some(ReaderState::None(reader));
            }
            Ok(CompressionHeader::Zlib) => {
                // Zlib compression
                self.state = Some(ReaderState::Zlib(ZlibDecoder::new(reader)));
            }
            Ok(CompressionHeader::Zstd) => {
                // Zstd compression
                self.state = Some(ReaderState::Zstd(ZstdDecoder::new(reader)));
            }
            Ok(CompressionHeader::Brotli) => {
                // Brotli compression
                self.state = Some(ReaderState::Brotli(BrotliDecoder::new(reader)));
            }
            Ok(CompressionHeader::Lzma) => {
                // LZMA compression
                self.state = Some(ReaderState::Lzma(LzmaDecoder::new(reader)));
            }
            Ok(CompressionHeader::Xz) => {
                // XZ compression
                self.state = Some(ReaderState::Xz(XzDecoder::new(reader)));
            }

            Err(()) => {
                // No valid header found - assume legacy zlib and rewind
                // Create a chain reader that includes the header bytes we already read
                let header_bytes = self.header_buffer.to_vec();
                let chain_reader = ChainReader::new(header_bytes, reader);
                self.state = Some(ReaderState::LegacyZlib(ZlibDecoder::new(BufReader::new(
                    chain_reader,
                ))));
            }
        }

        self.format_detected = true;
        Poll::Ready(Ok(()))
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for WoodstockCompressionReader<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        // First ensure format is detected
        ready!(self.detect_format(cx))?;

        // Now delegate to the appropriate reader
        match self.state.as_mut() {
            Some(ReaderState::None(reader)) => Pin::new(reader).poll_read(cx, buf),
            Some(ReaderState::Zlib(decoder)) => Pin::new(decoder).poll_read(cx, buf),
            Some(ReaderState::Zstd(decoder)) => Pin::new(decoder).poll_read(cx, buf),
            Some(ReaderState::Brotli(decoder)) => Pin::new(decoder).poll_read(cx, buf),
            Some(ReaderState::Lzma(decoder)) => Pin::new(decoder).poll_read(cx, buf),
            Some(ReaderState::Xz(decoder)) => Pin::new(decoder).poll_read(cx, buf),
            Some(ReaderState::LegacyZlib(decoder)) => Pin::new(decoder).poll_read(cx, buf),
            None => Poll::Ready(Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "Reader in invalid state",
            ))),
        }
    }
}

/// Compression format to use for writing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionFormat {
    /// No compression
    None = 0,
    /// Zlib compression
    Zlib,
    /// Zstd compression
    Zstd,
    /// Brotli compression
    Brotli,
    /// LZMA compression
    Lzma,
    /// XZ compression
    Xz,
}

impl TryFrom<&str> for CompressionFormat {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "none" => Ok(CompressionFormat::None),
            "zlib" => Ok(CompressionFormat::Zlib),
            "zstd" => Ok(CompressionFormat::Zstd),
            "brotli" => Ok(CompressionFormat::Brotli),
            "lzma" => Ok(CompressionFormat::Lzma),
            "xz" => Ok(CompressionFormat::Xz),
            _ => Err(()),
        }
    }
}

impl Display for CompressionFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str_name())
    }
}

impl CompressionFormat {
    /// Convert to compression header value
    fn to_header(self) -> CompressionHeader {
        match self {
            CompressionFormat::None => CompressionHeader::None,
            CompressionFormat::Zlib => CompressionHeader::Zlib,
            CompressionFormat::Zstd => CompressionHeader::Zstd,
            CompressionFormat::Brotli => CompressionHeader::Brotli,
            CompressionFormat::Lzma => CompressionHeader::Lzma,
            CompressionFormat::Xz => CompressionHeader::Xz,
        }
    }

    fn as_str_name(&self) -> &'static str {
        match self {
            CompressionFormat::None => "none",
            CompressionFormat::Zlib => "zlib",
            CompressionFormat::Zstd => "zstd",
            CompressionFormat::Brotli => "brotli",
            CompressionFormat::Lzma => "lzma",
            CompressionFormat::Xz => "xz",
        }
    }
}

/// Internal writer state after initialization
enum WriterState<W> {
    /// Raw data without compression
    None(BufWriter<W>),
    /// Zlib compressed data
    Zlib(ZlibEncoder<BufWriter<W>>),
    /// Zstd compressed data
    Zstd(ZstdEncoder<BufWriter<W>>),
    /// Brotli compressed data
    Brotli(BrotliEncoder<BufWriter<W>>),
    /// LZMA compressed data
    Lzma(LzmaEncoder<BufWriter<W>>),
    /// XZ compressed data
    Xz(XzEncoder<BufWriter<W>>),
}

/// State of header writing process
#[derive(Debug)]
enum HeaderState {
    /// Header needs to be written
    Pending([u8; 4]),
    /// Header partially written (bytes_written)
    Writing([u8; 4], usize),
    /// Header completely written
    Complete,
}

/// Writer for Woodstock format with compression support
///
/// This writer automatically adds the appropriate compression header based on the format
/// selected during construction, then applies the specified compression to the data.
///
/// The writer supports:
/// - No compression (raw data with header)
/// - Zlib compression
/// - Zstd compression
///
/// The header is written immediately upon creation to identify the compression format.
pub struct WoodstockCompressionWriter<W: AsyncWrite + Unpin> {
    /// Current state of the writer
    state: Option<WriterState<W>>,
    /// State of header writing
    header_state: HeaderState,
}

impl<W: AsyncWrite + Unpin> WoodstockCompressionWriter<W> {
    /// Create a new WoodstockCompressionWriter
    ///
    /// The compression format is specified during construction and the appropriate header
    /// will be written to identify the format used.
    ///
    /// # Arguments
    /// * `writer` - The underlying writer to write to
    /// * `format` - The compression format to use
    ///
    /// # Returns
    /// A new WoodstockCompressionWriter instance
    pub fn new(writer: BufWriter<W>, format: CompressionFormat) -> Self {
        let state = match format {
            CompressionFormat::None => Some(WriterState::None(writer)),
            CompressionFormat::Zlib => Some(WriterState::Zlib(ZlibEncoder::new(writer))),
            CompressionFormat::Zstd => Some(WriterState::Zstd(ZstdEncoder::new(writer))),
            CompressionFormat::Brotli => Some(WriterState::Brotli(BrotliEncoder::new(writer))),
            CompressionFormat::Lzma => Some(WriterState::Lzma(LzmaEncoder::new(writer))),
            CompressionFormat::Xz => Some(WriterState::Xz(XzEncoder::new(writer))),
        };

        let header_bytes = (format.to_header() as u32).to_be_bytes();

        WoodstockCompressionWriter {
            state,
            header_state: HeaderState::Pending(header_bytes),
        }
    }

    /// Write the compression header synchronously
    /// Returns Poll::Ready(Ok(true)) if header is complete, Poll::Ready(Ok(false)) if more work needed
    /// Returns Poll::Pending if the underlying writer would block
    fn write_header_sync(&mut self, cx: &mut Context<'_>) -> Poll<Result<bool, std::io::Error>> {
        match &mut self.header_state {
            HeaderState::Complete => return Poll::Ready(Ok(true)),
            HeaderState::Pending(header_bytes) => {
                // Start writing header
                self.header_state = HeaderState::Writing(*header_bytes, 0);
            }
            HeaderState::Writing(_, _) => {}
        }

        // Continue writing header
        if let HeaderState::Writing(header_bytes, bytes_written) = &mut self.header_state {
            while *bytes_written < 4 {
                let remaining = &header_bytes[*bytes_written..];

                // Write to the underlying writer directly, not through the encoder
                let bytes_written_this_call = match self.state.as_mut() {
                    Some(WriterState::None(writer)) => {
                        match Pin::new(writer).poll_write(cx, remaining) {
                            Poll::Ready(Ok(n)) => n,
                            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                            Poll::Pending => return Poll::Pending,
                        }
                    }
                    Some(WriterState::Zlib(encoder)) => {
                        match Pin::new(encoder.get_mut()).poll_write(cx, remaining) {
                            Poll::Ready(Ok(n)) => n,
                            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                            Poll::Pending => return Poll::Pending,
                        }
                    }
                    Some(WriterState::Zstd(encoder)) => {
                        match Pin::new(encoder.get_mut()).poll_write(cx, remaining) {
                            Poll::Ready(Ok(n)) => n,
                            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                            Poll::Pending => return Poll::Pending,
                        }
                    }
                    Some(WriterState::Brotli(encoder)) => {
                        match Pin::new(encoder.get_mut()).poll_write(cx, remaining) {
                            Poll::Ready(Ok(n)) => n,
                            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                            Poll::Pending => return Poll::Pending,
                        }
                    }
                    Some(WriterState::Lzma(encoder)) => {
                        match Pin::new(encoder.get_mut()).poll_write(cx, remaining) {
                            Poll::Ready(Ok(n)) => n,
                            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                            Poll::Pending => return Poll::Pending,
                        }
                    }
                    Some(WriterState::Xz(encoder)) => {
                        match Pin::new(encoder.get_mut()).poll_write(cx, remaining) {
                            Poll::Ready(Ok(n)) => n,
                            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                            Poll::Pending => return Poll::Pending,
                        }
                    }
                    None => {
                        return Poll::Ready(Err(std::io::Error::new(
                            ErrorKind::InvalidData,
                            "Writer in invalid state",
                        )))
                    }
                };

                if bytes_written_this_call == 0 {
                    return Poll::Ready(Err(std::io::Error::new(
                        ErrorKind::WriteZero,
                        "Failed to write header bytes",
                    )));
                }

                *bytes_written += bytes_written_this_call;
            }

            // Header is complete
            self.header_state = HeaderState::Complete;
        }

        Poll::Ready(Ok(true))
    }

    /// Shutdown the writer and flush any remaining data
    ///
    /// This is important for compression encoders to ensure all data is properly written.
    pub async fn shutdown(&mut self) -> IoResult<()> {
        // Ensure header is written (this should be done synchronously in normal flow)
        if !matches!(self.header_state, HeaderState::Complete) {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "Header not written before shutdown",
            ));
        }

        match self.state.as_mut() {
            Some(WriterState::None(writer)) => {
                writer.flush().await?;
            }
            Some(WriterState::Zlib(encoder)) => {
                encoder.shutdown().await?;
            }
            Some(WriterState::Zstd(encoder)) => {
                encoder.shutdown().await?;
            }
            Some(WriterState::Brotli(encoder)) => {
                encoder.shutdown().await?;
            }
            Some(WriterState::Lzma(encoder)) => {
                encoder.shutdown().await?;
            }
            Some(WriterState::Xz(encoder)) => {
                encoder.shutdown().await?;
            }
            None => {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidData,
                    "Writer in invalid state",
                ))
            }
        }

        Ok(())
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for WoodstockCompressionWriter<W> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        // Ensure header is written first
        match ready!(self.write_header_sync(cx))? {
            true => {}                     // Header complete, continue
            false => return Poll::Pending, // Header not complete yet
        }

        // Delegate to the appropriate writer
        match self.state.as_mut() {
            Some(WriterState::None(writer)) => Pin::new(writer).poll_write(cx, buf),
            Some(WriterState::Zlib(encoder)) => Pin::new(encoder).poll_write(cx, buf),
            Some(WriterState::Zstd(encoder)) => Pin::new(encoder).poll_write(cx, buf),
            Some(WriterState::Brotli(encoder)) => Pin::new(encoder).poll_write(cx, buf),
            Some(WriterState::Lzma(encoder)) => Pin::new(encoder).poll_write(cx, buf),
            Some(WriterState::Xz(encoder)) => Pin::new(encoder).poll_write(cx, buf),
            // If we reach here, the writer is in an invalid state
            None => Poll::Ready(Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "Writer in invalid state",
            ))),
        }
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        // Ensure header is written before flushing
        match ready!(self.write_header_sync(cx))? {
            true => {}                     // Header complete, continue
            false => return Poll::Pending, // Header not complete yet
        }

        // Delegate to the appropriate writer
        match self.state.as_mut() {
            Some(WriterState::None(writer)) => Pin::new(writer).poll_flush(cx),
            Some(WriterState::Zlib(encoder)) => Pin::new(encoder).poll_flush(cx),
            Some(WriterState::Zstd(encoder)) => Pin::new(encoder).poll_flush(cx),
            Some(WriterState::Brotli(encoder)) => Pin::new(encoder).poll_flush(cx),
            Some(WriterState::Lzma(encoder)) => Pin::new(encoder).poll_flush(cx),
            Some(WriterState::Xz(encoder)) => Pin::new(encoder).poll_flush(cx),
            // If we reach here, the writer is in an invalid state
            None => Poll::Ready(Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "Writer in invalid state",
            ))),
        }
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        // Ensure header is written before shutdown
        match ready!(self.write_header_sync(cx))? {
            true => {}                     // Header complete, continue
            false => return Poll::Pending, // Header not complete yet
        }

        // Delegate to the appropriate writer for shutdown
        match self.state.as_mut() {
            Some(WriterState::None(writer)) => Pin::new(writer).poll_flush(cx),
            Some(WriterState::Zlib(encoder)) => Pin::new(encoder).poll_shutdown(cx),
            Some(WriterState::Zstd(encoder)) => Pin::new(encoder).poll_shutdown(cx),
            Some(WriterState::Brotli(encoder)) => Pin::new(encoder).poll_shutdown(cx),
            Some(WriterState::Lzma(encoder)) => Pin::new(encoder).poll_shutdown(cx),
            Some(WriterState::Xz(encoder)) => Pin::new(encoder).poll_shutdown(cx),
            // If we reach here, the writer is in an invalid state
            None => Poll::Ready(Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "Writer in invalid state",
            ))),
        }
    }
}
