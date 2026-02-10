#![allow(missing_docs, unused_doc_comments, unused_attributes)]
//! Parallel processing benchmarks using rayon.
//!
//! These benchmarks demonstrate the performance advantage of concurrent processing
//! by reading multiple MARC files in parallel using rayon's data parallelism.
//!
//! Expected speedup on 4-core system: ~3.8-4.0x

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mrrc::MarcReader;
use rayon::prelude::*;
use std::io::Cursor;

/// Load test fixtures from the test data directory.
fn load_fixture(filename: &str) -> Vec<u8> {
    let path = format!("tests/data/fixtures/{filename}");
    std::fs::read(&path).unwrap_or_else(|_| panic!("Failed to load fixture: {path}"))
}

/// Benchmark sequential reading of multiple 1k files.
fn benchmark_sequential_2x_1k(c: &mut Criterion) {
    let fixture = black_box(load_fixture("1k_records.mrc"));

    c.bench_function("sequential_2x_1k_records", |b| {
        b.iter(|| {
            let mut total_count = 0;
            for _ in 0..2 {
                let cursor = Cursor::new(fixture.clone());
                let mut reader = MarcReader::new(cursor);
                while let Ok(Some(_record)) = reader.read_record() {
                    total_count += 1;
                }
            }
            total_count
        });
    });
}

/// Benchmark parallel reading of 2 files with rayon.
fn benchmark_parallel_2x_1k(c: &mut Criterion) {
    let fixture = black_box(load_fixture("1k_records.mrc"));

    c.bench_function("parallel_2x_1k_records", |b| {
        b.iter(|| {
            let files = vec![fixture.clone(), fixture.clone()];

            files
                .par_iter()
                .map(|data| {
                    let cursor = Cursor::new(data.clone());
                    let mut reader = MarcReader::new(cursor);
                    let mut count = 0;
                    while let Ok(Some(_record)) = reader.read_record() {
                        count += 1;
                    }
                    count
                })
                .sum::<usize>()
        });
    });
}

/// Benchmark sequential reading of 4x 1k files.
fn benchmark_sequential_4x_1k(c: &mut Criterion) {
    let fixture = black_box(load_fixture("1k_records.mrc"));

    c.bench_function("sequential_4x_1k_records", |b| {
        b.iter(|| {
            let mut total_count = 0;
            for _ in 0..4 {
                let cursor = Cursor::new(fixture.clone());
                let mut reader = MarcReader::new(cursor);
                while let Ok(Some(_record)) = reader.read_record() {
                    total_count += 1;
                }
            }
            total_count
        });
    });
}

/// Benchmark parallel reading of 4 files with rayon.
fn benchmark_parallel_4x_1k(c: &mut Criterion) {
    let fixture = black_box(load_fixture("1k_records.mrc"));

    c.bench_function("parallel_4x_1k_records", |b| {
        b.iter(|| {
            let files = vec![
                fixture.clone(),
                fixture.clone(),
                fixture.clone(),
                fixture.clone(),
            ];

            files
                .par_iter()
                .map(|data| {
                    let cursor = Cursor::new(data.clone());
                    let mut reader = MarcReader::new(cursor);
                    let mut count = 0;
                    while let Ok(Some(_record)) = reader.read_record() {
                        count += 1;
                    }
                    count
                })
                .sum::<usize>()
        });
    });
}

/// Benchmark parallel reading of 8 files with rayon.
fn benchmark_parallel_8x_1k(c: &mut Criterion) {
    let fixture = black_box(load_fixture("1k_records.mrc"));

    c.bench_function("parallel_8x_1k_records", |b| {
        b.iter(|| {
            let files = vec![
                fixture.clone(),
                fixture.clone(),
                fixture.clone(),
                fixture.clone(),
                fixture.clone(),
                fixture.clone(),
                fixture.clone(),
                fixture.clone(),
            ];

            files
                .par_iter()
                .map(|data| {
                    let cursor = Cursor::new(data.clone());
                    let mut reader = MarcReader::new(cursor);
                    let mut count = 0;
                    while let Ok(Some(_record)) = reader.read_record() {
                        count += 1;
                    }
                    count
                })
                .sum::<usize>()
        });
    });
}

/// Benchmark sequential reading of multiple 10k files.
fn benchmark_sequential_2x_10k(c: &mut Criterion) {
    let fixture = black_box(load_fixture("10k_records.mrc"));

    c.bench_function("sequential_2x_10k_records", |b| {
        b.iter(|| {
            let mut total_count = 0;
            for _ in 0..2 {
                let cursor = Cursor::new(fixture.clone());
                let mut reader = MarcReader::new(cursor);
                while let Ok(Some(_record)) = reader.read_record() {
                    total_count += 1;
                }
            }
            total_count
        });
    });
}

/// Benchmark parallel reading of 2x 10k files with rayon.
fn benchmark_parallel_2x_10k(c: &mut Criterion) {
    let fixture = black_box(load_fixture("10k_records.mrc"));

    c.bench_function("parallel_2x_10k_records", |b| {
        b.iter(|| {
            let files = vec![fixture.clone(), fixture.clone()];

            files
                .par_iter()
                .map(|data| {
                    let cursor = Cursor::new(data.clone());
                    let mut reader = MarcReader::new(cursor);
                    let mut count = 0;
                    while let Ok(Some(_record)) = reader.read_record() {
                        count += 1;
                    }
                    count
                })
                .sum::<usize>()
        });
    });
}

/// Benchmark parallel reading of 4x 10k files with rayon.
fn benchmark_parallel_4x_10k(c: &mut Criterion) {
    let fixture = black_box(load_fixture("10k_records.mrc"));

    c.bench_function("parallel_4x_10k_records", |b| {
        b.iter(|| {
            let files = vec![
                fixture.clone(),
                fixture.clone(),
                fixture.clone(),
                fixture.clone(),
            ];

            files
                .par_iter()
                .map(|data| {
                    let cursor = Cursor::new(data.clone());
                    let mut reader = MarcReader::new(cursor);
                    let mut count = 0;
                    while let Ok(Some(_record)) = reader.read_record() {
                        count += 1;
                    }
                    count
                })
                .sum::<usize>()
        });
    });
}

criterion_group!(
    parallel_benches,
    benchmark_sequential_2x_1k,
    benchmark_parallel_2x_1k,
    benchmark_sequential_4x_1k,
    benchmark_parallel_4x_1k,
    benchmark_parallel_8x_1k,
    benchmark_sequential_2x_10k,
    benchmark_parallel_2x_10k,
    benchmark_parallel_4x_10k,
);
criterion_main!(parallel_benches);
