use arrow::array::{StringBuilder, UInt16Builder, UInt32Builder};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
/// Rust-only Arrow benchmark for MARC to columnar conversion
use mrrc::{MarcReader, Record};
use std::fs::File;
use std::sync::Arc;
use std::time::Instant;

/// Convert MARC records to Arrow columnar format (long format)
fn marc_to_arrow(records: &[Record]) -> RecordBatch {
    let mut record_ids = UInt32Builder::new();
    let mut field_tags = StringBuilder::new();
    let mut indicator1s = StringBuilder::new();
    let mut indicator2s = StringBuilder::new();
    let mut subfield_codes = StringBuilder::new();
    let mut subfield_values = StringBuilder::new();
    let mut field_sequences = UInt16Builder::new();
    let mut subfield_sequences = UInt16Builder::new();

    for (record_id, record) in records.iter().enumerate() {
        let record_id = record_id as u32 + 1;

        // Iterate over all fields (both control and variable fields)
        for (field_seq, (tag, field_list)) in record.fields.iter().enumerate() {
            if field_list.is_empty() {
                // Control field
                if let Some(value) = record.control_fields.get(tag) {
                    record_ids.append_value(record_id);
                    field_tags.append_value(tag);
                    indicator1s.append_null();
                    indicator2s.append_null();
                    subfield_codes.append_null();
                    subfield_values.append_value(value);
                    field_sequences.append_value(field_seq as u16);
                    subfield_sequences.append_null();
                }
            } else {
                // Variable fields
                for field in field_list {
                    let ind1 = field.indicator1.to_string();
                    let ind2 = field.indicator2.to_string();

                    if field.subfields.is_empty() {
                        // Empty field
                        record_ids.append_value(record_id);
                        field_tags.append_value(&field.tag);
                        indicator1s.append_value(&ind1);
                        indicator2s.append_value(&ind2);
                        subfield_codes.append_null();
                        subfield_values.append_value("");
                        field_sequences.append_value(field_seq as u16);
                        subfield_sequences.append_null();
                    } else {
                        // Field with subfields
                        for (subf_seq, subfield) in field.subfields.iter().enumerate() {
                            record_ids.append_value(record_id);
                            field_tags.append_value(&field.tag);
                            indicator1s.append_value(&ind1);
                            indicator2s.append_value(&ind2);
                            subfield_codes.append_value(&subfield.code.to_string());
                            subfield_values.append_value(&subfield.value);
                            field_sequences.append_value(field_seq as u16);
                            subfield_sequences.append_value(subf_seq as u16);
                        }
                    }
                }
            }
        }
    }

    let schema = Arc::new(Schema::new(vec![
        Field::new("record_id", DataType::UInt32, false),
        Field::new("field_tag", DataType::Utf8, false),
        Field::new("indicator1", DataType::Utf8, true),
        Field::new("indicator2", DataType::Utf8, true),
        Field::new("subfield_code", DataType::Utf8, true),
        Field::new("subfield_value", DataType::Utf8, false),
        Field::new("field_sequence", DataType::UInt16, false),
        Field::new("subfield_sequence", DataType::UInt16, true),
    ]));

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(record_ids.finish()),
            Arc::new(field_tags.finish()),
            Arc::new(indicator1s.finish()),
            Arc::new(indicator2s.finish()),
            Arc::new(subfield_codes.finish()),
            Arc::new(subfield_values.finish()),
            Arc::new(field_sequences.finish()),
            Arc::new(subfield_sequences.finish()),
        ],
    )
    .expect("Failed to create RecordBatch")
}

fn main() {
    println!("\n════════════════════════════════════════════════════════════════════");
    println!("POLARS + ARROW EVALUATION (RUST-ONLY)");
    println!("════════════════════════════════════════════════════════════════════\n");

    // Load test data
    let file = File::open("tests/data/fixtures/10k_records.mrc").expect("Test data not found");
    let mut reader = MarcReader::new(file);
    let mut records = Vec::new();
    while let Some(record) = reader.read_record().expect("Failed to read record") {
        records.push(record);
    }

    println!("✓ Loaded {} records\n", records.len());

    // Benchmark: MARC to Arrow serialization
    println!("MARC → Arrow (10k records):");
    let start = Instant::now();
    let arrow_table = marc_to_arrow(&records);
    let duration = start.elapsed();
    let ms = duration.as_secs_f64() * 1000.0;
    let throughput = records.len() as f64 / duration.as_secs_f64();

    println!("  Time: {:.1} ms", ms);
    println!("  Throughput: {:.0} rec/sec", throughput);
    println!(
        "  Arrow table: {} rows × {} cols",
        arrow_table.num_rows(),
        arrow_table.num_columns()
    );

    // Estimate memory size (rough approximation)
    let mem_mb = (arrow_table.num_rows() * 100) as f64 / (1024.0 * 1024.0); // rough estimate
    println!("  Est. memory: ~{:.1} MB\n", mem_mb);

    // Benchmark: Column access
    println!("Column Operations:");
    let start = Instant::now();
    let col = arrow_table.column(1); // field_tag column
    let _ = col.len();
    let duration = start.elapsed();
    println!(
        "  Column access (field_tag): {:.2} μs\n",
        duration.as_secs_f64() * 1_000_000.0
    );

    println!("════════════════════════════════════════════════════════════════════");
    println!("BENCHMARK COMPLETE");
    println!("════════════════════════════════════════════════════════════════════\n");
}
