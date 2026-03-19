use std::env;
use std::hint::black_box;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use tempfile::tempdir;
use woodstock::pool::{IndexedChunk, PoolIndex};
use woodstock::utils::compression::CompressionFormat;

const MACHINE_COUNT: u64 = 10;
const TARGET_TOTAL_CHUNKS: u64 = 4_000_000 * MACHINE_COUNT;
const DEFAULT_BENCHMARK_SIZES: [u64; 3] = [100, 1_000, 5_000];
const DEFAULT_SAMPLE_SIZE: usize = 10;

#[derive(Debug, Clone)]
struct WorkloadResult {
    chunks_per_machine: u64,
    total_chunks: u64,
    elapsed: Duration,
}

impl WorkloadResult {
    fn chunks_per_second(&self) -> f64 {
        self.total_chunks as f64 / self.elapsed.as_secs_f64()
    }

    fn estimated_target_duration(&self) -> Duration {
        Duration::from_secs_f64(TARGET_TOTAL_CHUNKS as f64 / self.chunks_per_second())
    }
}

fn build_chunk(machine_id: u64, chunk_index: u64) -> IndexedChunk {
    let unique = machine_id
        .checked_mul(4_000_000)
        .and_then(|base| base.checked_add(chunk_index))
        .expect("chunk identifier overflow");
    let mut hash = vec![0_u8; 32];
    hash[..8].copy_from_slice(&machine_id.to_be_bytes());
    hash[8..16].copy_from_slice(&chunk_index.to_be_bytes());
    hash[16..24].copy_from_slice(&unique.to_be_bytes());
    hash[24..32].copy_from_slice(&(unique ^ 0xA5A5_A5A5_A5A5_A5A5).to_be_bytes());

    IndexedChunk {
        hash,
        size: 4096,
        compressed_size: 1024,
        compression_format: CompressionFormat::Zstd,
        ref_count: 1,
        segment_id: machine_id + 1,
        offset: chunk_index.saturating_mul(128),
        chunk_header_size: 24,
    }
}

fn run_machine(machine_id: u64, chunks_per_machine: u64, index: Arc<PoolIndex>) {
    for chunk_index in 0..chunks_per_machine {
        let chunk = build_chunk(machine_id, chunk_index);
        index.add_chunk(&chunk).unwrap_or_else(|error| {
            panic!("failed to insert chunk for machine {machine_id}: {error}")
        });
    }
}

fn run_open_create_close_workload(chunks_per_machine: u64) -> WorkloadResult {
    let directory = tempdir().expect("failed to create benchmark tempdir");
    let index_path = directory.path().join("index");
    let index = Arc::new(initialize_index(&index_path));

    let start = Instant::now();
    thread::scope(|scope| {
        for machine_id in 0..MACHINE_COUNT {
            let index = Arc::clone(&index);
            scope.spawn(move || run_machine(machine_id, chunks_per_machine, index));
        }
    });
    let elapsed = start.elapsed();

    WorkloadResult {
        chunks_per_machine,
        total_chunks: chunks_per_machine * MACHINE_COUNT,
        elapsed,
    }
}

fn initialize_index(path: &std::path::Path) -> PoolIndex {
    PoolIndex::open_or_create(path).expect("failed to initialize index")
}

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs_f64();
    if total_seconds >= 3600.0 {
        format!("{:.2} h", total_seconds / 3600.0)
    } else if total_seconds >= 60.0 {
        format!("{:.2} min", total_seconds / 60.0)
    } else {
        format!("{total_seconds:.2} s")
    }
}

fn configured_sizes() -> Vec<u64> {
    let Some(value) = env::var_os("WOODSTOCK_POOL_INDEX_BENCH_SIZES") else {
        return DEFAULT_BENCHMARK_SIZES.to_vec();
    };

    let parsed = value
        .to_string_lossy()
        .split(',')
        .filter_map(|entry| {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.parse::<u64>().unwrap_or_else(|error| {
                    panic!("invalid WOODSTOCK_POOL_INDEX_BENCH_SIZES entry {trimmed}: {error}")
                }))
            }
        })
        .collect::<Vec<_>>();

    assert!(
        !parsed.is_empty(),
        "WOODSTOCK_POOL_INDEX_BENCH_SIZES must contain at least one positive integer"
    );

    parsed
}

fn configured_sample_size() -> usize {
    env::var("WOODSTOCK_POOL_INDEX_SAMPLE_SIZE")
        .ok()
        .map(|value| {
            value.parse::<usize>().unwrap_or_else(|error| {
                panic!("invalid WOODSTOCK_POOL_INDEX_SAMPLE_SIZE value {value}: {error}")
            })
        })
        .unwrap_or(DEFAULT_SAMPLE_SIZE)
}

fn print_projection(result: &WorkloadResult) {
    let throughput = result.chunks_per_second();
    let per_machine_throughput = throughput / MACHINE_COUNT as f64;
    let estimate = result.estimated_target_duration();

    println!(
        "scenario={} chunks/machine total_chunks={} elapsed={} throughput={:.2} chunks/s throughput/machine={:.2} chunks/s estimate_for_40M={}",
        result.chunks_per_machine,
        result.total_chunks,
        format_duration(result.elapsed),
        throughput,
        per_machine_throughput,
        format_duration(estimate),
    );
}

fn report_projections() {
    let benchmark_sizes = configured_sizes();
    println!(
        "\n=== pool_index_open_close projections: {} machines, one shared heed env and serialized writes ===",
        MACHINE_COUNT
    );
    println!("configured_sizes_per_machine={benchmark_sizes:?}");
    let mut slowest = None::<WorkloadResult>;
    let mut fastest = None::<WorkloadResult>;

    for chunks_per_machine in benchmark_sizes {
        let result = run_open_create_close_workload(chunks_per_machine);
        print_projection(&result);

        if slowest
            .as_ref()
            .is_none_or(|candidate| result.chunks_per_second() < candidate.chunks_per_second())
        {
            slowest = Some(result.clone());
        }
        if fastest
            .as_ref()
            .is_none_or(|candidate| result.chunks_per_second() > candidate.chunks_per_second())
        {
            fastest = Some(result);
        }
    }

    if let Some(result) = fastest {
        println!(
            "best_case_estimate_for_40M={} from scenario={} chunks/machine",
            format_duration(result.estimated_target_duration()),
            result.chunks_per_machine,
        );
    }
    if let Some(result) = slowest {
        println!(
            "worst_case_estimate_for_40M={} from scenario={} chunks/machine\n",
            format_duration(result.estimated_target_duration()),
            result.chunks_per_machine,
        );
    }
}

fn benchmark_pool_index_open_close(c: &mut Criterion) {
    let benchmark_sizes = configured_sizes();
    let sample_size = configured_sample_size();
    report_projections();

    let mut group = c.benchmark_group("pool_index_shared_heed_writers");
    group.sample_size(sample_size);

    for chunks_per_machine in benchmark_sizes {
        let total_chunks = chunks_per_machine * MACHINE_COUNT;
        group.throughput(Throughput::Elements(total_chunks));
        group.bench_with_input(
            BenchmarkId::new("open_create_close", chunks_per_machine),
            &chunks_per_machine,
            |b, &chunks_per_machine| {
                b.iter_custom(|iterations| {
                    let start = Instant::now();
                    for _ in 0..iterations {
                        let result = run_open_create_close_workload(chunks_per_machine);
                        black_box(result);
                    }
                    start.elapsed()
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_pool_index_open_close);
criterion_main!(benches);
