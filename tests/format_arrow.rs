//! Arrow format evaluation tests for MARC records
//! Tests round-trip fidelity, field/subfield ordering, and error handling

use mrrc::arrow_impl::{
    arrow_batch_to_records, records_to_arrow_batch, ArrowMarcTable, ArrowReader, ArrowWriter,
};
use mrrc::formats::{FormatReader, FormatWriter};
use mrrc::{Field, Leader, MarcReader, Record};
use std::fs;
use std::io::Cursor;

mod common;
use common::create_test_leader;

#[test]
fn test_arrow_basic_roundtrip() {
    // Create a simple record
    let leader = Leader {
        record_length: 100,
        record_status: 'n',
        record_type: 'a',
        bibliographic_level: 'm',
        control_record_type: ' ',
        character_coding: 'a',
        indicator_count: 2,
        subfield_code_count: 2,
        data_base_address: 50,
        encoding_level: ' ',
        cataloging_form: 'a',
        multipart_level: ' ',
        reserved: "4500".to_string(),
    };

    let mut record = Record::new(leader.clone());

    // Add a field
    let mut field = mrrc::record::Field {
        tag: "245".to_string(),
        indicator1: '1',
        indicator2: '0',
        subfields: smallvec::smallvec![],
    };
    field.subfields.push(mrrc::record::Subfield {
        code: 'a',
        value: "Test Title".to_string(),
    });
    record.add_field(field);

    // Serialize to Arrow
    let batch = records_to_arrow_batch(&[record.clone()]).expect("Failed to serialize");
    assert_eq!(batch.num_rows(), 1);

    // Deserialize back
    let recovered = arrow_batch_to_records(&batch).expect("Failed to deserialize");
    assert_eq!(recovered.len(), 1);

    let recovered_record = &recovered[0];
    assert_eq!(
        recovered_record.leader.record_length,
        record.leader.record_length
    );
    assert_eq!(recovered_record.fields().count(), 1);
}

#[test]
fn test_arrow_field_ordering() {
    let leader = Leader {
        record_length: 100,
        record_status: 'n',
        record_type: 'a',
        bibliographic_level: 'm',
        control_record_type: ' ',
        character_coding: 'a',
        indicator_count: 2,
        subfield_code_count: 2,
        data_base_address: 50,
        encoding_level: ' ',
        cataloging_form: 'a',
        multipart_level: ' ',
        reserved: "4500".to_string(),
    };

    let mut record = Record::new(leader);

    // Add fields in specific order: 650, 245, 001
    for &(tag, ind1, ind2, code, value) in &[
        ("650", ' ', '0', 'a', "Subject 1"),
        ("245", '1', '0', 'a', "Title"),
        ("001", ' ', ' ', 'a', "ID123"),
    ] {
        let mut field = mrrc::record::Field {
            tag: tag.to_string(),
            indicator1: ind1,
            indicator2: ind2,
            subfields: smallvec::smallvec![],
        };
        field.subfields.push(mrrc::record::Subfield {
            code,
            value: value.to_string(),
        });
        record.add_field(field);
    }

    // Serialize and deserialize
    let batch = records_to_arrow_batch(&[record]).expect("Failed to serialize");
    let recovered = arrow_batch_to_records(&batch).expect("Failed to deserialize");

    let recovered_record = &recovered[0];
    let tags: Vec<_> = recovered_record.fields().map(|f| f.tag.as_str()).collect();

    // Verify ordering is preserved: 650, 245, 001
    assert_eq!(tags[0], "650");
    assert_eq!(tags[1], "245");
    assert_eq!(tags[2], "001");
}

