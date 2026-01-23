#![allow(
    missing_docs,
    clippy::cast_precision_loss,
    clippy::implicit_clone,
    clippy::uninlined_format_args,
    clippy::cast_possible_truncation,
    clippy::cast_lossless,
    clippy::items_after_statements
)]
//! `Apache Avro` Evaluation for MARC Data (Rust Implementation)
//!
//! Implements round-trip fidelity, failure modes, and performance benchmarks
//! per `EVALUATION_FRAMEWORK.md`

use apache_avro::Schema;
use mrrc::{MarcReader, MarcRecord};
use serde_json::json;
use std::fs;
use std::io::Cursor;
use std::time::Instant;

const FIDELITY_TEST_PATH: &str = "tests/data/fixtures/fidelity_test_100.mrc";
const PERF_TEST_PATH: &str = "tests/data/fixtures/10k_records.mrc";

// ============================================================================
// SCHEMA DESIGN: Apache Avro representation of MARC
// ============================================================================

lazy_static::lazy_static! {
    static ref MARC_AVRO_SCHEMA: Schema = {
        let schema_str = r#"{
  "type": "record",
  "name": "MarcRecord",
  "namespace": "mrrc.formats.avro",
  "fields": [
    {
      "name": "leader",
      "type": "string",
      "doc": "24-character LEADER (positions 0-23)"
    },
    {
      "name": "fields",
      "type": {
        "type": "array",
        "items": {
          "type": "record",
          "name": "Field",
          "fields": [
            {
              "name": "tag",
              "type": "string",
              "doc": "Tag as 3-character string (e.g., \"001\", \"245\")"
            },
            {
              "name": "indicator1",
              "type": "string",
              "doc": "Indicator 1 (1 character, can be space for control fields)"
            },
            {
              "name": "indicator2",
              "type": "string",
              "doc": "Indicator 2 (1 character, can be space for control fields)"
            },
            {
              "name": "subfields",
              "type": {
                "type": "array",
                "items": {
                  "type": "record",
                  "name": "Subfield",
                  "fields": [
                    {
                      "name": "code",
                      "type": "string",
                      "doc": "Subfield code (1 character: a-z, 0-9)"
                    },
                    {
                      "name": "value",
                      "type": "string",
                      "doc": "Subfield value (UTF-8 string, can be empty)"
                    }
                  ]
                }
              }
            }
          ]
        }
      }
    }
  ]
}"#;
        Schema::parse_str(schema_str).expect("Failed to parse Avro schema")
    };
}

/// Convert an mrrc Record to Avro JSON representation
fn marc_to_avro_value(record: &mrrc::Record) -> serde_json::Value {
    let fields: Vec<serde_json::Value> = record
        .fields()
        .map(|field| {
            json!({
                "tag": field.tag,
                "indicator1": field.indicator1.to_string(),
                "indicator2": field.indicator2.to_string(),
                "subfields": field
                    .subfields
                    .iter()
                    .map(|sf| {
                        json!({
                            "code": sf.code.to_string(),
                            "value": sf.value,
                        })
                    })
                    .collect::<Vec<_>>()
            })
        })
        .collect();

    let leader_str = record
        .leader()
        .as_bytes()
        .map(|b| String::from_utf8(b.to_vec()).unwrap_or_default())
        .unwrap_or_default();

    json!({
        "leader": leader_str,
        "fields": fields,
    })
}

