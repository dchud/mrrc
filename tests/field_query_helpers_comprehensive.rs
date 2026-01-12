//! Comprehensive tests for Phase 2 field query helpers
//!
//! Tests cover:
//! - Subfield pattern matching with regex
//! - Value-based filtering (exact and partial)
//! - Convenience helper methods
//! - Integration with realistic MARC records
//! - Edge cases and error handling
//! - Performance with large datasets

mod common;

use common::make_leader;
use mrrc::{Field, FieldQueryHelpers, Record};

/// Create a realistic bibliographic record for testing
fn create_realistic_record() -> Record {
    let mut record = Record::new(make_leader());

    // Control fields
    record.add_control_field_str("001", "ocm43089250");
    record.add_control_field_str("008", "970616s1997    enka   j      000 0 eng d");

    // Title
    let mut field_245 = Field::new("245".to_string(), '1', '0');
    field_245.add_subfield_str('a', "The Great Gatsby");
    field_245.add_subfield_str('c', "F. Scott Fitzgerald");
    record.add_field(field_245);

    // ISBN fields
    let mut isbn1 = Field::new("020".to_string(), ' ', ' ');
    isbn1.add_subfield_str('a', "978-0-7432-7356-5");
    record.add_field(isbn1);

    let mut isbn2 = Field::new("020".to_string(), ' ', ' ');
    isbn2.add_subfield_str('a', "979-10-90636-07-1");
    record.add_field(isbn2);

    let mut isbn3 = Field::new("020".to_string(), ' ', ' ');
    isbn3.add_subfield_str('a', "978-1-111111-11-1");
    record.add_field(isbn3);

    // Author with dates
    let mut field_100 = Field::new("100".to_string(), '1', ' ');
    field_100.add_subfield_str('a', "Fitzgerald, F. Scott");
    field_100.add_subfield_str('d', "1896-1940");
    record.add_field(field_100);

    // Added authors
    let mut field_700a = Field::new("700".to_string(), '1', ' ');
    field_700a.add_subfield_str('a', "Smith, John");
    field_700a.add_subfield_str('d', "1873-1944");
    record.add_field(field_700a);

    let mut field_700b = Field::new("700".to_string(), '1', ' ');
    field_700b.add_subfield_str('a', "Doe, Jane");
    field_700b.add_subfield_str('d', "1902-1989");
    record.add_field(field_700b);

    let mut field_700c = Field::new("700".to_string(), '1', ' ');
    field_700c.add_subfield_str('a', "Johnson, Robert");
    record.add_field(field_700c);

    // Corporate author
    let mut field_710 = Field::new("710".to_string(), '2', ' ');
    field_710.add_subfield_str('a', "Scribner");
    record.add_field(field_710);

    // Subject headings with subdivisions
    let mut subject1 = Field::new("650".to_string(), ' ', '0');
    subject1.add_subfield_str('a', "Novels");
    subject1.add_subfield_str('x', "American");
    subject1.add_subfield_str('y', "20th century");
    record.add_field(subject1);

    let mut subject2 = Field::new("650".to_string(), ' ', '0');
    subject2.add_subfield_str('a', "Coming of age");
    subject2.add_subfield_str('x', "Fiction");
    record.add_field(subject2);

    let mut subject3 = Field::new("650".to_string(), ' ', '0');
    subject3.add_subfield_str('a', "World");
    subject3.add_subfield_str('x', "History");
    subject3.add_subfield_str('y', "20th century");
    record.add_field(subject3);

    let mut subject4 = Field::new("650".to_string(), ' ', '0');
    subject4.add_subfield_str('a', "Philosophy");
    subject4.add_subfield_str('x', "History");
    record.add_field(subject4);

    let mut subject5 = Field::new("650".to_string(), ' ', '0');
    subject5.add_subfield_str('a', "Science");
    subject5.add_subfield_str('y', "Geography");
    record.add_field(subject5);

    // Geographic subject
    let mut geo_subject = Field::new("651".to_string(), ' ', '0');
    geo_subject.add_subfield_str('a', "United States");
    geo_subject.add_subfield_str('x', "Fiction");
    record.add_field(geo_subject);

    // Name subject
    let mut name_subject = Field::new("600".to_string(), '1', '0');
    name_subject.add_subfield_str('a', "Gatsby, Jay");
    name_subject.add_subfield_str('c', "Fictional character");
    record.add_field(name_subject);

    record
}

// =============================================================================
// REGEX PATTERN MATCHING TESTS
// =============================================================================

