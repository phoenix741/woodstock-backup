use eyre::Result;
use tempfile::TempDir;
use woodstock::pool::{IndexedChunk, PoolIndex};
use woodstock::utils::compression::CompressionFormat;

pub fn build_chunk(seed: usize) -> IndexedChunk {
    let mut hash = vec![0_u8; 32];
    hash[0] = (seed % 256) as u8;
    hash[1..9].copy_from_slice(&(seed as u64).to_be_bytes());

    IndexedChunk {
        hash,
        size: 4096 + (seed % 2048) as u64,
        compressed_size: 1024 + (seed % 512) as u64,
        compression_format: CompressionFormat::Zstd,
        ref_count: 1,
        segment_id: 1 + (seed / 4096) as u64,
        offset: (seed as u64) * 8192,
        chunk_header_size: 24,
    }
}

pub fn build_chunks(start: usize, count: usize) -> Vec<IndexedChunk> {
    (start..start + count).map(build_chunk).collect()
}

pub fn create_populated_index(
    chunk_count: usize,
) -> Result<(TempDir, PoolIndex, Vec<IndexedChunk>)> {
    let tempdir = tempfile::tempdir()?;
    let index = PoolIndex::open_or_create(tempdir.path().join("index"))?;
    let chunks = build_chunks(0, chunk_count);
    index.add_chunks(&chunks)?;
    Ok((tempdir, index, chunks))
}
