//! Property-based tests for MARC records.
//!
//! The binary round-trip property is the oldest: any structurally valid
//! record we can build should serialize to ISO 2709 and parse back to an
//! equal record. Around it sits a family of structural invariants that
//! inspect the emitted bytes directly (leader length, directory tiling,
//! indicator byte set, subfield code shape), plus round-trip properties
//! for MARCXML and MARCJSON that deliberately exercise format-specific
//! escaping (`< > & " '` for XML; `\t \n \r \\ "` for JSON).
//!
//! # Runtime and configuration
//!
//! `ProptestConfig::cases = 64` keeps the suite under ~10s locally (actual
//! on an Apple-silicon laptop: ~1s). Override with `PROPTEST_CASES=N cargo
//! test` when you want deeper coverage for a single run.
//!
//! # Regression seeds
//!
//! When a property fails, proptest writes the failing seed to
//! `tests/proptest-regressions/properties.txt`. Commit that file — the
//! saved seeds are permanent regression guards that re-run on every test
//! invocation. Only `*.pending` files are gitignored (see `.gitignore`).

use mrrc::{marcjson, marcxml, Field, Leader, MarcReader, MarcWriter, Record, Subfield};
use proptest::prelude::*;
use smallvec::SmallVec;
use std::io::Cursor;

// ============================================================================
// Strategies for generating arbitrary MARC components
// ============================================================================

/// Generate a valid MARC leader.
///
/// `record_length` and `data_base_address` are set to 0 because the writer
/// overwrites them during serialization.
fn arb_leader() -> impl Strategy<Value = Leader> {
    (
        prop_oneof![Just('a'), Just('c'), Just('d'), Just('n'), Just('p')],
        prop_oneof![
            Just('a'),
            Just('c'),
            Just('d'),
            Just('e'),
            Just('f'),
            Just('g'),
            Just('i'),
            Just('j'),
            Just('k'),
            Just('m'),
            Just('o'),
            Just('p'),
            Just('r'),
            Just('t'),
        ],
        prop_oneof![
            Just('a'),
            Just('b'),
            Just('c'),
            Just('d'),
            Just('i'),
            Just('m'),
            Just('s'),
        ],
        prop_oneof![Just(' '), Just('a')], // character coding: MARC-8 or UTF-8
        prop_oneof![Just(' '), Just('a'), Just(' ')], // control_record_type
        prop_oneof![Just(' '), Just('1'), Just('3'), Just('7')], // encoding_level
        prop_oneof![Just(' '), Just('a'), Just('c'), Just('i')], // cataloging_form
        prop_oneof![Just(' '), Just('a'), Just('b'), Just('c')], // multipart_level
    )
        .prop_map(
            |(
                record_status,
                record_type,
                bibliographic_level,
                character_coding,
                control_record_type,
                encoding_level,
                cataloging_form,
                multipart_level,
            )| {
                Leader {
                    record_length: 0,
                    record_status,
                    record_type,
                    bibliographic_level,
                    control_record_type,
                    character_coding,
                    indicator_count: 2,
                    subfield_code_count: 2,
                    data_base_address: 0,
                    encoding_level,
                    cataloging_form,
                    multipart_level,
                    reserved: "4500".to_string(),
                }
            },
        )
}

/// Generate a valid control field tag (001-009).
fn arb_control_tag() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("001".to_string()),
        Just("003".to_string()),
        Just("005".to_string()),
        Just("006".to_string()),
        Just("007".to_string()),
        Just("008".to_string()),
    ]
}

/// Generate ASCII content for a control field (no special MARC delimiters).
fn arb_control_value() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ./:;-]{1,40}".prop_filter("no MARC delimiters", |s| {
        !s.bytes().any(|b| b == 0x1D || b == 0x1E || b == 0x1F)
    })
}

/// Generate a valid data field tag (010-999).
fn arb_data_tag() -> impl Strategy<Value = String> {
    (10u16..1000).prop_map(|n| format!("{n:03}"))
}

/// Generate a valid indicator character (digit or space).
fn arb_indicator() -> impl Strategy<Value = char> {
    prop_oneof![
        Just(' '),
        Just('0'),
        Just('1'),
        Just('2'),
        Just('3'),
        Just('4'),
        Just('5'),
        Just('6'),
        Just('7'),
        Just('8'),
        Just('9'),
    ]
}

