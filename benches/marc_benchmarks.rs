#![allow(missing_docs, unused_doc_comments, unused_attributes)]
//! Benchmarks for MRRC MARC library.
//!
//! This benchmark suite tests the performance of reading, writing, and processing
//! MARC records using Criterion.rs for statistical analysis.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mrrc::{json, xml, MarcReader, MarcWriter, RecordHelpers};
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
                let _ = xml::record_to_xml(&record);
                count += 1;
            }
            count
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
    benchmark_read_with_field_access_1k,
    benchmark_read_with_field_access_10k,
    benchmark_serialization_to_json_1k,
    benchmark_serialization_to_xml_1k,
    benchmark_roundtrip_1k,
    benchmark_roundtrip_10k,
);
criterion_main!(benches);