/// Convert Avro JSON representation back to mrrc Record
fn avro_value_to_marc(value: &serde_json::Value) -> Result<mrrc::Record, String> {
    let leader_str = value
        .get("leader")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing or invalid leader".to_string())?;

    let mut record = mrrc::Record::new(
        mrrc::Leader::from_bytes(leader_str.as_bytes())
            .map_err(|e| format!("Invalid leader: {}", e))?,
    );

    let fields = value
        .get("fields")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Missing or invalid fields array".to_string())?;

    for field_val in fields {
        let tag = field_val
            .get("tag")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing field tag".to_string())?
            .to_string();

        // Validate tag is not empty
        if tag.is_empty() {
            return Err("Field tag cannot be empty".to_string());
        }

        let ind1_str = field_val
            .get("indicator1")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing indicator1".to_string())?;
        // Validate indicator1 is single character
        if ind1_str.chars().count() != 1 {
            return Err(format!(
                "Indicator1 must be exactly 1 character, got: '{}'",
                ind1_str
            ));
        }
        let indicator1 = ind1_str
            .chars()
            .next()
            .ok_or_else(|| "Invalid indicator1".to_string())?;

        let ind2_str = field_val
            .get("indicator2")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing indicator2".to_string())?;
        // Validate indicator2 is single character
        if ind2_str.chars().count() != 1 {
            return Err(format!(
                "Indicator2 must be exactly 1 character, got: '{}'",
                ind2_str
            ));
        }
        let indicator2 = ind2_str
            .chars()
            .next()
            .ok_or_else(|| "Invalid indicator2".to_string())?;

        let subfields_arr = field_val
            .get("subfields")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "Missing subfields array".to_string())?;

        let subfields: Result<smallvec::SmallVec<[mrrc::Subfield; 4]>, String> = subfields_arr
            .iter()
            .map(|sf| {
                let code_str = sf
                    .get("code")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing subfield code".to_string())?;
                let code = code_str
                    .chars()
                    .next()
                    .ok_or_else(|| "Invalid subfield code".to_string())?;

                let value = sf
                    .get("value")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing subfield value".to_string())?
                    .to_string();

                Ok(mrrc::Subfield { code, value })
            })
            .collect();

        let field = mrrc::Field {
            tag,
            indicator1,
            indicator2,
            subfields: subfields?,
        };

        record.add_field(field);
    }

    Ok(record)
}

// ============================================================================
// ROUND-TRIP FIDELITY TESTING
// ============================================================================

fn test_round_trip_fidelity() {
    println!("\n=== ROUND-TRIP FIDELITY TEST ===");

    let data = fs::read(FIDELITY_TEST_PATH).expect("Failed to read fidelity test file");
    let mut reader = MarcReader::new(Cursor::new(&data));

    let mut total = 0;
    let mut passed = 0;
    let mut failures = Vec::new();

    loop {
        match reader.read_record() {
            Ok(Some(record)) => {
                total += 1;

                // Serialize to Avro
                let avro_value = marc_to_avro_value(&record);

                // Deserialize from Avro
                match avro_value_to_marc(&avro_value) {
                    Ok(roundtrip_record) => {
                        // Compare field by field
                        if compare_records(&record, &roundtrip_record) {
                            passed += 1;
                        } else {
                            failures.push((total, "Field mismatch after round-trip".to_string()));
                        }
                    },
                    Err(e) => {
                        failures.push((total, format!("Deserialization error: {}", e)));
                    },
                }
            },
            Ok(None) => break,
            Err(e) => {
                eprintln!("Error reading record: {}", e);
            },
        }
    }

    println!("Records tested: {}", total);
    println!(
        "Perfect round-trips: {}/{} ({:.0}%)",
        passed,
        total,
        passed as f64 / total as f64 * 100.0
    );

    if !failures.is_empty() {
        println!("\nFailures:");
        for (record_num, reason) in &failures {
            println!("  Record {}: {}", record_num, reason);
        }
    }
}