/// Generate a valid subfield code (lowercase letter or digit).
fn arb_subfield_code() -> impl Strategy<Value = char> {
    prop_oneof![
        (b'a'..=b'z').prop_map(|b| b as char),
        (b'0'..=b'9').prop_map(|b| b as char),
    ]
}

/// Generate subfield value content for the binary round-trip (ASCII, no
/// MARC delimiters).
fn arb_subfield_value() -> BoxedStrategy<String> {
    "[a-zA-Z0-9 .,:;()'/&-]{1,80}"
        .prop_filter("no MARC delimiters", |s: &String| {
            !s.bytes().any(|b| b == 0x1D || b == 0x1E || b == 0x1F)
        })
        .boxed()
}

/// Generate subfield values that exercise XML escaping edge cases.
///
/// Deliberately includes `<`, `>`, `&`, `"`, `'`, and arbitrary whitespace
/// — the MARCXML reader preserves whitespace-only and whitespace-edge
/// text content verbatim.
fn arb_subfield_value_xml() -> BoxedStrategy<String> {
    "[a-zA-Z0-9 .,:;()<>&\"'/-]{1,80}"
        .prop_filter("no MARC delimiters", |s: &String| {
            !s.bytes().any(|b| b == 0x1D || b == 0x1E || b == 0x1F)
        })
        .boxed()
}

/// Generate subfield values that exercise JSON escaping edge cases.
///
/// Includes control characters (`\t`, `\n`, `\r`), plus `\\` and `"` — the
/// characters `serde_json` must escape to keep the emitted JSON valid.
fn arb_subfield_value_json() -> BoxedStrategy<String> {
    prop::collection::vec(
        prop_oneof![
            Just('\t'),
            Just('\n'),
            Just('\r'),
            Just('\\'),
            Just('"'),
            (b'a'..=b'z').prop_map(|b| b as char),
            (b'A'..=b'Z').prop_map(|b| b as char),
            (b'0'..=b'9').prop_map(|b| b as char),
            Just(' '),
            Just('.'),
            Just(','),
        ],
        1..=80,
    )
    .prop_map(|chars: Vec<char>| chars.into_iter().collect::<String>())
    .prop_filter("no MARC delimiters", |s: &String| {
        !s.bytes().any(|b| b == 0x1D || b == 0x1E || b == 0x1F)
    })
    .boxed()
}

/// Generate a single subfield with the given value strategy.
fn arb_subfield_with(value: BoxedStrategy<String>) -> BoxedStrategy<Subfield> {
    (arb_subfield_code(), value)
        .prop_map(|(code, value)| Subfield { code, value })
        .boxed()
}

/// Generate a data field with indicators and 1-5 subfields using the given
/// value strategy.
fn arb_data_field_with(value: BoxedStrategy<String>) -> BoxedStrategy<Field> {
    (
        arb_data_tag(),
        arb_indicator(),
        arb_indicator(),
        prop::collection::vec(arb_subfield_with(value), 1..=5),
    )
        .prop_map(|(tag, ind1, ind2, subfields)| Field {
            tag,
            indicator1: ind1,
            indicator2: ind2,
            subfields: SmallVec::from_vec(subfields),
        })
        .boxed()
}

/// Control-field value strategy for MARCXML round-trips — pass-through to
/// the shared strategy. The MARCXML reader preserves whitespace-only
/// control values so no extra filtering is needed here.
fn arb_control_value_xml() -> BoxedStrategy<String> {
    arb_control_value().boxed()
}

/// Generate a structurally valid MARC record using the given subfield-value
/// and control-value strategies.
///
/// Ranges cover edge cases called out in bd-ouji: `0..=3` control fields
/// (including zero → no 001, no record-control-number dependency) and
/// `0..=10` data fields (including zero → control-only records).
fn arb_record_with(
    subfield_value: BoxedStrategy<String>,
    control_value: BoxedStrategy<String>,
) -> BoxedStrategy<Record> {
    (
        arb_leader(),
        prop::collection::vec((arb_control_tag(), control_value), 0..=3),
        prop::collection::vec(arb_data_field_with(subfield_value), 0..=10),
    )
        .prop_map(|(leader, control_fields, data_fields)| {
            let mut record = Record::new(leader);
            let mut seen_tags = std::collections::HashSet::new();
            for (tag, value) in control_fields {
                if seen_tags.insert(tag.clone()) {
                    record.add_control_field(tag, value);
                }
            }
            for field in data_fields {
                record.add_field(field);
            }
            record
        })
        .boxed()
}

