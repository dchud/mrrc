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

use mrrc::{
    AuthorityMarcReader, AuthorityMarcWriter, AuthorityRecord, Field, HoldingsMarcReader,
    HoldingsMarcWriter, HoldingsRecord, Leader, MarcError, MarcReader, MarcWriter, Record,
    RecoveryMode, Subfield, ValidationLevel, marcjson, marcxml,
};
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
/// Ranges cover the structural edge cases: `0..=3` control fields
/// (including zero, so the no-001 / no-record-control-number case is
/// exercised) and `0..=10` data fields (including zero, so control-only
/// records are exercised).
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

/// Assert the ISO 2709 framing invariants on emitted bytes: the leader's
/// record-length field matches the byte count, the record ends with a
/// `RECORD_TERMINATOR`, and the directory entries tile the data area exactly
/// (consecutive starts, terminator-inclusive lengths, no entry past the
/// data area). Shared by the bibliographic, authority, and holdings writer
/// properties — all three writers emit the same framing.
fn assert_iso2709_framing(buffer: &[u8]) -> Result<(), TestCaseError> {
    let declared = parse_record_length(buffer);
    prop_assert_eq!(
        declared,
        buffer.len(),
        "leader.record_length ({}) != emitted byte count ({})",
        declared,
        buffer.len()
    );

    let base = parse_data_base_address(buffer);
    let record_length = parse_record_length(buffer);
    let entries = parse_directory(buffer);

    let data_area_size = record_length - base;
    prop_assert_eq!(
        buffer[record_length - 1],
        RECORD_TERMINATOR,
        "last byte should be RECORD_TERMINATOR"
    );

    let mut expected_start = 0usize;
    let mut total_length = 0usize;
    for entry in &entries {
        prop_assert_eq!(
            entry.start,
            expected_start,
            "entry {} start {} != expected {}",
            entry.tag,
            entry.start,
            expected_start
        );
        prop_assert!(entry.length >= 1, "entry {} length is zero", entry.tag);
        let field_end_in_data = entry.start + entry.length;
        prop_assert!(
            field_end_in_data < data_area_size,
            "entry {} extends past data area (end={}, data_area_size={})",
            entry.tag,
            field_end_in_data,
            data_area_size
        );
        // Each field's declared length includes its trailing FIELD_TERMINATOR.
        let term_pos = base + entry.start + entry.length - 1;
        prop_assert_eq!(
            buffer[term_pos],
            FIELD_TERMINATOR,
            "entry {} should end with FIELD_TERMINATOR at {}",
            entry.tag,
            term_pos
        );
        expected_start += entry.length;
        total_length += entry.length;
    }

    // Lengths plus the trailing record-terminator byte should tile the
    // entire data area.
    prop_assert_eq!(
        total_length + 1,
        data_area_size,
        "sum of field lengths ({}) + RECORD_TERMINATOR != data area size ({})",
        total_length,
        data_area_size
    );

    Ok(())
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

    /// ISO 2709 framing invariants on the emitted bytes: the leader's
    /// record-length field matches the byte count, and directory entries
    /// tile the data area exactly (consecutive starts, terminator-inclusive
    /// lengths, no entry pointing outside the data area).
    #[test]
    fn bibliographic_writer_emits_valid_iso2709_framing(record in arb_record()) {
        let buffer = emit_binary(&record);
        assert_iso2709_framing(&buffer)?;
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

// ============================================================================
// Malformation strategies — generate INVALID inputs from valid records
// ============================================================================
//
// Each strategy starts from a structurally valid record (or a controlled
// truncation) and applies a single, named mutation. The companion property
// asserts the parser produces a specific MarcError variant with populated
// positional context. Together with `tests/error_coverage.rs` (named-case
// integration coverage seeded from `tests/error_coverage.toml`) and the
// fuzz harnesses under `fuzz/fuzz_targets/`, this covers parser-error
// wiring at all three test layers; proptest's shrinking surfaces minimal
// counter-examples that integration tests would miss.

/// Generate a valid MARC record that always has at least one data field
/// (and therefore at least one subfield, per `arb_data_field_with`).
/// Mutation strategies that target indicators, subfield codes, or
/// directory entries need a non-empty data area to mutate.
fn arb_record_with_data_field() -> BoxedStrategy<Record> {
    (
        arb_leader(),
        prop::collection::vec((arb_control_tag(), arb_control_value()), 0..=2),
        prop::collection::vec(arb_data_field_with(arb_subfield_value()), 1..=5),
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

/// Generate a byte that is neither an ASCII digit nor a space — i.e., a
/// byte that fails the universal indicator-validity check.
fn arb_non_digit_non_space() -> impl Strategy<Value = u8> {
    (0u8..=255u8).prop_filter("not digit or space", |b| !b.is_ascii_digit() && *b != b' ')
}

/// Generate a byte that, placed in a leader's 5-byte numeric field
/// (record-length 0..5 or base-address 12..17), causes `parse_digits` to
/// return `None`. The parser uses `u32::from_str`, which accepts a
/// leading `'+'` or `'-'` — so excluding digits alone is insufficient
/// for the record-length-field and base-address mutations to reliably
/// fire E001/E003.
fn arb_non_numeric_byte() -> impl Strategy<Value = u8> {
    (0u8..=255u8).prop_filter("not digit or sign", |b| {
        !b.is_ascii_digit() && *b != b'+' && *b != b'-'
    })
}

/// Generate a byte outside the ASCII-graphic range (0x21..=0x7E) — i.e.,
/// a byte that fails the subfield-code-validity check at `strict_marc`.
/// The parser uses `is_ascii_graphic` for this check; any printable
/// character (including punctuation) is accepted, only control bytes and
/// 0x80+ trip `E202`.
fn arb_non_graphic_subfield_byte() -> impl Strategy<Value = u8> {
    (0u8..=255u8).prop_filter("not ASCII graphic", |b| !b.is_ascii_graphic())
}

/// First data-field offset in `buffer` (start of the indicator-1 byte)
/// plus the directory entry's tag. Returns `None` if the record has only
/// control fields, which `arb_record_with_data_field()` rules out.
fn first_data_field_offset(buffer: &[u8]) -> Option<(String, usize)> {
    let base = parse_data_base_address(buffer);
    parse_directory(buffer)
        .into_iter()
        .find(|e| e.tag.as_str() >= "010")
        .map(|e| (e.tag, base + e.start))
}

/// First subfield-delimiter (0x1F) offset in `buffer`. The byte that
/// follows is the subfield code targeted for mutation.
fn first_subfield_delimiter_offset(buffer: &[u8]) -> Option<usize> {
    buffer.iter().position(|b| *b == SUBFIELD_DELIMITER)
}

/// Truncated leader (< 24 bytes): any sub-leader byte string the reader
/// must reject.
fn arb_invalid_leader_truncated() -> BoxedStrategy<Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..LEADER_LEN).boxed()
}

/// Valid record with one byte in 0..5 (record-length field) replaced
/// with a non-numeric byte. Triggers a leader-stage error (`E001` or
/// `E002`).
fn arb_invalid_leader_bad_length_field() -> BoxedStrategy<Vec<u8>> {
    (arb_record(), 0usize..5, arb_non_numeric_byte())
        .prop_map(|(record, pos, bad_byte)| {
            let mut buffer = emit_binary(&record);
            buffer[pos] = bad_byte;
            buffer
        })
        .boxed()
}

/// Valid record with one byte in 12..17 (base-address field) replaced
/// with a non-numeric byte. Triggers `E003` `BaseAddressInvalid`.
fn arb_invalid_leader_bad_base_address() -> BoxedStrategy<Vec<u8>> {
    (arb_record(), 12usize..17, arb_non_numeric_byte())
        .prop_map(|(record, pos, bad_byte)| {
            let mut buffer = emit_binary(&record);
            buffer[pos] = bad_byte;
            buffer
        })
        .boxed()
}

/// Valid record with one indicator byte mutated to a value outside the
/// universal "digit or space" set. Returns `(bytes, expected_tag)` so
/// the property can assert the variant carries the right field tag.
fn arb_record_with_bad_indicator() -> BoxedStrategy<(Vec<u8>, String)> {
    (
        arb_record_with_data_field(),
        arb_non_digit_non_space(),
        0u8..=1u8,
    )
        .prop_map(|(record, bad_byte, ind_pos)| {
            let mut buffer = emit_binary(&record);
            let (tag, offset) = first_data_field_offset(&buffer)
                .expect("arb_record_with_data_field guarantees a data field");
            buffer[offset + ind_pos as usize] = bad_byte;
            (buffer, tag)
        })
        .boxed()
}

/// Valid record with the byte after the first subfield delimiter (the
/// subfield code) mutated to a value outside the lowercase-alnum set.
/// Triggers `E202` `BadSubfieldCode` at `strict_marc` validation.
///
/// Uses a single 020 (ISBN) field with both indicators set to space so
/// the per-tag `IndicatorValidator` (which runs at `strict_marc`
/// alongside the universal byte-validity check) doesn't fire before the
/// parser reaches the subfield-code check. 020's per-tag rule is
/// `Undefined`/`Undefined`, which accepts space.
fn arb_record_with_bad_subfield_code() -> BoxedStrategy<Vec<u8>> {
    (arb_subfield_value(), arb_non_graphic_subfield_byte())
        .prop_map(|(value, bad_byte)| {
            let leader = Leader {
                record_length: 0,
                record_status: 'n',
                record_type: 'a',
                bibliographic_level: 'm',
                control_record_type: ' ',
                character_coding: 'a',
                indicator_count: 2,
                subfield_code_count: 2,
                data_base_address: 0,
                encoding_level: ' ',
                cataloging_form: ' ',
                multipart_level: ' ',
                reserved: "4500".to_string(),
            };
            let mut record = Record::new(leader);
            record.add_field(Field {
                tag: "020".to_string(),
                indicator1: ' ',
                indicator2: ' ',
                subfields: SmallVec::from_vec(vec![Subfield { code: 'a', value }]),
            });
            let mut buffer = emit_binary(&record);
            let delim = first_subfield_delimiter_offset(&buffer).expect("020 field has a subfield");
            buffer[delim + 1] = bad_byte;
            buffer
        })
        .boxed()
}

/// Valid record with the directory's terminating `FIELD_TERMINATOR`
/// (the byte at base-1) replaced with an ASCII digit, producing a
/// partial trailing entry. Triggers `E101` `DirectoryInvalid`.
fn arb_record_with_directory_violation() -> BoxedStrategy<Vec<u8>> {
    arb_record_with_data_field()
        .prop_map(|record| {
            let mut buffer = emit_binary(&record);
            let base = parse_data_base_address(&buffer);
            buffer[base - 1] = b'0';
            buffer
        })
        .boxed()
}

/// Valid record truncated after a parametrized prefix length in
/// [`LEADER_LEN`, total). Triggers `E005` `TruncatedRecord` (or, when
/// the directory is partially readable, an earlier structural error).
fn arb_record_truncated_after_offset() -> BoxedStrategy<Vec<u8>> {
    (arb_record(), any::<u32>())
        .prop_map(|(record, seed)| {
            let buffer = emit_binary(&record);
            let total = buffer.len();
            let span = (total - LEADER_LEN).max(1);
            let prefix_len = LEADER_LEN + (seed as usize) % span;
            buffer[..prefix_len].to_vec()
        })
        .boxed()
}

/// Valid record with the final `RECORD_TERMINATOR` byte replaced with
/// 0x00. Triggers `E006` `EndOfRecordNotFound`.
fn arb_record_missing_record_terminator() -> BoxedStrategy<Vec<u8>> {
    arb_record()
        .prop_map(|record| {
            let mut buffer = emit_binary(&record);
            let last = buffer.len() - 1;
            buffer[last] = 0x00;
            buffer
        })
        .boxed()
}

/// One malformed but recoverable record: the leader and `record_length`
/// are intact (so the reader can advance to the next record), but the
/// length field of the first directory entry is corrupted to non-digit
/// bytes. Used to build pathological streams for the recovery-cap
/// invariant. Mirrors `build_bad_record` in `src/reader.rs` tests.
fn arb_malformed_recoverable_record() -> BoxedStrategy<Vec<u8>> {
    arb_record_with_data_field()
        .prop_map(|record| {
            let mut buffer = emit_binary(&record);
            // Directory entry bytes 27..31 are the 4-byte length field of
            // the first entry. Replace each with 'X' so directory parse
            // fails per-record but leader/length stay valid for stream
            // advancement.
            for byte in &mut buffer[27..31] {
                *byte = b'X';
            }
            buffer
        })
        .boxed()
}

// ============================================================================
// Malformation properties — assert variant + positional context
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64,
        ..ProptestConfig::default()
    })]

    /// A leader truncated to fewer than 24 bytes must not parse as a
    /// record. Either the reader returns Ok(None) or Err(_); never
    /// Ok(Some(_)) and never a panic.
    #[test]
    fn truncated_leader_never_yields_record(bytes in arb_invalid_leader_truncated()) {
        let mut reader = MarcReader::new(Cursor::new(&bytes[..]));
        let result = reader.read_record();
        prop_assert!(
            !matches!(&result, Ok(Some(_))),
            "truncated leader should not parse as a record, got {result:?}"
        );
    }

    /// A non-digit byte in the leader's record-length field (0..5) must
    /// surface as a leader-stage error (E001 RecordLengthInvalid or, when
    /// the malformed leader fails an earlier check, E002 InvalidLeader)
    /// with positional context populated.
    #[test]
    fn malformed_record_length_yields_leader_error(
        bytes in arb_invalid_leader_bad_length_field(),
    ) {
        let mut reader = MarcReader::new(Cursor::new(&bytes[..]));
        let result = reader.read_record();
        match result {
            Err(MarcError::RecordLengthInvalid {
                record_index, byte_offset, ..
            }) => {
                prop_assert!(record_index.is_some(), "record_index missing");
                prop_assert!(byte_offset.is_some(), "byte_offset missing");
            },
            Err(MarcError::InvalidLeader {
                record_index, byte_offset, ..
            }) => {
                prop_assert!(record_index.is_some(), "record_index missing");
                prop_assert!(byte_offset.is_some(), "byte_offset missing");
            },
            other => prop_assert!(
                false,
                "expected RecordLengthInvalid or InvalidLeader, got {other:?}"
            ),
        }
    }

    /// A non-digit byte in the leader's base-address field (12..17) must
    /// surface as E003 BaseAddressInvalid with positional context.
    #[test]
    fn malformed_base_address_yields_e003(
        bytes in arb_invalid_leader_bad_base_address(),
    ) {
        let mut reader = MarcReader::new(Cursor::new(&bytes[..]));
        let result = reader.read_record();
        match result {
            Err(MarcError::BaseAddressInvalid {
                record_index, byte_offset, ..
            }) => {
                prop_assert!(record_index.is_some());
                prop_assert!(byte_offset.is_some());
            },
            other => prop_assert!(false, "expected BaseAddressInvalid, got {other:?}"),
        }
    }

    /// A non-digit-non-space indicator byte must surface as E201
    /// InvalidIndicator at strict_marc validation, with field_tag,
    /// indicator_position, and the standard positional fields populated.
    #[test]
    fn malformed_indicator_yields_e201(
        (bytes, expected_tag) in arb_record_with_bad_indicator(),
    ) {
        let mut reader = MarcReader::new(Cursor::new(&bytes[..]))
            .with_validation_level(ValidationLevel::StrictMarc);
        let result = reader.read_record();
        match result {
            Err(MarcError::InvalidIndicator {
                field_tag,
                indicator_position,
                record_index,
                byte_offset,
                record_byte_offset,
                ..
            }) => {
                prop_assert_eq!(field_tag.as_deref(), Some(expected_tag.as_str()));
                prop_assert!(indicator_position.is_some());
                prop_assert!(record_index.is_some());
                prop_assert!(byte_offset.is_some());
                prop_assert!(record_byte_offset.is_some());
            },
            other => prop_assert!(false, "expected InvalidIndicator, got {other:?}"),
        }
    }

    /// A non-printable subfield-code byte must surface as E202
    /// BadSubfieldCode at strict_marc validation, with field_tag and the
    /// standard positional fields populated.
    #[test]
    fn malformed_subfield_code_yields_e202(
        bytes in arb_record_with_bad_subfield_code(),
    ) {
        let mut reader = MarcReader::new(Cursor::new(&bytes[..]))
            .with_validation_level(ValidationLevel::StrictMarc);
        let result = reader.read_record();
        match result {
            Err(MarcError::BadSubfieldCode {
                field_tag,
                record_index,
                byte_offset,
                record_byte_offset,
                ..
            }) => {
                prop_assert!(field_tag.is_some());
                prop_assert!(record_index.is_some());
                prop_assert!(byte_offset.is_some());
                prop_assert!(record_byte_offset.is_some());
            },
            other => prop_assert!(false, "expected BadSubfieldCode, got {other:?}"),
        }
    }

    /// A directory missing its terminating FIELD_TERMINATOR must surface
    /// as E101 DirectoryInvalid with positional context.
    #[test]
    fn directory_violation_yields_e101(
        bytes in arb_record_with_directory_violation(),
    ) {
        let mut reader = MarcReader::new(Cursor::new(&bytes[..]));
        let result = reader.read_record();
        match result {
            Err(MarcError::DirectoryInvalid {
                record_index, byte_offset, ..
            }) => {
                prop_assert!(record_index.is_some());
                prop_assert!(byte_offset.is_some());
            },
            other => prop_assert!(false, "expected DirectoryInvalid, got {other:?}"),
        }
    }

    /// A record truncated before its leader-claimed length is reached
    /// must surface as a parse error (typically E005 TruncatedRecord;
    /// some prefix lengths land on a partially-readable directory and
    /// fire DirectoryInvalid or InvalidField instead). The unifying
    /// invariant is: the malformation is detected, never parsed as a
    /// successful record.
    #[test]
    fn truncated_record_never_yields_record(
        bytes in arb_record_truncated_after_offset(),
    ) {
        let mut reader = MarcReader::new(Cursor::new(&bytes[..]));
        let result = reader.read_record();
        prop_assert!(
            !matches!(&result, Ok(Some(_))),
            "truncated record should not parse as a record, got {result:?}"
        );
    }

    /// A record whose final byte is not RECORD_TERMINATOR must surface
    /// as E006 EndOfRecordNotFound with positional context.
    #[test]
    fn missing_record_terminator_yields_e006(
        bytes in arb_record_missing_record_terminator(),
    ) {
        let mut reader = MarcReader::new(Cursor::new(&bytes[..]));
        let result = reader.read_record();
        match result {
            Err(MarcError::EndOfRecordNotFound {
                record_index, byte_offset, ..
            }) => {
                prop_assert!(record_index.is_some());
                prop_assert!(byte_offset.is_some());
            },
            other => prop_assert!(false, "expected EndOfRecordNotFound, got {other:?}"),
        }
    }
}

