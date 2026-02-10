//! Integration tests for advanced field query patterns

mod common;

use common::create_realistic_record;
use mrrc::{FieldQuery, TagRangeQuery};

#[test]
fn test_fields_by_indicator_lcsh() {
    let record = create_realistic_record();

    // Get all 650 fields with indicator2='0' (LCSH)
    let lcsh_fields: Vec<_> = record.fields_by_indicator("650", None, Some('0')).collect();

    assert_eq!(lcsh_fields.len(), 2);
    for field in lcsh_fields {
        assert_eq!(field.tag, "650");
        assert_eq!(field.indicator2, '0');
    }
}

#[test]
fn test_fields_by_indicator_specific() {
    let record = create_realistic_record();

    // Get all fields with indicator1='1', indicator2='0'
    let fields: Vec<_> = record
        .fields_by_indicator("245", Some('1'), Some('0'))
        .collect();

    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].tag, "245");
    assert_eq!(fields[0].indicator1, '1');
    assert_eq!(fields[0].indicator2, '0');
}

#[test]
fn test_fields_by_indicator_wildcard() {
    let record = create_realistic_record();

    // Get all 650 fields regardless of indicator1
    let fields: Vec<_> = record.fields_by_indicator("650", None, Some('0')).collect();

    assert_eq!(fields.len(), 2);
    for field in fields {
        assert_eq!(field.indicator2, '0');
    }
}

#[test]
fn test_fields_in_range_subjects() {
    let record = create_realistic_record();

    // Get all subject fields (600-699)
    let subject_fields: Vec<_> = record.fields_in_range("600", "699").collect();

    assert_eq!(subject_fields.len(), 4); // 600, 650 (2x), 651
    for field in subject_fields {
        assert!(field.tag.starts_with('6'));
    }
}

#[test]
fn test_fields_in_range_names() {
    let record = create_realistic_record();

    // Get all name-related fields (700-799)
    let name_fields: Vec<_> = record.fields_in_range("700", "799").collect();

    assert_eq!(name_fields.len(), 2); // 700, 710
    for field in name_fields {
        assert!(field.tag.as_str() >= "700" && field.tag.as_str() <= "799");
    }
}

#[test]
fn test_fields_with_subfield_a() {
    let record = create_realistic_record();

    // Find all 650 fields with subfield 'a'
    let fields_with_a: Vec<_> = record.fields_with_subfield("650", 'a').collect();

    assert_eq!(fields_with_a.len(), 2);
    for field in fields_with_a {
        assert!(field.get_subfield('a').is_some());
    }
}

#[test]
fn test_fields_with_subfield_nonexistent() {
    let record = create_realistic_record();

    // Find all 650 fields with subfield 'z' (unlikely)
    let fields_with_z: Vec<_> = record.fields_with_subfield("650", 'z').collect();

    assert_eq!(fields_with_z.len(), 0);
}

#[test]
fn test_fields_with_subfields_multiple() {
    let record = create_realistic_record();

    // Find all 650 fields with both 'a' and 'x'
    let fields_with_ax: Vec<_> = record.fields_with_subfields("650", &['a', 'x']).collect();

    assert_eq!(fields_with_ax.len(), 2);
    for field in fields_with_ax {
        assert!(field.get_subfield('a').is_some());
        assert!(field.get_subfield('x').is_some());
    }
}

#[test]
fn test_fields_with_subfields_partial_match() {
    let record = create_realistic_record();

    // Find all 650 fields with 'a' and 'z' (only a exists)
    let fields_with_az: Vec<_> = record.fields_with_subfields("650", &['a', 'z']).collect();

    assert_eq!(fields_with_az.len(), 0);
}

#[test]
fn test_field_query_builder() {
    let record = create_realistic_record();

    // Build a query for 650 fields with indicator2='0' and subfield 'a'
    let query = FieldQuery::new()
        .tag("650")
        .indicator2(Some('0'))
        .has_subfield('a');

    let matching: Vec<_> = record.fields_matching(&query).collect();

    assert_eq!(matching.len(), 2);
    for field in matching {
        assert_eq!(field.tag, "650");
        assert_eq!(field.indicator2, '0');
        assert!(field.get_subfield('a').is_some());
    }
}

#[test]
fn test_field_query_multiple_subfields() {
    let record = create_realistic_record();

    // Query for fields with multiple required subfields
    let query = FieldQuery::new()
        .tag("650")
        .has_subfield('a')
        .has_subfield('x');

    let matching: Vec<_> = record.fields_matching(&query).collect();

    assert_eq!(matching.len(), 2);
}

#[test]
fn test_field_query_no_tag() {
    let record = create_realistic_record();

    // Query matching any tag with indicator value
    let query = FieldQuery::new().indicator1(Some('1'));

    let matching: Vec<_> = record.fields_matching(&query).collect();

    // Should match 245, 600, and possibly others with indicator1='1'
    assert!(matching.len() >= 2);
    for field in matching {
        assert_eq!(field.indicator1, '1');
    }
}

#[test]
fn test_tag_range_query() {
    let query = TagRangeQuery {
        start_tag: "600".to_string(),
        end_tag: "699".to_string(),
        indicator1: None,
        indicator2: Some('0'),
        required_subfields: vec!['a'],
    };

    let record = create_realistic_record();
    let matching: Vec<_> = record.fields_matching_range(&query).collect();

    // Should match fields in range 600-699 with ind2='0' and subfield 'a'
    // 600 (1), 650 (2), 651 (1) = 4 fields total
    assert_eq!(matching.len(), 4);
    for field in matching {
        assert!(field.tag.as_str() >= "600" && field.tag.as_str() <= "699");
        assert_eq!(field.indicator2, '0');
        assert!(field.get_subfield('a').is_some());
    }
}

#[test]
fn test_combined_queries() {
    let record = create_realistic_record();

    // Complex scenario: Find LCSH subject headings
    let lcsh_subjects: Vec<_> = record
        .fields_by_indicator("650", None, Some('0'))
        .filter_map(|f| f.get_subfield('a'))
        .collect();

    assert_eq!(lcsh_subjects.len(), 2);
    assert!(lcsh_subjects.contains(&"Novels"));
    assert!(lcsh_subjects.contains(&"Coming of age"));
}

#[test]
fn test_range_query_boundaries() {
    let record = create_realistic_record();

    // Test exact boundary matching
    let exactly_600_799: Vec<_> = record.fields_in_range("600", "799").collect();
    let includes_650_and_700: Vec<_> = record.fields_in_range("650", "700").collect();

    // exactly_600_799 should be 600, 650 (x2), 651, 700, 710
    assert!(exactly_600_799.len() >= 5);

    // includes_650_and_700 should include 650 (x2) and 700
    assert!(includes_650_and_700.len() >= 3);
}

#[test]
fn test_query_default() {
    let record = create_realistic_record();

    // Default query should match all fields
    let query = FieldQuery::default();
    let all_fields: Vec<_> = record.fields_matching(&query).collect();

    assert_eq!(all_fields.len(), record.fields().count());
}

#[test]
fn test_query_no_matches() {
    let record = create_realistic_record();

    // Query that should match nothing
    let query = FieldQuery::new()
        .tag("999")
        .indicator1(Some('X'))
        .has_subfield('z');

    let matching: Vec<_> = record.fields_matching(&query).collect();

    assert_eq!(matching.len(), 0);
}
