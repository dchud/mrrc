//! Code for `docs/reference/rust-api.md`.

use mrrc::{Field, Leader, Record};

fn record() -> mrrc::Result<()> {
    // --8<-- [start:record]
    use mrrc::{Field, Leader, Record, RecordHelpers};

    // Create with builder
    let record = Record::builder(Leader::from_bytes(b"00000nam a2200000 i 4500")?)
        .control_field_str("001", "123456789")
        .field(
            Field::builder("245".to_string(), '1', '0')
                .subfield_str('a', "Title")
                .build(),
        )
        .build();

    // Access fields
    if let Some(title) = record.title() {
        println!("{title}");
    }

    for field in record.fields_by_tag("650") {
        if let Some(subject) = field.get_subfield('a') {
            println!("Subject: {subject}");
        }
    }
    // --8<-- [end:record]
    Ok(())
}

#[allow(clippy::let_and_return)]
fn record_builder(leader: Leader, title_field: Field, author_field: Field) -> Record {
    // --8<-- [start:record-builder]
    use mrrc::Record;

    let record = Record::builder(leader)
        .control_field_str("001", "12345")
        .control_field_str("008", "040520s2023    xxu")
        .field(title_field)
        .field(author_field)
        .build();
    // --8<-- [end:record-builder]
    record
}

fn field() {
    // --8<-- [start:field]
    use mrrc::Field;

    // Create with builder
    let field = Field::builder("245".to_string(), '1', '0')
        .subfield_str('a', "Main title :")
        .subfield_str('b', "subtitle /")
        .subfield_str('c', "by Author.")
        .build();

    // Create directly
    let mut field = Field::new("650".to_string(), ' ', '0');
    field.add_subfield('a', "Subject heading".to_string());

    // Access data
    println!("Tag: {}", field.tag);
    println!("Ind1: {}", field.indicator1);
    if let Some(value) = field.get_subfield('a') {
        println!("$a: {value}");
    }
    // --8<-- [end:field]
}

fn subfield() {
    // --8<-- [start:subfield]
    use mrrc::Subfield;

    let sf = Subfield {
        code: 'a',
        value: "value".to_string(),
    };
    println!("Code: {}", sf.code);
    println!("Value: {}", sf.value);
    // --8<-- [end:subfield]
}

fn leader() -> mrrc::Result<()> {
    // --8<-- [start:leader]
    use mrrc::Leader;

    let mut leader = Leader::from_bytes(b"00000nam a2200000 i 4500")?;
    leader.record_type = 'a'; // Language material
    leader.bibliographic_level = 'm'; // Monograph
    leader.character_coding = 'a'; // UTF-8
    // --8<-- [end:leader]
    Ok(())
}

fn marc_reader() -> mrrc::Result<()> {
    // --8<-- [start:marc-reader]
    use mrrc::{MarcReader, RecordHelpers};
    use std::fs::File;

    let file = File::open("records.mrc")?;
    let mut reader = MarcReader::new(file);

    while let Some(record) = reader.read_record()? {
        println!("{:?}", record.title());
    }
    // --8<-- [end:marc-reader]
    Ok(())
}

fn marc_writer(record: Record) -> mrrc::Result<()> {
    // --8<-- [start:marc-writer]
    use mrrc::MarcWriter;
    use std::fs::File;

    let file = File::create("output.mrc")?;
    let mut writer = MarcWriter::new(file);

    writer.write_record(&record)?;
    // --8<-- [end:marc-writer]
    Ok(())
}

fn parse_from_bytes() -> mrrc::Result<()> {
    // --8<-- [start:parse-from-bytes]
    use mrrc::{RecoveryMode, ValidationLevel, parse_record_from_bytes};

    let bytes: Vec<u8> = std::fs::read("one_record.mrc")?;
    let record = parse_record_from_bytes(bytes, RecoveryMode::Strict, ValidationLevel::Structural)?;
    // --8<-- [end:parse-from-bytes]
    Ok(())
}

fn authority_record() -> mrrc::Result<()> {
    // --8<-- [start:authority-record]
    use mrrc::{AuthorityRecord, Field, Leader};

    let record = AuthorityRecord::builder(Leader::from_bytes(b"00000nz  a2200000n  4500")?)
        .control_field("001".to_string(), "n12345678".to_string())
        .heading(
            Field::builder("100".to_string(), '1', ' ')
                .subfield_str('a', "Smith, John")
                .build(),
        )
        .build();
    // --8<-- [end:authority-record]
    assert!(record.heading().is_some());
    Ok(())
}

fn holdings_record() -> mrrc::Result<()> {
    // --8<-- [start:holdings-record]
    use mrrc::{Field, HoldingsRecord, Leader};

    let record = HoldingsRecord::builder(Leader::from_bytes(b"00000nx  a2200000   4500")?)
        .control_field("001".to_string(), "h12345".to_string())
        .location(
            Field::builder("852".to_string(), ' ', ' ')
                .subfield_str('a', "Main Library")
                .build(),
        )
        .build();
    // --8<-- [end:holdings-record]
    assert_eq!(record.locations().len(), 1);
    Ok(())
}