// ============================================================================
// Recovery-mode invariant properties
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64,
        ..ProptestConfig::default()
    })]

    /// Lenient mode must not panic on any byte sequence. This is the
    /// same property the libfuzzer harness covers, but proptest's
    /// shrinking turns any failure into a minimal counter-example.
    #[test]
    fn lenient_mode_never_panics_on_arbitrary_bytes(
        bytes in prop::collection::vec(any::<u8>(), 0..2048),
    ) {
        let mut reader = MarcReader::new(Cursor::new(&bytes[..]))
            .with_recovery_mode(RecoveryMode::Lenient);
        while let Ok(Some(_)) = reader.read_record() {}
    }

    /// On the lenient path, the record-body buffer's allocation is bounded
    /// by the bytes the stream actually delivers (plus one growth chunk),
    /// never by the leader's claimed record length: a tiny stub claiming
    /// the ISO 2709 maximum must not force a maximum-length allocation.
    #[test]
    fn lenient_read_allocation_bounded_by_input_size(
        record_length in (LEADER_LEN + 1)..=99_999usize,
        available_seed in any::<u32>(),
    ) {
        let expected_body = record_length - LEADER_LEN;
        // Strictly fewer bytes than claimed, so the lenient short-read
        // path runs (0 up to expected_body - 1 bytes available).
        let available = (available_seed as usize) % expected_body;
        let body = vec![b'x'; available];

        let mut cursor = Cursor::new(&body[..]);
        let ctx = mrrc::iso2709::ParseContext::new();
        let (data, bytes_read) = mrrc::iso2709::read_record_data(
            &mut cursor,
            record_length,
            RecoveryMode::Lenient,
            &ctx,
        ).expect("lenient short read must not error");

        prop_assert_eq!(bytes_read, available);
        prop_assert_eq!(data.len(), available, "no padding to the claimed length");
        prop_assert!(
            data.capacity() <= available + mrrc::iso2709::READ_CHUNK_LEN,
            "capacity {} exceeds available {} + chunk {}",
            data.capacity(),
            available,
            mrrc::iso2709::READ_CHUNK_LEN
        );
    }

    /// Permissive mode must swallow record-internal malformations: when
    /// the leader parses cleanly (so the reader can advance to the next
    /// record boundary) but the field area is malformed, no error
    /// propagates to the caller. Stream-level errors (unparseable leader
    /// length, etc.) are out of scope — the reader cannot establish
    /// record boundaries to skip past them, so they correctly surface.
    #[test]
    fn permissive_mode_swallows_field_level_malformations(
        bytes in prop_oneof![
            arb_record_with_directory_violation(),
            arb_record_missing_record_terminator(),
        ],
    ) {
        let mut reader = MarcReader::new(Cursor::new(&bytes[..]))
            .with_recovery_mode(RecoveryMode::Permissive);
        loop {
            match reader.read_record() {
                Ok(Some(_)) => {},
                Ok(None) => break,
                Err(e) => {
                    prop_assert!(
                        false,
                        "permissive should swallow field-level malformations; got {e:?}"
                    );
                    break;
                },
            }
        }
    }

    /// In Lenient mode with `with_max_errors(N)`, a stream of strictly
    /// more than N malformed records must eventually surface
    /// FatalReaderError. After that, the reader is exhausted.
    #[test]
    fn recovery_cap_eventually_trips_on_pathological_inputs(
        n in 1usize..=4usize,
        records in prop::collection::vec(arb_malformed_recoverable_record(), 6..=10),
    ) {
        let mut stream = Vec::new();
        for r in &records {
            stream.extend_from_slice(r);
        }
        let mut reader = MarcReader::new(Cursor::new(&stream[..]))
            .with_recovery_mode(RecoveryMode::Lenient)
            .with_max_errors(n);

        let mut got_fatal = false;
        loop {
            match reader.read_record() {
                Ok(Some(_)) => {},
                Ok(None) => break,
                Err(MarcError::FatalReaderError { cap, errors_seen, .. }) => {
                    prop_assert_eq!(cap, n);
                    prop_assert!(errors_seen > n);
                    got_fatal = true;
                    break;
                },
                Err(e) => prop_assert!(false, "unexpected error in Lenient mode: {e:?}"),
            }
        }
        prop_assert!(
            got_fatal,
            "cap N={} should have tripped on {} malformed records",
            n, records.len()
        );

        // After FatalReaderError, the reader is exhausted.
        prop_assert!(matches!(reader.read_record(), Ok(None)));
    }
}

