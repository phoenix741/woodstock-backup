// Inspired from https://stackoverflow.com/questions/73145503/iterator-for-reading-file-chunks

use std::io::{self, Read};

// Create chunks from a reader

pub struct ToChunks<R> {
    reader: R,
    chunk_size: usize,
}

impl<R: Read> Iterator for ToChunks<R> {
    type Item = io::Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = vec![0u8; self.chunk_size];
        match self.reader.read(&mut buffer) {
            Ok(0) => None, // End of file reached
            Ok(n) => {
                buffer.truncate(n); // Resize buffer to actual read size
                Some(Ok(buffer))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

// Create a trait for iterating chunks from a reader

pub trait IterChunks {
    type Output;

    fn iter_chunks(self, len: usize) -> Self::Output;
}

impl<R: Read> IterChunks for R {
    type Output = ToChunks<R>;

    fn iter_chunks(self, len: usize) -> Self::Output {
        ToChunks {
            reader: self,
            chunk_size: len,
        }
    }
}
