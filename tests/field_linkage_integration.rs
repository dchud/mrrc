//! Integration tests for MARC 880 field linkage and linked field navigation

mod common;

use common::make_leader;
use mrrc::{Field, LinkageInfo, Record};

/// Create a record with linked 880 fields (Arabic author with romanized form)
fn create_linked_record() -> Record {
    let mut record = Record::new(make_leader());

    // Original field 100 - Author with Arabic name
    // $6 880-01 means it links to 880 occurrence 01
    let mut field_100 = Field::new("100".to_string(), '1', ' ');
    field_100.add_subfield_str('6', "880-01");
    field_100.add_subfield_str('a', "سميث، جون"); // Arabic: "Smith, John"
    field_100.add_subfield_str('d', "1850-1925");
    record.add_field(field_100);

    // 880 field - Romanized version of 100
    // $6 100-01 points back to the original 100 field occurrence 01
    let mut field_880_100 = Field::new("880".to_string(), '1', ' ');
    field_880_100.add_subfield_str('6', "100-01");
    field_880_100.add_subfield_str('a', "Smith, John");
    field_880_100.add_subfield_str('d', "1850-1925");
    record.add_field(field_880_100);

    // Original field 245 - Title in Arabic
    let mut field_245 = Field::new("245".to_string(), '1', '0');
    field_245.add_subfield_str('6', "880-02");
    field_245.add_subfield_str('a', "كتاب عن الحياة");
    record.add_field(field_245);

    // 880 field - Romanized title
    let mut field_880_245 = Field::new("880".to_string(), '1', '0');
    field_880_245.add_subfield_str('6', "245-02");
    field_880_245.add_subfield_str('a', "Kitab ʻan al-hayah");
    record.add_field(field_880_245);

    // Field 700 - Without linkage (no 880)
    let mut field_700 = Field::new("700".to_string(), '1', ' ');
    field_700.add_subfield_str('a', "Jones, Jane");
    record.add_field(field_700);

    record
}

// =============================================================================
// LINKAGE INFO PARSING TESTS
// =============================================================================

#[test]
fn test_linkage_info_parse_basic() {
    let info = LinkageInfo::parse("100-01").unwrap();
    assert_eq!(info.occurrence(), "01");
    assert!(!info.is_reverse());
}

#[test]
fn test_linkage_info_parse_with_reverse() {
    let info = LinkageInfo::parse("245-02/r").unwrap();
    assert_eq!(info.occurrence(), "02");
    assert!(info.is_reverse());
}

#[test]
fn test_linkage_info_parse_invalid() {
    assert!(LinkageInfo::parse("10001").is_none());
    assert!(LinkageInfo::parse("880-").is_none());
    assert!(LinkageInfo::parse("").is_none());
}

// =============================================================================
// GET LINKED FIELD TESTS (Original -> 880)
// =============================================================================

#[test]
fn test_get_linked_field_basic() {
    let record = create_linked_record();

    let field_100 = record.get_field("100").unwrap();
    let field_880 = record.get_linked_field(field_100).unwrap();

    assert_eq!(field_880.tag, "880");
    assert_eq!(field_880.get_subfield('a'), Some("Smith, John"));
}

#[test]
fn test_get_linked_field_multiple_occurrences() {
    let record = create_linked_record();

    let field_245 = record.get_field("245").unwrap();
    let field_880 = record.get_linked_field(field_245).unwrap();

    assert_eq!(field_880.tag, "880");
    assert_eq!(field_880.get_subfield('6'), Some("245-02"));
}

#[test]
fn test_get_linked_field_no_linkage() {
    let record = create_linked_record();

    // Field 700 has no subfield 6
    let field_700 = record.get_field("700").unwrap();
    let result = record.get_linked_field(field_700);

    assert!(result.is_none());
}

#[test]
fn test_get_linked_field_no_880_match() {
    let mut record = Record::new(make_leader());

    // Create field with linkage but no matching 880
    let mut field = Field::new("100".to_string(), '1', ' ');
    field.add_subfield_str('6', "880-99");
    field.add_subfield_str('a', "Author");
    record.add_field(field);

    let field_100 = record.get_field("100").unwrap();
    let result = record.get_linked_field(field_100);

    assert!(result.is_none());
}

#[test]
fn test_get_linked_field_malformed_linkage() {
    let mut record = Record::new(make_leader());

    // Create field with malformed subfield 6
    let mut field = Field::new("100".to_string(), '1', ' ');
    field.add_subfield_str('6', "invalid-format");
    field.add_subfield_str('a', "Author");
    record.add_field(field);

    let field_100 = record.get_field("100").unwrap();
    let result = record.get_linked_field(field_100);

    assert!(result.is_none());
}

// =============================================================================
// GET ORIGINAL FIELD TESTS (880 -> Original)
// =============================================================================

