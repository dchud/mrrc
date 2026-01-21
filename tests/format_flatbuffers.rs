//! `FlatBuffers` format evaluation tests for MARC records
//! Tests round-trip fidelity, field/subfield ordering, and error handling

use mrrc::flatbuffers_impl::{FlatbuffersReader, FlatbuffersWriter};
use mrrc::formats::{FormatReader, FormatWriter};
use mrrc::{Field, MarcReader, Record};
use std::fs;
use std::io::Cursor;

mod common;
use common::create_test_leader;

// ============================================================================
// Basic Round-trip Tests
// ============================================================================

#[test]
fn test_flatbuffers_basic_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let mut record = Record::new(create_test_leader());
    record.add_control_field("001".to_string(), "test123".to_string());

    let mut field = Field::new("245".to_string(), '1', '0');
    field.add_subfield('a', "Test Title".to_string());
    record.add_field(field);

    // Write using FlatbuffersWriter
    let mut buffer = Vec::new();
    {
        let mut writer = FlatbuffersWriter::new(&mut buffer);
        writer.write_record(&record)?;
        writer.finish()?;
    }

    // Read using FlatbuffersReader
    let cursor = Cursor::new(buffer);
    let mut reader = FlatbuffersReader::new(cursor);
    let restored = reader.read_record()?.expect("Should read one record");

    // Verify content
    assert_eq!(restored.get_control_field("001"), Some("test123"));
    let field_245 = restored.get_field("245").expect("Should have 245");
    assert_eq!(field_245.get_subfield('a'), Some("Test Title"));

    Ok(())
}

#[test]
fn test_flatbuffers_field_ordering() -> Result<(), Box<dyn std::error::Error>> {
    let mut record = Record::new(create_test_leader());

    // Add fields in specific order: 650, 245, control fields
    record.add_control_field("001".to_string(), "ID123".to_string());
    record.add_control_field("005".to_string(), "20260121".to_string());

    let mut field_650 = Field::new("650".to_string(), ' ', '0');
    field_650.add_subfield('a', "Subject 1".to_string());
    record.add_field(field_650);

    let mut field_245 = Field::new("245".to_string(), '1', '0');
    field_245.add_subfield('a', "Title".to_string());
    record.add_field(field_245);

    // Write and read back
    let mut buffer = Vec::new();
    {
        let mut writer = FlatbuffersWriter::new(&mut buffer);
        writer.write_record(&record)?;
        writer.finish()?;
    }

    let cursor = Cursor::new(buffer);
    let mut reader = FlatbuffersReader::new(cursor);
    let restored = reader.read_record()?.unwrap();

    // Verify variable field ordering is preserved: 650, 245
    let tags: Vec<_> = restored.fields().map(|f| f.tag.as_str()).collect();
    assert_eq!(tags[0], "650");
    assert_eq!(tags[1], "245");

    Ok(())
}

#[test]
fn test_flatbuffers_subfield_ordering() -> Result<(), Box<dyn std::error::Error>> {
    let mut record = Record::new(create_test_leader());

    // Add subfields in non-alphabetical order: c, a, b
    let mut field = Field::new("245".to_string(), '1', '0');
    field.add_subfield('c', "Author".to_string());
    field.add_subfield('a', "Title".to_string());
    field.add_subfield('b', "Subtitle".to_string());
    record.add_field(field);

    // Write and read back
    let mut buffer = Vec::new();
    {
        let mut writer = FlatbuffersWriter::new(&mut buffer);
        writer.write_record(&record)?;
        writer.finish()?;
    }

    let cursor = Cursor::new(buffer);
    let mut reader = FlatbuffersReader::new(cursor);
    let restored = reader.read_record()?.unwrap();

    // Verify subfield ordering is preserved: c, a, b
    let f = restored.get_field("245").unwrap();
    let codes: Vec<char> = f.subfields.iter().map(|s| s.code).collect();
    assert_eq!(codes, vec!['c', 'a', 'b']);

    Ok(())
}

