#[path = "support/pool_index_benchmark_support.rs"]
mod support;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use tempfile::TempDir;
use woodstock::pool::PoolIndex;

struct ReadCase {
    _tempdir: TempDir,
    index: PoolIndex,
    hashes: Vec<Vec<u8>>,
}

fn create_read_case(chunk_count: usize) -> ReadCase {
    let (tempdir, index, chunks) = support::create_populated_index(chunk_count).unwrap();
    ReadCase {
        _tempdir: tempdir,
        index,
        hashes: chunks.into_iter().map(|chunk| chunk.hash).collect(),
    }
}

fn pool_index_get_chunk_read_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("pool_index_read_get_chunk");
    group.sample_size(10);

    let cases = [10_000_usize, 100_000]
        .into_iter()
        .map(|chunk_count| (chunk_count, create_read_case(chunk_count)))
        .collect::<Vec<_>>();

    for (chunk_count, case) in &cases {
        group.throughput(Throughput::Elements(1));
        let mut query_index = 0_usize;
        group.bench_with_input(BenchmarkId::from_parameter(chunk_count), case, |b, case| {
            b.iter(|| {
                let hash = &case.hashes[query_index % case.hashes.len()];
                query_index += 1;
                black_box(case.index.get_chunk(black_box(hash)).unwrap())
            })
        });
    }

    group.finish();
}

criterion_group!(benches, pool_index_get_chunk_read_benchmark);
criterion_main!(benches);
