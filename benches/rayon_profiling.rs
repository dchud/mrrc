//! Rayon concurrency profiling harness.
//!
//! This harness isolates the rayon parallel iteration performance
//! to identify bottlenecks and compare with sequential baseline.

#![allow(clippy::cast_precision_loss)]

use mrrc::MarcReader;
use rayon::prelude::*;
use std::io::Cursor;
use std::time::Instant;

fn load_fixture(filename: &str) -> Vec<u8> {
    let path = format!("tests/data/fixtures/{filename}");
    std::fs::read(&path).unwrap_or_else(|_| panic!("Failed to load fixture: {path}"))
}

/// Profile sequential baseline
fn profile_sequential(data: &[Vec<u8>]) -> (u128, usize) {
    let start = Instant::now();
    let mut total_count = 0;

    for chunk in data {
        let cursor = Cursor::new(chunk.clone());
        let mut reader = MarcReader::new(cursor);
        while let Ok(Some(_record)) = reader.read_record() {
            total_count += 1;
        }
    }

    let elapsed = start.elapsed().as_nanos();
    (elapsed, total_count)
}

/// Profile rayon parallel iteration
fn profile_rayon(data: &[Vec<u8>]) -> (u128, usize) {
    let start = Instant::now();

    let count: usize = data
        .par_iter()
        .map(|chunk| {
            let cursor = Cursor::new(chunk.clone());
            let mut reader = MarcReader::new(cursor);
            let mut count = 0;
            while let Ok(Some(_record)) = reader.read_record() {
                count += 1;
            }
            count
        })
        .sum();

    let elapsed = start.elapsed().as_nanos();
    (elapsed, count)
}

/// Profile rayon with work stealing analysis
fn profile_rayon_chunked(data: &[Vec<u8>], chunk_size: usize) -> (u128, usize) {
    let start = Instant::now();

    let count: usize = data
        .par_chunks(chunk_size)
        .map(|chunks| {
            chunks
                .iter()
                .map(|chunk| {
                    let cursor = Cursor::new(chunk.clone());
                    let mut reader = MarcReader::new(cursor);
                    let mut count = 0;
                    while let Ok(Some(_record)) = reader.read_record() {
                        count += 1;
                    }
                    count
                })
                .sum::<usize>()
        })
        .sum();

    let elapsed = start.elapsed().as_nanos();
    (elapsed, count)
}

fn main() {
    println!("=== Rayon Concurrency Profiling ===\n");

    // Load test data
    let fixture_1k = load_fixture("1k_records.mrc");
    let fixture_10k = load_fixture("10k_records.mrc");

    // Test configurations
    let configs = vec![
        ("4x 1k files", vec![fixture_1k.clone(); 4]),
        ("4x 10k files", vec![fixture_10k.clone(); 4]),
        ("8x 1k files", vec![fixture_1k.clone(); 8]),
        ("2x 10k files", vec![fixture_10k.clone(); 2]),
    ];

    println!(
        "{:<20} {:<15} {:<15} {:<10}",
        "Config", "Sequential", "Rayon", "Speedup"
    );
    println!("{}", "-".repeat(60));

    for (name, data) in configs {
        let (seq_time, seq_count) = profile_sequential(&data);
        let (par_time, par_count) = profile_rayon(&data);

        let speedup = (seq_time as f64) / (par_time as f64);

        println!(
            "{:<20} {:<15.2} {:<15.2} {:<10.2}x",
            name,
            seq_time as f64 / 1_000_000.0,
            par_time as f64 / 1_000_000.0,
            speedup
        );

        assert_eq!(
            seq_count, par_count,
            "Record count mismatch for {name}: seq={seq_count}, par={par_count}",
        );
    }

    println!("\n=== Rayon Chunk Size Analysis ===\n");
    println!(
        "{:<20} {:<15} {:<15} {:<10}",
        "Chunk Size", "Time (ms)", "Records", "Speedup"
    );
    println!("{}", "-".repeat(60));

    let data = vec![fixture_10k.clone(); 4];
    let (seq_time, _) = profile_sequential(&data);

    for chunk_size in &[1, 2, 4, 8] {
        let (par_time, count) = profile_rayon_chunked(&data, *chunk_size);
        let speedup = (seq_time as f64) / (par_time as f64);

        println!(
            "{:<20} {:<15.2} {:<15} {:<10.2}x",
            chunk_size,
            par_time as f64 / 1_000_000.0,
            count,
            speedup
        );
    }

    println!("\n=== Analysis Complete ===");
    println!("Note: Run with perf or flamegraph for detailed CPU profiling");
    println!("  perf record -g target/release/deps/rayon_profiling");
    println!("  flamegraph -b target/release/deps/rayon_profiling");
}
