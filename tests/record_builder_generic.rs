//! Integration tests for `GenericRecordBuilder` across all MARC record types.
//!
//! Tests that `GenericRecordBuilder` provides a unified interface for building
//! any record type.

mod common;

use common::make_leader;
use mrrc::{AuthorityRecord, GenericRecordBuilder, HoldingsRecord, MarcRecord, Record};

#[test]
fn test_generic_builder_with_record() {
    let record = GenericRecordBuilder::new(Record::new(make_leader()))
        .control_field("001", "12345")
        .control_field("003", "OCoLC")
        .build();

    assert_eq!(record.get_control_field("001"), Some("12345"));
    assert_eq!(record.get_control_field("003"), Some("OCoLC"));
}

#[test]
fn test_generic_builder_with_authority_record() {
    let mut leader = make_leader();
    leader.record_type = 'z';

    let record = GenericRecordBuilder::new(AuthorityRecord::new(leader))
        .control_field("001", "auth001")
        .control_field("003", "DLC")
        .build();

    assert_eq!(record.get_control_field("001"), Some("auth001"));
    assert_eq!(record.get_control_field("003"), Some("DLC"));
    assert_eq!(record.leader().record_type, 'z');
}

#[test]
fn test_generic_builder_with_holdings_record() {
    let mut leader = make_leader();
    leader.record_type = 'y';

    let record = GenericRecordBuilder::new(HoldingsRecord::new(leader))
        .control_field("001", "hold001")
        .control_field("005", "20250101120000")
        .build();

    assert_eq!(record.get_control_field("001"), Some("hold001"));
    assert_eq!(record.get_control_field("005"), Some("20250101120000"));
    assert_eq!(record.leader().record_type, 'y');
}

#[test]
fn test_generic_builder_mutable_access() {
    let mut builder = GenericRecordBuilder::new(Record::new(make_leader()));

    // Add via generic interface
    builder = builder.control_field("001", "12345");

    // Modify via mutable reference for record-specific methods
    builder
        .record_mut()
        .add_control_field("003".to_string(), "test".to_string());

    let record = builder.build();

    assert_eq!(record.get_control_field("001"), Some("12345"));
    assert_eq!(record.get_control_field("003"), Some("test"));
}

#[test]
fn test_generic_builder_read_access() {
    let builder =
        GenericRecordBuilder::new(Record::new(make_leader())).control_field("001", "12345");

    // Read record properties
    assert_eq!(builder.record().get_control_field("001"), Some("12345"));
    assert_eq!(builder.record().leader().record_type, 'a');

    // Still can build
    let record = builder.build();
    assert_eq!(record.get_control_field("001"), Some("12345"));
}

#[test]
fn test_generic_builder_multiple_control_fields() {
    let record = GenericRecordBuilder::new(Record::new(make_leader()))
        .control_field("001", "id1")
        .control_field("003", "source1")
        .control_field("005", "timestamp")
        .control_field("008", "250126s2024    xx||||||||||||||||eng||")
        .build();

    assert_eq!(record.get_control_field("001"), Some("id1"));
    assert_eq!(record.get_control_field("003"), Some("source1"));
    assert_eq!(record.get_control_field("005"), Some("timestamp"));
    assert_eq!(
        record.get_control_field("008"),
        Some("250126s2024    xx||||||||||||||||eng||")
    );
}

#[test]
fn test_generic_builder_overwrite_field() {
    let record = GenericRecordBuilder::new(Record::new(make_leader()))
        .control_field("001", "first_value")
        .control_field("001", "second_value") // Overwrite
        .build();

    assert_eq!(record.get_control_field("001"), Some("second_value"));
}
