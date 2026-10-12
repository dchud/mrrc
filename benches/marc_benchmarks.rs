#![allow(missing_docs, unused_doc_comments, unused_attributes)]
//! Benchmarks for MRRC MARC library.
//!
//! This benchmark suite tests the performance of reading, writing, and processing
//! MARC records using Criterion.rs for statistical analysis.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use mrrc::{LinkageInfo, MarcReader, MarcWriter, Record, RecordHelpers, json, marcxml};
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

/// Read every record in a fixture held in memory and return the count.
fn read_all(fixture: &[u8]) -> usize {
    let mut reader = MarcReader::new(Cursor::new(fixture.to_vec()));
    let mut count = 0;
    while let Ok(Some(_record)) = reader.read_record() {
        count += 1;
    }
    count
}

/// Parse every record in a fixture once, for benches that time work on
/// parsed records.
fn parse_fixture(filename: &str) -> Vec<Record> {
    let mut reader = MarcReader::new(Cursor::new(load_fixture(filename)));
    let mut records = Vec::new();
    while let Ok(Some(record)) = reader.read_record() {
        records.push(record);
    }
    records
}

/// Benchmark reading the realistic fixture: 2,000 records of 12-30 fields
/// with real-length text, so parsing rather than per-record overhead
/// dominates.
fn benchmark_read_realistic(c: &mut Criterion) {
    let fixture = black_box(load_fixture("realistic.mrc"));
    c.bench_function("read_realistic_records", |b| b.iter(|| read_all(&fixture)));
}

/// Benchmark reading 100 large records (150-250 fields each).
fn benchmark_read_large(c: &mut Criterion) {
    let fixture = black_box(load_fixture("large_records.mrc"));
    c.bench_function("read_large_records", |b| b.iter(|| read_all(&fixture)));
}

/// Benchmark reading 10 very large records (800-1,000 fields each).
fn benchmark_read_very_large(c: &mut Criterion) {
    let fixture = black_box(load_fixture("very_large_records.mrc"));
    c.bench_function("read_very_large_records", |b| b.iter(|| read_all(&fixture)));
}

/// Benchmark reading 1,000 MARC-8 records (leader position 09 blank) with
/// ANSEL diacritics and Cyrillic and Greek text reached through escape
/// sequences. The only fixture that exercises MARC-8 decoding.
fn benchmark_read_marc8(c: &mut Criterion) {
    let fixture = black_box(load_fixture("realistic_marc8.mrc"));
    c.bench_function("read_marc8_records", |b| b.iter(|| read_all(&fixture)));
}

/// Data-field tags the BIBFRAME and MODS converters look up on each record.
/// Several are absent from most records, so those lookups miss.
const LOOKUP_TAGS: &[&str] = &[
    "010", "020", "022", "024", "035", "041", "050", "082", "100", "110", "111", "240", "245",
    "246", "250", "260", "264", "300", "310", "490", "500", "504", "505", "520", "600", "610",
    "611", "630", "650", "651", "655", "700", "710", "711", "730", "830", "852", "856", "880",
];

/// Look up every tag in [`LOOKUP_TAGS`] on every record, the way a
/// converter does, through `get_field`, `fields_by_tag`, and
/// `get_control_field`.
fn lookup_tags(records: &[Record]) -> usize {
    let mut found = 0;
    for record in records {
        found += usize::from(record.get_control_field(black_box("001")).is_some());
        for tag in LOOKUP_TAGS {
            let tag = black_box(*tag);
            if let Some(field) = record.get_field(tag) {
                found += field.subfields().count();
            }
            found += record.fields_by_tag(tag).count();
        }
    }
    found
}

/// Benchmark tag lookups on the realistic fixture.
fn benchmark_lookup_tags_realistic(c: &mut Criterion) {
    let records = parse_fixture("realistic.mrc");
    c.bench_function("lookup_tags_realistic", |b| {
        b.iter(|| lookup_tags(&records));
    });
}

/// Benchmark tag lookups on large records, where per-lookup cost that grows
/// with field count shows.
fn benchmark_lookup_tags_large(c: &mut Criterion) {
    let records = parse_fixture("large_records.mrc");
    c.bench_function("lookup_tags_large", |b| b.iter(|| lookup_tags(&records)));
}

/// Benchmark tag lookups on very large records.
fn benchmark_lookup_tags_very_large(c: &mut Criterion) {
    let records = parse_fixture("very_large_records.mrc");
    c.bench_function("lookup_tags_very_large", |b| {
        b.iter(|| lookup_tags(&records));
    });
}

/// Benchmark writing 100 large records to ISO 2709 binary: an in-order walk
/// of every field.
fn benchmark_write_large(c: &mut Criterion) {
    let records = parse_fixture("large_records.mrc");
    let mut output = Vec::with_capacity(2 << 20);
    c.bench_function("write_large_records", |b| {
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
    benchmark_read_realistic,
    benchmark_read_large,
    benchmark_read_very_large,
    benchmark_read_marc8,
    benchmark_lookup_tags_realistic,
    benchmark_lookup_tags_large,
    benchmark_lookup_tags_very_large,
    benchmark_write_large,
);
criterion_main!(benches);