#[test]
fn test_flatbuffers_empty_subfield_value() -> Result<(), Box<dyn std::error::Error>> {
    let mut record = Record::new(create_test_leader());

    let mut field = Field::new("650".to_string(), ' ', '0');
    field.add_subfield('a', String::new()); // Empty value
    field.add_subfield('b', "Non-empty".to_string());
    record.add_field(field);

    // Write and read back
    let mut buffer = Vec::new();
    {
        let mut writer = FlatbuffersWriter::new(&mut buffer);
        writer.write_record(&record)?;
        writer.finish()?;
    }

    let cursor = Cursor::new(buffer);
    let mut reader = FlatbuffersReader::new(cursor);
    let restored = reader.read_record()?.unwrap();

    let field = restored.get_field("650").unwrap();
    assert_eq!(field.subfields[0].value, "");
    assert_eq!(field.subfields[1].value, "Non-empty");

    Ok(())
}

#[test]
fn test_flatbuffers_multiple_records() -> Result<(), Box<dyn std::error::Error>> {
    let records: Vec<Record> = (0..10)
        .map(|i| {
            let mut record = Record::new(create_test_leader());
            record.add_control_field("001".to_string(), format!("rec{i:03}"));

            let mut field = Field::new("245".to_string(), '1', '0');
            field.add_subfield('a', format!("Title {i}"));
            record.add_field(field);
            record
        })
        .collect();

    // Write all records
    let mut buffer = Vec::new();
    {
        let mut writer = FlatbuffersWriter::new(&mut buffer);
        for record in &records {
            writer.write_record(record)?;
        }
        writer.finish()?;
    }

    // Read all records back
    let cursor = Cursor::new(buffer);
    let mut reader = FlatbuffersReader::new(cursor);
    let restored = reader.read_all()?;

    assert_eq!(restored.len(), 10);
    for (i, record) in restored.iter().enumerate() {
        assert_eq!(
            record.get_control_field("001"),
            Some(format!("rec{i:03}").as_str())
        );
    }

    Ok(())
}

#[test]
fn test_flatbuffers_utf8_content() -> Result<(), Box<dyn std::error::Error>> {
    let mut record = Record::new(create_test_leader());

    let mut field = Field::new("245".to_string(), '1', '0');
    field.add_subfield('a', "Tïtlé with Ümlàuts".to_string());
    field.add_subfield('b', "中文字符和日本語テキスト".to_string());
    record.add_field(field);

    // Write and read back
    let mut buffer = Vec::new();
    {
        let mut writer = FlatbuffersWriter::new(&mut buffer);
        writer.write_record(&record)?;
        writer.finish()?;
    }

    let cursor = Cursor::new(buffer);
    let mut reader = FlatbuffersReader::new(cursor);
    let restored = reader.read_record()?.unwrap();

    let f = restored.get_field("245").unwrap();
    assert_eq!(f.subfields[0].value, "Tïtlé with Ümlàuts");
    assert_eq!(f.subfields[1].value, "中文字符和日本語テキスト");

    Ok(())
}

#[test]
fn test_flatbuffers_indicator_preservation() -> Result<(), Box<dyn std::error::Error>> {
    let mut record = Record::new(create_test_leader());

    let mut field = Field::new("245".to_string(), '1', '4');
    field.add_subfield('a', "The Title".to_string());
    record.add_field(field);

    let mut field2 = Field::new("650".to_string(), ' ', '0');
    field2.add_subfield('a', "Subject".to_string());
    record.add_field(field2);

    // Write and read back
    let mut buffer = Vec::new();
    {
        let mut writer = FlatbuffersWriter::new(&mut buffer);
        writer.write_record(&record)?;
        writer.finish()?;
    }

    let cursor = Cursor::new(buffer);
    let mut reader = FlatbuffersReader::new(cursor);
    let restored = reader.read_record()?.unwrap();

    let f245 = restored.get_field("245").unwrap();
    assert_eq!(f245.indicator1, '1');
    assert_eq!(f245.indicator2, '4');

    let f650 = restored.get_field("650").unwrap();
    assert_eq!(f650.indicator1, ' ');
    assert_eq!(f650.indicator2, '0');

    Ok(())
}

