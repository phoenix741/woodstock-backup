use std::env;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use criterion::{BenchmarkId, Criterion, Throughput};
use eyre::{ensure, Result, WrapErr};
use tempfile::tempdir;
use woodstock::pool::{IndexedChunk, PoolIndex};
use woodstock::utils::compression::CompressionFormat;

const DEFAULT_PROCESS_COUNT: u64 = 2;
const DEFAULT_BENCHMARK_SIZES: [u64; 3] = [1_000, 5_000, 10_000];
const DEFAULT_SAMPLE_SIZE: usize = 10;

#[derive(Debug, Clone)]
struct WorkloadResult {
    chunks_per_process: u64,
    total_chunks: u64,
    elapsed: Duration,
}

impl WorkloadResult {
    fn chunks_per_second(&self) -> f64 {
        self.total_chunks as f64 / self.elapsed.as_secs_f64()
    }
}

#[derive(Debug)]
struct ChildArgs {
    index_path: PathBuf,
    process_id: u64,
    chunks_per_process: u64,
}

fn build_chunk(process_id: u64, chunk_index: u64) -> IndexedChunk {
    let unique = process_id
        .checked_mul(10_000_000)
        .and_then(|base| base.checked_add(chunk_index))
        .expect("chunk identifier overflow");
    let mut hash = vec![0_u8; 32];
    hash[..8].copy_from_slice(&process_id.to_be_bytes());
    hash[8..16].copy_from_slice(&chunk_index.to_be_bytes());
    hash[16..24].copy_from_slice(&unique.to_be_bytes());
    hash[24..32].copy_from_slice(&(unique ^ 0x5A5A_5A5A_5A5A_5A5A).to_be_bytes());

    IndexedChunk {
        hash,
        size: 4096,
        compressed_size: 1024,
        compression_format: CompressionFormat::Zstd,
        ref_count: 1,
        segment_id: process_id + 1,
        offset: chunk_index.saturating_mul(128),
        chunk_header_size: 24,
    }
}

fn configured_process_count() -> u64 {
    env::var("WOODSTOCK_POOL_INDEX_BENCH_PROCESS_COUNT")
        .ok()
        .map(|value| {
            value.parse::<u64>().unwrap_or_else(|error| {
                panic!("invalid WOODSTOCK_POOL_INDEX_BENCH_PROCESS_COUNT value {value}: {error}")
            })
        })
        .unwrap_or(DEFAULT_PROCESS_COUNT)
}

fn configured_sizes() -> Vec<u64> {
    let Some(value) = env::var_os("WOODSTOCK_POOL_INDEX_MULTIPROCESS_BENCH_SIZES") else {
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
                    panic!("invalid WOODSTOCK_POOL_INDEX_MULTIPROCESS_BENCH_SIZES entry {trimmed}: {error}")
                }))
            }
        })
        .collect::<Vec<_>>();

    assert!(
        !parsed.is_empty(),
        "WOODSTOCK_POOL_INDEX_MULTIPROCESS_BENCH_SIZES must contain at least one positive integer"
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

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs_f64();
    if total_seconds >= 60.0 {
        format!("{:.2} min", total_seconds / 60.0)
    } else {
        format!("{total_seconds:.2} s")
    }
}

fn parse_child_args() -> Option<ChildArgs> {
    let mut args = env::args_os();
    let _program = args.next();
    let mode = args.next()?;
    if mode != "--woodstock-pool-index-child" {
        return None;
    }

    let index_path = PathBuf::from(args.next().expect("missing child index path"));
    let process_id = args
        .next()
        .expect("missing child process id")
        .to_string_lossy()
        .parse::<u64>()
        .expect("invalid child process id");
    let chunks_per_process = args
        .next()
        .expect("missing child chunk count")
        .to_string_lossy()
        .parse::<u64>()
        .expect("invalid child chunk count");

    Some(ChildArgs {
        index_path,
        process_id,
        chunks_per_process,
    })
}

