//! The ISO 2709 readers decode field data in the encoding leader position 09
//! declares, as pymarc does: `a` is UTF-8 and anything else is MARC-8.
//! `CharacterCoding` overrides that choice.

use mrrc::{
    AuthorityMarcReader, CharacterCoding, HoldingsMarcReader, MarcReader, RecoveryMode,
    ValidationLevel, parse_record_from_bytes,
};
use std::io::Cursor;

const FIELD_TERMINATOR: u8 = 0x1E;
const RECORD_TERMINATOR: u8 = 0x1D;

/// "Café" in MARC-8: ANSEL 0xE2 (combining acute) precedes its base letter.
const CAFE_MARC8: &[u8] = b"Caf\xE2e";
const CAFE: &str = "Caf\u{e9}";

/// Build one ISO 2709 record. `kind` is leader positions 05-08 (status,
/// type of record, bibliographic level, type of control) and `coding` is
/// position 09.
fn record_bytes(kind: [u8; 4], coding: u8, fields: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut directory = Vec::new();
    let mut data = Vec::new();
    for (tag, value) in fields {
        let start = data.len();
        data.extend_from_slice(value);
        data.push(FIELD_TERMINATOR);
        directory.extend_from_slice(tag.as_bytes());
        directory.extend_from_slice(format!("{:04}{:05}", data.len() - start, start).as_bytes());
    }
    directory.push(FIELD_TERMINATOR);
    let base = 24 + directory.len();
    let total = base + data.len() + 1;

    let mut record = format!("{total:05}").into_bytes();
    record.extend_from_slice(&kind);
    record.push(coding);
    record.extend_from_slice(format!("22{base:05}   4500").as_bytes());
    record.extend_from_slice(&directory);
    record.extend_from_slice(&data);
    record.push(RECORD_TERMINATOR);
    record
}

/// A data field: two indicators, then `$a` with `value`.
fn subfield_a(value: &[u8]) -> Vec<u8> {
    let mut field = b"10\x1Fa".to_vec();
    field.extend_from_slice(value);
    field
}

fn bib(coding: u8, title: &[u8]) -> Vec<u8> {
    record_bytes(*b"nam ", coding, &[("245", subfield_a(title))])
}

fn read_title(bytes: Vec<u8>, coding: CharacterCoding) -> String {
    let record = MarcReader::new(Cursor::new(bytes))
        .with_character_coding(coding)
        .read_record()
        .unwrap()
        .unwrap();
    record
        .get_field("245")
        .unwrap()
        .get_subfield('a')
        .unwrap()
        .to_string()
}

#[test]
fn marc8_leader_decodes_marc8() {
    assert_eq!(
        read_title(bib(b' ', CAFE_MARC8), CharacterCoding::Leader),
        CAFE
    );
}

#[test]
fn utf8_leader_decodes_utf8() {
    assert_eq!(
        read_title(bib(b'a', CAFE.as_bytes()), CharacterCoding::Leader),
        CAFE
    );
}

#[test]
fn leader_coding_is_the_default() {
    let record = MarcReader::new(Cursor::new(bib(b' ', CAFE_MARC8)))
        .read_record()
        .unwrap()
        .unwrap();
    assert_eq!(
        record.get_field("245").unwrap().get_subfield('a'),
        Some(CAFE)
    );
}

#[test]
fn position_09_other_than_a_is_marc8() {
    // pymarc's rule: only `a` means UTF-8.
    assert_eq!(
        read_title(bib(b'q', CAFE_MARC8), CharacterCoding::Leader),
        CAFE
    );
}

#[test]
fn control_fields_decode_as_marc8() {
    let bytes = record_bytes(*b"nam ", b' ', &[("001", b"ab\xE2e".to_vec())]);
    let record = MarcReader::new(Cursor::new(bytes))
        .read_record()
        .unwrap()
        .unwrap();
    assert_eq!(record.get_control_field("001"), Some("ab\u{e9}"));
}

#[test]
fn mislabelled_utf8_follows_the_leader_by_default() {
    // UTF-8 bytes under a MARC-8 leader decode as MARC-8, as in pymarc:
    // 0xC3 and 0xA9 are ANSEL's copyright and music flat signs.
    assert_eq!(
        read_title(bib(b' ', CAFE.as_bytes()), CharacterCoding::Leader),
        "Caf\u{a9}\u{266d}"
    );
}

#[test]
fn utf8_coding_ignores_the_leader() {
    assert_eq!(
        read_title(bib(b' ', CAFE.as_bytes()), CharacterCoding::Utf8),
        CAFE
    );
}

