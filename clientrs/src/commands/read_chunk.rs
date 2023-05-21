use std::{
    error::Error,
    fs::File,
    io::{stdout, BufReader, Read, Write},
    path::Path,
};

use flate2::bufread::ZlibDecoder;

use crate::{pool::PoolChunkWrapper, scanner::BUFFER_SIZE};

pub fn read_chunk(pool_path: &Path, chunk: &str) -> Result<(), Box<dyn Error>> {
    let chunk = hex::decode(chunk)?;
    let chunk = PoolChunkWrapper::new(pool_path, Some(&chunk));

    if !chunk.exists() {
        return Err("Chunk doesn't exist".into());
    }

    let chunk_path = chunk.chunk_path();

    // Read all the chunk content
    let file = File::open(chunk_path)?;
    let file = BufReader::new(file);
    let mut file = ZlibDecoder::new(file);

    let mut buffer = [0; BUFFER_SIZE];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }

        stdout().write_all(&buffer[..read]).unwrap();
    }

    Ok(())
}