/// Default record strategy: ASCII subfield values, safe for binary
/// round-trips and structural-invariant inspection.
fn arb_record() -> BoxedStrategy<Record> {
    arb_record_with(arb_subfield_value(), arb_control_value().boxed())
}

/// Record strategy for MARCXML round-trips.
fn arb_record_xml() -> BoxedStrategy<Record> {
    arb_record_with(arb_subfield_value_xml(), arb_control_value_xml())
}

/// Record strategy for MARCJSON round-trips.
fn arb_record_json() -> BoxedStrategy<Record> {
    arb_record_with(arb_subfield_value_json(), arb_control_value().boxed())
}

// ============================================================================
// Byte-level helpers for structural invariants
// ============================================================================

const LEADER_LEN: usize = 24;
const DIRECTORY_ENTRY_LEN: usize = 12;
const FIELD_TERMINATOR: u8 = 0x1E;
const RECORD_TERMINATOR: u8 = 0x1D;
const SUBFIELD_DELIMITER: u8 = 0x1F;

/// Parse the 5-byte ASCII record length from the leader.
fn parse_record_length(bytes: &[u8]) -> usize {
    std::str::from_utf8(&bytes[0..5])
        .expect("leader length is ASCII")
        .parse()
        .expect("leader length is numeric")
}

/// Parse the 5-byte ASCII data-base-address from the leader.
fn parse_data_base_address(bytes: &[u8]) -> usize {
    std::str::from_utf8(&bytes[12..17])
        .expect("leader base address is ASCII")
        .parse()
        .expect("leader base address is numeric")
}

/// A single directory entry: `(tag, length, start)`.
#[derive(Debug)]
struct DirEntry {
    tag: String,
    length: usize,
    start: usize,
}

/// Parse all directory entries from the serialized record.
fn parse_directory(bytes: &[u8]) -> Vec<DirEntry> {
    let base = parse_data_base_address(bytes);
    // Directory runs from LEADER_LEN up to base-1 (the byte at base-1 is a
    // FIELD_TERMINATOR separating directory from data area).
    let dir = &bytes[LEADER_LEN..base - 1];
    dir.chunks_exact(DIRECTORY_ENTRY_LEN)
        .map(|chunk| DirEntry {
            tag: std::str::from_utf8(&chunk[0..3])
                .expect("tag is ASCII")
                .to_string(),
            length: std::str::from_utf8(&chunk[3..7])
                .expect("length is ASCII")
                .parse()
                .expect("length is numeric"),
            start: std::str::from_utf8(&chunk[7..12])
                .expect("start is ASCII")
                .parse()
                .expect("start is numeric"),
        })
        .collect()
}

/// Serialize a record to ISO 2709 bytes (test convenience).
fn emit_binary(record: &Record) -> Vec<u8> {
    let mut buffer = Vec::new();
    {
        let mut writer = MarcWriter::new(&mut buffer);
        writer.write_record(record).expect("write should succeed");
    }
    buffer
}

