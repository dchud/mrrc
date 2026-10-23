//! Code for `docs/reference/validators.md`.

use mrrc::{Field, Record};

fn indicator_validator(field: Field) -> mrrc::Result<()> {
    // --8<-- [start:indicator-validator]
    use mrrc::IndicatorValidator;

    let v = IndicatorValidator::new();
    v.validate_field(&field)?; // by Field
    v.validate_indicators("245", '0', '1')?; // by tag + chars
    // --8<-- [end:indicator-validator]
    Ok(())
}

fn record_structure_validator(record: Record) -> mrrc::Result<()> {
    // --8<-- [start:record-structure-validator]
    use mrrc::RecordStructureValidator;

    RecordStructureValidator::validate_leader(&record.leader)?;
    RecordStructureValidator::validate_record(&record)?;
    RecordStructureValidator::validate_directory_structure(&record)?;
    // --8<-- [end:record-structure-validator]
    Ok(())
}

fn isbn_validator() {
    // --8<-- [start:isbn-validator]
    use mrrc::IsbnValidator;

    assert!(IsbnValidator::validate_isbn10("0306406152"));
    assert!(IsbnValidator::validate_isbn13("9780306406157"));

    // Either length; dashes and spaces are ignored
    assert!(IsbnValidator::validate("978-0-306-40615-7"));
    assert_eq!(IsbnValidator::normalize("0-306-40615-2"), "0306406152");
    // --8<-- [end:isbn-validator]
}

fn encoding_validator(record: Record) -> mrrc::Result<()> {
    // --8<-- [start:encoding-validator]
    use mrrc::{EncodingAnalysis, EncodingValidator};

    if let EncodingAnalysis::Mixed {
        primary,
        secondary,
        field_count,
    } = EncodingValidator::analyze_encoding(&record)?
    {
        // `field_count` values hold MARC-8 escape sequences.
    } else {
        // No mismatch visible in the decoded values.
    }
    // --8<-- [end:encoding-validator]
    Ok(())
}

fn field_245() -> Field {
    Field::builder("245".to_string(), '1', '0')
        .subfield_str('a', "Title")
        .build()
}

#[test]
fn fragments_run() {
    use mrrc::Leader;

    indicator_validator(field_245()).unwrap();
    let record = Record::builder(Leader::from_bytes(b"00000nam a2200000 i 4500").unwrap())
        .control_field_str("001", "12345")
        .control_field_str("008", "200101s2020    xxu||||||||||||||||eng||")
        .field(field_245())
        .build();
    record_structure_validator(record).unwrap();
    isbn_validator();
    for record in crate::fixtures::records() {
        encoding_validator(record).unwrap();
    }
}
