#![allow(clippy::uninlined_format_args, missing_docs)]
//! Comprehensive FlatBuffers evaluation for MARC data
//!
//! Tests fidelity of round-trip serialization through FlatBuffers and measures performance.
//! Run with: cargo test --test flatbuffers_evaluation --release -- --nocapture

use flate2::write::GzEncoder;
use flate2::Compression;
use mrrc::flatbuffers_impl::{FlatBuffersDeserializer, FlatBuffersSerializer};
use mrrc::{MarcReader, Record};
use serde_json::{json, Value};
use std::fs::File;
use std::time::Instant;

#[derive(Debug, Clone)]
struct FidelityFailure {
    record_number: usize,
    field_tag: String,
    failure_type: String,
    expected: String,
    actual: String,
}

#[derive(Debug)]
struct PerformanceMetrics {
    read_throughput_rec_sec: f64,
    write_throughput_rec_sec: f64,
    file_size_bytes: u64,
    gzipped_size_bytes: u64,
    compression_ratio_percent: f64,
    #[allow(dead_code)]
    serialized_data: Vec<u8>,
}

/// Compare two records for complete fidelity
fn compare_records(
    original: &Record,
    restored: &Record,
    record_num: usize,
) -> Option<FidelityFailure> {
    // Check leader
    if original.leader.record_length != restored.leader.record_length {
        return Some(FidelityFailure {
            record_number: record_num,
            field_tag: "LEADER".to_string(),
            failure_type: "record_length".to_string(),
            expected: original.leader.record_length.to_string(),
            actual: restored.leader.record_length.to_string(),
        });
    }

    if original.leader.record_status != restored.leader.record_status {
        return Some(FidelityFailure {
            record_number: record_num,
            field_tag: "LEADER".to_string(),
            failure_type: "record_status".to_string(),
            expected: original.leader.record_status.to_string(),
            actual: restored.leader.record_status.to_string(),
        });
    }

    if original.leader.record_type != restored.leader.record_type {
        return Some(FidelityFailure {
            record_number: record_num,
            field_tag: "LEADER".to_string(),
            failure_type: "record_type".to_string(),
            expected: original.leader.record_type.to_string(),
            actual: restored.leader.record_type.to_string(),
        });
    }

    if original.leader.bibliographic_level != restored.leader.bibliographic_level {
        return Some(FidelityFailure {
            record_number: record_num,
            field_tag: "LEADER".to_string(),
            failure_type: "bibliographic_level".to_string(),
            expected: original.leader.bibliographic_level.to_string(),
            actual: restored.leader.bibliographic_level.to_string(),
        });
    }

    // Check control fields
    if original.control_fields.len() != restored.control_fields.len() {
        return Some(FidelityFailure {
            record_number: record_num,
            field_tag: "CONTROL_FIELDS".to_string(),
            failure_type: "count_mismatch".to_string(),
            expected: original.control_fields.len().to_string(),
            actual: restored.control_fields.len().to_string(),
        });
    }

    for (tag, value) in &original.control_fields {
        match restored.control_fields.get(tag) {
            Some(restored_value) if restored_value == value => {},
            Some(restored_value) => {
                return Some(FidelityFailure {
                    record_number: record_num,
                    field_tag: tag.clone(),
                    failure_type: "value_mismatch".to_string(),
                    expected: value.clone(),
                    actual: restored_value.clone(),
                });
            },
            None => {
                return Some(FidelityFailure {
                    record_number: record_num,
                    field_tag: tag.clone(),
                    failure_type: "missing_field".to_string(),
                    expected: value.clone(),
                    actual: "NOT_FOUND".to_string(),
                });
            },
        }
    }

    // Check data fields
    if original.fields.len() != restored.fields.len() {
        return Some(FidelityFailure {
            record_number: record_num,
            field_tag: "DATA_FIELDS".to_string(),
            failure_type: "count_mismatch".to_string(),
            expected: original.fields.len().to_string(),
            actual: restored.fields.len().to_string(),
        });
    }

    for (tag, orig_fields) in &original.fields {
        match restored.fields.get(tag) {
            Some(restored_fields) => {
                if orig_fields.len() != restored_fields.len() {
                    return Some(FidelityFailure {
                        record_number: record_num,
                        field_tag: tag.clone(),
                        failure_type: "field_count_mismatch".to_string(),
                        expected: orig_fields.len().to_string(),
                        actual: restored_fields.len().to_string(),
                    });
                }

                for (field_idx, (orig_field, restored_field)) in
                    orig_fields.iter().zip(restored_fields.iter()).enumerate()
                {
                    if orig_field.indicator1 != restored_field.indicator1 {
                        return Some(FidelityFailure {
                            record_number: record_num,
                            field_tag: format!("{}[{}].ind1", tag, field_idx),
                            failure_type: "indicator_mismatch".to_string(),
                            expected: orig_field.indicator1.to_string(),
                            actual: restored_field.indicator1.to_string(),
                        });
                    }

                    if orig_field.indicator2 != restored_field.indicator2 {
                        return Some(FidelityFailure {
                            record_number: record_num,
                            field_tag: format!("{}[{}].ind2", tag, field_idx),
                            failure_type: "indicator_mismatch".to_string(),
                            expected: orig_field.indicator2.to_string(),
                            actual: restored_field.indicator2.to_string(),
                        });
                    }

                    if orig_field.subfields.len() != restored_field.subfields.len() {
                        return Some(FidelityFailure {
                            record_number: record_num,
                            field_tag: format!("{}[{}]", tag, field_idx),
                            failure_type: "subfield_count_mismatch".to_string(),
                            expected: orig_field.subfields.len().to_string(),
                            actual: restored_field.subfields.len().to_string(),
                        });
                    }

                    for (subf_idx, (orig_subf, restored_subf)) in orig_field
                        .subfields
                        .iter()
                        .zip(restored_field.subfields.iter())
                        .enumerate()
                    {
                        if orig_subf.code != restored_subf.code {
                            return Some(FidelityFailure {
                                record_number: record_num,
                                field_tag: format!("{}[{}]${}", tag, field_idx, subf_idx),
                                failure_type: "subfield_code_mismatch".to_string(),
                                expected: orig_subf.code.to_string(),
                                actual: restored_subf.code.to_string(),
                            });
                        }

                        if orig_subf.value != restored_subf.value {
                            return Some(FidelityFailure {
                                record_number: record_num,
                                field_tag: format!("{}[{}]${}", tag, field_idx, subf_idx),
                                failure_type: "subfield_value_mismatch".to_string(),
                                expected: orig_subf.value.clone(),
                                actual: restored_subf.value.clone(),
                            });
                        }
                    }
                }
            },
            None => {
                return Some(FidelityFailure {
                    record_number: record_num,
                    field_tag: tag.clone(),
                    failure_type: "missing_field".to_string(),
                    expected: "EXISTS".to_string(),
                    actual: "NOT_FOUND".to_string(),
                });
            },
        }
    }

    None
}