#[test]
fn test_isbns_matching_basic_978() {
    let record = create_realistic_record();

    let results = record.isbns_matching(r"^978-.*").unwrap();
    assert_eq!(results.len(), 2);

    for field in results {
        if let Some(isbn) = field.get_subfield('a') {
            assert!(isbn.starts_with("978-"));
        }
    }
}

#[test]
fn test_isbns_matching_basic_979() {
    let record = create_realistic_record();

    let results = record.isbns_matching(r"^979-.*").unwrap();
    assert_eq!(results.len(), 1);

    for field in results {
        if let Some(isbn) = field.get_subfield('a') {
            assert!(isbn.starts_with("979-"));
        }
    }
}

#[test]
fn test_isbns_matching_digit_pattern() {
    let record = create_realistic_record();

    // Match ISBNs with specific digit sequence (978-0-7432-7356-5)
    let results = record.isbns_matching(r"978-0-7432-.*").unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_isbns_matching_complex_pattern() {
    let record = create_realistic_record();

    // Match ISBNs containing "7432"
    let results = record.isbns_matching(r".*7432.*").unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_isbns_matching_invalid_pattern() {
    let record = create_realistic_record();

    let result = record.isbns_matching(r"[invalid(pattern");
    assert!(result.is_err());
}

#[test]
fn test_isbns_matching_no_matches() {
    let record = create_realistic_record();

    let results = record.isbns_matching(r"^000-.*").unwrap();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_isbns_matching_empty_record() {
    let record = Record::new(make_leader());

    let results = record.isbns_matching(r"^978-.*").unwrap();
    assert_eq!(results.len(), 0);
}

// =============================================================================
// SUBJECT SUBDIVISION MATCHING TESTS
// =============================================================================

#[test]
fn test_subjects_with_subdivision_x_history() {
    let record = create_realistic_record();

    let results = record.subjects_with_subdivision('x', "History");
    assert_eq!(results.len(), 2);

    for field in results {
        assert_eq!(field.tag, "650");
        if let Some(subfield) = field.get_subfield('x') {
            assert_eq!(subfield, "History");
        }
    }
}

#[test]
fn test_subjects_with_subdivision_x_american() {
    let record = create_realistic_record();

    let results = record.subjects_with_subdivision('x', "American");
    assert_eq!(results.len(), 1);
}

#[test]
fn test_subjects_with_subdivision_y_geography() {
    let record = create_realistic_record();

    let results = record.subjects_with_subdivision('y', "Geography");
    assert_eq!(results.len(), 1);
}

#[test]
fn test_subjects_with_subdivision_no_match() {
    let record = create_realistic_record();

    let results = record.subjects_with_subdivision('x', "Nonexistent");
    assert_eq!(results.len(), 0);
}

#[test]
fn test_subjects_with_subdivision_empty_record() {
    let record = Record::new(make_leader());

    let results = record.subjects_with_subdivision('x', "History");
    assert_eq!(results.len(), 0);
}

// =============================================================================
// AUTHOR DATE EXTRACTION TESTS
// =============================================================================

#[test]
fn test_authors_with_dates_primary_and_added() {
    let record = create_realistic_record();

    let results = record.authors_with_dates();
    assert_eq!(results.len(), 3);

    // Check that dates are paired with names
    assert_eq!(results[0], ("Fitzgerald, F. Scott", "1896-1940"));
    assert_eq!(results[1], ("Smith, John", "1873-1944"));
    assert_eq!(results[2], ("Doe, Jane", "1902-1989"));
}

#[test]
fn test_authors_with_dates_empty_record() {
    let record = Record::new(make_leader());

    let results = record.authors_with_dates();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_authors_with_dates_author_without_dates() {
    let record = create_realistic_record();

    // Johnson, Robert has no dates in the test record
    let results = record.authors_with_dates();

    // Should only return authors WITH dates
    assert_eq!(results.len(), 3);

    // Ensure Johnson is not included (no dates)
    for (name, _) in &results {
        assert_ne!(*name, "Johnson, Robert");
    }
}

#[test]
fn test_authors_with_dates_date_formats() {
    let record = create_realistic_record();

    let results = record.authors_with_dates();

    // Verify all results have dates (format YYYY-YYYY or similar)
    for (name, dates) in results {
        assert!(!name.is_empty());
        assert!(!dates.is_empty());
        assert!(dates.contains('-'));
    }
}

// =============================================================================
// NAMES IN RANGE TESTS
// =============================================================================

#[test]
fn test_names_in_range_700_711() {
    let record = create_realistic_record();

    let results = record.names_in_range("700", "711");
    assert_eq!(results.len(), 4); // All three 700 fields
}

#[test]
fn test_names_in_range_700_only() {
    let record = create_realistic_record();

    let results = record.names_in_range("700", "700");
    assert_eq!(results.len(), 3); // Only 700 fields
}

#[test]
fn test_names_in_range_600_699() {
    let record = create_realistic_record();

    // Note: This includes subject headings too
    let results = record.names_in_range("600", "699");
    assert!(results.len() > 3); // 600 + multiple 650s + 651
}

#[test]
fn test_names_in_range_no_matches() {
    let record = create_realistic_record();

    let results = record.names_in_range("900", "999");
    assert_eq!(results.len(), 0);
}

#[test]
fn test_names_in_range_empty_record() {
    let record = Record::new(make_leader());

    let results = record.names_in_range("100", "799");
    assert_eq!(results.len(), 0);
}

// =============================================================================
// SUBJECTS WITH NOTE TESTS
// =============================================================================

#[test]
fn test_subjects_with_note_history() {
    let record = create_realistic_record();

    let results = record.subjects_with_note("History");
    assert_eq!(results.len(), 2); // Two subjects with "History" subdivision
}

#[test]
fn test_subjects_with_note_fiction() {
    let record = create_realistic_record();

    let results = record.subjects_with_note("Fiction");
    assert_eq!(results.len(), 1);
}

#[test]
fn test_subjects_with_note_partial_match() {
    let record = create_realistic_record();

    let results = record.subjects_with_note("Hist");
    assert_eq!(results.len(), 2); // Partial match on "History"
}

#[test]
fn test_subjects_with_note_no_match() {
    let record = create_realistic_record();

    let results = record.subjects_with_note("Nonexistent");
    assert_eq!(results.len(), 0);
}

#[test]
fn test_subjects_with_note_case_sensitive() {
    let record = create_realistic_record();

    // Should be case-sensitive
    let results_upper = record.subjects_with_note("History");
    let results_lower = record.subjects_with_note("history");

    assert_eq!(results_upper.len(), 2);
    assert_eq!(results_lower.len(), 0);
}

// =============================================================================
// EDGE CASES AND ERROR HANDLING
// =============================================================================

#[test]
fn test_pattern_query_empty_subfield_value() {
    let mut record = Record::new(make_leader());

    let mut field = Field::new("020".to_string(), ' ', ' ');
    field.add_subfield_str('a', "");
    record.add_field(field);

    let results = record.isbns_matching(r"^978-.*").unwrap();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_pattern_query_special_regex_chars() {
    let mut record = Record::new(make_leader());

    let mut field = Field::new("020".to_string(), ' ', ' ');
    field.add_subfield_str('a', "ISBN-123.456");
    record.add_field(field);

    // Match with escaped period
    let results = record.isbns_matching(r".*\..*").unwrap();
    assert_eq!(results.len(), 1);

    // Match with literal period should fail
    let results = record.isbns_matching(r".*-.*").unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_value_query_unicode_characters() {
    let mut record = Record::new(make_leader());

    let mut subject = Field::new("650".to_string(), ' ', '0');
    subject.add_subfield_str('a', "Café");
    subject.add_subfield_str('x', "Français");
    record.add_field(subject);

    let results = record.subjects_with_subdivision('x', "Français");
    assert_eq!(results.len(), 1);
}

#[test]
fn test_value_query_whitespace_sensitivity() {
    let mut record = Record::new(make_leader());

    let mut subject = Field::new("650".to_string(), ' ', '0');
    subject.add_subfield_str('a', "Modern history");
    subject.add_subfield_str('x', "20th century");
    record.add_field(subject);

    // Exact match should be case and whitespace sensitive
    let exact = record.subjects_with_subdivision('x', "20th century");
    assert_eq!(exact.len(), 1);

    // Different whitespace should not match
    let different = record.subjects_with_subdivision('x', "20thcentury");
    assert_eq!(different.len(), 0);
}

// =============================================================================
// INTEGRATION TESTS WITH MULTIPLE HELPERS
// =============================================================================

#[test]
fn test_combined_author_and_isbn_queries() {
    let record = create_realistic_record();

    let authors = record.authors_with_dates();
    let isbns = record.isbns_matching(r"^978-.*").unwrap();

    assert!(!authors.is_empty());
    assert!(!isbns.is_empty());

    // Verify we can use both simultaneously
    for (name, dates) in authors {
        assert!(!name.is_empty());
        assert!(!dates.is_empty());
    }
}

#[test]
fn test_combined_subject_queries() {
    let record = create_realistic_record();

    let with_history = record.subjects_with_subdivision('x', "History");
    let with_note = record.subjects_with_note("History");

    // Both should find the same subjects (for exact match)
    assert_eq!(with_history.len(), with_note.len());
}

#[test]
fn test_query_independence() {
    let record = create_realistic_record();

    // Multiple queries on same record should be independent
    let _isbns = record.isbns_matching(r"^978-.*").unwrap();
    let _authors = record.authors_with_dates();
    let _subjects = record.subjects_with_subdivision('x', "History");

    // Record should still be intact
    assert!(record.title().is_some());
}

// =============================================================================
// PERFORMANCE TESTS WITH LARGE DATASETS
// =============================================================================

#[test]
fn test_large_record_isbn_matching_performance() {
    let mut record = Record::new(make_leader());

    // Add 1000 ISBN fields
    for i in 0..1000 {
        let mut field = Field::new("020".to_string(), ' ', ' ');
        let isbn = if i % 3 == 0 {
            format!("978-0-{:06}-{:02}-5", i, i % 10)
        } else {
            format!("979-10-{:06}-{:02}-1", i, i % 10)
        };
        field.add_subfield_str('a', &isbn);
        record.add_field(field);
    }

    // Should efficiently find ISBNs
    let results = record.isbns_matching(r"^978-.*").unwrap();
    assert_eq!(results.len(), 334); // ~1000/3
}

#[test]
fn test_large_record_subject_matching_performance() {
    let mut record = Record::new(make_leader());

    // Add 500 subject fields
    let subdivisions = ["History", "Geography", "Fiction", "Literature"];
    for i in 0..500 {
        let mut field = Field::new("650".to_string(), ' ', '0');
        let subject = format!("Subject {i}");
        field.add_subfield_str('a', &subject);
        field.add_subfield_str('x', subdivisions[i % subdivisions.len()]);
        record.add_field(field);
    }

    // Should efficiently find subjects with specific subdivision
    let results = record.subjects_with_subdivision('x', "History");
    assert_eq!(results.len(), 125); // 500/4
}

#[test]
fn test_large_record_names_with_dates_performance() {
    let mut record = Record::new(make_leader());

    // Add 200 author fields with dates
    for i in 0..200 {
        let tag = if i == 0 { "100" } else { "700" };
        let mut field = Field::new(tag.to_string(), '1', ' ');
        let name = format!("Author, Number {i}");
        let dates = format!("18{:02}-19{:02}", i % 100, (i + 50) % 100);
        field.add_subfield_str('a', &name);
        field.add_subfield_str('d', &dates);
        record.add_field(field);
    }

    // Should efficiently extract all author dates
    let results = record.authors_with_dates();
    assert_eq!(results.len(), 200);
}

// =============================================================================
// BATCH OPERATIONS WITH FIELD QUERIES
// =============================================================================

#[test]
fn test_subjects_with_subdivision_multiple_codes() {
    let record = create_realistic_record();

    // Test different subdivision codes
    let x_results = record.subjects_with_subdivision('x', "History");
    let y_results = record.subjects_with_subdivision('y', "20th century");

    assert!(!x_results.is_empty());
    assert!(!y_results.is_empty());
    // Both should have the same count (2) in this test record
}

#[test]
fn test_empty_field_value_handling() {
    let mut record = Record::new(make_leader());

    // Add fields with empty subfield values
    let mut field1 = Field::new("650".to_string(), ' ', '0');
    field1.add_subfield_str('a', "Subject");
    field1.add_subfield_str('x', "");
    record.add_field(field1);

    let mut field2 = Field::new("650".to_string(), ' ', '0');
    field2.add_subfield_str('a', "Subject");
    field2.add_subfield_str('x', "History");
    record.add_field(field2);

    let results = record.subjects_with_subdivision('x', "History");
    assert_eq!(results.len(), 1); // Only the one with actual value
}

#[test]
fn test_multiple_subfields_same_code() {
    let mut record = Record::new(make_leader());

    // Field with multiple 'x' subfields
    let mut field = Field::new("650".to_string(), ' ', '0');
    field.add_subfield_str('a', "Subject");
    field.add_subfield_str('x', "History");
    field.add_subfield_str('x', "Sources");
    record.add_field(field);

    // Should match because it has an 'x' with value "History"
    let results = record.subjects_with_subdivision('x', "History");
    assert_eq!(results.len(), 1);
}
