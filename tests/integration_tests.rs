//! Integration tests for the mrrc library

use mrrc::{AuthorityMarcReader, AuthorityMarcWriter, MarcReader, MarcWriter};
use std::fs::File;
use std::io::Cursor;

#[test]
fn test_read_simple_book_record() {
    let file = File::open("tests/data/simple_book.mrc").expect("Could not open test file");
    let mut reader = MarcReader::new(file);

    let record = reader.read_record().expect("Failed to read record");
    let record = record.expect("No record found");

    // Verify record structure
    assert_eq!(record.leader.record_type, 'a'); // Book
    assert_eq!(record.leader.bibliographic_level, 'm'); // Monograph

    // Check title field
    let title_fields = record.get_fields("245").expect("No title field");
    assert!(!title_fields.is_empty());
    assert_eq!(title_fields[0].get_subfield('a'), Some("The Great Gatsby"));
    assert_eq!(
        title_fields[0].get_subfield('c'),
        Some("F. Scott Fitzgerald")
    );

    // Check author field
    let author_fields = record.get_fields("100").expect("No author field");
    assert!(!author_fields.is_empty());
    assert_eq!(
        author_fields[0].get_subfield('a'),
        Some("Fitzgerald, F. Scott")
    );
}

#[test]
fn test_read_music_record() {
    let file = File::open("tests/data/music_score.mrc").expect("Could not open test file");
    let mut reader = MarcReader::new(file);

    let record = reader.read_record().expect("Failed to read record");
    let record = record.expect("No record found");

    // Verify it's a music record
    assert_eq!(record.leader.record_type, 'c'); // Musical notation

    // Check title
    let title_fields = record.get_fields("245").expect("No title field");
    assert_eq!(
        title_fields[0].get_subfield('a'),
        Some("Beethovens Ninth Symphony")
    );
}

#[test]
fn test_read_record_with_control_fields() {
    let file = File::open("tests/data/with_control_fields.mrc").expect("Could not open test file");
    let mut reader = MarcReader::new(file);

    let record = reader.read_record().expect("Failed to read record");
    let record = record.expect("No record found");

    // Check control field
    let field_008 = record.get_control_field("008");
    assert!(field_008.is_some());
    assert!(field_008.unwrap().starts_with("200101"));
}

#[test]
fn test_read_multiple_records() {
    let file = File::open("tests/data/multi_records.mrc").expect("Could not open test file");
    let mut reader = MarcReader::new(file);

    // Read first record
    let record1 = reader.read_record().expect("Failed to read first record");
    assert!(record1.is_some());

    // Read second record
    let record2 = reader.read_record().expect("Failed to read second record");
    assert!(record2.is_some());

    // Read third record
    let record3 = reader.read_record().expect("Failed to read third record");
    assert!(record3.is_some());

    // No more records
    let record4 = reader
        .read_record()
        .expect("Failed to check for fourth record");
    assert!(record4.is_none());
}

#[test]
fn test_roundtrip_book_record() {
    // Read the original file
    let file = File::open("tests/data/simple_book.mrc").expect("Could not open test file");
    let mut reader = MarcReader::new(file);
    let original = reader.read_record().expect("Failed to read record");
    let original = original.expect("No record found");

    // Write to buffer
    let mut buffer = Vec::new();
    {
        let mut writer = MarcWriter::new(&mut buffer);
        writer
            .write_record(&original)
            .expect("Failed to write record");
    }

    // Read back from buffer
    let cursor = Cursor::new(buffer);
    let mut reader = MarcReader::new(cursor);
    let restored = reader
        .read_record()
        .expect("Failed to read restored record");
    let restored = restored.expect("No restored record");

    // Verify the roundtrip preserved data
    assert_eq!(original.leader.record_type, restored.leader.record_type);
    assert_eq!(
        original.leader.bibliographic_level,
        restored.leader.bibliographic_level
    );

    let orig_title = original.get_fields("245").unwrap()[0].get_subfield('a');
    let restored_title = restored.get_fields("245").unwrap()[0].get_subfield('a');
    assert_eq!(orig_title, restored_title);

    let orig_author = original.get_fields("100").unwrap()[0].get_subfield('a');
    let restored_author = restored.get_fields("100").unwrap()[0].get_subfield('a');
    assert_eq!(orig_author, restored_author);
}

