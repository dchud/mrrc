//! Property-based tests for MARC record round-trip fidelity.
//!
//! Uses proptest to generate arbitrary structurally valid MARC records
//! and verify that serialization → parsing produces identical results.

use mrrc::{Field, Leader, MarcReader, MarcWriter, Record, Subfield};
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

/// Generate subfield value content (ASCII, no MARC delimiters).
fn arb_subfield_value() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 .,:;()'/&-]{1,80}".prop_filter("no MARC delimiters", |s| {
        !s.bytes().any(|b| b == 0x1D || b == 0x1E || b == 0x1F)
    })
}

/// Generate a single subfield.
fn arb_subfield() -> impl Strategy<Value = Subfield> {
    (arb_subfield_code(), arb_subfield_value()).prop_map(|(code, value)| Subfield { code, value })
}

/// Generate a data field with indicators and 1-5 subfields.
fn arb_data_field() -> impl Strategy<Value = Field> {
    (
        arb_data_tag(),
        arb_indicator(),
        arb_indicator(),
        prop::collection::vec(arb_subfield(), 1..=5),
    )
        .prop_map(|(tag, ind1, ind2, subfields)| Field {
            tag,
            indicator1: ind1,
            indicator2: ind2,
            subfields: SmallVec::from_vec(subfields),
        })
}

/// Generate a complete, structurally valid MARC record.
fn arb_record() -> impl Strategy<Value = Record> {
    (
        arb_leader(),
        prop::collection::vec((arb_control_tag(), arb_control_value()), 0..=3),
        prop::collection::vec(arb_data_field(), 1..=10),
    )
        .prop_map(|(leader, control_fields, data_fields)| {
            let mut record = Record::new(leader);
            for (tag, value) in control_fields {
                record.add_control_field(tag, value);
            }
            for field in data_fields {
                record.add_field(field);
            }
            record
        })
}

// ============================================================================
// Property tests
// ============================================================================

proptest! {
    /// Binary round-trip: any record we can build, we can serialize to
    /// ISO 2709 and parse back to a structurally identical record.
    #[test]
    fn binary_roundtrip(record in arb_record()) {
        // Serialize to binary MARC
        let mut buffer = Vec::new();
        {
            let mut writer = MarcWriter::new(&mut buffer);
            writer.write_record(&record).expect("write should succeed");
        }

        // Parse back
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
}