#[test]
fn test_arrow_empty_subfield_value() {
    let leader = Leader {
        record_length: 100,
        record_status: 'n',
        record_type: 'a',
        bibliographic_level: 'm',
        control_record_type: ' ',
        character_coding: 'a',
        indicator_count: 2,
        subfield_code_count: 2,
        data_base_address: 50,
        encoding_level: ' ',
        cataloging_form: 'a',
        multipart_level: ' ',
        reserved: "4500".to_string(),
    };

    let mut record = Record::new(leader);

    let mut field = mrrc::record::Field {
        tag: "650".to_string(),
        indicator1: ' ',
        indicator2: '0',
        subfields: smallvec::smallvec![],
    };
    field.subfields.push(mrrc::record::Subfield {
        code: 'a',
        value: String::new(), // Empty value
    });
    record.add_field(field);

    // Serialize and deserialize
    let batch = records_to_arrow_batch(&[record]).expect("Failed to serialize");
    let recovered = arrow_batch_to_records(&batch).expect("Failed to deserialize");

    let recovered_record = &recovered[0];
    let field = recovered_record.fields().next().unwrap();
    assert_eq!(field.subfields[0].value, "");
}

#[test]
fn test_arrow_multiple_records() {
    let mut records = vec![];

    for i in 0..10 {
        let leader = Leader {
            record_length: 100,
            record_status: 'n',
            record_type: 'a',
            bibliographic_level: 'm',
            control_record_type: ' ',
            character_coding: 'a',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 50,
            encoding_level: ' ',
            cataloging_form: 'a',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        };

        let mut record = Record::new(leader);

        let mut field = mrrc::record::Field {
            tag: "245".to_string(),
            indicator1: '1',
            indicator2: '0',
            subfields: smallvec::smallvec![],
        };
        field.subfields.push(mrrc::record::Subfield {
            code: 'a',
            value: format!("Title {i}"),
        });
        record.add_field(field);

        records.push(record);
    }

    // Serialize and deserialize
    let batch = records_to_arrow_batch(&records).expect("Failed to serialize");
    let recovered = arrow_batch_to_records(&batch).expect("Failed to deserialize");

    assert_eq!(recovered.len(), 10);
}

#[test]
fn test_arrow_marc_table() {
    let leader = Leader {
        record_length: 100,
        record_status: 'n',
        record_type: 'a',
        bibliographic_level: 'm',
        control_record_type: ' ',
        character_coding: 'a',
        indicator_count: 2,
        subfield_code_count: 2,
        data_base_address: 50,
        encoding_level: ' ',
        cataloging_form: 'a',
        multipart_level: ' ',
        reserved: "4500".to_string(),
    };

    let mut record = Record::new(leader);

    let mut field = mrrc::record::Field {
        tag: "245".to_string(),
        indicator1: '1',
        indicator2: '0',
        subfields: smallvec::smallvec![],
    };
    field.subfields.push(mrrc::record::Subfield {
        code: 'a',
        value: "Test Title".to_string(),
    });
    record.add_field(field);

    let table = ArrowMarcTable::from_records(&[record]).expect("Failed to create Arrow table");

    let schema = table.schema();
    assert_eq!(schema.fields().len(), 8);

    let recovered = table.to_records().expect("Failed to recover records");
    assert_eq!(recovered.len(), 1);
}

// ============================================================================
// Fidelity Tests using ArrowReader/ArrowWriter
// ============================================================================