// ============================================================================
// Authority and holdings strategies and round-trip properties
// ============================================================================
//
// The authority and holdings writers share the ISO 2709 framing rules with
// the bibliographic writer but are separate implementations. These
// properties give both writers the same generative verification the
// bibliographic writer has: anything the strategies can build must
// serialize, parse back structurally identical through the paired reader,
// and satisfy the byte-level framing invariants.

/// Generate a leader for an authority record (leader/06 = `'z'`).
///
/// Record status draws from the MARC 21 Authority position-5 set.
/// Positions 7 and 8 are undefined for authority records, so the strategy
/// includes the fill character there. Encoding level draws from the
/// authority position-17 set {n, o}.
fn arb_authority_leader() -> impl Strategy<Value = Leader> {
    (
        prop_oneof![
            Just('a'),
            Just('c'),
            Just('d'),
            Just('n'),
            Just('o'),
            Just('s'),
            Just('x'),
        ],
        prop_oneof![Just(' '), Just('|')], // position 7: undefined
        prop_oneof![Just(' '), Just('|')], // position 8: undefined
        prop_oneof![Just(' '), Just('a')], // character coding
        prop_oneof![Just('n'), Just('o')], // encoding level
    )
        .prop_map(
            |(
                record_status,
                bibliographic_level,
                control_record_type,
                character_coding,
                encoding_level,
            )| {
                Leader {
                    record_length: 0,
                    record_status,
                    record_type: 'z',
                    bibliographic_level,
                    control_record_type,
                    character_coding,
                    indicator_count: 2,
                    subfield_code_count: 2,
                    data_base_address: 0,
                    encoding_level,
                    cataloging_form: ' ',
                    multipart_level: ' ',
                    reserved: "4500".to_string(),
                }
            },
        )
}

