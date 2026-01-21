#![allow(
    missing_docs,
    clippy::cast_precision_loss,
    clippy::implicit_clone,
    clippy::uninlined_format_args,
    clippy::cast_possible_truncation,
    clippy::cast_lossless,
    clippy::too_many_lines
)]
//! `FlatBuffers` benchmarks for MARC record serialization.
//!
//! Benchmarks read/write throughput and compression for the `FlatBuffers` format.
//! Target: 259k records/second and 64% memory savings vs ISO 2709.

use mrrc::flatbuffers_impl::{FlatbuffersReader, FlatbuffersWriter};
use mrrc::formats::FormatReader;
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

    // Serialize all records to FlatBuffers
    let mut serialized = Vec::new();
    {
        let mut writer = FlatbuffersWriter::new(&mut serialized);
        for record in &records {
            writer.write_record(record)?;
        }
        writer.finish()?;
    }

    // Deserialize from FlatBuffers
    let cursor = Cursor::new(&serialized);
    let mut fb_reader = FlatbuffersReader::new(cursor);
    let recovered = fb_reader.read_all()?;

    let mut perfect_count = 0;
    let mut failures = Vec::new();

    for (idx, (original, restored)) in records.iter().zip(recovered.iter()).enumerate() {
        // Compare leaders
        let orig_leader_bytes = original.leader.as_bytes().unwrap_or_default();
        let rec_leader_bytes = restored.leader.as_bytes().unwrap_or_default();
        let match_leader = orig_leader_bytes == rec_leader_bytes;

        // Compare control fields
        let orig_control: Vec<_> = original.control_fields_iter().collect();
        let rec_control: Vec<_> = restored.control_fields_iter().collect();
        let match_control = orig_control == rec_control;

        // Compare data fields
        let orig_fields: Vec<_> = original.fields().collect();
        let rec_fields: Vec<_> = restored.fields().collect();
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
        let mut writer = FlatbuffersWriter::new(&mut serialized);
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
    let mut reader = FlatbuffersReader::new(cursor);
    let records = reader.read_all().unwrap();
    let duration = start.elapsed();
    assert_eq!(records.len(), expected_count, "Record count mismatch");
    records.len() as f64 / duration.as_secs_f64()
}

fn benchmark_10k() -> Result<(), Box<dyn std::error::Error>> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    let data = fs::read(PERF_TEST_10K_PATH)?;
    let iso_size = data.len();
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
        let mut writer = FlatbuffersWriter::new(&mut warmup_buf);
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
            let mut buf = Vec::new();
            {
                let mut writer = FlatbuffersWriter::new(&mut buf);
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
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&serialized)?;
    let gzipped = encoder.finish()?;

    // Calculate size ratio vs ISO 2709
    let size_ratio = (1.0 - serialized.len() as f64 / iso_size as f64) * 100.0;

    println!("| Metric | Value |");
    println!("|--------|-------|");
    println!("| Write Throughput | {:.0} rec/sec |", avg_write);
    println!("| Read Throughput | {:.0} rec/sec |", avg_read);
    println!(
        "| FlatBuffers Size | {} bytes ({:.2} KB/rec) |",
        serialized.len(),
        serialized.len() as f64 / records.len() as f64 / 1024.0
    );
    println!(
        "| ISO 2709 Size | {} bytes ({:.2} KB/rec) |",
        iso_size,
        iso_size as f64 / records.len() as f64 / 1024.0
    );
    println!(
        "| Size vs ISO | {:.1}% {} |",
        size_ratio.abs(),
        if size_ratio >= 0.0 {
            "smaller"
        } else {
            "larger"
        }
    );
    println!(
        "| Gzipped FlatBuffers Size | {} bytes ({:.1}% gzip compression) |",
        gzipped.len(),
        (1.0 - gzipped.len() as f64 / serialized.len() as f64) * 100.0
    );
    println!();

    // Check against targets
    let throughput_target = 259_000.0;
    let memory_target = 64.0; // 64% smaller than ISO 2709

    if avg_write >= throughput_target {
        println!(
            "**Write Target (259k rec/sec):** PASS ({:.0} >= {:.0})",
            avg_write, throughput_target
        );
    } else {
        println!(
            "**Write Target (259k rec/sec):** Below target ({:.0} < {:.0})",
            avg_write, throughput_target
        );
    }
    if avg_read >= throughput_target {
        println!(
            "**Read Target (259k rec/sec):** PASS ({:.0} >= {:.0})",
            avg_read, throughput_target
        );
    } else {
        println!(
            "**Read Target (259k rec/sec):** Below target ({:.0} < {:.0})",
            avg_read, throughput_target
        );
    }
    if size_ratio >= memory_target {
        println!(
            "**Memory Target (64% savings):** PASS ({:.1}% >= {:.0}%)",
            size_ratio, memory_target
        );
    } else if size_ratio > 0.0 {
        println!(
            "**Memory Target (64% savings):** Below target ({:.1}% < {:.0}%)",
            size_ratio, memory_target
        );
    } else {
        println!(
            "**Memory Target (64% savings):** FAIL - FlatBuffers is {:.1}% larger than ISO",
            -size_ratio
        );
    }
    println!();

    Ok(())
}

