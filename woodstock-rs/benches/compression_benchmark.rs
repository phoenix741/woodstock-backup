use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::fs;
use std::io::Cursor;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::runtime::Runtime;

use woodstock::utils::compression::{
    CompressionFormat, WoodstockCompressionReader, WoodstockCompressionWriter,
};

/// Real-world test files for benchmarks
const TEXT_FILE: &str = "../client-rs/src/storage/accessor.rs"; // Rust source code (~42KB)
const IMAGE_FILE: &str = "data/Flux.1_00176_.png"; // PNG image (~1.2MB)
const BINARY_FILE: &str = "../target/release/ws_sync"; // Binary executable (~90MB)

/// Load real test data from files
fn load_test_data() -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let text_data = fs::read(TEXT_FILE).expect("Failed to read text file");
    let image_data = fs::read(IMAGE_FILE).expect("Failed to read image file");

    // For binary file, take only first 10MB to keep benchmarks reasonable
    let binary_data = fs::read(BINARY_FILE)
        .expect("Failed to read binary file")
        .into_iter()
        .take(10 * 1024 * 1024) // 10MB limit
        .collect::<Vec<u8>>();

    (text_data, image_data, binary_data)
}

/// Benchmark Woodstock compression writing
async fn benchmark_woodstock_write(data: &[u8], format: CompressionFormat) -> Vec<u8> {
    let mut output = Vec::new();
    let buf_writer = BufWriter::new(Cursor::new(&mut output));
    let mut writer = WoodstockCompressionWriter::new(buf_writer, format);

    writer.write_all(data).await.unwrap();
    writer.shutdown().await.unwrap();

    output
}

/// Benchmark Woodstock compression reading
async fn benchmark_woodstock_read(compressed_data: &[u8]) -> Vec<u8> {
    let cursor = Cursor::new(compressed_data);
    let mut reader = WoodstockCompressionReader::new(cursor);

    let mut output = Vec::new();
    reader.read_to_end(&mut output).await.unwrap();

    output
}

fn compression_write_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Load real-world test data
    let (text_data, _image_data, _binary_data) = load_test_data();

    let mut group = c.benchmark_group("compression_write");
    group.throughput(Throughput::Bytes(text_data.len() as u64));
    group.sample_size(15); // Reasonable sample size

    // Only test a subset of formats for speed benchmarks
    for &format in &[
        CompressionFormat::None,
        CompressionFormat::Zlib,
        CompressionFormat::Zstd,
        CompressionFormat::Brotli,
    ] {
        group.bench_with_input(
            BenchmarkId::new("text", format!("{}", format)),
            &text_data,
            |b, data| {
                b.iter(|| {
                    rt.block_on(async {
                        black_box(benchmark_woodstock_write(black_box(data), format).await)
                    })
                })
            },
        );
    }

    group.finish();
}

fn compression_read_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Load real-world test data
    let (text_data, image_data, binary_data) = load_test_data();

    // Pre-compress all data with all formats
    let text_compressed: Vec<(CompressionFormat, Vec<u8>)> = vec![
        CompressionFormat::None,
        CompressionFormat::Zlib,
        CompressionFormat::Zstd,
        CompressionFormat::Brotli,
        CompressionFormat::Lzma,
        CompressionFormat::Xz,
    ]
    .into_iter()
    .map(|format| {
        let compressed = rt.block_on(benchmark_woodstock_write(&text_data, format));
        (format, compressed)
    })
    .collect();

    let image_compressed: Vec<(CompressionFormat, Vec<u8>)> = vec![
        CompressionFormat::None,
        CompressionFormat::Zlib,
        CompressionFormat::Zstd,
        CompressionFormat::Brotli,
        CompressionFormat::Lzma,
        CompressionFormat::Xz,
    ]
    .into_iter()
    .map(|format| {
        let compressed = rt.block_on(benchmark_woodstock_write(&image_data, format));
        (format, compressed)
    })
    .collect();

    let binary_compressed: Vec<(CompressionFormat, Vec<u8>)> = vec![
        CompressionFormat::None,
        CompressionFormat::Zlib,
        CompressionFormat::Zstd,
        CompressionFormat::Brotli,
        CompressionFormat::Lzma,
        CompressionFormat::Xz,
    ]
    .into_iter()
    .map(|format| {
        let compressed = rt.block_on(benchmark_woodstock_write(&binary_data, format));
        (format, compressed)
    })
    .collect();

    let mut group = c.benchmark_group("compression_read_real_data");

    // Text file decompression benchmarks
    group.throughput(Throughput::Bytes(text_data.len() as u64));

    for (format, compressed_data) in &text_compressed {
        group.bench_with_input(
            BenchmarkId::new(format!("text_{:?}", format).to_lowercase(), "42KB"),
            compressed_data,
            |b, data| {
                b.iter(|| {
                    rt.block_on(async {
                        black_box(benchmark_woodstock_read(black_box(data)).await)
                    })
                })
            },
        );
    }

    // Image file decompression benchmarks
    group.throughput(Throughput::Bytes(image_data.len() as u64));

    for (format, compressed_data) in &image_compressed {
        group.bench_with_input(
            BenchmarkId::new(format!("image_{:?}", format).to_lowercase(), "1.2MB"),
            compressed_data,
            |b, data| {
                b.iter(|| {
                    rt.block_on(async {
                        black_box(benchmark_woodstock_read(black_box(data)).await)
                    })
                })
            },
        );
    }

    // Binary file decompression benchmarks
    group.throughput(Throughput::Bytes(binary_data.len() as u64));

    for (format, compressed_data) in &binary_compressed {
        group.bench_with_input(
            BenchmarkId::new(format!("binary_{:?}", format).to_lowercase(), "10MB"),
            compressed_data,
            |b, data| {
                b.iter(|| {
                    rt.block_on(async {
                        black_box(benchmark_woodstock_read(black_box(data)).await)
                    })
                })
            },
        );
    }

    group.finish();
}

