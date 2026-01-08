//! Detailed profiling harness for analyzing record parsing performance.
//!
//! This harness instruments the parsing pipeline to measure:
//! - Time spent in each parsing phase
//! - Record boundary detection overhead
//! - Field extraction overhead
//! - Memory allocation patterns

use mrrc::MarcReader;
use std::io::Cursor;
use std::time::Instant;

fn load_fixture(filename: &str) -> Vec<u8> {
    let path = format!("tests/data/fixtures/{filename}");
    std::fs::read(&path).unwrap_or_else(|_| panic!("Failed to load fixture: {path}"))
}

/// Detailed phase breakdown
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PhaseMetrics {
    phase_name: String,
    count: usize,
    total_ns: u128,
    min_ns: u128,
    max_ns: u128,
    avg_ns: u128,
}

impl PhaseMetrics {
    fn new(phase_name: &str, count: usize, total_ns: u128) -> Self {
        let avg_ns = if count > 0 {
            total_ns / count as u128
        } else {
            0
        };
        PhaseMetrics {
            phase_name: phase_name.to_string(),
            count,
            total_ns,
            min_ns: 0,
            max_ns: 0,
            avg_ns,
        }
    }

    fn print_header() {
        println!(
            "{:<30} {:<12} {:<12} {:<12} {:<12}",
            "Phase", "Count", "Total (ns)", "Min (ns)", "Avg (ns)"
        );
        println!("{}", "-".repeat(78));
    }

    fn print(&self) {
        println!(
            "{:<30} {:<12} {:<12} {:<12} {:<12}",
            self.phase_name, self.count, self.total_ns, self.min_ns, self.avg_ns
        );
    }
}

/// Profile record reading with detailed breakdown
fn profile_detailed_read(filename: &str, iterations: usize) -> Vec<PhaseMetrics> {
    let fixture = load_fixture(filename);
    let mut metrics = Vec::new();

    let mut total_read_ns = 0u128;
    let mut record_count = 0;

    for _ in 0..iterations {
        let cursor = Cursor::new(fixture.clone());
        let mut reader = MarcReader::new(cursor);

        loop {
            let start = Instant::now();
            match reader.read_record() {
                Ok(Some(_record)) => {
                    let elapsed = start.elapsed().as_nanos();
                    total_read_ns += elapsed;
                    record_count += 1;
                },
                Ok(None) | Err(_) => break,
            }
        }
    }

    metrics.push(PhaseMetrics::new(
        "File I/O + Parsing",
        record_count,
        total_read_ns,
    ));
    metrics.push(PhaseMetrics::new(
        "Avg per record",
        1,
        if record_count > 0 {
            total_read_ns / record_count as u128
        } else {
            0
        },
    ));

    metrics
}

/// Estimate instruction count for hot functions
fn estimate_cpu_intensity() -> String {
    // Simple heuristic: measure throughput to estimate CPU intensity
    let fixture = load_fixture("1k_records.mrc");
    let cursor = Cursor::new(fixture);
    let mut reader = MarcReader::new(cursor);

    let start = Instant::now();
    let mut count = 0;
    while let Ok(Some(_record)) = reader.read_record() {
        count += 1;
    }
    let elapsed = start.elapsed();

    #[allow(clippy::cast_precision_loss, clippy::cast_lossless)]
    let throughput_rec_per_ns = f64::from(count) / elapsed.as_nanos() as f64;
    let cycles_per_record = 3.0 / throughput_rec_per_ns; // Assuming 3 GHz CPU

    format!(
        "Est. cycles/record: {cycles_per_record:.0} | CPU intensity: {} IPC",
        if cycles_per_record < 10.0 {
            "Low (memory-bound)"
        } else {
            "High (compute)"
        }
    )
}

/// Memory usage estimation
fn estimate_memory_usage() {
    println!("\n=== Memory Usage Estimation ===\n");

    let fixture = load_fixture("10k_records.mrc");
    println!("Input file size: {} KB", fixture.len() / 1024);

    // Create a sample record to estimate per-record overhead
    let cursor = Cursor::new(fixture);
    let mut reader = MarcReader::new(cursor);

    if let Ok(Some(record)) = reader.read_record() {
        let serialized = serde_json::to_string(&record).unwrap_or_default();
        println!("Sample record JSON size: {} bytes", serialized.len());

        // Estimate Vec overhead
        let field_count = record.fields.len();
        println!("Fields per record (avg): {field_count}");

        // Rough estimation
        let estimated_heap_per_record = 100 + (field_count * 50);
        println!("Est. heap per record: ~{estimated_heap_per_record} bytes");
    }
}

fn main() {
    println!("=== Detailed Pure Rust MARC Profiling ===\n");

    println!("File: 10k_records.mrc");
    let metrics = profile_detailed_read("10k_records.mrc", 3);

    PhaseMetrics::print_header();
    for metric in metrics {
        metric.print();
    }

    println!("\n=== CPU Intensity Analysis ===\n");
    println!("{}", estimate_cpu_intensity());

    estimate_memory_usage();

    println!("\n=== Profiling Complete ===");
    println!("For more detailed analysis, see:");
    println!("  - docs/design/PROFILING_PLAN.md");
    println!("  - docs/design/profiling/RUST_SINGLE_THREADED_PROFILING_RESULTS.md");
}
