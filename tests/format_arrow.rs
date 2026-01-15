//! Arrow format evaluation tests for MARC records
//! Tests round-trip fidelity, field/subfield ordering, and error handling

use mrrc::arrow_impl::{arrow_batch_to_records, records_to_arrow_batch, ArrowMarcTable};
use mrrc::{Leader, Record};

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
        subfields: vec![],
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
            subfields: vec![],
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
        subfields: vec![],
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
            subfields: vec![],
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
        subfields: vec![],
    };
    field.subfields.push(mrrc::record::Subfield {
        code: 'a',
        value: "Test Title".to_string(),
    });
    record.add_field(field);

    let table = ArrowMarcTable::from_records(&[record]).expect("Failed to create Arrow table");

    let schema = table.schema();
    assert_eq!(schema.fields().len(), 7);

    let recovered = table.to_records().expect("Failed to recover records");
    assert_eq!(recovered.len(), 1);
}
