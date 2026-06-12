#![allow(missing_docs, unused_doc_comments, unused_attributes)]
//! Benchmarks for serialization formats beyond ISO 2709/JSON/MARCXML, plus
//! the single-thread parser-pool path.
//!
//! Covers CSV, MODS, Dublin Core, BIBFRAME (RDF), and MARC-in-JSON
//! serialization, the parse-from direction where one exists, and the
//! boundary-scan + batch-parse path used by the producer-consumer pipeline,
//! pinned to one rayon worker so the per-record instruction cost is a
//! deterministic single-thread signal.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use mrrc::bibframe::{BibframeConfig, RdfFormat, marc_to_bibframe};
use mrrc::boundary_scanner::RecordBoundaryScanner;
use mrrc::rayon_parser_pool::parse_batch_parallel;
use mrrc::{MarcReader, Record, csv, dublin_core, marcjson, mods};
use std::io::Cursor;

/// Load test fixtures from the test data directory.
fn load_fixture(filename: &str) -> Vec<u8> {
    let path = format!("tests/data/fixtures/{filename}");
    std::fs::read(&path).unwrap_or_else(|_| panic!("Failed to load fixture: {path}"))
}

/// Parse every record in a fixture once, for benches that measure
/// serialization or conversion in isolation.
fn parse_fixture(filename: &str) -> Vec<Record> {
    let fixture = load_fixture(filename);
    let mut reader = MarcReader::new(Cursor::new(fixture));
    let mut records = Vec::new();
    while let Ok(Some(record)) = reader.read_record() {
        records.push(record);
    }
    records
}

/// Benchmark CSV serialization of 1,000 MARC records.
fn benchmark_serialize_to_csv_1k(c: &mut Criterion) {
    let records = black_box(parse_fixture("1k_records.mrc"));

    c.bench_function("serialize_1k_to_csv", |b| {
        b.iter(|| {
            let output = csv::records_to_csv(&records).unwrap();
            output.len()
        });
    });
}

/// Benchmark MARC-in-JSON serialization of 1,000 MARC records.
fn benchmark_serialize_to_marcjson_1k(c: &mut Criterion) {
    let records = black_box(parse_fixture("1k_records.mrc"));

    c.bench_function("serialize_1k_to_marcjson", |b| {
        b.iter(|| {
            let mut count = 0;
            for record in &records {
                let _ = marcjson::record_to_marcjson(record);
                count += 1;
            }
            count
        });
    });
}

/// Benchmark MARC-in-JSON parsing of 1,000 records.
fn benchmark_parse_from_marcjson_1k(c: &mut Criterion) {
    let records = parse_fixture("1k_records.mrc");
    let values: Vec<_> = records
        .iter()
        .map(|r| marcjson::record_to_marcjson(r).unwrap())
        .collect();
    let values = black_box(values);

    c.bench_function("parse_1k_from_marcjson", |b| {
        b.iter(|| {
            let mut count = 0;
            for value in &values {
                if marcjson::marcjson_to_record(value).is_ok() {
                    count += 1;
                }
            }
            count
        });
    });
}

/// Benchmark MODS XML serialization of 1,000 MARC records.
fn benchmark_serialize_to_mods_1k(c: &mut Criterion) {
    let records = black_box(parse_fixture("1k_records.mrc"));

    c.bench_function("serialize_1k_to_mods_xml", |b| {
        b.iter(|| {
            let mut count = 0;
            for record in &records {
                let _ = mods::record_to_mods_xml(record);
                count += 1;
            }
            count
        });
    });
}

/// Benchmark MODS XML parsing of 1,000 records.
fn benchmark_parse_from_mods_1k(c: &mut Criterion) {
    let records = parse_fixture("1k_records.mrc");
    let documents: Vec<_> = records
        .iter()
        .map(|r| mods::record_to_mods_xml(r).unwrap())
        .collect();
    let documents = black_box(documents);

    c.bench_function("parse_1k_from_mods_xml", |b| {
        b.iter(|| {
            let mut count = 0;
            for xml in &documents {
                if mods::mods_xml_to_record(xml).is_ok() {
                    count += 1;
                }
            }
            count
        });
    });
}

/// Benchmark Dublin Core XML serialization of 1,000 MARC records.
fn benchmark_serialize_to_dublin_core_1k(c: &mut Criterion) {
    let records = black_box(parse_fixture("1k_records.mrc"));

    c.bench_function("serialize_1k_to_dublin_core_xml", |b| {
        b.iter(|| {
            let mut count = 0;
            for record in &records {
                let _ = dublin_core::record_to_dublin_core_xml(record);
                count += 1;
            }
            count
        });
    });
}

/// Benchmark BIBFRAME conversion plus Turtle serialization of 1,000 MARC
/// records.
fn benchmark_serialize_to_bibframe_turtle_1k(c: &mut Criterion) {
    let records = black_box(parse_fixture("1k_records.mrc"));
    let config = BibframeConfig::default();

    c.bench_function("serialize_1k_to_bibframe_turtle", |b| {
        b.iter(|| {
            let mut count = 0;
            for record in &records {
                let graph = marc_to_bibframe(record, &config);
                if graph.serialize(RdfFormat::Turtle).is_ok() {
                    count += 1;
                }
            }
            count
        });
    });
}

/// Benchmark the single-thread parser-pool path over 1,000 records.
///
/// Drives the boundary scan and `parse_batch_parallel` exactly as the
/// producer-consumer pipeline does, inside a one-worker rayon pool so the
/// measurement is a deterministic single-thread instruction count rather
/// than a parallel-throughput number.
fn benchmark_parser_pool_single_thread_1k(c: &mut Criterion) {
    let buffer = black_box(load_fixture("1k_records.mrc"));
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .expect("failed to build single-thread rayon pool");

    c.bench_function("parser_pool_single_thread_1k", |b| {
        b.iter(|| {
            pool.install(|| {
                let mut scanner = RecordBoundaryScanner::new();
                let boundaries = scanner.scan(&buffer).unwrap();
                let records = parse_batch_parallel(&boundaries, &buffer).unwrap();
                records.len()
            })
        });
    });
}

criterion_group!(
    benches,
    benchmark_serialize_to_csv_1k,
    benchmark_serialize_to_marcjson_1k,
    benchmark_parse_from_marcjson_1k,
    benchmark_serialize_to_mods_1k,
    benchmark_parse_from_mods_1k,
    benchmark_serialize_to_dublin_core_1k,
    benchmark_serialize_to_bibframe_turtle_1k,
    benchmark_parser_pool_single_thread_1k,
);
criterion_main!(benches);
