//! Custom profiling harness for detailed performance analysis.
//!
//! This harness provides:
//! 1. Detailed timing breakdown for different phases
//! 2. Memory allocation tracking
//! 3. Records-per-second measurements
//! 4. Cache-friendly output for analysis

use mrrc::MarcReader;
use std::io::Cursor;
use std::time::Instant;

fn load_fixture(filename: &str) -> Vec<u8> {
    let path = format!("tests/data/fixtures/{filename}");
    std::fs::read(&path).unwrap_or_else(|_| panic!("Failed to load fixture: {path}"))
}

/// Detailed breakdown of a single read pass
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ProfileResult {
    filename: String,
    total_time_ms: f64,
    record_count: usize,
    records_per_sec: f64,
    avg_time_per_record_us: f64,
    bytes_total: usize,
}

impl ProfileResult {
    fn print_header() {
        println!(
            "{:<30} {:<15} {:<15} {:<15} {:<15}",
            "File", "Records", "Time (ms)", "Rec/sec", "µs/rec"
        );
        println!("{}", "-".repeat(90));
    }

    fn print(&self) {
        println!(
            "{:<30} {:<15} {:<15.2} {:<15.0} {:<15.2}",
            self.filename,
            self.record_count,
            self.total_time_ms,
            self.records_per_sec,
            self.avg_time_per_record_us
        );
    }
}

/// Profile a single file read
fn profile_file(filename: &str, repetitions: usize) -> ProfileResult {
    let fixture = load_fixture(filename);
    let bytes_total = fixture.len();

    let mut total_duration = std::time::Duration::ZERO;
    let mut total_records = 0;

    for _ in 0..repetitions {
        let cursor = Cursor::new(fixture.clone());
        let mut reader = MarcReader::new(cursor);

        let start = Instant::now();
        let mut count = 0;
        while let Ok(Some(_record)) = reader.read_record() {
            count += 1;
        }
        let duration = start.elapsed();

        total_duration += duration;
        total_records += count;
    }

    let total_ms = total_duration.as_secs_f64() * 1000.0;
    #[allow(clippy::cast_precision_loss)]
    let total_records_f64 = total_records as f64;
    let records_per_sec = (total_records_f64 / total_duration.as_secs_f64()).round();
    #[allow(clippy::cast_precision_loss)]
    let avg_us = (total_duration.as_micros() as f64) / total_records_f64;

    ProfileResult {
        filename: filename.to_string(),
        total_time_ms: total_ms,
        record_count: total_records,
        records_per_sec,
        avg_time_per_record_us: avg_us,
        bytes_total,
    }
}

/// Warmup run to stabilize CPU frequency
fn warmup() {
    println!("Running warmup...");
    let fixture = load_fixture("1k_records.mrc");
    let cursor = Cursor::new(fixture);
    let mut reader = MarcReader::new(cursor);
    while let Ok(Some(_record)) = reader.read_record() {}
}

fn main() {
    println!("=== Pure Rust (mrrc) Single-Threaded Profiling ===\n");

    warmup();
    std::thread::sleep(std::time::Duration::from_millis(100));

    ProfileResult::print_header();

    // Profile different file sizes
    let results = vec![
        profile_file("1k_records.mrc", 10),
        profile_file("10k_records.mrc", 10),
        // profile_file("100k_records.mrc", 3),  // Uncomment for full profile
    ];

    for result in &results {
        result.print();
    }

    println!("\n=== Summary ===");
    println!(
        "Total records processed: {}",
        results.iter().map(|r| r.record_count).sum::<usize>()
    );

    #[allow(clippy::cast_precision_loss)]
    let avg_rps = results.iter().map(|r| r.records_per_sec).sum::<f64>() / results.len() as f64;
    println!("Average throughput: {avg_rps:.0} rec/sec");

    #[allow(clippy::cast_precision_loss)]
    let avg_us = results
        .iter()
        .map(|r| r.avg_time_per_record_us)
        .sum::<f64>()
        / results.len() as f64;
    println!("Average time per record: {avg_us:.2} µs");
}