#[test]
fn test_flatbuffers_format_traits() -> Result<(), Box<dyn std::error::Error>> {
    let mut record = Record::new(create_test_leader());
    let mut field = Field::new("245".to_string(), '1', '0');
    field.add_subfield('a', "Trait Test".to_string());
    record.add_field(field);

    // Use via FormatWriter trait
    let mut buffer = Vec::new();
    {
        let mut writer = FlatbuffersWriter::new(&mut buffer);
        FormatWriter::write_record(&mut writer, &record)?;
        assert_eq!(FormatWriter::records_written(&writer), Some(1));
        FormatWriter::finish(&mut writer)?;
    }

    // Use via FormatReader trait
    let cursor = Cursor::new(buffer);
    let mut reader = FlatbuffersReader::new(cursor);
    let restored = FormatReader::read_all(&mut reader)?;

    assert_eq!(restored.len(), 1);
    assert_eq!(FormatReader::records_read(&reader), Some(1));

    Ok(())
}

// ============================================================================
// Fidelity Tests using FIDELITY_TEST_SET (100 records)
// ============================================================================

#[test]
#[allow(clippy::too_many_lines, clippy::uninlined_format_args)]
fn test_flatbuffers_roundtrip_fidelity_100_records() -> Result<(), Box<dyn std::error::Error>> {
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

    // Serialize to FlatBuffers format using FlatbuffersWriter
    let mut buffer = Vec::new();
    {
        let mut writer = FlatbuffersWriter::new(&mut buffer);
        for record in &original_records {
            writer.write_record(record)?;
        }
        writer.finish()?;
    }

    // Deserialize from FlatBuffers format using FlatbuffersReader
    let cursor = Cursor::new(buffer);
    let mut fb_reader = FlatbuffersReader::new(cursor);
    let restored_records = fb_reader.read_all()?;

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
        "FlatBuffers round-trip fidelity: {}/{} records perfect ({:.1}%)",
        perfect_count, n_records, percent
    );

    // Require 100% fidelity for FlatBuffers
    assert_eq!(
        perfect_count, n_records,
        "Expected 100% perfect records, got {}/{}",
        perfect_count, n_records
    );

    Ok(())
}

#[test]
fn test_flatbuffers_whitespace_preservation() -> Result<(), Box<dyn std::error::Error>> {
    let mut record = Record::new(create_test_leader());

    let mut field = Field::new("245".to_string(), '1', '0');
    field.add_subfield('a', "  Leading and trailing  ".to_string());
    field.add_subfield('b', "\ttab\there\t".to_string());
    record.add_field(field);

    // Write and read back
    let mut buffer = Vec::new();
    {
        let mut writer = FlatbuffersWriter::new(&mut buffer);
        writer.write_record(&record)?;
        writer.finish()?;
    }

    let cursor = Cursor::new(buffer);
    let mut reader = FlatbuffersReader::new(cursor);
    let restored = reader.read_record()?.unwrap();

    let f = restored.get_field("245").unwrap();
    assert_eq!(f.get_subfield('a'), Some("  Leading and trailing  "));
    assert_eq!(f.get_subfield('b'), Some("\ttab\there\t"));

    Ok(())
}

#[test]
fn test_flatbuffers_repeated_fields() -> Result<(), Box<dyn std::error::Error>> {
    let mut record = Record::new(create_test_leader());

    // Add multiple 650 fields (common in MARC records)
    for i in 1..=5 {
        let mut field = Field::new("650".to_string(), ' ', '0');
        field.add_subfield('a', format!("Subject {i}"));
        record.add_field(field);
    }

    // Write and read back
    let mut buffer = Vec::new();
    {
        let mut writer = FlatbuffersWriter::new(&mut buffer);
        writer.write_record(&record)?;
        writer.finish()?;
    }

    let cursor = Cursor::new(buffer);
    let mut reader = FlatbuffersReader::new(cursor);
    let restored = reader.read_record()?.unwrap();

    // Verify all 5 fields are present in order
    let subjects: Vec<_> = restored.fields_by_tag("650").collect();
    assert_eq!(subjects.len(), 5);
    for (i, field) in subjects.iter().enumerate() {
        assert_eq!(
            field.get_subfield('a'),
            Some(format!("Subject {}", i + 1).as_str())
        );
    }

    Ok(())
}

#[test]
fn test_flatbuffers_empty_file() -> Result<(), Box<dyn std::error::Error>> {
    let buffer: Vec<u8> = Vec::new();
    let cursor = Cursor::new(buffer);
    let mut reader = FlatbuffersReader::new(cursor);

    assert!(reader.read_record()?.is_none());
    assert_eq!(reader.records_read(), 0);

    Ok(())
}