/// Generate a leader for a holdings record (leader/06 in {x, y, v, u}).
///
/// Record status draws from the MARC 21 Holdings position-5 set {c, d, n};
/// encoding level from the holdings position-17 set.
fn arb_holdings_leader() -> impl Strategy<Value = Leader> {
    (
        prop_oneof![Just('c'), Just('d'), Just('n')],
        prop_oneof![Just('x'), Just('y'), Just('v'), Just('u')],
        prop_oneof![Just(' '), Just('|')], // position 7: undefined
        prop_oneof![Just(' '), Just('|')], // position 8: undefined
        prop_oneof![Just(' '), Just('a')], // character coding
        prop_oneof![
            Just('1'),
            Just('2'),
            Just('3'),
            Just('4'),
            Just('5'),
            Just('m'),
            Just('u'),
            Just('z'),
        ],
    )
        .prop_map(
            |(
                record_status,
                record_type,
                bibliographic_level,
                control_record_type,
                character_coding,
                encoding_level,
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
                    cataloging_form: ' ',
                    multipart_level: ' ',
                    reserved: "4500".to_string(),
                }
            },
        )
}

/// Generate a structurally valid authority record: `0..=3` control fields
/// (repeated tags allowed — `AuthorityRecord` stores multiple values per
/// control tag) and `0..=10` data fields.
fn arb_authority_record() -> BoxedStrategy<AuthorityRecord> {
    (
        arb_authority_leader(),
        prop::collection::vec((arb_control_tag(), arb_control_value()), 0..=3),
        prop::collection::vec(arb_data_field_with(arb_subfield_value()), 0..=10),
    )
        .prop_map(|(leader, control_fields, data_fields)| {
            let mut record = AuthorityRecord::new(leader);
            for (tag, value) in control_fields {
                record.add_control_field(tag, value);
            }
            for field in data_fields {
                record.add_field(field);
            }
            record
        })
        .boxed()
}

