//! Invalid UTF-8 in a record decoded as UTF-8 follows `Utf8Handling`,
//! whose values and default match pymarc's `utf8_handling`: `Strict` (the
//! default) makes it an E301 error for the recovery mode to handle,
//! `Replace` substitutes U+FFFD, and `Ignore` drops the bytes.

use mrrc::{MarcReader, RecoveryMode, Utf8Handling, ValidationLevel};
use std::io::Cursor;

mod common;
use common::{record_bytes, subfield_a};

/// A UTF-8 record whose 245$a and 001 each hold one invalid byte (0xFF).
fn invalid_utf8_record() -> Vec<u8> {
    record_bytes(
        *b"nam ",
        b'a',
        &[
            ("001", b"id\xFF1".to_vec()),
            ("245", subfield_a(b"Caf\xFFe")),
        ],
    )
}

fn reader(utf8: Utf8Handling, recovery: RecoveryMode) -> MarcReader<Cursor<Vec<u8>>> {
    MarcReader::new(Cursor::new(invalid_utf8_record()))
        .with_utf8_handling(utf8)
        .with_recovery_mode(recovery)
}

#[test]
fn strict_is_the_default() {
    let err = MarcReader::new(Cursor::new(invalid_utf8_record()))
        .read_record()
        .unwrap_err();
    assert_eq!(err.code(), "E301", "{err}");
}

#[test]
fn strict_is_independent_of_validation_level() {
    for level in [ValidationLevel::Structural, ValidationLevel::StrictMarc] {
        let err = reader(Utf8Handling::Strict, RecoveryMode::Strict)
            .with_validation_level(level)
            .read_record()
            .unwrap_err();
        assert_eq!(err.code(), "E301", "{level:?}: {err}");
    }
}

#[test]
fn strict_under_permissive_recovery_drops_the_fields_and_records_e301() {
    let record = reader(Utf8Handling::Strict, RecoveryMode::Permissive)
        .read_record()
        .unwrap()
        .unwrap();
    assert!(record.get_field("245").is_none());
    assert!(record.get_control_field("001").is_none());
    let codes: Vec<&str> = record.errors.iter().map(mrrc::MarcError::code).collect();
    assert_eq!(codes, ["E301", "E301"]);
}

#[test]
fn replace_substitutes_the_replacement_character() {
    let record = reader(Utf8Handling::Replace, RecoveryMode::Strict)
        .read_record()
        .unwrap()
        .unwrap();
    assert_eq!(
        record.get_field("245").unwrap().get_subfield('a'),
        Some("Caf\u{fffd}e")
    );
    assert_eq!(record.get_control_field("001"), Some("id\u{fffd}1"));
    assert!(record.errors.is_empty());
}

#[test]
fn ignore_drops_the_invalid_bytes() {
    let record = reader(Utf8Handling::Ignore, RecoveryMode::Strict)
        .read_record()
        .unwrap()
        .unwrap();
    assert_eq!(
        record.get_field("245").unwrap().get_subfield('a'),
        Some("Cafe")
    );
    assert_eq!(record.get_control_field("001"), Some("id1"));
}

#[test]
fn lenient_handling_holds_under_strict_marc() {
    // utf8_handling alone decides invalid UTF-8; strict_marc does not
    // override it.
    let record = reader(Utf8Handling::Replace, RecoveryMode::Strict)
        .with_validation_level(ValidationLevel::StrictMarc)
        .read_record()
        .unwrap()
        .unwrap();
    assert_eq!(
        record.get_field("245").unwrap().get_subfield('a'),
        Some("Caf\u{fffd}e")
    );
}

#[test]
fn marc8_records_ignore_utf8_handling() {
    // 0xFF has no MARC-8 mapping; it is a MARC-8 decoding matter (U+FFFD at
    // structural), not a UTF-8 one.
    let bytes = record_bytes(*b"nam ", b' ', &[("245", subfield_a(b"Caf\xFFe"))]);
    let record = MarcReader::new(Cursor::new(bytes))
        .read_record()
        .unwrap()
        .unwrap();
    assert_eq!(
        record.get_field("245").unwrap().get_subfield('a'),
        Some("Caf\u{fffd}e")
    );
}