fn query_dsl(record: Record) {
    // --8<-- [start:query-dsl]
    use mrrc::{FieldQuery, SubfieldValueQuery};

    // Find fields by tag
    let query = FieldQuery::new().tag("245");
    for field in record.fields_matching(&query) {
        println!("{}", field.tag);
    }

    // Find by tag range
    let query = FieldQuery::new().tag_range("600", "699");
    let subject_fields = record.fields_matching_range(&query);

    // Find by subfield value
    let query = SubfieldValueQuery::new("100", 'a', "Smith");
    let matches = record.fields_matching_value(&query);

    // Find by subfield presence only
    let query = FieldQuery::new().tag("100").has_subfield('a');
    let with_name = record.fields_matching(&query);
    // --8<-- [end:query-dsl]
}

fn core_formats(record: Record, collection_xml: String) -> mrrc::Result<()> {
    // --8<-- [start:core-formats]
    use mrrc::marcxml::{marcxml_to_record, marcxml_to_records, record_to_marcxml};

    let json = mrrc::json::record_to_json(&record)?;
    let marcjson = mrrc::marcjson::record_to_marcjson(&record)?;

    // MARCXML conversion (bidirectional)
    let xml = record_to_marcxml(&record)?;
    let restored = marcxml_to_record(&xml)?;

    // Parse a MARCXML collection (multiple records)
    let records = marcxml_to_records(&collection_xml)?;

    // MODS conversion (bidirectional)
    let mods = mrrc::mods::record_to_mods_xml(&record)?;
    let restored = mrrc::mods::mods_xml_to_record(&mods)?;
    // --8<-- [end:core-formats]
    Ok(())
}

fn bibframe_conversion(record: Record) -> mrrc::Result<()> {
    // --8<-- [start:bibframe-conversion]
    use mrrc::bibframe::{BibframeConfig, RdfFormat, marc_to_bibframe};

    let config = BibframeConfig {
        base_uri: Some("http://example.org/".to_string()),
        ..Default::default()
    };

    let graph = marc_to_bibframe(&record, &config);

    // Serialize to various RDF formats
    let turtle = graph.serialize(RdfFormat::Turtle)?;
    let rdfxml = graph.serialize(RdfFormat::RdfXml)?;
    let jsonld = graph.serialize(RdfFormat::JsonLd)?;
    // --8<-- [end:bibframe-conversion]
    Ok(())
}

fn character_encoding(leader: Leader, bytes: Vec<u8>) -> mrrc::Result<()> {
    // --8<-- [start:character-encoding]
    use mrrc::encoding::{MarcEncoding, decode_bytes, encode_string};

    // The encoding leader position 09 declares, by the readers' rule:
    // `a` is UTF-8 and any other value is MARC-8
    let encoding = MarcEncoding::declared_by_leader(leader.character_coding);

    // Decode bytes
    let text = decode_bytes(&bytes, encoding)?;

    // Encode string
    let bytes = encode_string(&text, encoding)?;
    // --8<-- [end:character-encoding]
    Ok(())
}

mod error_handling {
    // --8<-- [start:error-handling]
    use mrrc::{MarcReader, Result};
    use std::fs::File;

    fn process(path: &str) -> Result<()> {
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
        process(crate::fixtures::RECORDS_PATH).unwrap();
    }
}

fn parallel_processing() -> mrrc::Result<()> {
    // --8<-- [start:parallel-processing]
    use mrrc::RecordHelpers;
    use mrrc::boundary_scanner::RecordBoundaryScanner;
    use mrrc::rayon_parser_pool::parse_batch_parallel;
    use rayon::prelude::*;

    // Read the whole file into one buffer
    let data = std::fs::read("records.mrc")?;

    // Locate each record as an (offset, length) pair
    let mut scanner = RecordBoundaryScanner::new();
    let boundaries = scanner.scan(&data)?;

    // Parse every record in parallel against the shared buffer
    let records = parse_batch_parallel(&boundaries, &data)?;

    // Process records in parallel
    records
        .par_iter()
        .filter(|r| r.title().is_some())
        .for_each(|r| println!("{:?}", r.title()));
    // --8<-- [end:parallel-processing]
    Ok(())
}

#[test]
fn fragments_run_against_fixtures() {
    record().unwrap();
    let leader_value = Leader::from_bytes(b"00000nam a2200000 i 4500").unwrap();
    let title_field = Field::builder("245".to_string(), '1', '0')
        .subfield_str('a', "Title")
        .build();
    let author_field = Field::builder("100".to_string(), '1', ' ')
        .subfield_str('a', "Author")
        .build();
    let built = record_builder(leader_value.clone(), title_field, author_field);
    assert_eq!(built.fields().count(), 2);
    field();
    subfield();
    leader().unwrap();
    authority_record().unwrap();
    holdings_record().unwrap();
    character_encoding(leader_value, "Caf\u{e9}".as_bytes().to_vec()).unwrap();

    let collection_xml =
        std::fs::read_to_string("tests/data/loc_collection.marcxml").expect("read fixture");
    for record in crate::fixtures::records() {
        query_dsl(record.clone());
        core_formats(record.clone(), collection_xml.clone()).unwrap();
        bibframe_conversion(record).unwrap();
    }
}
