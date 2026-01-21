#![allow(
    missing_docs,
    clippy::cast_precision_loss,
    clippy::implicit_clone,
    clippy::uninlined_format_args,
    clippy::cast_possible_truncation,
    clippy::cast_lossless,
    clippy::too_many_lines
)]
//! Arrow benchmarks for MARC record serialization.
//!
//! Benchmarks read/write throughput and compression for the Arrow IPC format.
//! Target: 1.77M records/second and 30% compression (vs ISO 2709).

use mrrc::arrow_impl::{ArrowReader, ArrowWriter};
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

    // Serialize all records to Arrow IPC
    let mut serialized = Vec::new();
    {
        let mut writer = ArrowWriter::new(&mut serialized);
        writer.write_batch(&records)?;
        writer.finish()?;
    }

    // Deserialize from Arrow IPC
    let cursor = Cursor::new(&serialized);
    let mut arrow_reader = ArrowReader::new(cursor)?;
    let recovered = arrow_reader.read_all()?;

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
        let mut writer = ArrowWriter::new(&mut serialized);
        writer.write_batch(records).unwrap();
        writer.finish().unwrap();
    }
    let duration = start.elapsed();
    let throughput = records.len() as f64 / duration.as_secs_f64();
    (throughput, serialized.len())
}

fn benchmark_read_throughput(serialized: &[u8], expected_count: usize) -> f64 {
    let start = Instant::now();
    let cursor = Cursor::new(serialized);
    let mut reader = ArrowReader::new(cursor).unwrap();
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
        let mut writer = ArrowWriter::new(&mut warmup_buf);
        writer.write_batch(&records[..100.min(records.len())])?;
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
                let mut writer = ArrowWriter::new(&mut buf);
                writer.write_batch(&records)?;
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

    // Calculate compression vs ISO 2709
    let compression_ratio = (1.0 - serialized.len() as f64 / iso_size as f64) * 100.0;

    println!("| Metric | Value |");
    println!("|--------|-------|");
    println!("| Write Throughput | {:.0} rec/sec |", avg_write);
    println!("| Read Throughput | {:.0} rec/sec |", avg_read);
    println!(
        "| Arrow IPC Size | {} bytes ({:.2} KB/rec) |",
        serialized.len(),
        serialized.len() as f64 / records.len() as f64 / 1024.0
    );
    println!(
        "| ISO 2709 Size | {} bytes ({:.2} KB/rec) |",
        iso_size,
        iso_size as f64 / records.len() as f64 / 1024.0
    );
    println!(
        "| Compression vs ISO | {:.1}% {} |",
        compression_ratio.abs(),
        if compression_ratio >= 0.0 {
            "smaller"
        } else {
            "larger"
        }
    );
    println!(
        "| Gzipped Arrow Size | {} bytes ({:.1}% gzip compression) |",
        gzipped.len(),
        (1.0 - gzipped.len() as f64 / serialized.len() as f64) * 100.0
    );
    println!();

    // Check against targets
    let throughput_target = 1_770_000.0;
    let compression_target = 30.0;

    if avg_write >= throughput_target {
        println!(
            "**Write Target (1.77M rec/sec):** PASS ({:.0} >= {:.0})",
            avg_write, throughput_target
        );
    } else {
        println!(
            "**Write Target (1.77M rec/sec):** Below target ({:.0} < {:.0})",
            avg_write, throughput_target
        );
    }
    if avg_read >= throughput_target {
        println!(
            "**Read Target (1.77M rec/sec):** PASS ({:.0} >= {:.0})",
            avg_read, throughput_target
        );
    } else {
        println!(
            "**Read Target (1.77M rec/sec):** Below target ({:.0} < {:.0})",
            avg_read, throughput_target
        );
    }
    if compression_ratio >= compression_target {
        println!(
            "**Compression Target (30%):** PASS ({:.1}% >= {:.0}%)",
            compression_ratio, compression_target
        );
    } else {
        println!(
            "**Compression Target (30%):** Below target ({:.1}% < {:.0}%)",
            compression_ratio, compression_target
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
                let mut writer = ArrowWriter::new(&mut buf);
                writer.write_batch(&records)?;
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

    // Calculate compression vs ISO 2709
    let compression_ratio = (1.0 - serialized.len() as f64 / iso_size as f64) * 100.0;

    println!("| Metric | Value |");
    println!("|--------|-------|");
    println!("| Write Throughput | {:.0} rec/sec |", avg_write);
    println!("| Read Throughput | {:.0} rec/sec |", avg_read);
    println!(
        "| Arrow IPC Size | {} bytes ({:.2} MB) |",
        serialized.len(),
        serialized.len() as f64 / 1024.0 / 1024.0
    );
    println!(
        "| ISO 2709 Size | {} bytes ({:.2} MB) |",
        iso_size,
        iso_size as f64 / 1024.0 / 1024.0
    );
    println!(
        "| Compression vs ISO | {:.1}% {} |",
        compression_ratio.abs(),
        if compression_ratio >= 0.0 {
            "smaller"
        } else {
            "larger"
        }
    );
    println!();

    // Check against targets
    let throughput_target = 1_770_000.0;
    let compression_target = 30.0;

    if avg_write >= throughput_target {
        println!(
            "**Write Target (1.77M rec/sec):** PASS ({:.0} >= {:.0})",
            avg_write, throughput_target
        );
    } else {
        println!(
            "**Write Target (1.77M rec/sec):** Below target ({:.0} < {:.0})",
            avg_write, throughput_target
        );
    }
    if avg_read >= throughput_target {
        println!(
            "**Read Target (1.77M rec/sec):** PASS ({:.0} >= {:.0})",
            avg_read, throughput_target
        );
    } else {
        println!(
            "**Read Target (1.77M rec/sec):** Below target ({:.0} < {:.0})",
            avg_read, throughput_target
        );
    }
    if compression_ratio >= compression_target {
        println!(
            "**Compression Target (30%):** PASS ({:.1}% >= {:.0}%)",
            compression_ratio, compression_target
        );
    } else {
        println!(
            "**Compression Target (30%):** Below target ({:.1}% < {:.0}%)",
            compression_ratio, compression_target
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

    // Serialize all records as a single batch
    let mut serialized = Vec::new();
    {
        let mut writer = ArrowWriter::new(&mut serialized);
        writer.write_batch(&records)?;
        writer.finish()?;
    }

    let arrow_bytes_per_record = serialized.len() as f64 / records.len() as f64;
    let iso_bytes_per_record = iso_size as f64 / records.len() as f64;

    println!("| Metric | ISO 2709 | Arrow IPC | Delta |");
    println!("|--------|----------|-----------|-------|");
    println!(
        "| Avg bytes/record | {:.1} | {:.1} | {:+.1}% |",
        iso_bytes_per_record,
        arrow_bytes_per_record,
        ((arrow_bytes_per_record - iso_bytes_per_record) / iso_bytes_per_record) * 100.0
    );
    println!(
        "| Total size (10k) | {} | {} | {:+.1}% |",
        iso_size,
        serialized.len(),
        ((serialized.len() as f64 - iso_size as f64) / iso_size as f64) * 100.0
    );
    println!();

    println!("**Note:** Arrow IPC uses columnar storage with efficient encoding.");
    println!("The columnar layout enables fast analytical queries via DuckDB/Polars.\n");

    Ok(())
}

// ============================================================================
// MAIN
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("# Arrow Benchmark Results\n");
    println!("**Issue:** mrrc-d4g.3.1.6");
    println!("**Targets:** 1.77M records/second, 30% compression vs ISO 2709\n");
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