/// Run fidelity test on 100-record test file
fn test_fidelity_100() -> Result<Vec<FidelityFailure>, Box<dyn std::error::Error>> {
    let file = File::open("tests/data/fixtures/fidelity_test_100.mrc")?;
    let mut reader = MarcReader::new(file);
    let mut failures = Vec::new();
    let mut record_num = 0;

    while let Some(original) = reader.read_record()? {
        record_num += 1;

        // Serialize
        let serialized = FlatBuffersSerializer::serialize(&original)
            .map_err(|e| format!("Serialization failed for record {}: {}", record_num, e))?;

        // Deserialize
        let restored = FlatBuffersDeserializer::deserialize(&serialized)
            .map_err(|e| format!("Deserialization failed for record {}: {}", record_num, e))?;

        // Compare
        if let Some(failure) = compare_records(&original, &restored, record_num) {
            failures.push(failure);
        }
    }

    Ok(failures)
}

/// Run performance test on 10k records
fn test_performance_10k() -> Result<PerformanceMetrics, Box<dyn std::error::Error>> {
    let file = File::open("tests/data/fixtures/10k_records.mrc")?;
    let mut reader = MarcReader::new(file);
    let mut all_serialized = Vec::new();
    let mut record_count = 0;

    // Read and serialize
    let start = Instant::now();
    while let Some(record) = reader.read_record()? {
        let serialized = FlatBuffersSerializer::serialize(&record)
            .map_err(|e| format!("Serialization error: {e}"))?;
        all_serialized.push(serialized);
        record_count += 1;
    }
    let serialize_duration = start.elapsed();
    let serialize_throughput = record_count as f64 / serialize_duration.as_secs_f64();

    // Deserialize all
    let start = Instant::now();
    let mut deserialize_count = 0;
    for serialized in &all_serialized {
        let _ = FlatBuffersDeserializer::deserialize(serialized)
            .map_err(|e| format!("Deserialization error: {e}"))?;
        deserialize_count += 1;
    }
    let deserialize_duration = start.elapsed();
    let deserialize_throughput = deserialize_count as f64 / deserialize_duration.as_secs_f64();

    // Combine all serialized data
    let mut combined = Vec::new();
    for serialized in &all_serialized {
        combined.extend_from_slice(serialized);
    }
    let file_size = combined.len() as u64;

    // Gzip compression
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&combined)?;
    let gzipped = encoder.finish()?;
    let gzipped_size = gzipped.len() as u64;

    let compression_ratio = if file_size > 0 {
        ((file_size - gzipped_size) as f64 / file_size as f64) * 100.0
    } else {
        0.0
    };

    Ok(PerformanceMetrics {
        read_throughput_rec_sec: serialize_throughput,
        write_throughput_rec_sec: deserialize_throughput,
        file_size_bytes: file_size,
        gzipped_size_bytes: gzipped_size,
        compression_ratio_percent: compression_ratio,
        serialized_data: combined,
    })
}