#[test]
#[allow(clippy::too_many_lines, clippy::uninlined_format_args)]
fn test_arrow_roundtrip_fidelity_100_records() -> Result<(), Box<dyn std::error::Error>> {
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

    // Serialize to Arrow IPC format using ArrowWriter
    let mut buffer = Vec::new();
    {
        let mut writer = ArrowWriter::new(&mut buffer);
        writer.write_batch(&original_records)?;
        writer.finish()?;
    }

    // Deserialize from Arrow IPC format using ArrowReader
    let cursor = Cursor::new(buffer);
    let mut arrow_reader = ArrowReader::new(cursor)?;
    let restored_records = arrow_reader.read_all()?;

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
        "Arrow round-trip fidelity: {}/{} records perfect ({:.1}%)",
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
fn test_arrow_writer_reader_single_record() -> Result<(), Box<dyn std::error::Error>> {
    let mut record = Record::new(create_test_leader());
    record.add_control_field("001".to_string(), "test12345".to_string());

    let mut field = Field::new("245".to_string(), '1', '0');
    field.add_subfield('a', "Test Title".to_string());
    field.add_subfield('b', "Test Subtitle".to_string());
    record.add_field(field);

    let records = vec![record.clone()];

    // Write to Arrow using ArrowWriter
    let mut buffer = Vec::new();
    {
        let mut writer = ArrowWriter::new(&mut buffer);
        writer.write_batch(&records)?;
        writer.finish()?;
    }

    // Read back using ArrowReader
    let cursor = Cursor::new(buffer);
    let mut reader = ArrowReader::new(cursor)?;
    let restored = reader.read_all()?;

    assert_eq!(restored.len(), 1);
    assert_eq!(record.control_fields, restored[0].control_fields);
    assert_eq!(record.fields, restored[0].fields);

    Ok(())
}

#[test]
fn test_arrow_writer_reader_utf8_content() -> Result<(), Box<dyn std::error::Error>> {
    let mut record = Record::new(create_test_leader());

    // Add fields with UTF-8 content
    let mut field = Field::new("245".to_string(), '1', '0');
    field.add_subfield('a', "Tïtlé with Ümlàuts".to_string());
    field.add_subfield('b', "中文字符和日本語テキスト".to_string());
    record.add_field(field);

    let records = vec![record];

    // Write to Arrow
    let mut buffer = Vec::new();
    {
        let mut writer = ArrowWriter::new(&mut buffer);
        writer.write_batch(&records)?;
        writer.finish()?;
    }

    // Read back
    let cursor = Cursor::new(buffer);
    let mut reader = ArrowReader::new(cursor)?;
    let restored = reader.read_all()?;

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
fn test_arrow_format_traits() -> Result<(), Box<dyn std::error::Error>> {
    let mut record = Record::new(create_test_leader());
    let mut field = Field::new("245".to_string(), '1', '0');
    field.add_subfield('a', "Trait Test".to_string());
    record.add_field(field);

    let records = vec![record];

    // Use via FormatWriter trait
    let mut buffer = Vec::new();
    {
        let mut writer = ArrowWriter::new(&mut buffer);
        FormatWriter::write_batch(&mut writer, &records)?;
        assert_eq!(FormatWriter::records_written(&writer), Some(1));
        FormatWriter::finish(&mut writer)?;
    }

    // Use via FormatReader trait
    let cursor = Cursor::new(buffer);
    let mut reader = ArrowReader::new(cursor)?;
    let restored = FormatReader::read_all(&mut reader)?;

    assert_eq!(restored.len(), 1);
    assert_eq!(FormatReader::records_read(&reader), Some(1));

    Ok(())
}

#[test]
fn test_arrow_preserves_subfield_order_via_ipc() -> Result<(), Box<dyn std::error::Error>> {
    let mut record = Record::new(create_test_leader());

    // Add field with subfields in non-alphabetical order
    let mut field = Field::new("245".to_string(), '1', '0');
    field.add_subfield('c', "Author First".to_string());
    field.add_subfield('a', "Title Second".to_string());
    field.add_subfield('b', "Subtitle Third".to_string());
    record.add_field(field);

    let records = vec![record];

    // Write to Arrow via ArrowWriter
    let mut buffer = Vec::new();
    {
        let mut writer = ArrowWriter::new(&mut buffer);
        writer.write_batch(&records)?;
        writer.finish()?;
    }

    // Read back via ArrowReader
    let cursor = Cursor::new(buffer);
    let mut reader = ArrowReader::new(cursor)?;
    let restored = reader.read_all()?;

    assert_eq!(restored.len(), 1);
    let f = restored[0].get_field("245").unwrap();
    let codes: Vec<char> = f.subfields.iter().map(|s| s.code).collect();
    assert_eq!(codes, vec!['c', 'a', 'b']);

    Ok(())
}