#[test]
fn test_json_serialization_with_file_data() {
    use mrrc::json;

    let file = File::open("tests/data/simple_book.mrc").expect("Could not open test file");
    let mut reader = MarcReader::new(file);
    let record = reader.read_record().expect("Failed to read record");
    let record = record.expect("No record found");

    // Convert to JSON
    let json = json::record_to_json(&record).expect("Failed to convert to JSON");

    // Verify JSON structure
    assert!(json.is_array());
    let array = json.as_array().unwrap();
    assert!(!array.is_empty());

    // Convert back from JSON
    let restored = json::json_to_record(&json).expect("Failed to restore from JSON");

    // Verify data
    assert_eq!(record.leader.record_type, restored.leader.record_type);
}

#[test]
fn test_xml_serialization_with_file_data() {
    use mrrc::xml;

    let file = File::open("tests/data/simple_book.mrc").expect("Could not open test file");
    let mut reader = MarcReader::new(file);
    let record = reader.read_record().expect("Failed to read record");
    let record = record.expect("No record found");

    // Convert to XML
    let xml_str = xml::record_to_xml(&record).expect("Failed to convert to XML");

    // Verify XML has expected elements
    assert!(xml_str.contains("<leader>"));
    assert!(xml_str.contains("245"));
    assert!(xml_str.contains("The Great Gatsby"));

    // Convert back from XML
    let restored = xml::xml_to_record(&xml_str).expect("Failed to restore from XML");

    // Verify data
    assert_eq!(record.leader.record_type, restored.leader.record_type);
}

#[test]
fn test_marcjson_serialization_with_file_data() {
    use mrrc::marcjson;

    let file = File::open("tests/data/simple_book.mrc").expect("Could not open test file");
    let mut reader = MarcReader::new(file);
    let record = reader.read_record().expect("Failed to read record");
    let record = record.expect("No record found");

    // Convert to MARCJSON
    let json = marcjson::record_to_marcjson(&record).expect("Failed to convert to MARCJSON");

    // Convert back from MARCJSON
    let restored = marcjson::marcjson_to_record(&json).expect("Failed to restore from MARCJSON");

    // Verify data
    assert_eq!(record.leader.record_type, restored.leader.record_type);

    let orig_title = record.get_fields("245").unwrap()[0].get_subfield('a');
    let restored_title = restored.get_fields("245").unwrap()[0].get_subfield('a');
    assert_eq!(orig_title, restored_title);
}

#[test]
fn test_read_simple_authority_record() {
    let file = File::open("tests/data/simple_authority.mrc").expect("Could not open test file");
    let mut reader = AuthorityMarcReader::new(file);

    let record = reader.read_record().expect("Failed to read record");
    let record = record.expect("No record found");

    // Verify record structure
    assert_eq!(record.leader.record_type, 'z'); // Authority record

    // Check control field 001 (control number)
    assert_eq!(record.get_control_field("001"), Some("n79021800"));

    // Check heading field
    assert!(record.heading().is_some());
    let heading = record.heading().unwrap();
    assert_eq!(heading.tag, "100");
    assert_eq!(heading.get_subfield('a'), Some("Smith, John"));
}

#[test]
fn test_authority_record_roundtrip() {
    // Read the original file
    let file = File::open("tests/data/simple_authority.mrc").expect("Could not open test file");
    let mut reader = AuthorityMarcReader::new(file);
    let original = reader.read_record().expect("Failed to read record");
    let original = original.expect("No record found");

    // Write to buffer
    let mut buffer = Vec::new();
    {
        let mut writer = AuthorityMarcWriter::new(&mut buffer);
        writer
            .write_record(&original)
            .expect("Failed to write record");
    }

    // Read back from buffer
    let cursor = Cursor::new(buffer);
    let mut reader = AuthorityMarcReader::new(cursor);
    let restored = reader
        .read_record()
        .expect("Failed to read restored record");
    let restored = restored.expect("No restored record");

    // Verify the roundtrip preserved data
    assert_eq!(original.leader.record_type, restored.leader.record_type);
    assert_eq!(
        original.get_control_field("001"),
        restored.get_control_field("001")
    );

    if let Some(orig_heading) = original.heading() {
        let restored_heading = restored.heading().expect("Restored heading missing");
        assert_eq!(orig_heading.tag, restored_heading.tag);
        assert_eq!(
            orig_heading.get_subfield('a'),
            restored_heading.get_subfield('a')
        );
    }
}
