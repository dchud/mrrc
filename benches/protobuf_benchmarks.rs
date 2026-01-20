#![allow(
    missing_docs,
    clippy::cast_precision_loss,
    clippy::implicit_clone,
    clippy::uninlined_format_args,
    clippy::cast_possible_truncation,
    clippy::cast_lossless
)]
//! Protobuf benchmarks for MARC record serialization.
//!
//! Benchmarks read/write throughput and memory usage for the protobuf format.
//! Target: 500k records/second for both read and write operations.

use mrrc::formats::protobuf::{ProtobufReader, ProtobufWriter};
use mrrc::MarcReader;
use std::fs;
use std::io::Cursor;
use std::time::Instant;

const FIDELITY_TEST_PATH: &str = "tests/data/fixtures/fidelity_test_100.mrc";
const PERF_TEST_10K_PATH: &str = "tests/data/fixtures/10k_records.mrc";
const PERF_TEST_100K_PATH: &str = "tests/data/fixtures/100k_records.mrc";

// ============================================================================
// ROUND-TRIP FIDELITY TEST
// ============================================================================

fn test_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(FIDELITY_TEST_PATH)?;
    let cursor = Cursor::new(&data);
    let mut reader = MarcReader::new(cursor);
    let mut records: Vec<mrrc::Record> = Vec::new();

    while let Some(record) = reader.read_record()? {
        records.push(record);
    }

    let mut perfect_count = 0;
    let mut failures = Vec::new();

    for (idx, original) in records.iter().enumerate() {
        // Serialize to protobuf using streaming writer
        let mut serialized = Vec::new();
        {
            let mut writer = ProtobufWriter::new(&mut serialized);
            writer.write_record(original)?;
            writer.finish()?;
        }

        // Deserialize from protobuf using streaming reader
        let cursor = Cursor::new(&serialized);
        let mut reader = ProtobufReader::new(cursor);
        let recovered = reader.read_record()?.expect("Should have one record");

        // Compare field by field
        let orig_leader_bytes = original.leader.as_bytes().unwrap_or_default();
        let rec_leader_bytes = recovered.leader.as_bytes().unwrap_or_default();
        let match_leader = orig_leader_bytes == rec_leader_bytes;

        let orig_control: Vec<_> = original.control_fields_iter().collect();
        let rec_control: Vec<_> = recovered.control_fields_iter().collect();
        let match_control = orig_control == rec_control;

        let orig_fields: Vec<_> = original.fields().collect();
        let rec_fields: Vec<_> = recovered.fields().collect();
        let match_field_count = orig_fields.len() == rec_fields.len();
        let match_fields = orig_fields.iter().zip(rec_fields.iter()).all(|(a, b)| {
            a.tag == b.tag
                && a.indicator1 == b.indicator1
                && a.indicator2 == b.indicator2
                && a.subfields == b.subfields
        });

        if match_leader && match_control && match_field_count && match_fields {
            perfect_count += 1;
        } else {
            failures.push(idx);
        }
    }

    let fidelity = (perfect_count as f64 / records.len() as f64) * 100.0;

    println!("### Round-Trip Fidelity Results\n");
    println!("**Test Set:** fidelity_test_100.mrc");
    println!("**Records Tested:** {}", records.len());
    println!(
        "**Perfect Round-Trips:** {}/{} ({:.1}%)",
        perfect_count,
        records.len(),
        fidelity
    );

    if failures.is_empty() {
        println!("**Status:** PASS - All records match perfectly\n");
    } else {
        println!("**Status:** FAIL");
        println!("Record IDs with failures: {:?}\n", failures);
    }

    Ok(())
}

// ============================================================================
// PERFORMANCE BENCHMARKS
// ============================================================================

fn benchmark_write_throughput(records: &[mrrc::Record]) -> (f64, usize) {
    let start = Instant::now();
    let mut serialized = Vec::new();
    {
        let mut writer = ProtobufWriter::new(&mut serialized);
        for record in records {
            writer.write_record(record).unwrap();
        }
        writer.finish().unwrap();
    }
    let duration = start.elapsed();
    let throughput = records.len() as f64 / duration.as_secs_f64();
    (throughput, serialized.len())
}

