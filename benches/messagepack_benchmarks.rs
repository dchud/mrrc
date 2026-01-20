#![allow(
    missing_docs,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_lossless
)]
//! MessagePack benchmarks for MARC record serialization.
//!
//! Target: 750k records/second for both read and write operations.

use mrrc::formats::messagepack::{MessagePackReader, MessagePackWriter};
use mrrc::MarcReader;
use std::fs;
use std::io::Cursor;
use std::time::Instant;

const FIDELITY_TEST_PATH: &str = "tests/data/fixtures/fidelity_test_100.mrc";
const PERF_TEST_10K_PATH: &str = "tests/data/fixtures/10k_records.mrc";
const PERF_TEST_100K_PATH: &str = "tests/data/fixtures/100k_records.mrc";

fn test_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(FIDELITY_TEST_PATH)?;
    let cursor = Cursor::new(&data);
    let mut reader = MarcReader::new(cursor);
    let mut records: Vec<mrrc::Record> = Vec::new();

    while let Some(record) = reader.read_record()? {
        records.push(record);
    }

    let mut perfect_count = 0;

    for original in &records {
        let mut serialized = Vec::new();
        {
            let mut writer = MessagePackWriter::new(&mut serialized);
            writer.write_record(original)?;
            writer.finish()?;
        }

        let cursor = Cursor::new(&serialized);
        let mut reader = MessagePackReader::new(cursor);
        let recovered = reader.read_record()?.expect("Should have one record");

        let orig_leader = original.leader.as_bytes().unwrap_or_default();
        let rec_leader = recovered.leader.as_bytes().unwrap_or_default();
        let match_leader = orig_leader == rec_leader;

        let orig_control: Vec<_> = original.control_fields_iter().collect();
        let rec_control: Vec<_> = recovered.control_fields_iter().collect();
        let match_control = orig_control == rec_control;

        let orig_fields: Vec<_> = original.fields().collect();
        let rec_fields: Vec<_> = recovered.fields().collect();
        let match_fields = orig_fields.len() == rec_fields.len()
            && orig_fields.iter().zip(rec_fields.iter()).all(|(a, b)| {
                a.tag == b.tag
                    && a.indicator1 == b.indicator1
                    && a.indicator2 == b.indicator2
                    && a.subfields == b.subfields
            });

        if match_leader && match_control && match_fields {
            perfect_count += 1;
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
    println!(
        "**Status:** {}",
        if perfect_count == records.len() {
            "PASS"
        } else {
            "FAIL"
        }
    );
    println!();

    Ok(())
}

fn benchmark_write_throughput(records: &[mrrc::Record]) -> (f64, usize) {
    let start = Instant::now();
    let mut serialized = Vec::new();
    {
        let mut writer = MessagePackWriter::new(&mut serialized);
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
    let mut reader = MessagePackReader::new(cursor);
    let mut count = 0;
    while reader.read_record().unwrap().is_some() {
        count += 1;
    }
    let duration = start.elapsed();
    assert_eq!(count, expected_count);
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

    // Warmup
    let mut warmup = Vec::new();
    {
        let mut w = MessagePackWriter::new(&mut warmup);
        for r in records.iter().take(100) {
            w.write_record(r)?;
        }
        w.finish()?;
    }

    // Write benchmark (5 iterations)
    let mut write_throughputs = Vec::new();
    let mut serialized = Vec::new();
    for i in 0..5 {
        let (throughput, _) = benchmark_write_throughput(&records);
        write_throughputs.push(throughput);
        if i == 0 {
            let mut buf = Vec::new();
            {
                let mut w = MessagePackWriter::new(&mut buf);
                for r in &records {
                    w.write_record(r)?;
                }
                w.finish()?;
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

    // Compression
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
    println!(
        "| Serialized Size | {} bytes ({:.2} KB/rec) |",
        serialized.len(),
        serialized.len() as f64 / records.len() as f64 / 1024.0
    );
    println!(
        "| Gzipped Size | {} bytes ({:.1}% compression) |",
        gzipped.len(),
        (1.0 - gzipped.len() as f64 / serialized.len() as f64) * 100.0
    );
    println!();

    let target = 750_000.0;
    println!(
        "**Write Target (750k rec/sec):** {} ({:.0} {} {:.0})",
        if avg_write >= target { "PASS" } else { "Below target" },
        avg_write,
        if avg_write >= target { ">=" } else { "<" },
        target
    );
    println!(
        "**Read Target (750k rec/sec):** {} ({:.0} {} {:.0})",
        if avg_read >= target { "PASS" } else { "Below target" },
        avg_read,
        if avg_read >= target { ">=" } else { "<" },
        target
    );
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

    let mut write_throughputs = Vec::new();
    let mut serialized = Vec::new();
    for i in 0..3 {
        let (throughput, _) = benchmark_write_throughput(&records);
        write_throughputs.push(throughput);
        if i == 0 {
            let mut buf = Vec::new();
            {
                let mut w = MessagePackWriter::new(&mut buf);
                for r in &records {
                    w.write_record(r)?;
                }
                w.finish()?;
            }
            serialized = buf;
        }
    }
    let avg_write = write_throughputs.iter().sum::<f64>() / write_throughputs.len() as f64;

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
    println!(
        "| Serialized Size | {} bytes ({:.2} MB) |",
        serialized.len(),
        serialized.len() as f64 / 1024.0 / 1024.0
    );
    println!();

    let target = 750_000.0;
    println!(
        "**Write Target (750k rec/sec):** {} ({:.0} {} {:.0})",
        if avg_write >= target { "PASS" } else { "Below target" },
        avg_write,
        if avg_write >= target { ">=" } else { "<" },
        target
    );
    println!(
        "**Read Target (750k rec/sec):** {} ({:.0} {} {:.0})",
        if avg_read >= target { "PASS" } else { "Below target" },
        avg_read,
        if avg_read >= target { ">=" } else { "<" },
        target
    );
    println!();

    Ok(())
}

fn memory_profile() -> Result<(), Box<dyn std::error::Error>> {
    println!("### Memory Profile\n");

    let data = fs::read(PERF_TEST_10K_PATH)?;
    let cursor = Cursor::new(&data);
    let mut reader = MarcReader::new(cursor);
    let mut records: Vec<mrrc::Record> = Vec::new();

    while let Some(record) = reader.read_record()? {
        records.push(record);
    }

    let mut total_serialized = 0usize;
    for record in &records {
        let mut buf = Vec::new();
        {
            let mut writer = MessagePackWriter::new(&mut buf);
            writer.write_record(record)?;
            writer.finish()?;
        }
        total_serialized += buf.len();
    }
    let avg_msgpack = total_serialized as f64 / records.len() as f64;
    let avg_iso = data.len() as f64 / records.len() as f64;

    println!("| Metric | ISO 2709 | MessagePack | Delta |");
    println!("|--------|----------|-------------|-------|");
    println!(
        "| Avg bytes/record | {:.1} | {:.1} | {:+.1}% |",
        avg_iso,
        avg_msgpack,
        ((avg_msgpack - avg_iso) / avg_iso) * 100.0
    );
    println!(
        "| Total size (10k) | {} | {} | {:+.1}% |",
        data.len(),
        total_serialized,
        ((total_serialized as f64 - data.len() as f64) / data.len() as f64) * 100.0
    );
    println!();

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("# MessagePack Benchmark Results\n");
    println!("**Issue:** mrrc-d4g.3.3.5");
    println!("**Target:** 750k records/second read and write throughput\n");
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

    Ok(())
}
