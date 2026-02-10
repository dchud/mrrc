//! Common test helpers and utilities shared across test suite.

use mrrc::{Field, Leader, Record};

/// Creates a default leader for test records.
///
/// This is used by many tests to establish a consistent baseline leader.
pub fn create_test_leader() -> Leader {
    Leader {
        record_length: 1000,
        record_status: 'a',
        record_type: 'a',
        bibliographic_level: 'm',
        control_record_type: 'a',
        character_coding: ' ',
        indicator_count: 2,
        subfield_code_count: 2,
        data_base_address: 100,
        encoding_level: ' ',
        cataloging_form: ' ',
        multipart_level: ' ',
        reserved: "4500".to_string(),
    }
}

/// Creates a default leader for field accessor tests.
///
/// Alias for `create_test_leader()` to support existing test naming conventions.
/// Used across multiple test files for consistency.
#[allow(dead_code)]
pub fn make_leader() -> Leader {
    create_test_leader()
}

/// Creates a simple test record with a basic leader.
pub fn create_test_record() -> Record {
    let leader = Leader {
        record_length: 0,
        record_status: 'n',
        record_type: 'a',
        bibliographic_level: 'm',
        control_record_type: ' ',
        character_coding: ' ',
        indicator_count: 2,
        subfield_code_count: 2,
        data_base_address: 0,
        encoding_level: ' ',
        cataloging_form: 'a',
        multipart_level: ' ',
        reserved: "4500".to_string(),
    };
    Record::new(leader)
}

/// Creates a realistic test record for field query testing.
///
/// Includes various field types (245, 600-651, 700-710) with subfields
/// to test field queries and indicators.
pub fn create_realistic_record() -> Record {
    let mut record = create_test_record();

    // Add title field
    let mut field_245 = Field::new("245".to_string(), '1', '0');
    field_245.add_subfield_str('a', "The Great Gatsby");
    field_245.add_subfield_str('c', "F. Scott Fitzgerald");
    record.add_field(field_245);

    // Add multiple 650 fields (LCSH subjects)
    let mut field_650_1 = Field::new("650".to_string(), ' ', '0');
    field_650_1.add_subfield_str('a', "Novels");
    field_650_1.add_subfield_str('x', "American");
    record.add_field(field_650_1);

    let mut field_650_2 = Field::new("650".to_string(), ' ', '0');
    field_650_2.add_subfield_str('a', "Coming of age");
    field_650_2.add_subfield_str('x', "Fiction");
    record.add_field(field_650_2);

    // Add a 651 field (geographic subject)
    let mut field_651 = Field::new("651".to_string(), ' ', '0');
    field_651.add_subfield_str('a', "United States");
    field_651.add_subfield_str('x', "Fiction");
    record.add_field(field_651);

    // Add a 600 field (name subject)
    let mut field_600 = Field::new("600".to_string(), '1', '0');
    field_600.add_subfield_str('a', "Gatsby, Jay");
    field_600.add_subfield_str('c', "Fictional character");
    record.add_field(field_600);

    // Add a 700 field (name added entry) with different indicators
    let mut field_700 = Field::new("700".to_string(), '1', ' ');
    field_700.add_subfield_str('a', "Fitzgerald, F. Scott");
    field_700.add_subfield_str('d', "1896-1940");
    record.add_field(field_700);

    // Add a 710 field (corporate body added entry)
    let mut field_710 = Field::new("710".to_string(), '2', ' ');
    field_710.add_subfield_str('a', "Scribner");
    record.add_field(field_710);

    record
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_leader_valid() {
        let leader = create_test_leader();
        assert_eq!(leader.record_type, 'a');
        assert_eq!(leader.indicator_count, 2);
    }

    #[test]
    fn test_create_test_record_valid() {
        let record = create_test_record();
        assert_eq!(record.fields().count(), 0);
    }

    #[test]
    fn test_create_realistic_record_has_expected_fields() {
        let record = create_realistic_record();
        // Should have 245, 650 (2x), 651, 600, 700, 710 = 7 fields
        assert_eq!(record.fields().count(), 7);
    }
}