fn compression_ratio_and_speed_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Load real-world test data
    let (text_data, image_data, binary_data) = load_test_data();

    // Test compression ratios first
    println!("\n=== COMPRESSION RATIOS - REAL WORLD DATA ===");

    for (name, data) in &[
        ("Rust Source Code", &text_data),
        ("PNG Image", &image_data),
        ("Binary Executable", &binary_data),
    ] {
        println!("\n{} file (original size: {} bytes):", name, data.len());

        let none_compressed = rt.block_on(benchmark_woodstock_write(data, CompressionFormat::None));
        let zlib_compressed = rt.block_on(benchmark_woodstock_write(data, CompressionFormat::Zlib));
        let zstd_compressed = rt.block_on(benchmark_woodstock_write(data, CompressionFormat::Zstd));
        let brotli_compressed =
            rt.block_on(benchmark_woodstock_write(data, CompressionFormat::Brotli));
        let lzma_compressed = rt.block_on(benchmark_woodstock_write(data, CompressionFormat::Lzma));
        let xz_compressed = rt.block_on(benchmark_woodstock_write(data, CompressionFormat::Xz));

        println!(
            "  Woodstock None:       {} bytes ({:.1}%)",
            none_compressed.len(),
            (none_compressed.len() as f64 / data.len() as f64) * 100.0
        );
        println!(
            "  Woodstock Zlib:       {} bytes ({:.1}%)",
            zlib_compressed.len(),
            (zlib_compressed.len() as f64 / data.len() as f64) * 100.0
        );
        println!(
            "  Woodstock Zstd:       {} bytes ({:.1}%)",
            zstd_compressed.len(),
            (zstd_compressed.len() as f64 / data.len() as f64) * 100.0
        );
        println!(
            "  Woodstock Brotli:     {} bytes ({:.1}%)",
            brotli_compressed.len(),
            (brotli_compressed.len() as f64 / data.len() as f64) * 100.0
        );
        println!(
            "  Woodstock LZMA:       {} bytes ({:.1}%)",
            lzma_compressed.len(),
            (lzma_compressed.len() as f64 / data.len() as f64) * 100.0
        );
        println!(
            "  Woodstock XZ:         {} bytes ({:.1}%)",
            xz_compressed.len(),
            (xz_compressed.len() as f64 / data.len() as f64) * 100.0
        );
    }

    // Now benchmark compression speed on text data (smaller dataset for reasonable benchmark time)
    let mut group = c.benchmark_group("compression_speed_text");
    group.throughput(Throughput::Bytes(text_data.len() as u64));
    group.sample_size(10); // Reduce sample size for faster benchmarks

    for &format in &[
        CompressionFormat::None,
        CompressionFormat::Zlib,
        CompressionFormat::Zstd,
        CompressionFormat::Brotli,
        CompressionFormat::Lzma,
        CompressionFormat::Xz,
    ] {
        group.bench_with_input(
            BenchmarkId::new("compress", format!("{}", format)),
            &text_data,
            |b, data| {
                b.iter(|| {
                    rt.block_on(async {
                        black_box(benchmark_woodstock_write(black_box(data), format).await)
                    })
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    compression_benches,
    compression_write_benchmark,
    compression_read_benchmark,
    compression_ratio_and_speed_benchmark
);
criterion_main!(compression_benches);
