//! Code for `docs/contributing/testing.md`.

// --8<-- [start:rust-test-example]
use mrrc::{MarcReader, RecordHelpers};
use std::fs::File;

#[test]
fn test_parse_simple_record() {
    let file = File::open("tests/data/simple_book.mrc").unwrap();
    let mut reader = MarcReader::new(file);
    let record = reader.read_record().unwrap().unwrap();
    assert_eq!(record.title(), Some("The Great Gatsby"));
}
// --8<-- [end:rust-test-example]