// ============================================================================
// Property tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64,
        ..ProptestConfig::default()
    })]

    /// Binary round-trip: any record we can build, we can serialize to
    /// ISO 2709 and parse back to a structurally identical record.
    #[test]
    fn binary_roundtrip(record in arb_record()) {
        let buffer = emit_binary(&record);

        let cursor = Cursor::new(&buffer);
        let mut reader = MarcReader::new(cursor);
        let parsed = reader
            .read_record()
            .expect("read should succeed")
            .expect("should get a record");

        // Compare leader fields (skip record_length and data_base_address
        // since those are computed by the writer and not part of the input)
        prop_assert_eq!(record.leader.record_status, parsed.leader.record_status);
        prop_assert_eq!(record.leader.record_type, parsed.leader.record_type);
        prop_assert_eq!(record.leader.bibliographic_level, parsed.leader.bibliographic_level);
        prop_assert_eq!(record.leader.control_record_type, parsed.leader.control_record_type);
        prop_assert_eq!(record.leader.character_coding, parsed.leader.character_coding);
        prop_assert_eq!(record.leader.indicator_count, parsed.leader.indicator_count);
        prop_assert_eq!(record.leader.subfield_code_count, parsed.leader.subfield_code_count);
        prop_assert_eq!(record.leader.encoding_level, parsed.leader.encoding_level);
        prop_assert_eq!(record.leader.cataloging_form, parsed.leader.cataloging_form);
        prop_assert_eq!(record.leader.multipart_level, parsed.leader.multipart_level);
        prop_assert_eq!(&record.leader.reserved, &parsed.leader.reserved);

        // Compare control fields
        prop_assert_eq!(record.control_fields.len(), parsed.control_fields.len());
        for (tag, value) in &record.control_fields {
            let parsed_value = parsed.control_fields.get(tag);
            prop_assert_eq!(Some(value), parsed_value,
                "control field {} mismatch", tag);
        }

        // Compare data fields
        let orig_fields: Vec<&Field> = record.fields().collect();
        let parsed_fields: Vec<&Field> = parsed.fields().collect();
        prop_assert_eq!(orig_fields.len(), parsed_fields.len(),
            "field count mismatch");

        for (orig, roundtripped) in orig_fields.iter().zip(parsed_fields.iter()) {
            prop_assert_eq!(&orig.tag, &roundtripped.tag);
            prop_assert_eq!(orig.indicator1, roundtripped.indicator1);
            prop_assert_eq!(orig.indicator2, roundtripped.indicator2);
            prop_assert_eq!(orig.subfields.len(), roundtripped.subfields.len(),
                "subfield count mismatch in field {}", orig.tag);

            for (orig_sf, parsed_sf) in orig.subfields.iter().zip(roundtripped.subfields.iter()) {
                prop_assert_eq!(orig_sf.code, parsed_sf.code);
                prop_assert_eq!(&orig_sf.value, &parsed_sf.value);
            }
        }

        // Verify no more records in the buffer
        let next = reader.read_record().expect("read should succeed");
        prop_assert!(next.is_none(), "expected exactly one record in buffer");
    }

    /// Serialization should always produce valid bytes (no panic, no error).
    #[test]
    fn serialization_never_panics(record in arb_record()) {
        let mut buf = Vec::new();
        let mut writer = MarcWriter::new(&mut buf);
        let result = writer.write_record(&record);
        prop_assert!(result.is_ok(), "MarcWriter failed: {:?}", result.err());
        prop_assert!(!buf.is_empty(), "Serialized record is empty");
    }

    /// The record-length field in the leader must match the number of bytes
    /// the writer actually emitted.
    #[test]
    fn leader_length_matches_emitted_bytes(record in arb_record()) {
        let buffer = emit_binary(&record);
        let declared = parse_record_length(&buffer);
        prop_assert_eq!(declared, buffer.len(),
            "leader.record_length ({}) != emitted byte count ({})",
            declared, buffer.len());
    }

    /// Every directory entry must tile the data area exactly: entries
    /// start immediately after one another, their lengths (each of which
    /// includes the field terminator) sum to the data-area size minus the
    /// record terminator, and no entry points outside the data area.
    #[test]
    fn directory_entries_tile_data_area(record in arb_record()) {
        let buffer = emit_binary(&record);
        let base = parse_data_base_address(&buffer);
        let record_length = parse_record_length(&buffer);
        let entries = parse_directory(&buffer);

        let data_area_size = record_length - base;
        prop_assert_eq!(buffer[record_length - 1], RECORD_TERMINATOR,
            "last byte should be RECORD_TERMINATOR");

        let mut expected_start = 0usize;
        let mut total_length = 0usize;
        for entry in &entries {
            prop_assert_eq!(entry.start, expected_start,
                "entry {} start {} != expected {}",
                entry.tag, entry.start, expected_start);
            prop_assert!(entry.length >= 1,
                "entry {} length is zero", entry.tag);
            let field_end_in_data = entry.start + entry.length;
            prop_assert!(field_end_in_data < data_area_size,
                "entry {} extends past data area (end={}, data_area_size={})",
                entry.tag, field_end_in_data, data_area_size);
            // Each field's declared length includes its trailing FIELD_TERMINATOR.
            let term_pos = base + entry.start + entry.length - 1;
            prop_assert_eq!(buffer[term_pos], FIELD_TERMINATOR,
                "entry {} should end with FIELD_TERMINATOR at {}",
                entry.tag, term_pos);
            expected_start += entry.length;
            total_length += entry.length;
        }

        // Lengths plus the trailing record-terminator byte should tile the
        // entire data area.
        prop_assert_eq!(total_length + 1, data_area_size,
            "sum of field lengths ({}) + RECORD_TERMINATOR != data area size ({})",
            total_length, data_area_size);
    }

    /// Every indicator byte emitted in a data field must be a digit or a
    /// space. The leader declares `indicator_count = 2`, so indicators
    /// occupy the first two bytes of each data-field block.
    #[test]
    fn indicator_bytes_in_valid_set(record in arb_record()) {
        let buffer = emit_binary(&record);
        let base = parse_data_base_address(&buffer);
        for entry in parse_directory(&buffer) {
            if entry.tag.as_str() < "010" {
                continue; // control fields have no indicators
            }
            let ind1 = buffer[base + entry.start];
            let ind2 = buffer[base + entry.start + 1];
            for (label, byte) in [("ind1", ind1), ("ind2", ind2)] {
                prop_assert!(
                    byte == b' ' || byte.is_ascii_digit(),
                    "{} for tag {} is 0x{:02x} (not digit or space)",
                    label, entry.tag, byte
                );
            }
        }
    }

    /// Every byte immediately following a SUBFIELD_DELIMITER (0x1F) is a
    /// subfield code, which must be a lowercase ASCII letter or digit.
    #[test]
    fn subfield_codes_are_lower_alnum(record in arb_record()) {
        let buffer = emit_binary(&record);
        for (i, byte) in buffer.iter().enumerate() {
            if *byte == SUBFIELD_DELIMITER {
                let code = buffer[i + 1];
                prop_assert!(
                    code.is_ascii_lowercase() || code.is_ascii_digit(),
                    "subfield code at offset {} is 0x{:02x} (expected [a-z0-9])",
                    i + 1, code
                );
            }
        }
    }

    /// MARCXML round-trip: a record with subfield values containing XML
    /// metacharacters (`< > & " '`) and arbitrary whitespace must survive
    /// serialize → parse, including whitespace-only and whitespace-edge
    /// text content.
    #[test]
    fn marcxml_roundtrip(record in arb_record_xml()) {
        let xml = marcxml::record_to_marcxml(&record).expect("MARCXML serialize");
        let parsed = marcxml::marcxml_to_record(&xml).expect("MARCXML parse");
        assert_records_equal(&record, &parsed)?;
    }

    /// MARCJSON round-trip: a record with subfield values containing
    /// JSON-problematic characters (control chars, `\\`, `"`) must survive
    /// serialize → parse.
    #[test]
    fn marcjson_roundtrip(record in arb_record_json()) {
        let json = marcjson::record_to_marcjson(&record).expect("MARCJSON serialize");
        let parsed = marcjson::marcjson_to_record(&json).expect("MARCJSON parse");
        assert_records_equal(&record, &parsed)?;
    }
}