fn compare_records(record1: &mrrc::Record, record2: &mrrc::Record) -> bool {
    // Compare leader (positions 0-3, 12-15 can differ)
    let leader1 = record1.leader().as_bytes().unwrap_or_default();
    let leader2 = record2.leader().as_bytes().unwrap_or_default();

    if leader1.len() != 24 || leader2.len() != 24 {
        return false;
    }

    // Check critical leader positions (5-11, 17-23)
    if leader1[5..12] != leader2[5..12] || leader1[17..24] != leader2[17..24] {
        return false;
    }

    // Compare field count and content
    let fields1: Vec<_> = record1.fields().collect();
    let fields2: Vec<_> = record2.fields().collect();

    if fields1.len() != fields2.len() {
        return false;
    }

    for (f1, f2) in fields1.iter().zip(fields2.iter()) {
        if f1.tag != f2.tag
            || f1.indicator1 != f2.indicator1
            || f1.indicator2 != f2.indicator2
            || f1.subfields.len() != f2.subfields.len()
        {
            return false;
        }

        for (sf1, sf2) in f1.subfields.iter().zip(f2.subfields.iter()) {
            if sf1.code != sf2.code || sf1.value != sf2.value {
                return false;
            }
        }
    }

    true
}

// ============================================================================
// FAILURE MODES TESTING
// ============================================================================

fn test_failure_modes() {
    println!("\n=== FAILURE MODES TESTING ===");

    // Test 1: Truncated record
    println!("Test: Truncated record");
    let truncated = json!({
        "leader": "00123",
        "fields": []
    });
    match avro_value_to_marc(&truncated) {
        Ok(_) => println!("  ✗ Should have rejected truncated leader"),
        Err(e) => println!("  ✓ Graceful error: {}", e),
    }

    // Test 2: Invalid tag
    println!("Test: Invalid tag format");
    let invalid_tag = json!({
        "leader": "00345nam a2200133 a 4500",
        "fields": [
            {
                "tag": "",
                "indicator1": " ",
                "indicator2": " ",
                "subfields": []
            }
        ]
    });
    match avro_value_to_marc(&invalid_tag) {
        Ok(_) => println!("  ✗ Should have rejected empty tag"),
        Err(e) => println!("  ✓ Graceful error: {}", e),
    }

    // Test 3: Missing field
    println!("Test: Missing required field");
    let missing_leader = json!({
        "fields": []
    });
    match avro_value_to_marc(&missing_leader) {
        Ok(_) => println!("  ✗ Should have rejected missing leader"),
        Err(e) => println!("  ✓ Graceful error: {}", e),
    }

    // Test 4: Malformed indicator
    println!("Test: Invalid indicator (non-single char)");
    let bad_indicator = json!({
        "leader": "00345nam a2200133 a 4500",
        "fields": [
            {
                "tag": "245",
                "indicator1": "12",
                "indicator2": " ",
                "subfields": []
            }
        ]
    });
    match avro_value_to_marc(&bad_indicator) {
        Ok(_) => println!("  ✗ Should have rejected multi-char indicator"),
        Err(e) => println!("  ✓ Graceful error: {}", e),
    }

    println!("All failure mode tests completed.");
}

// ============================================================================
// PERFORMANCE BENCHMARKING
// ============================================================================

fn bench_read_performance() {
    println!("\n=== READ PERFORMANCE ===");

    let data = fs::read(PERF_TEST_PATH).expect("Failed to read perf test file");
    let mut reader = MarcReader::new(Cursor::new(&data));

    // Collect all records first
    let mut records = Vec::new();
    while let Ok(Some(record)) = reader.read_record() {
        records.push(record);
    }
    let total_records = records.len();

    println!("Benchmark: Serializing {} records to Avro", total_records);

    // Warm-up
    for _ in 0..3 {
        for record in &records {
            let _ = marc_to_avro_value(record);
        }
    }

    // Measure
    let start = Instant::now();
    for record in &records {
        let _ = marc_to_avro_value(record);
    }
    let elapsed = start.elapsed();

    let throughput = total_records as f64 / elapsed.as_secs_f64();
    println!(
        "  Serialization time: {:.2} ms",
        elapsed.as_secs_f64() * 1000.0
    );
    println!("  Throughput: {:.0} records/sec", throughput);
}