fn benchmark_read_throughput(serialized: &[u8], expected_count: usize) -> f64 {
    let start = Instant::now();
    let cursor = Cursor::new(serialized);
    let mut reader = ProtobufReader::new(cursor);
    let mut count = 0;
    while let Ok(Some(_)) = reader.read_record() {
        count += 1;
    }
    let duration = start.elapsed();
    assert_eq!(count, expected_count, "Record count mismatch");
    count as f64 / duration.as_secs_f64()
}

fn benchmark_10k() -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(PERF_TEST_10K_PATH)?;
    let cursor = Cursor::new(&data);
    let mut reader = MarcReader::new(cursor);
    let mut records: Vec<mrrc::Record> = Vec::new();

    while let Some(record) = reader.read_record()? {
        records.push(record);
    }

    println!("### 10k Records Benchmark\n");
    println!("**Records:** {}", records.len());

    // Warm-up run
    let mut warmup_buf = Vec::new();
    {
        let mut writer = ProtobufWriter::new(&mut warmup_buf);
        for record in records.iter().take(100) {
            writer.write_record(record)?;
        }
        writer.finish()?;
    }

    // Write benchmark (5 iterations)
    let mut write_throughputs = Vec::new();
    let mut serialized = Vec::new();
    for _ in 0..5 {
        let (throughput, _size) = benchmark_write_throughput(&records);
        write_throughputs.push(throughput);
        if serialized.is_empty() {
            // Serialize once for read benchmarks
            let mut buf = Vec::new();
            {
                let mut writer = ProtobufWriter::new(&mut buf);
                for record in &records {
                    writer.write_record(record)?;
                }
                writer.finish()?;
            }
            serialized = buf;
        }
    }
    let avg_write = write_throughputs.iter().sum::<f64>() / write_throughputs.len() as f64;

    // Read benchmark (5 iterations)
    let mut read_throughputs = Vec::new();
    for _ in 0..5 {
        let throughput = benchmark_read_throughput(&serialized, records.len());
        read_throughputs.push(throughput);
    }
    let avg_read = read_throughputs.iter().sum::<f64>() / read_throughputs.len() as f64;

    // Compression stats
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&serialized)?;
    let gzipped = encoder.finish()?;

    println!("| Metric | Value |");
    println!("|--------|-------|");
    println!("| Write Throughput | {:.0} rec/sec |", avg_write);
    println!("| Read Throughput | {:.0} rec/sec |", avg_read);
    println!("| Serialized Size | {} bytes ({:.2} KB/rec) |",
             serialized.len(),
             serialized.len() as f64 / records.len() as f64 / 1024.0);
    println!("| Gzipped Size | {} bytes ({:.1}% compression) |",
             gzipped.len(),
             (1.0 - gzipped.len() as f64 / serialized.len() as f64) * 100.0);
    println!();

    // Check against target
    let target = 500_000.0;
    if avg_write >= target {
        println!("**Write Target (500k rec/sec):** PASS ({:.0} >= {:.0})", avg_write, target);
    } else {
        println!("**Write Target (500k rec/sec):** Below target ({:.0} < {:.0})", avg_write, target);
    }
    if avg_read >= target {
        println!("**Read Target (500k rec/sec):** PASS ({:.0} >= {:.0})", avg_read, target);
    } else {
        println!("**Read Target (500k rec/sec):** Below target ({:.0} < {:.0})", avg_read, target);
    }
    println!();

    Ok(())
}

