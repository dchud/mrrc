//! Integration tests for `MarcRecord` trait across all record types
//!
//! Tests that the `MarcRecord` trait enables generic code working with any MARC record.

mod common;

use common::make_leader;
use mrrc::{AuthorityRecord, HoldingsRecord, MarcRecord, Record};

/// Helper function demonstrating generic code using `MarcRecord` trait.
fn set_and_verify_control_field<T: MarcRecord>(record: &mut T, tag: &str, value: &str) {
    record.add_control_field(tag, value);
    assert_eq!(record.get_control_field(tag), Some(value));
}

/// Helper function to verify leader access through trait.
fn verify_leader_type<T: MarcRecord>(record: &T, expected_type: char) {
    assert_eq!(record.leader().record_type, expected_type);
}

#[test]
fn test_record_implements_marc_record() {
    let mut record = Record::new(make_leader());
    set_and_verify_control_field(&mut record, "001", "12345");
    verify_leader_type(&record, 'a');
}

#[test]
fn test_authority_record_implements_marc_record() {
    let mut leader = make_leader();
    leader.record_type = 'z';
    let mut record = AuthorityRecord::new(leader);
    set_and_verify_control_field(&mut record, "001", "auth001");
    verify_leader_type(&record, 'z');
}

#[test]
fn test_holdings_record_implements_marc_record() {
    let mut leader = make_leader();
    leader.record_type = 'y';
    let mut record = HoldingsRecord::new(leader);
    set_and_verify_control_field(&mut record, "001", "hold001");
    verify_leader_type(&record, 'y');
}

#[test]
fn test_control_fields_iter_all_types() {
    // Test Record
    let mut record = Record::new(make_leader());
    record.add_control_field("001".to_string(), "id1".to_string());
    record.add_control_field("003".to_string(), "source".to_string());
    let cf_count: usize = record.control_fields_iter().count();
    assert_eq!(cf_count, 2);

    // Test AuthorityRecord
    let mut auth = AuthorityRecord::new(make_leader());
    auth.add_control_field("001".to_string(), "id2".to_string());
    auth.add_control_field("003".to_string(), "source2".to_string());
    let cf_count: usize = auth.control_fields_iter().count();
    assert_eq!(cf_count, 2);

    // Test HoldingsRecord
    let mut holdings = HoldingsRecord::new(make_leader());
    holdings.add_control_field("001".to_string(), "id3".to_string());
    holdings.add_control_field("003".to_string(), "source3".to_string());
    let cf_count: usize = holdings.control_fields_iter().count();
    assert_eq!(cf_count, 2);
}

#[test]
fn test_leader_mutation_through_trait() {
    let mut record = Record::new(make_leader());
    assert_eq!(record.leader().record_status, 'a');

    // Mutate through trait
    record.leader_mut().record_status = 'c';
    assert_eq!(record.leader().record_status, 'c');

    // Same with AuthorityRecord
    let mut auth = AuthorityRecord::new(make_leader());
    auth.leader_mut().record_status = 'd';
    assert_eq!(auth.leader().record_status, 'd');
}
