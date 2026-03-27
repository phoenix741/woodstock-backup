//! Benchmark: `copy_file_range` (COW) vs buffered `tokio::io::copy`.
//!
//! Measures both strategies across a range of payload sizes to empirically
//! determine the threshold below which the `spawn_blocking` + fd-clone overhead
//! of `copy_file_range` outweighs its benefits.
//!
//! Run with:
//!   cargo bench -p woodstock-rs --bench cow_copy_benchmark
//!
//! Typical interpretation:
//!   - Small sizes  → buffered copy wins (less overhead)
//!   - Large sizes  → copy_file_range wins (zero user-space copy + COW on btrfs/XFS)
//!   - Cross-over   → choose as COW_THRESHOLD in cow_copy/mod.rs

use std::io::Write;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use tempfile::tempdir;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::runtime::Runtime;
use woodstock::utils::cow_copy;

/// Payload sizes to benchmark (bytes).
const SIZES: &[u64] = &[
    4 * 1024,        // 4 KiB   — tiny chunk
    16 * 1024,       // 16 KiB
    64 * 1024,       // 64 KiB
    256 * 1024,      // 256 KiB
    1 * 1024 * 1024, // 1 MiB
    4 * 1024 * 1024, // 4 MiB   — large chunk
];

/// Creates a source file filled with pseudo-random-looking bytes of the given size.
fn create_source(dir: &std::path::Path, size: u64) -> std::path::PathBuf {
    let path = dir.join(format!("source_{size}.bin"));
    let mut f = std::fs::File::create(&path).unwrap();
    // Simple repeating pattern — content doesn't matter for the copy benchmark.
    let chunk = vec![0xABu8; 4096];
    let mut written = 0u64;
    while written < size {
        let to_write = ((size - written) as usize).min(chunk.len());
        f.write_all(&chunk[..to_write]).unwrap();
        written += to_write as u64;
    }
    path
}

fn bench_cow_copy(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let dir = tempdir().unwrap();

    let mut group = c.benchmark_group("copy_strategy");
    group.sample_size(20);

    for &size in SIZES {
        let source_path = create_source(dir.path(), size);
        group.throughput(Throughput::Bytes(size));

        // --- COW copy (copy_file_range on Linux) ---
        group.bench_with_input(
            BenchmarkId::new("copy_file_range", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    rt.block_on(async {
                        let dest_path = dir.path().join(format!("dest_cow_{size}.bin"));
                        let mut dest = File::create(&dest_path).await.unwrap();
                        dest.set_len(size).await.unwrap(); // pre-allocate
                        dest.seek(std::io::SeekFrom::Start(0)).await.unwrap();

                        cow_copy::copy_file_to_writer(&source_path, &mut dest, size, 0)
                            .await
                            .unwrap();
                    })
                })
            },
        );

        // --- Buffered copy (tokio::io::copy, 8 KiB buffer) ---
        group.bench_with_input(
            BenchmarkId::new("buffered_copy", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    rt.block_on(async {
                        let dest_path = dir.path().join(format!("dest_buf_{size}.bin"));
                        let mut dest = File::create(&dest_path).await.unwrap();

                        let src = File::open(&source_path).await.unwrap();
                        let mut limited = src.take(size);
                        tokio::io::copy(&mut limited, &mut dest).await.unwrap();
                    })
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_cow_copy);
criterion_main!(benches);