fn benchmark_100k() -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(PERF_TEST_100K_PATH)?;
    let cursor = Cursor::new(&data);
    let mut reader = MarcReader::new(cursor);
    let mut records: Vec<mrrc::Record> = Vec::new();

    while let Some(record) = reader.read_record()? {
        records.push(record);
    }

    println!("### 100k Records Benchmark\n");
    println!("**Records:** {}", records.len());

    // Write benchmark (3 iterations for larger dataset)
    let mut write_throughputs = Vec::new();
    let mut serialized = Vec::new();
    for i in 0..3 {
        let (throughput, _) = benchmark_write_throughput(&records);
        write_throughputs.push(throughput);
        if i == 0 {
            // Serialize once for read benchmarks
            let mut buf = Vec::new();
            {
                let mut writer = ProtobufWriter::new(&mut buf);
                for record in &records {
                    writer.write_record(record)?;
                }
                writer.finish()?;
            }
            serialized = buf;
        }
    }
    let avg_write = write_throughputs.iter().sum::<f64>() / write_throughputs.len() as f64;

    // Read benchmark (3 iterations)
    let mut read_throughputs = Vec::new();
    for _ in 0..3 {
        let throughput = benchmark_read_throughput(&serialized, records.len());
        read_throughputs.push(throughput);
    }
    let avg_read = read_throughputs.iter().sum::<f64>() / read_throughputs.len() as f64;

    println!("| Metric | Value |");
    println!("|--------|-------|");
    println!("| Write Throughput | {:.0} rec/sec |", avg_write);
    println!("| Read Throughput | {:.0} rec/sec |", avg_read);
    println!("| Serialized Size | {} bytes ({:.2} MB) |",
             serialized.len(),
             serialized.len() as f64 / 1024.0 / 1024.0);
    println!();

    // Check against target
    let target = 500_000.0;
    if avg_write >= target {
        println!("**Write Target (500k rec/sec):** PASS ({:.0} >= {:.0})", avg_write, target);
    } else {
        println!("**Write Target (500k rec/sec):** Below target ({:.0} < {:.0})", avg_write, target);
    }
    if avg_read >= target {
        println!("**Read Target (500k rec/sec):** PASS ({:.0} >= {:.0})", avg_read, target);
    } else {
        println!("**Read Target (500k rec/sec):** Below target ({:.0} < {:.0})", avg_read, target);
    }
    println!();

    Ok(())
}

// ============================================================================
// MEMORY PROFILING
// ============================================================================

fn memory_profile() -> Result<(), Box<dyn std::error::Error>> {
    println!("### Memory Profile\n");

    // Load 10k records
    let data = fs::read(PERF_TEST_10K_PATH)?;
    let cursor = Cursor::new(&data);
    let mut reader = MarcReader::new(cursor);
    let mut records: Vec<mrrc::Record> = Vec::new();

    while let Some(record) = reader.read_record()? {
        records.push(record);
    }

    // Measure serialized size per record
    let mut total_serialized = 0usize;
    for record in &records {
        let mut buf = Vec::new();
        {
            let mut writer = ProtobufWriter::new(&mut buf);
            writer.write_record(record)?;
            writer.finish()?;
        }
        total_serialized += buf.len();
    }
    let avg_bytes_per_record = total_serialized as f64 / records.len() as f64;

    // Compare to original ISO 2709 size
    let iso_size = data.len();
    let iso_bytes_per_record = iso_size as f64 / records.len() as f64;

    println!("| Metric | ISO 2709 | Protobuf | Delta |");
    println!("|--------|----------|----------|-------|");
    println!(
        "| Avg bytes/record | {:.1} | {:.1} | {:+.1}% |",
        iso_bytes_per_record,
        avg_bytes_per_record,
        ((avg_bytes_per_record - iso_bytes_per_record) / iso_bytes_per_record) * 100.0
    );
    println!(
        "| Total size (10k) | {} | {} | {:+.1}% |",
        iso_size,
        total_serialized,
        ((total_serialized as f64 - iso_size as f64) / iso_size as f64) * 100.0
    );
    println!();

    println!("**Note:** Protobuf uses length-delimited encoding for streaming support.");
    println!("Memory during processing depends on batch size, not total record count.\n");

    Ok(())
}

// ============================================================================
// MAIN
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("# Protobuf Benchmark Results\n");
    println!("**Issue:** mrrc-d4g.2.2.6");
    println!("**Target:** 500k records/second read and write throughput\n");
    println!("---\n");

    println!("## 1. Round-Trip Fidelity\n");
    test_round_trip()?;

    println!("---\n");

    println!("## 2. Performance Benchmarks\n");
    benchmark_10k()?;
    benchmark_100k()?;

    println!("---\n");

    println!("## 3. Memory Profile\n");
    memory_profile()?;

    println!("---\n");

    Ok(())
}