/// Generate a structurally valid holdings record, same shape as
/// [`arb_authority_record`].
fn arb_holdings_record() -> BoxedStrategy<HoldingsRecord> {
    (
        arb_holdings_leader(),
        prop::collection::vec((arb_control_tag(), arb_control_value()), 0..=3),
        prop::collection::vec(arb_data_field_with(arb_subfield_value()), 0..=10),
    )
        .prop_map(|(leader, control_fields, data_fields)| {
            let mut record = HoldingsRecord::new(leader);
            for (tag, value) in control_fields {
                record.add_control_field(tag, value);
            }
            for field in data_fields {
                record.add_field(field);
            }
            record
        })
        .boxed()
}

/// Serialize an authority record to ISO 2709 bytes (test convenience).
fn emit_authority_binary(record: &AuthorityRecord) -> Vec<u8> {
    let mut buffer = Vec::new();
    {
        let mut writer = AuthorityMarcWriter::new(&mut buffer);
        writer.write_record(record).expect("write should succeed");
    }
    buffer
}

/// Serialize a holdings record to ISO 2709 bytes (test convenience).
fn emit_holdings_binary(record: &HoldingsRecord) -> Vec<u8> {
    let mut buffer = Vec::new();
    {
        let mut writer = HoldingsMarcWriter::new(&mut buffer);
        writer.write_record(record).expect("write should succeed");
    }
    buffer
}

