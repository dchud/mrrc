#![allow(missing_docs, unused_doc_comments, unused_attributes)]
//! Benchmarks for MRRC MARC library.
//!
//! This benchmark suite tests the performance of reading, writing, and processing
//! MARC records using Criterion.rs for statistical analysis.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use mrrc::{LinkageInfo, MarcReader, MarcWriter, RecordHelpers, json, marcxml};
use std::io::Cursor;

/// Load test fixtures from the test data directory.
fn load_fixture(filename: &str) -> Vec<u8> {
    let path = format!("tests/data/fixtures/{filename}");
    std::fs::read(&path).unwrap_or_else(|_| panic!("Failed to load fixture: {path}"))
}

/// Benchmark reading 1,000 MARC records.
fn benchmark_read_1k(c: &mut Criterion) {
    let fixture = black_box(load_fixture("1k_records.mrc"));

    c.bench_function("read_1k_records", |b| {
        b.iter(|| {
            let cursor = Cursor::new(fixture.clone());
            let mut reader = MarcReader::new(cursor);
            let mut count = 0;
            while let Ok(Some(_record)) = reader.read_record() {
                count += 1;
            }
            count
        });
    });
}

/// Benchmark reading 10,000 MARC records.
fn benchmark_read_10k(c: &mut Criterion) {
    let fixture = black_box(load_fixture("10k_records.mrc"));

    c.bench_function("read_10k_records", |b| {
        b.iter(|| {
            let cursor = Cursor::new(fixture.clone());
            let mut reader = MarcReader::new(cursor);
            let mut count = 0;
            while let Ok(Some(_record)) = reader.read_record() {
                count += 1;
            }
            count
        });
    });
}

/// Benchmark reading 1,000 MARC records from a file path.
///
/// Unlike the cursor benchmarks above, this exercises the real file-I/O
/// read loop, so it is sensitive to syscall count and buffering.
fn benchmark_read_1k_from_path(c: &mut Criterion) {
    c.bench_function("read_1k_records_from_path", |b| {
        b.iter(|| {
            let mut reader = MarcReader::from_path("tests/data/fixtures/1k_records.mrc").unwrap();
            let mut count = 0;
            while let Ok(Some(_record)) = reader.read_record() {
                count += 1;
            }
            black_box(count)
        });
    });
}

/// Benchmark reading 10,000 MARC records from a file path.
fn benchmark_read_10k_from_path(c: &mut Criterion) {
    c.bench_function("read_10k_records_from_path", |b| {
        b.iter(|| {
            let mut reader = MarcReader::from_path("tests/data/fixtures/10k_records.mrc").unwrap();
            let mut count = 0;
            while let Ok(Some(_record)) = reader.read_record() {
                count += 1;
            }
            black_box(count)
        });
    });
}

/// Benchmark reading 1,000 MARC records with field access.
fn benchmark_read_with_field_access_1k(c: &mut Criterion) {
    let fixture = black_box(load_fixture("1k_records.mrc"));

    c.bench_function("read_1k_with_field_access", |b| {
        b.iter(|| {
            let cursor = Cursor::new(fixture.clone());
            let mut reader = MarcReader::new(cursor);
            let mut count = 0;
            while let Ok(Some(record)) = reader.read_record() {
                // Access title field (245)
                let _ = record.title();
                // Access field 100
                let _ = record.get_fields("100");
                count += 1;
            }
            count
        });
    });
}

/// Benchmark reading 10,000 MARC records with field access.
fn benchmark_read_with_field_access_10k(c: &mut Criterion) {
    let fixture = black_box(load_fixture("10k_records.mrc"));

    c.bench_function("read_10k_with_field_access", |b| {
        b.iter(|| {
            let cursor = Cursor::new(fixture.clone());
            let mut reader = MarcReader::new(cursor);
            let mut count = 0;
            while let Ok(Some(record)) = reader.read_record() {
                let _ = record.title();
                let _ = record.get_fields("100");
                count += 1;
            }
            count
        });
    });
}

/// Benchmark writing 1,000 MARC records to ISO 2709 binary.
///
/// Records are parsed once up front; only serialization is timed. This is
/// the regression sensor for the binary writer's hot path — the existing
/// roundtrip benches bundle write with read (which dominates), so a write-path
/// change is invisible there. The output buffer is reused across iterations
/// so the benchmark measures the writer, not allocator warmup.
fn benchmark_write_1k(c: &mut Criterion) {
    let fixture = load_fixture("1k_records.mrc");
    let mut reader = MarcReader::new(Cursor::new(fixture));
    let mut records = Vec::new();
    while let Ok(Some(record)) = reader.read_record() {
        records.push(record);
    }
    let mut output = Vec::with_capacity(1 << 20);

    c.bench_function("write_1k_records", |b| {
        b.iter(|| {
            output.clear();
            let mut writer = MarcWriter::new(&mut output);
            for record in &records {
                writer.write_record(record).unwrap();
            }
            black_box(output.len())
        });
    });
}

