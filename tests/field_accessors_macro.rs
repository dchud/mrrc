//! Integration tests for the `define_field_accessors!` macro.
//!
//! Demonstrates usage of the macro to eliminate boilerplate in field collection management.

mod common;

use common::make_leader;
use mrrc::{define_field_accessors, Field};

// Example record type for testing
#[derive(Debug)]
struct ExampleRecord {
    #[allow(dead_code)]
    leader: mrrc::Leader,
    first_collection: Vec<Field>,
    second_collection: Vec<Field>,
}

impl ExampleRecord {
    fn new(leader: mrrc::Leader) -> Self {
        ExampleRecord {
            leader,
            first_collection: Vec::new(),
            second_collection: Vec::new(),
        }
    }

    // Use the macro to generate methods
    define_field_accessors!(first_collection, add_first_field, first_fields);
    define_field_accessors!(second_collection, add_second_field, second_fields);
}

#[test]
fn test_macro_generates_add_method() {
    let mut record = ExampleRecord::new(make_leader());
    let field = Field::new("245".to_string(), '1', '0');

    record.add_first_field(field);
    assert_eq!(record.first_fields().len(), 1);
}

#[test]
fn test_macro_generates_get_method() {
    let mut record = ExampleRecord::new(make_leader());
    let field1 = Field::new("245".to_string(), '1', '0');
    let field2 = Field::new("650".to_string(), ' ', '0');

    record.add_first_field(field1);
    record.add_first_field(field2);

    let fields = record.first_fields();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].tag, "245");
    assert_eq!(fields[1].tag, "650");
}

#[test]
fn test_macro_multiple_collections() {
    let mut record = ExampleRecord::new(make_leader());

    let field1 = Field::new("100".to_string(), '1', ' ');
    let field2 = Field::new("700".to_string(), '1', ' ');

    record.add_first_field(field1);
    record.add_second_field(field2);

    assert_eq!(record.first_fields().len(), 1);
    assert_eq!(record.second_fields().len(), 1);
    assert_eq!(record.first_fields()[0].tag, "100");
    assert_eq!(record.second_fields()[0].tag, "700");
}

#[test]
fn test_macro_empty_collection() {
    let record = ExampleRecord::new(make_leader());
    assert_eq!(record.first_fields().len(), 0);
    assert_eq!(record.second_fields().len(), 0);
}

#[test]
fn test_macro_method_chaining() {
    let mut record = ExampleRecord::new(make_leader());

    // Add multiple fields
    for i in 0..5 {
        let field = Field::new(format!("65{i}"), ' ', '0');
        record.add_first_field(field);
    }

    // Verify all were added
    assert_eq!(record.first_fields().len(), 5);
}

#[test]
fn test_macro_get_method_returns_slice() {
    let mut record = ExampleRecord::new(make_leader());
    let field = Field::new("245".to_string(), '1', '0');
    record.add_first_field(field);

    // Test that we can use slice operations
    let fields = record.first_fields();
    assert!(fields.iter().all(|f| f.tag.starts_with("24")));
}
