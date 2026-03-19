#[path = "support/pool_index_benchmark_support.rs"]
mod support;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;

fn pool_index_add_chunks_write_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("pool_index_write_add_chunks");
    group.sample_size(10);

    for (initial_chunk_count, batch_chunk_count) in [(0_usize, 10_000_usize), (100_000, 10_000)] {
        let batch = support::build_chunks(initial_chunk_count, batch_chunk_count);
        group.throughput(Throughput::Elements(batch_chunk_count as u64));
        group.bench_with_input(
            BenchmarkId::new("initial_chunks", initial_chunk_count),
            &batch,
            |b, batch| {
                b.iter_batched_ref(
                    || {
                        let (tempdir, index, _) =
                            support::create_populated_index(initial_chunk_count).unwrap();
                        (tempdir, index)
                    },
                    |(tempdir, index)| {
                        index.add_chunks(black_box(batch)).unwrap();
                    },
                    BatchSize::LargeInput,
                )
            },
        );
    }

    group.finish();
}

criterion_group!(benches, pool_index_add_chunks_write_benchmark);
criterion_main!(benches);
