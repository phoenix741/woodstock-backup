use std::{
    fs::File,
    io::{Read, Write},
};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tempfile::NamedTempFile;
use woodstock::{config::CHUNK_SIZE, utils::chunk_hasher::create_chunk_hasher, ChunkAlgorithm};

fn hash_data(file: File, algo: ChunkAlgorithm) -> Vec<u8> {
    let mut hasher = create_chunk_hasher(algo);

    let mut reader = std::io::BufReader::new(file);

    // Read by block of CHUNK_SIZE
    let mut buffer = vec![0; CHUNK_SIZE]; // 1MB buffer
    loop {
        let bytes_read = reader.read(&mut buffer).expect("Failed to read file");
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    hasher.finalize()
}

// Create a temporary file with 1Gb of random data
fn create_temp_file() -> NamedTempFile {
    let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");

    let random_data = vec![0u8; 1_073_741_824]; // 1Gb of random data
    temp_file
        .write_all(&random_data)
        .expect("Failed to write to file");

    temp_file
}

fn criterion_benchmark(c: &mut Criterion) {
    let temp_file = create_temp_file();
    let algorithm = [
        ChunkAlgorithm::Sha3256,
        ChunkAlgorithm::Sha2256,
        ChunkAlgorithm::Blake3,
    ];

    for algo in &algorithm {
        c.bench_function(format!("hashing_{}", algo.as_str_name()).as_str(), |b| {
            b.iter(|| {
                let file = temp_file.reopen().expect("Failed to reopen temp file");
                hash_data(black_box(file), black_box(*algo));
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