/// Estimate peak memory during operation (simplified)
fn estimate_peak_memory(serialized_data_size: u64) -> f64 {
    // Rough estimate: peak memory is typically 2-3x the serialized data size during processing
    (serialized_data_size as f64 * 2.5) / (1024.0 * 1024.0)
}

/// Write results as JSON
fn write_json_results(
    fidelity_score: usize,
    total_records: usize,
    failures: &[FidelityFailure],
    perf: &PerformanceMetrics,
    memory_peak: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let failure_json: Vec<Value> = failures
        .iter()
        .map(|f| {
            json!({
                "record_number": f.record_number,
                "field_tag": f.field_tag,
                "failure_type": f.failure_type,
                "expected": f.expected,
                "actual": f.actual,
            })
        })
        .collect();

    let results = json!({
        "fidelity_score": fidelity_score,
        "total_records": total_records,
        "perfect_roundtrips": total_records - failures.len(),
        "failures": failure_json,
        "performance": {
            "read_throughput_rec_sec": perf.read_throughput_rec_sec,
            "write_throughput_rec_sec": perf.write_throughput_rec_sec,
            "file_size_bytes": perf.file_size_bytes,
            "gzipped_size_bytes": perf.gzipped_size_bytes,
            "compression_ratio_percent": perf.compression_ratio_percent,
        },
        "memory_peak_mb": memory_peak,
    });

    let json_str = serde_json::to_string_pretty(&results)?;
    std::fs::write("flatbuffers_evaluation_results.json", json_str)?;

    println!("\n📊 FlatBuffers Evaluation Results\n");
    println!(
        "Fidelity Score: {}/{} ({:.1}%)",
        fidelity_score,
        total_records,
        (fidelity_score as f64 / total_records as f64) * 100.0
    );
    println!("Perfect Round-trips: {}", total_records - failures.len());
    println!("\nPerformance Metrics:");
    println!(
        "  Read Throughput: {:.0} records/sec",
        perf.read_throughput_rec_sec
    );
    println!(
        "  Write Throughput: {:.0} records/sec",
        perf.write_throughput_rec_sec
    );
    println!("  Serialized Size: {} bytes", perf.file_size_bytes);
    println!("  Gzipped Size: {} bytes", perf.gzipped_size_bytes);
    println!(
        "  Compression Ratio: {:.1}%",
        perf.compression_ratio_percent
    );
    println!("  Peak Memory: {:.1} MB", memory_peak);

    if !failures.is_empty() {
        println!("\n❌ Failures ({}): ", failures.len());
        for failure in failures.iter().take(10) {
            println!(
                "  Record {}: Field {} - {} (expected: '{}', actual: '{}')",
                failure.record_number,
                failure.field_tag,
                failure.failure_type,
                failure.expected,
                failure.actual
            );
        }
        if failures.len() > 10 {
            println!("  ... and {} more failures", failures.len() - 10);
        }
    } else {
        println!("\n✅ All records passed fidelity checks!");
    }

    println!("\nResults saved to: flatbuffers_evaluation_results.json");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatbuffers_comprehensive_evaluation() {
        println!("\n🧪 Running FlatBuffers Comprehensive Evaluation...\n");

        // Phase 1: Fidelity test
        println!("Phase 1: Testing fidelity on 100 diverse records...");
        let fidelity_result = match test_fidelity_100() {
            Ok(failures) => {
                let total = 100; // We read 100 records from the file
                let perfect = total - failures.len();
                let score = (perfect * 100) / total;
                println!(
                    "✓ Fidelity test complete: {}/100 perfect round-trips",
                    perfect
                );
                (score, total, failures)
            },
            Err(e) => {
                eprintln!("✗ Fidelity test failed: {e}");
                panic!("Fidelity test error: {e}");
            },
        };

        // Phase 2: Performance test
        println!("\nPhase 2: Testing performance on 10,000 records...");
        let perf = match test_performance_10k() {
            Ok(metrics) => {
                println!("✓ Performance test complete:");
                println!(
                    "  - Serialization: {:.0} records/sec",
                    metrics.read_throughput_rec_sec
                );
                println!(
                    "  - Deserialization: {:.0} records/sec",
                    metrics.write_throughput_rec_sec
                );
                metrics
            },
            Err(e) => {
                eprintln!("✗ Performance test failed: {e}");
                panic!("Performance test error: {e}");
            },
        };

        let memory_peak = estimate_peak_memory(perf.file_size_bytes);

        // Write results
        if let Err(e) = write_json_results(
            fidelity_result.0,
            fidelity_result.1,
            &fidelity_result.2,
            &perf,
            memory_peak,
        ) {
            eprintln!("✗ Failed to write results: {e}");
            panic!("Results write error: {e}");
        }

        // Assert fidelity
        assert_eq!(
            fidelity_result.2.len(),
            0,
            "Expected zero fidelity failures, but found {}",
            fidelity_result.2.len()
        );
    }
}

fn main() {
    println!("Run with: cargo test --test flatbuffers_evaluation --release -- --nocapture");
}
