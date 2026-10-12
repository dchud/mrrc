//! Code for `docs/reference/encoding.md`.

use mrrc::Record;
use std::fs::File;

fn reading_utf8(file: File) -> mrrc::Result<()> {
    // --8<-- [start:reading-utf8]
    use mrrc::{MarcReader, RecordHelpers};

    let mut reader = MarcReader::new(file);
    while let Some(record) = reader.read_record()? {
        // All strings are Rust String (UTF-8)
        println!("{:?}", record.title());
    }
    // --8<-- [end:reading-utf8]
    Ok(())
}

fn encoding_detection(record: Record) {
    // --8<-- [start:encoding-detection]
    use mrrc::encoding::MarcEncoding;

    // Leader position 9 (`character_coding`) declares the scheme, by the rule
    // the readers use: `a` is UTF-8 and any other value is MARC-8
    let encoding = MarcEncoding::declared_by_leader(record.leader.character_coding);
    match encoding {
        MarcEncoding::Utf8 => println!("UTF-8"),
        MarcEncoding::Marc8 => println!("MARC-8"),
    }
    // --8<-- [end:encoding-detection]
}

#[allow(unused_mut)]
fn choosing_the_encoding(file: File) {
    // --8<-- [start:choosing-the-encoding]
    use mrrc::{CharacterCoding, MarcReader};

    let mut reader = MarcReader::new(file).with_character_coding(CharacterCoding::Detect);
    // --8<-- [end:choosing-the-encoding]
}

fn mixed_encoding(record: Record) -> mrrc::Result<()> {
    // --8<-- [start:mixed-encoding]
    use mrrc::{EncodingAnalysis, EncodingValidator};

    if let EncodingAnalysis::Mixed { primary, .. } = EncodingValidator::analyze_encoding(&record)? {
        println!("Warning: MARC-8 escape sequences in a record declared {primary:?}");
    }
    // --8<-- [end:mixed-encoding]
    Ok(())
}

#[test]
fn fragments_run_against_fixtures() {
    let open = || File::open(crate::fixtures::RECORDS_PATH).expect("open fixture file");
    reading_utf8(open()).unwrap();
    choosing_the_encoding(open());
    for record in crate::fixtures::records() {
        encoding_detection(record.clone());
        mixed_encoding(record).unwrap();
    }
}
