//! Rayon concurrency profiling with actual file I/O.
//!
//! This harness profiles rayon parallel iteration using actual file reads
//! instead of in-memory buffers, which better reflects real-world performance.
//!
//! This is critical for accurate comparison with Python's `ProducerConsumerPipeline`,
//! which reads from actual files and may benefit from I/O blocking between threads.

#![allow(clippy::cast_precision_loss)]

use mrrc::MarcReader;
use rayon::prelude::*;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::time::Instant;

fn load_fixture(filename: &str) -> Vec<u8> {
    let path = format!("tests/data/fixtures/{filename}");
    std::fs::read(&path).unwrap_or_else(|_| panic!("Failed to load fixture: {path}"))
}

/// Create temporary test files for profiling with real file I/O
fn create_test_files(count: usize, fixture_name: &str, temp_dir: &str) -> Vec<String> {
    fs::create_dir_all(temp_dir).ok();

    let fixture_data = load_fixture(fixture_name);
    let mut file_paths = Vec::new();

    for i in 0..count {
        let path = format!("{temp_dir}/test_file_{i}.mrc");
        let mut file = File::create(&path).expect("Failed to create temp file");
        file.write_all(&fixture_data)
            .expect("Failed to write temp file");
        file_paths.push(path);
    }

    file_paths
}

/// Profile sequential file reading
fn profile_sequential_files(file_paths: &[String]) -> (u128, usize) {
    let start = Instant::now();
    let mut total_count = 0;

    for path in file_paths {
        let file = File::open(path).expect("Failed to open file");
        let reader = BufReader::new(file);
        let mut marc_reader = MarcReader::new(reader);

        while let Ok(Some(_record)) = marc_reader.read_record() {
            total_count += 1;
        }
    }

    let elapsed = start.elapsed().as_nanos();
    (elapsed, total_count)
}

/// Profile rayon parallel file reading
fn profile_rayon_files(file_paths: &[String]) -> (u128, usize) {
    let start = Instant::now();

    let count: usize = file_paths
        .par_iter()
        .map(|path| {
            let file = File::open(path).expect("Failed to open file");
            let reader = BufReader::new(file);
            let mut marc_reader = MarcReader::new(reader);
            let mut count = 0;

            while let Ok(Some(_record)) = marc_reader.read_record() {
                count += 1;
            }
            count
        })
        .sum();

    let elapsed = start.elapsed().as_nanos();
    (elapsed, count)
}

/// Profile rayon with chunk-based iteration for large file batches
fn profile_rayon_files_chunked(file_paths: &[String], chunk_size: usize) -> (u128, usize) {
    let start = Instant::now();

    let count: usize = file_paths
        .par_chunks(chunk_size)
        .map(|chunks| {
            chunks
                .iter()
                .map(|path| {
                    let file = File::open(path).expect("Failed to open file");
                    let reader = BufReader::new(file);
                    let mut marc_reader = MarcReader::new(reader);
                    let mut count = 0;

                    while let Ok(Some(_record)) = marc_reader.read_record() {
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
    println!("=== Rayon Concurrency Profiling with File I/O ===\n");

    let temp_dir = "/tmp/mrrc_profile_test";

    // Test configurations with file I/O
    let configs = vec![
        ("4x 1k files", 4, "1k_records.mrc"),
        ("4x 10k files", 4, "10k_records.mrc"),
        ("8x 1k files", 8, "1k_records.mrc"),
        ("2x 10k files", 2, "10k_records.mrc"),
    ];

    println!(
        "{:<20} {:<15} {:<15} {:<10}",
        "Config", "Sequential", "Rayon", "Speedup"
    );
    println!("{}", "-".repeat(60));

    for (name, count, fixture) in &configs {
        // Create test files
        let file_paths = create_test_files(*count, fixture, temp_dir);

        // Profile sequential
        let (seq_time, seq_count) = profile_sequential_files(&file_paths);

        // Profile rayon
        let (par_time, par_count) = profile_rayon_files(&file_paths);

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

        // Cleanup
        for path in file_paths {
            let _ = std::fs::remove_file(path);
        }
    }

    println!("\n=== Rayon Chunk Size Analysis (4x 10k files) ===\n");
    println!(
        "{:<20} {:<15} {:<15} {:<10}",
        "Chunk Size", "Time (ms)", "Records", "Speedup"
    );
    println!("{}", "-".repeat(60));

    let file_paths = create_test_files(4, "10k_records.mrc", temp_dir);
    let (seq_time, _) = profile_sequential_files(&file_paths);

    for chunk_size in &[1, 2, 4, 8] {
        let (par_time, count) = profile_rayon_files_chunked(&file_paths, *chunk_size);
        let speedup = (seq_time as f64) / (par_time as f64);

        println!(
            "{:<20} {:<15.2} {:<15} {:<10.2}x",
            chunk_size,
            par_time as f64 / 1_000_000.0,
            count,
            speedup
        );
    }

    // Cleanup test directory
    let _ = fs::remove_dir_all(temp_dir);

    println!("\n=== File I/O Profiling Complete ===");
    println!("\nNote: These results use actual file I/O, more realistic than in-memory buffers");
    println!("Expected speedup should be closer to Python baseline (3.74x) due to:");
    println!("  - I/O blocking between threads");
    println!("  - Better interleaving of reading and parsing");
    println!("  - More realistic memory access patterns");
}
