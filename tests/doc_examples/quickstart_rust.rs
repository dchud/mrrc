//! Code for `docs/getting-started/quickstart-rust.md`.

use mrrc::Record;

mod read_records {
    // --8<-- [start:read-records]
    use mrrc::{MarcReader, RecordHelpers};
    use std::fs::File;

    fn main() -> mrrc::Result<()> {
        let file = File::open("records.mrc")?;
        let mut reader = MarcReader::new(file);

        while let Some(record) = reader.read_record()? {
            if let Some(title) = record.title() {
                println!("{title}");
            }
        }

        Ok(())
    }
    // --8<-- [end:read-records]
}

fn access_fields(record: Record) {
    // --8<-- [start:access-fields]
    use mrrc::RecordHelpers;

    // Get a specific field by tag
    if let Some(field) = record.get_field("245") {
        if let Some(title) = field.get_subfield('a') {
            println!("Title: {title}");
        }
    }

    // Get all fields with a tag
    for field in record.fields_by_tag("650") {
        if let Some(subject) = field.get_subfield('a') {
            println!("Subject: {subject}");
        }
    }

    // Use convenience methods
    println!("Title: {:?}", record.title());
    println!("Author: {:?}", record.author());
    println!("ISBNs: {:?}", record.isbns());
    // --8<-- [end:access-fields]
}

fn create_record() -> mrrc::Result<Record> {
    // --8<-- [start:create-record]
    use mrrc::{Field, Leader, Record};

    // Create a new record with builder pattern
    let record = Record::builder(Leader::from_bytes(b"00000nam a2200000 i 4500")?)
        .control_field_str("001", "123456789")
        .field(
            Field::builder("245".to_string(), '1', '0')
                .subfield_str('a', "My Book Title")
                .subfield_str('c', "by Author Name")
                .build(),
        )
        .build();
    // --8<-- [end:create-record]
    Ok(record)
}

fn write_record(record: Record) -> mrrc::Result<()> {
    // --8<-- [start:write-record]
    use mrrc::MarcWriter;
    use std::fs::File;

    let file = File::create("output.mrc")?;
    let mut writer = MarcWriter::new(file);

    writer.write_record(&record)?;
    // --8<-- [end:write-record]
    Ok(())
}

fn convert_formats(record: Record) -> mrrc::Result<()> {
    // --8<-- [start:convert-formats]
    // To JSON
    let json = mrrc::json::record_to_json(&record)?;

    // To MARCXML
    let xml_str = mrrc::marcxml::record_to_marcxml(&record)?;

    // To MARCJSON (LOC standard)
    let marcjson = mrrc::marcjson::record_to_marcjson(&record)?;
    // --8<-- [end:convert-formats]
    Ok(())
}

mod error_handling {
    // --8<-- [start:error-handling]
    use mrrc::{MarcReader, Result};
    use std::fs::File;

    fn process_file(path: &str) -> Result<()> {
        let file = File::open(path)?;
        let mut reader = MarcReader::new(file);

        while let Some(record) = reader.read_record()? {
            // Process record
        }

        Ok(())
    }
    // --8<-- [end:error-handling]

    #[test]
    fn processes_fixture_file() {
        process_file(crate::fixtures::RECORDS_PATH).unwrap();
    }
}

#[test]
fn fragments_run_against_fixtures() {
    for record in crate::fixtures::records() {
        access_fields(record.clone());
        convert_formats(record).unwrap();
    }
    let record = create_record().unwrap();
    assert_eq!(record.get_control_field("001"), Some("123456789"));
}