#[test]
fn test_get_original_field_from_880() {
    let record = create_linked_record();

    let all_880 = record.get_all_880_fields();
    assert!(all_880.len() >= 2);

    // Get the first 880 and find its original
    let field_880 = all_880[0];
    let original = record.get_original_field(field_880).unwrap();

    assert_eq!(original.tag, "100");
}

#[test]
fn test_get_original_field_multiple() {
    let record = create_linked_record();

    let all_880 = record.get_all_880_fields();

    for field_880 in all_880 {
        if field_880
            .get_subfield('6')
            .is_some_and(|v| v.contains("100"))
        {
            let original = record.get_original_field(field_880).unwrap();
            assert_eq!(original.tag, "100");
        }
        if field_880
            .get_subfield('6')
            .is_some_and(|v| v.contains("245"))
        {
            let original = record.get_original_field(field_880).unwrap();
            assert_eq!(original.tag, "245");
        }
    }
}

#[test]
fn test_get_original_field_not_880() {
    let record = create_linked_record();

    let field_100 = record.get_field("100").unwrap();
    let result = record.get_original_field(field_100);

    // Should return None because it's not an 880
    assert!(result.is_none());
}

#[test]
fn test_get_original_field_no_subfield_6() {
    let mut record = Record::new(make_leader());

    // Create 880 without subfield 6
    let mut field_880 = Field::new("880".to_string(), '1', ' ');
    field_880.add_subfield_str('a', "Some text");
    record.add_field(field_880);

    let field = record.get_all_880_fields()[0];
    let result = record.get_original_field(field);

    assert!(result.is_none());
}

// =============================================================================
// GET ALL 880 FIELDS TESTS
// =============================================================================

#[test]
fn test_get_all_880_fields_count() {
    let record = create_linked_record();

    let fields_880 = record.get_all_880_fields();
    assert_eq!(fields_880.len(), 2);
}

#[test]
fn test_get_all_880_fields_all_880() {
    let record = create_linked_record();

    let fields_880 = record.get_all_880_fields();
    for field in fields_880 {
        assert_eq!(field.tag, "880");
    }
}

#[test]
fn test_get_all_880_fields_empty() {
    let record = Record::new(make_leader());

    let fields_880 = record.get_all_880_fields();
    assert!(fields_880.is_empty());
}

// =============================================================================
// GET FIELD PAIRS TESTS
// =============================================================================

#[test]
fn test_get_field_pairs_both_linked() {
    let record = create_linked_record();

    let pairs = record.get_field_pairs("100");
    assert_eq!(pairs.len(), 1);

    let (orig, linked) = pairs[0];
    assert_eq!(orig.tag, "100");
    assert!(linked.is_some());
    assert_eq!(linked.unwrap().tag, "880");
}

#[test]
fn test_get_field_pairs_no_880() {
    let record = create_linked_record();

    let pairs = record.get_field_pairs("700");
    assert_eq!(pairs.len(), 1);

    let (orig, linked) = pairs[0];
    assert_eq!(orig.tag, "700");
    assert!(linked.is_none());
}

#[test]
fn test_get_field_pairs_multiple() {
    let mut record = Record::new(make_leader());

    // Add multiple 100 fields (unusual but valid)
    let mut field_100a = Field::new("100".to_string(), '1', ' ');
    field_100a.add_subfield_str('6', "880-01");
    field_100a.add_subfield_str('a', "Author 1");
    record.add_field(field_100a);

    let mut field_100b = Field::new("100".to_string(), '1', ' ');
    field_100b.add_subfield_str('a', "Author 2");
    record.add_field(field_100b);

    let mut field_880 = Field::new("880".to_string(), '1', ' ');
    field_880.add_subfield_str('6', "100-01");
    field_880.add_subfield_str('a', "Author 1 romanized");
    record.add_field(field_880);

    let pairs = record.get_field_pairs("100");
    assert_eq!(pairs.len(), 2);

    // First pair should have linked 880
    assert!(pairs[0].1.is_some());

    // Second pair should not have linked 880
    assert!(pairs[1].1.is_none());
}

#[test]
fn test_get_field_pairs_245() {
    let record = create_linked_record();

    let pairs = record.get_field_pairs("245");
    assert_eq!(pairs.len(), 1);

    let (orig, linked) = pairs[0];
    assert_eq!(orig.tag, "245");
    assert!(linked.is_some());
}

// =============================================================================
// FIND LINKED BY OCCURRENCE TESTS
// =============================================================================

#[test]
fn test_find_linked_by_occurrence_01() {
    let record = create_linked_record();

    // Field 100 has linkage "880-01", so occurrence is "01"
    let fields = record.find_linked_by_occurrence("01");
    assert!(fields.len() >= 2); // Should find both 100 and its 880
}

#[test]
fn test_find_linked_by_occurrence_02() {
    let record = create_linked_record();

    // Field 245 has linkage "880-02", so occurrence is "02"
    let fields = record.find_linked_by_occurrence("02");
    assert!(fields.len() >= 2); // Should find both 245 and its 880
}