fn run_child_process(args: ChildArgs) -> Result<()> {
    let index = PoolIndex::open_or_create(&args.index_path)?;
    for chunk_index in 0..args.chunks_per_process {
        let chunk = build_chunk(args.process_id, chunk_index);
        index.add_chunk(&chunk)?;
    }
    Ok(())
}

fn initialize_index(path: &Path) -> Result<()> {
    let index = PoolIndex::open_or_create(path)?;
    drop(index);
    Ok(())
}

fn run_multiprocess_workload(
    process_count: u64,
    chunks_per_process: u64,
) -> Result<WorkloadResult> {
    let directory = tempdir()?;
    let index_path = directory.path().join("index");
    initialize_index(&index_path)?;

    let executable = env::current_exe().wrap_err("failed to locate benchmark executable")?;
    let start = Instant::now();
    let mut children = Vec::new();

    for process_id in 0..process_count {
        let mut command = Command::new(&executable);
        command
            .arg("--woodstock-pool-index-child")
            .arg(&index_path)
            .arg(process_id.to_string())
            .arg(chunks_per_process.to_string());
        children.push((
            process_id,
            command
                .spawn()
                .wrap_err("failed to spawn child benchmark process")?,
        ));
    }

    for (process_id, mut child) in children {
        let status = child
            .wait()
            .wrap_err("failed to wait for child benchmark process")?;
        ensure!(
            status.success(),
            "child benchmark process {process_id} failed with status {status}"
        );
    }

    let elapsed = start.elapsed();
    let index = PoolIndex::open_or_create(&index_path)?;
    let total_chunks = process_count * chunks_per_process;
    ensure!(
        index.list_chunks()?.len() == total_chunks as usize,
        "unexpected chunk count after multiprocess benchmark"
    );

    Ok(WorkloadResult {
        chunks_per_process,
        total_chunks,
        elapsed,
    })
}

fn report_projections(process_count: u64, benchmark_sizes: &[u64]) {
    println!(
        "\n=== pool_index_multiprocess projections: {process_count} independent processes on one heed env ==="
    );

    for &chunks_per_process in benchmark_sizes {
        let result = run_multiprocess_workload(process_count, chunks_per_process)
            .unwrap_or_else(|error| panic!("failed multiprocess projection: {error:?}"));
        println!(
            "scenario={} chunks/process total_chunks={} elapsed={} throughput={:.2} chunks/s",
            result.chunks_per_process,
            result.total_chunks,
            format_duration(result.elapsed),
            result.chunks_per_second(),
        );
    }

    println!();
}

fn benchmark_pool_index_multiprocess(c: &mut Criterion) {
    let process_count = configured_process_count();
    let benchmark_sizes = configured_sizes();
    let sample_size = configured_sample_size();
    report_projections(process_count, &benchmark_sizes);

    let mut group = c.benchmark_group("pool_index_multiprocess_writers");
    group.sample_size(sample_size);

    for chunks_per_process in benchmark_sizes {
        let total_chunks = process_count * chunks_per_process;
        group.throughput(Throughput::Elements(total_chunks));
        group.bench_with_input(
            BenchmarkId::new("chunks_per_process", chunks_per_process),
            &chunks_per_process,
            |b, &chunks_per_process| {
                b.iter_custom(|iterations| {
                    let start = Instant::now();
                    for _ in 0..iterations {
                        let result = run_multiprocess_workload(process_count, chunks_per_process)
                            .unwrap_or_else(|error| {
                                panic!("failed multiprocess benchmark run: {error:?}")
                            });
                        black_box(result);
                    }
                    start.elapsed()
                });
            },
        );
    }

    group.finish();
}

fn main() {
    if let Some(args) = parse_child_args() {
        run_child_process(args)
            .unwrap_or_else(|error| panic!("child benchmark process failed: {error:?}"));
        return;
    }

    let mut criterion = Criterion::default().configure_from_args();
    benchmark_pool_index_multiprocess(&mut criterion);
    criterion.final_summary();
}