/// Compare two records for structural equality, ignoring leader fields the
/// writer computes (`record_length`, `data_base_address`).
fn assert_records_equal(orig: &Record, parsed: &Record) -> Result<(), TestCaseError> {
    prop_assert_eq!(orig.leader.record_status, parsed.leader.record_status);
    prop_assert_eq!(orig.leader.record_type, parsed.leader.record_type);
    prop_assert_eq!(
        orig.leader.bibliographic_level,
        parsed.leader.bibliographic_level
    );
    prop_assert_eq!(orig.leader.character_coding, parsed.leader.character_coding);
    prop_assert_eq!(&orig.leader.reserved, &parsed.leader.reserved);

    prop_assert_eq!(orig.control_fields.len(), parsed.control_fields.len());
    for (tag, values) in &orig.control_fields {
        let parsed_values = parsed.control_fields.get(tag);
        prop_assert_eq!(
            Some(values),
            parsed_values,
            "control field {} mismatch",
            tag
        );
    }

    let orig_fields: Vec<&Field> = orig.fields().collect();
    let parsed_fields: Vec<&Field> = parsed.fields().collect();
    prop_assert_eq!(
        orig_fields.len(),
        parsed_fields.len(),
        "field count mismatch"
    );

    for (orig_f, parsed_f) in orig_fields.iter().zip(parsed_fields.iter()) {
        prop_assert_eq!(&orig_f.tag, &parsed_f.tag);
        prop_assert_eq!(orig_f.indicator1, parsed_f.indicator1);
        prop_assert_eq!(orig_f.indicator2, parsed_f.indicator2);
        prop_assert_eq!(
            orig_f.subfields.len(),
            parsed_f.subfields.len(),
            "subfield count in {} mismatch",
            orig_f.tag
        );
        for (orig_sf, parsed_sf) in orig_f.subfields.iter().zip(parsed_f.subfields.iter()) {
            prop_assert_eq!(orig_sf.code, parsed_sf.code);
            prop_assert_eq!(&orig_sf.value, &parsed_sf.value);
        }
    }

    Ok(())
}