#[test]
fn detect_reads_valid_utf8_under_a_marc8_leader_as_utf8() {
    assert_eq!(
        read_title(bib(b' ', CAFE.as_bytes()), CharacterCoding::Detect),
        CAFE
    );
}

#[test]
fn detect_reads_marc8_as_marc8() {
    assert_eq!(
        read_title(bib(b' ', CAFE_MARC8), CharacterCoding::Detect),
        CAFE
    );
}

#[test]
fn detect_reads_ascii_with_escapes_as_marc8() {
    // Pure-ASCII MARC-8 with escape sequences is valid UTF-8 too, but has no
    // non-ASCII bytes, so the leader decides.
    assert_eq!(
        read_title(bib(b' ', b"H\x1Bb2\x1BsO"), CharacterCoding::Detect),
        "H\u{2082}O"
    );
}

#[test]
fn unmapped_marc8_is_replaced_under_structural() {
    // 0xAF has no mapping in ANSEL.
    let record = MarcReader::new(Cursor::new(bib(b' ', b"a\xAFb")))
        .read_record()
        .unwrap()
        .unwrap();
    assert_eq!(
        record.get_field("245").unwrap().get_subfield('a'),
        Some("a\u{fffd}b")
    );
}

#[test]
fn unmapped_marc8_raises_e302_under_strict_marc() {
    let err = MarcReader::new(Cursor::new(bib(b' ', b"a\xAFb")))
        .with_validation_level(ValidationLevel::StrictMarc)
        .read_record()
        .unwrap_err();
    assert_eq!(err.code(), "E302", "{err}");
    assert_eq!(err.slug(), "marc8_invalid");
}

#[test]
fn unmapped_marc8_in_a_control_field_raises_e302_under_strict_marc() {
    let bytes = record_bytes(*b"nam ", b' ', &[("001", b"a\xAFb".to_vec())]);
    let err = MarcReader::new(Cursor::new(bytes))
        .with_validation_level(ValidationLevel::StrictMarc)
        .read_record()
        .unwrap_err();
    assert_eq!(err.code(), "E302", "{err}");
}

#[test]
fn unmapped_marc8_is_recorded_on_the_record_in_permissive_strict_marc() {
    let record = MarcReader::new(Cursor::new(bib(b' ', b"a\xAFb")))
        .with_validation_level(ValidationLevel::StrictMarc)
        .with_recovery_mode(RecoveryMode::Permissive)
        .read_record()
        .unwrap()
        .unwrap();
    let codes: Vec<&str> = record.errors.iter().map(mrrc::MarcError::code).collect();
    assert_eq!(codes, ["E302"]);
}

#[test]
fn parse_record_from_bytes_decodes_marc8() {
    let record = parse_record_from_bytes(
        bib(b' ', CAFE_MARC8),
        RecoveryMode::Strict,
        ValidationLevel::Structural,
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        record.get_field("245").unwrap().get_subfield('a'),
        Some(CAFE)
    );
}

#[test]
fn authority_reader_decodes_marc8() {
    let bytes = record_bytes(*b"nz  ", b' ', &[("100", subfield_a(CAFE_MARC8))]);
    let record = AuthorityMarcReader::new(Cursor::new(bytes))
        .read_record()
        .unwrap()
        .unwrap();
    assert_eq!(record.heading().unwrap().get_subfield('a'), Some(CAFE));
}

#[test]
fn authority_reader_honours_character_coding() {
    let bytes = record_bytes(*b"nz  ", b' ', &[("100", subfield_a(CAFE.as_bytes()))]);
    let record = AuthorityMarcReader::new(Cursor::new(bytes))
        .with_character_coding(CharacterCoding::Utf8)
        .read_record()
        .unwrap()
        .unwrap();
    assert_eq!(record.heading().unwrap().get_subfield('a'), Some(CAFE));
}

#[test]
fn holdings_reader_decodes_marc8() {
    let bytes = record_bytes(*b"ny  ", b' ', &[("852", subfield_a(CAFE_MARC8))]);
    let record = HoldingsMarcReader::new(Cursor::new(bytes))
        .read_record()
        .unwrap()
        .unwrap();
    assert_eq!(record.locations()[0].get_subfield('a'), Some(CAFE));
}

#[test]
fn holdings_reader_honours_character_coding() {
    let bytes = record_bytes(*b"ny  ", b' ', &[("852", subfield_a(CAFE.as_bytes()))]);
    let record = HoldingsMarcReader::new(Cursor::new(bytes))
        .with_character_coding(CharacterCoding::Detect)
        .read_record()
        .unwrap()
        .unwrap();
    assert_eq!(record.locations()[0].get_subfield('a'), Some(CAFE));
}
