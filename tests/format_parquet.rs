#![allow(missing_docs)]
//! Parquet format tests for MARC records.
//!
//! Tests Parquet serialization/deserialization with fidelity verification
//! and failure mode handling.

use mrrc::parquet_impl;
use mrrc::{MarcReader, Record};
use std::fs;
use std::io::Cursor;
use tempfile::NamedTempFile;

mod common;
use common::create_test_leader;

#[test]
#[allow(clippy::too_many_lines, clippy::uninlined_format_args)]
fn test_parquet_roundtrip_fidelity_100_records() -> Result<(), Box<dyn std::error::Error>> {
    // Load fidelity_test_100.mrc
    let mrc_data = fs::read("tests/data/fixtures/fidelity_test_100.mrc")?;
    let mut reader = MarcReader::new(Cursor::new(&mrc_data));

    let mut original_records = Vec::new();
    while let Some(result) = reader.read_record()? {
        original_records.push(result);
    }

    let n_records = original_records.len();
    assert!(
        n_records >= 100,
        "Expected at least 100 records, got {}",
        n_records
    );

    // Serialize to Parquet
    let temp_parquet = NamedTempFile::new()?;
    let parquet_path = temp_parquet.path().to_string_lossy().to_string();

    parquet_impl::serialize_to_parquet(&original_records, &parquet_path)?;

    // Deserialize from Parquet
    let restored_records = parquet_impl::deserialize_from_parquet(&parquet_path)?;

    assert_eq!(
        original_records.len(),
        restored_records.len(),
        "Record count mismatch"
    );

    // Compare field-by-field
    let mut perfect_count = 0;

    for (idx, (orig, restored)) in original_records
        .iter()
        .zip(restored_records.iter())
        .enumerate()
    {
        // Compare leaders
        if orig.leader != restored.leader {
            eprintln!("Record {}: Leader mismatch", idx);
            continue;
        }

        // Compare control fields
        if orig.control_fields.len() != restored.control_fields.len() {
            eprintln!(
                "Record {}: Control field count mismatch: {} vs {}",
                idx,
                orig.control_fields.len(),
                restored.control_fields.len()
            );
            continue;
        }

        let mut cf_match = true;
        for (tag, value) in &orig.control_fields {
            if restored.control_fields.get(tag) != Some(value) {
                eprintln!("Record {}: Control field {} mismatch", idx, tag);
                cf_match = false;
                break;
            }
        }

        if !cf_match {
            continue;
        }

        // Compare data fields
        if orig.fields.len() != restored.fields.len() {
            eprintln!(
                "Record {}: Field count mismatch: {} vs {}",
                idx,
                orig.fields.len(),
                restored.fields.len()
            );
            continue;
        }

        let mut fields_match = true;
        for (tag, orig_field_list) in &orig.fields {
            match restored.fields.get(tag) {
                None => {
                    eprintln!("Record {}: Field {} missing in restored", idx, tag);
                    fields_match = false;
                    break;
                },
                Some(restored_field_list) => {
                    if orig_field_list.len() != restored_field_list.len() {
                        eprintln!("Record {}: Field {} count mismatch", idx, tag);
                        fields_match = false;
                        break;
                    }

                    for (field_idx, (orig_field, restored_field)) in orig_field_list
                        .iter()
                        .zip(restored_field_list.iter())
                        .enumerate()
                    {
                        if orig_field != restored_field {
                            eprintln!(
                                "Record {}: Field {} instance {} mismatch",
                                idx, tag, field_idx
                            );
                            fields_match = false;
                            break;
                        }
                    }
                },
            }

            if !fields_match {
                break;
            }
        }

        if fields_match {
            perfect_count += 1;
        }
    }

    #[allow(clippy::cast_precision_loss)]
    let percent = (perfect_count as f64 / n_records as f64) * 100.0;
    println!(
        "Parquet round-trip fidelity: {}/{} records perfect ({:.1}%)",
        perfect_count, n_records, percent
    );
    assert!(
        perfect_count >= n_records * 95 / 100,
        "Expected at least 95% perfect records, got {}/{}",
        perfect_count,
        n_records
    );

    Ok(())
}

#[test]
fn test_parquet_empty_records() -> Result<(), Box<dyn std::error::Error>> {
    let records: Vec<Record> = Vec::new();

    let temp_parquet = NamedTempFile::new()?;
    let parquet_path = temp_parquet.path().to_string_lossy().to_string();

    parquet_impl::serialize_to_parquet(&records, &parquet_path)?;
    let restored = parquet_impl::deserialize_from_parquet(&parquet_path)?;

    assert_eq!(restored.len(), 0);
    Ok(())
}

