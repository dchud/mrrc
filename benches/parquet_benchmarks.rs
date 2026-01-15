#![allow(
    missing_docs,
    clippy::cast_precision_loss,
    clippy::uninlined_format_args,
    clippy::semicolon_if_nothing_returned,
    clippy::too_many_lines
)]
//! Benchmarks for Parquet serialization/deserialization.
//!
//! Measures throughput and file size characteristics compared to ISO 2709.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mrrc::parquet_impl;
use mrrc::MarcReader;
use std::fs;
use std::io::Cursor;
use std::time::Instant;
use tempfile::NamedTempFile;

/// Load MARC records from an MRC file.
fn load_test_records(path: &str) -> Vec<mrrc::Record> {
    let mrc_data = fs::read(path).expect("Failed to read MRC file");
    let mut reader = MarcReader::new(Cursor::new(&mrc_data));
    let mut records = Vec::new();
    loop {
        match reader.read_record() {
            Ok(Some(record)) => records.push(record),
            Ok(None) => break,
            Err(e) => panic!("Failed to read record: {}", e),
        }
    }
    records
}

/// Benchmark Parquet write performance.
fn benchmark_parquet_write(c: &mut Criterion) {
    let records = load_test_records("tests/data/fixtures/10k_records.mrc");

    c.bench_function("parquet_write_10k", |b| {
        b.iter_with_setup(
            || NamedTempFile::new().expect("Failed to create temp file"),
            |temp_file| {
                let path = temp_file.path().to_string_lossy().to_string();
                parquet_impl::serialize_to_parquet(black_box(&records), &path)
                    .expect("Failed to serialize")
            },
        )
    });

    // Calculate throughput
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let path = temp_file.path().to_string_lossy().to_string();

    let start = Instant::now();
    parquet_impl::serialize_to_parquet(&records, &path).expect("Failed to serialize");
    let elapsed = start.elapsed();

    #[allow(clippy::cast_precision_loss)]
    let throughput = records.len() as f64 / elapsed.as_secs_f64();
    println!(
        "Parquet write throughput: {:.0} rec/sec ({} records in {:.2}s)",
        throughput,
        records.len(),
        elapsed.as_secs_f64()
    );

    let parquet_size = fs::metadata(&path).expect("Failed to get file size").len();
    #[allow(clippy::cast_precision_loss)]
    {
        println!(
            "Parquet file size: {} bytes ({:.2} MB)",
            parquet_size,
            parquet_size as f64 / 1_048_576.0
        );
    }
}

/// Benchmark Parquet read performance.
fn benchmark_parquet_read(c: &mut Criterion) {
    let records = load_test_records("tests/data/fixtures/10k_records.mrc");

    // Create a temporary Parquet file and keep it for the benchmark
    let parquet_path = "target/bench_temp.parquet".to_string();
    parquet_impl::serialize_to_parquet(&records, &parquet_path).expect("Failed to serialize");

    c.bench_function("parquet_read_10k", |b| {
        b.iter(|| {
            parquet_impl::deserialize_from_parquet(&parquet_path).expect("Failed to deserialize")
        })
    });

    // Calculate throughput
    let start = Instant::now();
    let restored =
        parquet_impl::deserialize_from_parquet(&parquet_path).expect("Failed to deserialize");
    let elapsed = start.elapsed();

    #[allow(clippy::cast_precision_loss)]
    let throughput = restored.len() as f64 / elapsed.as_secs_f64();
    println!(
        "Parquet read throughput: {:.0} rec/sec ({} records in {:.2}s)",
        throughput,
        restored.len(),
        elapsed.as_secs_f64()
    );

    // Clean up
    let _ = std::fs::remove_file(&parquet_path);
}

/// Benchmark Parquet round-trip performance.
fn benchmark_parquet_roundtrip(c: &mut Criterion) {
    let records = load_test_records("tests/data/fixtures/10k_records.mrc");

    c.bench_function("parquet_roundtrip_10k", |b| {
        b.iter_with_setup(
            || "target/bench_roundtrip.parquet".to_string(),
            |path| {
                parquet_impl::serialize_to_parquet(black_box(&records), &path)
                    .expect("Failed to serialize");
                let result =
                    parquet_impl::deserialize_from_parquet(&path).expect("Failed to deserialize");
                let _ = std::fs::remove_file(&path);
                result
            },
        )
    });
}

/// Benchmark Parquet with smaller record sets (100 records).
fn benchmark_100_records(c: &mut Criterion) {
    let records = load_test_records("tests/data/fixtures/fidelity_test_100.mrc");

    c.bench_function("parquet_write_100", |b| {
        b.iter_with_setup(
            || "target/bench_100_write.parquet".to_string(),
            |path| {
                let result = parquet_impl::serialize_to_parquet(black_box(&records), &path)
                    .expect("Failed to serialize");
                let _ = std::fs::remove_file(&path);
                result
            },
        )
    });

    c.bench_function("parquet_read_100", |b| {
        b.iter_with_setup(
            || {
                let path = "target/bench_100_read.parquet".to_string();
                parquet_impl::serialize_to_parquet(&records, &path).expect("Failed to serialize");
                path
            },
            |path| {
                let result =
                    parquet_impl::deserialize_from_parquet(&path).expect("Failed to deserialize");
                let _ = std::fs::remove_file(&path);
                result
            },
        );
    });
}

criterion_group!(
    benches,
    benchmark_100_records,
    benchmark_parquet_write,
    benchmark_parquet_read,
    benchmark_parquet_roundtrip
);
criterion_main!(benches);