/// Clone a leader with the writer-computed positions zeroed
/// (`record_length`, `data_base_address`) so leaders can be compared
/// before and after a round-trip.
fn leader_ignoring_computed(leader: &Leader) -> Leader {
    Leader {
        record_length: 0,
        data_base_address: 0,
        ..leader.clone()
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64,
        ..ProptestConfig::default()
    })]

    /// Authority binary round-trip: any authority record the strategy can
    /// build must serialize through `AuthorityMarcWriter` and parse back
    /// structurally identical through `AuthorityMarcReader`.
    #[test]
    fn authority_binary_roundtrip(record in arb_authority_record()) {
        let buffer = emit_authority_binary(&record);
        let mut reader = AuthorityMarcReader::new(Cursor::new(&buffer));
        let parsed = reader
            .read_record()
            .expect("read should succeed")
            .expect("should get a record");

        prop_assert_eq!(
            leader_ignoring_computed(&record.leader),
            leader_ignoring_computed(&parsed.leader)
        );
        prop_assert_eq!(&record.control_fields, &parsed.control_fields);
        prop_assert_eq!(&record.fields, &parsed.fields);

        let next = reader.read_record().expect("read should succeed");
        prop_assert!(next.is_none(), "expected exactly one record in buffer");
    }

    /// The authority writer must emit bytes satisfying the ISO 2709
    /// framing invariants (leader length, directory tiling).
    #[test]
    fn authority_writer_emits_valid_iso2709_framing(record in arb_authority_record()) {
        let buffer = emit_authority_binary(&record);
        assert_iso2709_framing(&buffer)?;
    }

    /// Holdings binary round-trip: any holdings record the strategy can
    /// build must serialize through `HoldingsMarcWriter` and parse back
    /// structurally identical through `HoldingsMarcReader`.
    #[test]
    fn holdings_binary_roundtrip(record in arb_holdings_record()) {
        let buffer = emit_holdings_binary(&record);
        let mut reader = HoldingsMarcReader::new(Cursor::new(&buffer));
        let parsed = reader
            .read_record()
            .expect("read should succeed")
            .expect("should get a record");

        prop_assert_eq!(
            leader_ignoring_computed(&record.leader),
            leader_ignoring_computed(&parsed.leader)
        );
        prop_assert_eq!(&record.control_fields, &parsed.control_fields);
        prop_assert_eq!(&record.fields, &parsed.fields);

        let next = reader.read_record().expect("read should succeed");
        prop_assert!(next.is_none(), "expected exactly one record in buffer");
    }

    /// The holdings writer must emit bytes satisfying the ISO 2709
    /// framing invariants (leader length, directory tiling).
    #[test]
    fn holdings_writer_emits_valid_iso2709_framing(record in arb_holdings_record()) {
        let buffer = emit_holdings_binary(&record);
        assert_iso2709_framing(&buffer)?;
    }
}