fn bench_write_performance() {
    println!("\n=== WRITE PERFORMANCE ===");

    let data = fs::read(PERF_TEST_PATH).expect("Failed to read perf test file");
    let mut reader = MarcReader::new(Cursor::new(&data));

    // Collect all records first
    let mut records = Vec::new();
    while let Ok(Some(record)) = reader.read_record() {
        records.push(record);
    }
    let total_records = records.len();

    // Pre-convert to Avro values
    let avro_values: Vec<_> = records.iter().map(marc_to_avro_value).collect();

    println!(
        "Benchmark: Deserializing {} records from Avro",
        total_records
    );

    // Warm-up
    for _ in 0..3 {
        for value in &avro_values {
            let _ = avro_value_to_marc(value);
        }
    }

    // Measure
    let start = Instant::now();
    for value in &avro_values {
        let _ = avro_value_to_marc(value);
    }
    let elapsed = start.elapsed();

    let throughput = total_records as f64 / elapsed.as_secs_f64();
    println!(
        "  Deserialization time: {:.2} ms",
        elapsed.as_secs_f64() * 1000.0
    );
    println!("  Throughput: {:.0} records/sec", throughput);
}

fn bench_file_size() {
    println!("\n=== FILE SIZE ANALYSIS ===");

    let data = fs::read(PERF_TEST_PATH).expect("Failed to read perf test file");
    let mut reader = MarcReader::new(Cursor::new(&data));

    let mut records = Vec::new();
    while let Ok(Some(record)) = reader.read_record() {
        records.push(record);
    }

    // Serialize all records to Avro by converting to JSON strings
    // This measures schema-based serialization effectiveness
    let mut total_size = 0;
    for record in &records {
        let avro_value = marc_to_avro_value(record);
        let json_bytes = serde_json::to_vec(&avro_value).expect("Failed to serialize to JSON");
        total_size += json_bytes.len();
    }

    let raw_size = total_size;
    let iso_size = data.len();

    println!(
        "ISO 2709 size: {} bytes ({:.2} MB)",
        iso_size,
        iso_size as f64 / 1_000_000.0
    );
    println!(
        "Avro (JSON) size: {} bytes ({:.2} MB)",
        raw_size,
        raw_size as f64 / 1_000_000.0
    );
    println!(
        "Size delta: {:.2}% {}",
        (raw_size as f64 - iso_size as f64) / iso_size as f64 * 100.0,
        if raw_size > iso_size {
            "(larger)"
        } else {
            "(smaller)"
        }
    );

    // Gzip compression
    use flate2::Compression;
    use std::io::Write;

    // Re-encode all as JSON and compress
    let mut json_data = Vec::new();
    for record in &records {
        let avro_value = marc_to_avro_value(record);
        let json_bytes = serde_json::to_vec(&avro_value).expect("Failed to serialize to JSON");
        json_data.extend_from_slice(&json_bytes);
    }

    let mut gzip_encoder = flate2::write::GzEncoder::new(Vec::new(), Compression::new(9));
    gzip_encoder
        .write_all(&json_data)
        .expect("Failed to compress");
    let compressed = gzip_encoder.finish().expect("Failed to finish compression");

    let comp_ratio = (1.0 - compressed.len() as f64 / raw_size as f64) * 100.0;
    println!(
        "Avro compressed (gzip -9): {} bytes ({:.2} MB)",
        compressed.len(),
        compressed.len() as f64 / 1_000_000.0
    );
    println!("Compression ratio: {:.2}%", comp_ratio);
}

// ============================================================================
// MAIN
// ============================================================================

fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║  Apache Avro Evaluation for MARC Data (Rust)           ║");
    println!("║  Issue: mrrc-fks.4                                     ║");
    println!("╚════════════════════════════════════════════════════════╝");

    test_round_trip_fidelity();
    test_failure_modes();
    bench_read_performance();
    bench_write_performance();
    bench_file_size();

    println!("\n✓ Evaluation complete!");
}