#[test]
fn test_find_linked_by_occurrence_no_match() {
    let record = create_linked_record();

    let fields = record.find_linked_by_occurrence("999");
    assert!(fields.is_empty());
}

// =============================================================================
// ROUNDTRIP TESTS (880 <-> Original)
// =============================================================================

#[test]
fn test_roundtrip_original_to_880_and_back() {
    let record = create_linked_record();

    let field_100 = record.get_field("100").unwrap();
    let field_880 = record.get_linked_field(field_100).unwrap();
    let field_100_again = record.get_original_field(field_880).unwrap();

    // Should end up back at the original
    assert_eq!(field_100.tag, field_100_again.tag);
    assert_eq!(
        field_100.get_subfield('a'),
        field_100_again.get_subfield('a')
    );
}

#[test]
fn test_roundtrip_880_to_original_and_back() {
    let record = create_linked_record();

    let all_880 = record.get_all_880_fields();
    let field_880 = all_880[0];

    let field_100 = record.get_original_field(field_880).unwrap();
    let field_880_again = record.get_linked_field(field_100).unwrap();

    // Should end up back at the 880
    assert_eq!(field_880.tag, field_880_again.tag);
    assert_eq!(
        field_880.get_subfield('a'),
        field_880_again.get_subfield('a')
    );
}

// =============================================================================
// EDGE CASES
// =============================================================================

#[test]
fn test_empty_record_no_880() {
    let record = Record::new(make_leader());

    let fields_880 = record.get_all_880_fields();
    assert!(fields_880.is_empty());
}

#[test]
fn test_record_only_880_fields() {
    let mut record = Record::new(make_leader());

    // Add only 880 fields without originals
    let mut field_880 = Field::new("880".to_string(), '1', ' ');
    field_880.add_subfield_str('6', "100-01");
    field_880.add_subfield_str('a', "Some author");
    record.add_field(field_880);

    let all_880 = record.get_all_880_fields();
    assert_eq!(all_880.len(), 1);

    // get_original_field should return None (no matching 100)
    let field = all_880[0];
    let original = record.get_original_field(field);
    assert!(original.is_none());
}

#[test]
fn test_multiple_880_same_occurrence() {
    // This is unusual but test it anyway
    let mut record = Record::new(make_leader());

    let mut field_100 = Field::new("100".to_string(), '1', ' ');
    field_100.add_subfield_str('6', "880-01");
    field_100.add_subfield_str('a', "Author");
    record.add_field(field_100);

    // Add two 880 fields with same occurrence (malformed, but test it)
    let mut field_880a = Field::new("880".to_string(), '1', ' ');
    field_880a.add_subfield_str('6', "100-01");
    field_880a.add_subfield_str('a', "Romanized 1");
    record.add_field(field_880a);

    let mut field_880b = Field::new("880".to_string(), '1', ' ');
    field_880b.add_subfield_str('6', "100-01");
    field_880b.add_subfield_str('a', "Romanized 2");
    record.add_field(field_880b);

    let field_100 = record.get_field("100").unwrap();
    let linked = record.get_linked_field(field_100).unwrap();

    // Should return the first match
    assert_eq!(linked.tag, "880");
}

#[test]
fn test_complex_multilingual_record() {
    let mut record = Record::new(make_leader());

    // English author
    let mut field_100 = Field::new("100".to_string(), '1', ' ');
    field_100.add_subfield_str('a', "Smith, John");
    record.add_field(field_100);

    // Arabic title with romanization
    let mut field_245_ar = Field::new("245".to_string(), '1', '0');
    field_245_ar.add_subfield_str('6', "880-01");
    field_245_ar.add_subfield_str('a', "كتاب");
    record.add_field(field_245_ar);

    let mut field_880_245 = Field::new("880".to_string(), '1', '0');
    field_880_245.add_subfield_str('6', "245-01");
    field_880_245.add_subfield_str('a', "Kitab");
    record.add_field(field_880_245);

    // Chinese subject with romanization
    let mut field_650_zh = Field::new("650".to_string(), ' ', '0');
    field_650_zh.add_subfield_str('6', "880-02");
    field_650_zh.add_subfield_str('a', "中国文化");
    record.add_field(field_650_zh);

    let mut field_880_650 = Field::new("880".to_string(), ' ', '0');
    field_880_650.add_subfield_str('6', "650-02");
    field_880_650.add_subfield_str('a', "Chinese culture");
    record.add_field(field_880_650);

    let all_880 = record.get_all_880_fields();
    assert_eq!(all_880.len(), 2);

    // Verify pairs
    let pairs_245 = record.get_field_pairs("245");
    assert_eq!(pairs_245.len(), 1);
    assert!(pairs_245[0].1.is_some());

    let pairs_650 = record.get_field_pairs("650");
    assert_eq!(pairs_650.len(), 1);
    assert!(pairs_650[0].1.is_some());

    let pairs_100 = record.get_field_pairs("100");
    assert_eq!(pairs_100.len(), 1);
    assert!(pairs_100[0].1.is_none()); // No 880 for 100
}