#[test]
fn test_parquet_single_record() -> Result<(), Box<dyn std::error::Error>> {
    let mut record = Record::new(create_test_leader());
    record.add_control_field("001".to_string(), "test12345".to_string());

    let mut field = mrrc::Field::new("245".to_string(), '1', '0');
    field.add_subfield('a', "Test Title".to_string());
    field.add_subfield('b', "Test Subtitle".to_string());
    record.add_field(field);

    let records = vec![record.clone()];

    let temp_parquet = NamedTempFile::new()?;
    let parquet_path = temp_parquet.path().to_string_lossy().to_string();

    parquet_impl::serialize_to_parquet(&records, &parquet_path)?;
    let restored = parquet_impl::deserialize_from_parquet(&parquet_path)?;

    assert_eq!(restored.len(), 1);
    assert_eq!(record.control_fields, restored[0].control_fields);
    assert_eq!(record.fields, restored[0].fields);

    Ok(())
}

#[test]
fn test_parquet_preserves_field_order() -> Result<(), Box<dyn std::error::Error>> {
    let mut record = Record::new(create_test_leader());

    // Add fields in specific order
    let mut field1 = mrrc::Field::new("100".to_string(), '1', ' ');
    field1.add_subfield('a', "Author Name".to_string());
    record.add_field(field1);

    let mut field2 = mrrc::Field::new("245".to_string(), '1', '0');
    field2.add_subfield('a', "Title".to_string());
    record.add_field(field2);

    let mut field3 = mrrc::Field::new("650".to_string(), ' ', '0');
    field3.add_subfield('a', "Subject".to_string());
    record.add_field(field3);

    let records = vec![record];

    let temp_parquet = NamedTempFile::new()?;
    let parquet_path = temp_parquet.path().to_string_lossy().to_string();

    parquet_impl::serialize_to_parquet(&records, &parquet_path)?;
    let restored = parquet_impl::deserialize_from_parquet(&parquet_path)?;

    let tags: Vec<_> = restored[0].fields.keys().cloned().collect();
    assert_eq!(tags, vec!["100", "245", "650"]);

    Ok(())
}

#[test]
fn test_parquet_handles_utf8_content() -> Result<(), Box<dyn std::error::Error>> {
    let mut record = Record::new(create_test_leader());

    // Add fields with UTF-8 content
    let mut field = mrrc::Field::new("245".to_string(), '1', '0');
    field.add_subfield('a', "Tïtlé with Ümlàuts".to_string());
    field.add_subfield('b', "中文字符和日本語テキスト".to_string());
    record.add_field(field);

    let records = vec![record];

    let temp_parquet = NamedTempFile::new()?;
    let parquet_path = temp_parquet.path().to_string_lossy().to_string();

    parquet_impl::serialize_to_parquet(&records, &parquet_path)?;
    let restored = parquet_impl::deserialize_from_parquet(&parquet_path)?;

    assert_eq!(restored.len(), 1);
    if let Some(fields) = restored[0].get_fields("245") {
        assert_eq!(fields.len(), 1);
        let subfields: Vec<_> = fields[0].subfields.iter().collect();
        assert_eq!(subfields.len(), 2);
        assert_eq!(subfields[0].value, "Tïtlé with Ümlàuts");
        assert_eq!(subfields[1].value, "中文字符和日本語テキスト");
    } else {
        panic!("Field 245 not found");
    }

    Ok(())
}

#[test]
fn test_parquet_file_size() -> Result<(), Box<dyn std::error::Error>> {
    // Load and serialize some records to check file size
    let mrc_data = fs::read("tests/data/fixtures/fidelity_test_100.mrc")?;
    let mut reader = MarcReader::new(Cursor::new(&mrc_data));

    let mut records = Vec::new();
    while let Some(record) = reader.read_record()? {
        records.push(record);
    }

    let temp_parquet = NamedTempFile::new()?;
    let parquet_path = temp_parquet.path().to_string_lossy().to_string();

    parquet_impl::serialize_to_parquet(&records, &parquet_path)?;

    #[allow(clippy::cast_precision_loss)]
    let parquet_size = fs::metadata(&parquet_path)?.len() as f64;
    #[allow(clippy::cast_precision_loss)]
    let mrc_size = mrc_data.len() as f64;

    println!(
        "File sizes: MRC: {} bytes, Parquet: {} bytes, Ratio: {:.2}",
        mrc_size,
        parquet_size,
        parquet_size / mrc_size
    );

    // Parquet with JSON encoding will be larger than ISO 2709
    // This is acceptable for the analytical use case
    Ok(())
}
