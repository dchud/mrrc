use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mrrc::{MarcReader, MarcWriter, json, xml};
use std::io::Cursor;

// Load test fixtures
fn load_fixture(filename: &str) -> Vec<u8> {
    let path = format!("tests/data/fixtures/{}", filename);
    std::fs::read(&path).expect(&format!("Failed to load fixture: {}", path))
}

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
        })
    });
}

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
        })
    });
}

fn benchmark_read_100k(c: &mut Criterion) {
    let fixture = black_box(load_fixture("100k_records.mrc"));
    
    let mut group = c.benchmark_group("slow");
    group.sample_size(10); // Reduce samples for slow tests
    
    group.bench_function("read_100k_records", |b| {
        b.iter(|| {
            let cursor = Cursor::new(fixture.clone());
            let mut reader = MarcReader::new(cursor);
            let mut count = 0;
            while let Ok(Some(_record)) = reader.read_record() {
                count += 1;
            }
            count
        })
    });
    group.finish();
}

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
        })
    });
}

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
        })
    });
}

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
        })
    });
}

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
        })
    });
}

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
        })
    });
}

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
        })
    });
}

criterion_group!(
    benches,
    benchmark_read_1k,
    benchmark_read_10k,
    benchmark_read_100k,
    benchmark_read_with_field_access_1k,
    benchmark_read_with_field_access_10k,
    benchmark_serialization_to_json_1k,
    benchmark_serialization_to_xml_1k,
    benchmark_roundtrip_1k,
    benchmark_roundtrip_10k,
);
criterion_main!(benches);