/// Benchmark writing 10,000 MARC records to ISO 2709 binary.
fn benchmark_write_10k(c: &mut Criterion) {
    let fixture = load_fixture("10k_records.mrc");
    let mut reader = MarcReader::new(Cursor::new(fixture));
    let mut records = Vec::new();
    while let Ok(Some(record)) = reader.read_record() {
        records.push(record);
    }
    let mut output = Vec::with_capacity(4 << 20);

    c.bench_function("write_10k_records", |b| {
        b.iter(|| {
            output.clear();
            let mut writer = MarcWriter::new(&mut output);
            for record in &records {
                writer.write_record(record).unwrap();
            }
            black_box(output.len())
        });
    });
}

/// Benchmark JSON serialization of 1,000 MARC records.
fn benchmark_serialization_to_json_1k(c: &mut Criterion) {
    let fixture = black_box(load_fixture("1k_records.mrc"));

    c.bench_function("serialize_1k_to_json", |b| {
        b.iter(|| {
            let cursor = Cursor::new(fixture.clone());
            let mut reader = MarcReader::new(cursor);
            let mut count = 0;
            while let Ok(Some(record)) = reader.read_record() {
                let _ = json::record_to_json(&record);
                count += 1;
            }
            count
        });
    });
}

/// Benchmark XML serialization of 1,000 MARC records.
fn benchmark_serialization_to_xml_1k(c: &mut Criterion) {
    let fixture = black_box(load_fixture("1k_records.mrc"));

    c.bench_function("serialize_1k_to_xml", |b| {
        b.iter(|| {
            let cursor = Cursor::new(fixture.clone());
            let mut reader = MarcReader::new(cursor);
            let mut count = 0;
            while let Ok(Some(record)) = reader.read_record() {
                let _ = marcxml::record_to_marcxml(&record);
                count += 1;
            }
            count
        });
    });
}

/// Benchmark MARCXML deserialization of a single record.
///
/// Exercises `strip_marcxml_ns` once per iteration — the per-call path PERF-7
/// made compile-free by hoisting its namespace regexes to statics. A
/// whole-document parse amortizes that cost over the whole collection and
/// hides it; this single-record form is the regression sensor for it (the
/// `serialize_1k_to_xml` bench only covers the write path).
fn benchmark_deserialize_marcxml_record(c: &mut Criterion) {
    let fixture = load_fixture("1k_records.mrc");
    let mut reader = MarcReader::new(Cursor::new(fixture));
    let record = reader
        .read_record()
        .expect("read fixture record")
        .expect("fixture has at least one record");
    let xml = black_box(marcxml::record_to_marcxml(&record).expect("serialize to MARCXML"));

    c.bench_function("deserialize_marcxml_record", |b| {
        b.iter(|| {
            let _ = black_box(marcxml::marcxml_to_record(black_box(&xml)));
        });
    });
}

/// Benchmark MARC subfield-6 (880 linkage) parsing.
///
/// `LinkageInfo::parse` runs per field during 880-linkage scans; PERF-7
/// hoisted its regex to a static. Regression sensor for that path.
fn benchmark_parse_linkage(c: &mut Criterion) {
    let values = black_box(["880-01", "100-02/(3/r", "245-01/(2/r", "650-03"]);

    c.bench_function("parse_linkage_subfield6", |b| {
        b.iter(|| {
            for v in values {
                black_box(LinkageInfo::parse(black_box(v)));
            }
        });
    });
}

/// Benchmark read + write roundtrip of 1,000 MARC records.
fn benchmark_roundtrip_1k(c: &mut Criterion) {
    let fixture = black_box(load_fixture("1k_records.mrc"));

    c.bench_function("roundtrip_1k_records", |b| {
        b.iter(|| {
            // Read records
            let cursor = Cursor::new(fixture.clone());
            let mut reader = MarcReader::new(cursor);
            let mut records = Vec::new();
            while let Ok(Some(record)) = reader.read_record() {
                records.push(record);
            }

            // Write records
            let mut output = Vec::new();
            let mut writer = MarcWriter::new(&mut output);
            for record in records {
                let _ = writer.write_record(&record);
            }

            output.len()
        });
    });
}

/// Benchmark read + write roundtrip of 10,000 MARC records.
fn benchmark_roundtrip_10k(c: &mut Criterion) {
    let fixture = black_box(load_fixture("10k_records.mrc"));

    c.bench_function("roundtrip_10k_records", |b| {
        b.iter(|| {
            let cursor = Cursor::new(fixture.clone());
            let mut reader = MarcReader::new(cursor);
            let mut records = Vec::new();
            while let Ok(Some(record)) = reader.read_record() {
                records.push(record);
            }

            let mut output = Vec::new();
            let mut writer = MarcWriter::new(&mut output);
            for record in records {
                let _ = writer.write_record(&record);
            }

            output.len()
        });
    });
}

criterion_group!(
    benches,
    benchmark_read_1k,
    benchmark_read_10k,
    benchmark_read_1k_from_path,
    benchmark_read_10k_from_path,
    benchmark_read_with_field_access_1k,
    benchmark_read_with_field_access_10k,
    benchmark_write_1k,
    benchmark_write_10k,
    benchmark_serialization_to_json_1k,
    benchmark_serialization_to_xml_1k,
    benchmark_deserialize_marcxml_record,
    benchmark_parse_linkage,
    benchmark_roundtrip_1k,
    benchmark_roundtrip_10k,
);
criterion_main!(benches);