fn benchmark_100k() -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(PERF_TEST_100K_PATH)?;
    let iso_size = data.len();
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
            let mut buf = Vec::new();
            {
                let mut writer = FlatbuffersWriter::new(&mut buf);
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

    // Calculate size ratio vs ISO 2709
    let size_ratio = (1.0 - serialized.len() as f64 / iso_size as f64) * 100.0;

    println!("| Metric | Value |");
    println!("|--------|-------|");
    println!("| Write Throughput | {:.0} rec/sec |", avg_write);
    println!("| Read Throughput | {:.0} rec/sec |", avg_read);
    println!(
        "| FlatBuffers Size | {} bytes ({:.2} MB) |",
        serialized.len(),
        serialized.len() as f64 / 1024.0 / 1024.0
    );
    println!(
        "| ISO 2709 Size | {} bytes ({:.2} MB) |",
        iso_size,
        iso_size as f64 / 1024.0 / 1024.0
    );
    println!(
        "| Size vs ISO | {:.1}% {} |",
        size_ratio.abs(),
        if size_ratio >= 0.0 {
            "smaller"
        } else {
            "larger"
        }
    );
    println!();

    // Check against targets
    let throughput_target = 259_000.0;
    let memory_target = 64.0;

    if avg_write >= throughput_target {
        println!(
            "**Write Target (259k rec/sec):** PASS ({:.0} >= {:.0})",
            avg_write, throughput_target
        );
    } else {
        println!(
            "**Write Target (259k rec/sec):** Below target ({:.0} < {:.0})",
            avg_write, throughput_target
        );
    }
    if avg_read >= throughput_target {
        println!(
            "**Read Target (259k rec/sec):** PASS ({:.0} >= {:.0})",
            avg_read, throughput_target
        );
    } else {
        println!(
            "**Read Target (259k rec/sec):** Below target ({:.0} < {:.0})",
            avg_read, throughput_target
        );
    }
    if size_ratio >= memory_target {
        println!(
            "**Memory Target (64% savings):** PASS ({:.1}% >= {:.0}%)",
            size_ratio, memory_target
        );
    } else if size_ratio > 0.0 {
        println!(
            "**Memory Target (64% savings):** Below target ({:.1}% < {:.0}%)",
            size_ratio, memory_target
        );
    } else {
        println!(
            "**Memory Target (64% savings):** FAIL - FlatBuffers is {:.1}% larger than ISO",
            -size_ratio
        );
    }
    println!();

    Ok(())
}

// ============================================================================
// MEMORY PROFILE
// ============================================================================

fn memory_profile() -> Result<(), Box<dyn std::error::Error>> {
    println!("### Memory Profile\n");

    // Load 10k records
    let data = fs::read(PERF_TEST_10K_PATH)?;
    let iso_size = data.len();
    let cursor = Cursor::new(&data);
    let mut reader = MarcReader::new(cursor);
    let mut records: Vec<mrrc::Record> = Vec::new();

    while let Some(record) = reader.read_record()? {
        records.push(record);
    }

    // Serialize all records
    let mut serialized = Vec::new();
    {
        let mut writer = FlatbuffersWriter::new(&mut serialized);
        for record in &records {
            writer.write_record(record)?;
        }
        writer.finish()?;
    }

    let fb_bytes_per_record = serialized.len() as f64 / records.len() as f64;
    let iso_bytes_per_record = iso_size as f64 / records.len() as f64;
    let memory_delta =
        ((fb_bytes_per_record - iso_bytes_per_record) / iso_bytes_per_record) * 100.0;

    println!("| Metric | ISO 2709 | FlatBuffers | Delta |");
    println!("|--------|----------|-------------|-------|");
    println!(
        "| Avg bytes/record | {:.1} | {:.1} | {:+.1}% |",
        iso_bytes_per_record, fb_bytes_per_record, memory_delta
    );
    println!(
        "| Total size (10k) | {} | {} | {:+.1}% |",
        iso_size,
        serialized.len(),
        ((serialized.len() as f64 - iso_size as f64) / iso_size as f64) * 100.0
    );
    println!();

    // Memory savings target check
    let memory_savings = -memory_delta;
    if memory_savings >= 64.0 {
        println!(
            "**Memory Savings Target (64%):** PASS ({:.1}% >= 64%)\n",
            memory_savings
        );
    } else if memory_savings > 0.0 {
        println!(
            "**Memory Savings Target (64%):** Below target ({:.1}% < 64%)\n",
            memory_savings
        );
    } else {
        println!(
            "**Memory Savings Target (64%):** FAIL - FlatBuffers uses {:.1}% MORE memory\n",
            -memory_savings
        );
    }

    println!("**Note:** FlatBuffers uses size-prefixed encoding for streaming support.");
    println!("The format enables zero-copy deserialization for high-performance reads.\n");

    Ok(())
}

// ============================================================================
// MAIN
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("# FlatBuffers Benchmark Results\n");
    println!("**Issue:** mrrc-d4g.3.2.6");
    println!("**Targets:** 259k records/second, 64% memory savings vs ISO 2709\n");
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
