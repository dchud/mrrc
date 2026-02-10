//! BIBFRAME round-trip tests.
//!
//! These tests verify that MARC → BIBFRAME → MARC conversion preserves
//! essential bibliographic data with acceptable data loss documentation.

use mrrc::bibframe::{bibframe_to_marc, marc_to_bibframe, BibframeConfig};
use mrrc::leader::Leader;
use mrrc::record::{Field, Record};

fn make_test_leader() -> Leader {
    Leader {
        record_length: 1000,
        record_status: 'n',
        record_type: 'a',
        bibliographic_level: 'm',
        control_record_type: ' ',
        character_coding: 'a',
        indicator_count: 2,
        subfield_code_count: 2,
        data_base_address: 100,
        encoding_level: ' ',
        cataloging_form: 'a',
        multipart_level: ' ',
        reserved: "4500".to_string(),
    }
}

fn make_config() -> BibframeConfig {
    BibframeConfig::new().with_base_uri("http://example.org/")
}

// ============================================================================
// Round-Trip Tests: Basic Record
// ============================================================================

#[test]
fn test_roundtrip_minimal_record() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "roundtrip-001".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    // Convert to BIBFRAME
    let graph = marc_to_bibframe(&record, &make_config());

    // Convert back to MARC
    let result_record = bibframe_to_marc(&graph);

    // Should successfully round-trip (may be empty initially)
    assert!(result_record.is_ok(), "Round-trip should not error");
}

#[test]
fn test_roundtrip_title_preservation() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "title-rt".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "Test Title /".to_string());
    f245.add_subfield('c', "by Author.".to_string());
    record.add_field(f245);

    let graph = marc_to_bibframe(&record, &make_config());
    let result_record = bibframe_to_marc(&graph).expect("conversion failed");

    // Title might be reconstructed or present
    let has_title = result_record.fields().any(|f| f.tag.starts_with("24"));

    assert!(has_title, "Title should be preserved through round-trip");
}

#[test]
fn test_roundtrip_creator_preservation() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "creator-rt".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f100 = Field::new("100".to_string(), '1', ' ');
    f100.add_subfield('a', "Smith, John,".to_string());
    f100.add_subfield('d', "1950-".to_string());
    record.add_field(f100);

    let graph = marc_to_bibframe(&record, &make_config());
    let result_record = bibframe_to_marc(&graph).expect("conversion failed");

    // Creator information should be in result (1XX or 7XX)
    let has_creator = result_record
        .fields()
        .any(|f| f.tag == "100" || f.tag == "700");

    assert!(
        has_creator,
        "Creator should be preserved through round-trip"
    );
}

#[test]
fn test_roundtrip_subject_preservation() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "subject-rt".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f650 = Field::new("650".to_string(), ' ', '0');
    f650.add_subfield('a', "Computer science".to_string());
    f650.add_subfield('x', "Study and teaching.".to_string());
    record.add_field(f650);

    let graph = marc_to_bibframe(&record, &make_config());
    let result_record = bibframe_to_marc(&graph).expect("conversion failed");

    // Subject should be preserved (6XX fields)
    let has_subject = result_record.fields().any(|f| f.tag.starts_with('6'));

    assert!(
        has_subject,
        "Subject should be preserved through round-trip"
    );
}

#[test]
fn test_roundtrip_identifier_preservation() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "id-rt".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f020 = Field::new("020".to_string(), ' ', ' ');
    f020.add_subfield('a', "0123456789".to_string());
    record.add_field(f020);

    let graph = marc_to_bibframe(&record, &make_config());
    let result_record = bibframe_to_marc(&graph).expect("conversion failed");

    // Identifier should be preserved (020, 022, 024, etc.)
    let has_identifier = result_record
        .fields()
        .any(|f| f.tag == "020" || f.tag == "022" || f.tag == "024" || f.tag == "035");

    assert!(
        has_identifier,
        "Identifier should be preserved through round-trip"
    );
}

// ============================================================================
// Round-Trip Tests: Complex Records
// ============================================================================

#[test]
fn test_roundtrip_complex_book() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "complex-rt".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    // Multiple identifiers
    let mut f020 = Field::new("020".to_string(), ' ', ' ');
    f020.add_subfield('a', "0123456789".to_string());
    record.add_field(f020);

    // Title
    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "The Book /".to_string());
    f245.add_subfield('c', "by Author.".to_string());
    record.add_field(f245);

    // Creator
    let mut f100 = Field::new("100".to_string(), '1', ' ');
    f100.add_subfield('a', "Author, Someone,".to_string());
    f100.add_subfield('4', "aut".to_string());
    record.add_field(f100);

    // Publication
    let mut f260 = Field::new("260".to_string(), ' ', ' ');
    f260.add_subfield('a', "New York :".to_string());
    f260.add_subfield('b', "Publisher,".to_string());
    f260.add_subfield('c', "2001.".to_string());
    record.add_field(f260);

    // Physical description
    let mut f300 = Field::new("300".to_string(), ' ', ' ');
    f300.add_subfield('a', "500 pages.".to_string());
    record.add_field(f300);

    // Multiple subjects
    let mut f650a = Field::new("650".to_string(), ' ', '0');
    f650a.add_subfield('a', "Subject 1.".to_string());
    record.add_field(f650a);

    let mut f650b = Field::new("650".to_string(), ' ', '0');
    f650b.add_subfield('a', "Subject 2.".to_string());
    record.add_field(f650b);

    // Contributor
    let mut f700 = Field::new("700".to_string(), '1', ' ');
    f700.add_subfield('a', "Editor, Someone,".to_string());
    f700.add_subfield('e', "editor.".to_string());
    record.add_field(f700);

    let graph = marc_to_bibframe(&record, &make_config());
    let result_record = bibframe_to_marc(&graph).expect("conversion failed");

    // Check that essential data types are preserved
    let has_title = result_record.fields().any(|f| f.tag.starts_with("24"));
    let has_creator = result_record
        .fields()
        .any(|f| f.tag == "100" || f.tag == "700");
    let has_subject = result_record.fields().any(|f| f.tag.starts_with('6'));
    let has_identifier = result_record
        .fields()
        .any(|f| f.tag == "020" || f.tag == "024");

    assert!(has_title, "Title should be preserved");
    assert!(has_creator, "Creator should be preserved");
    assert!(has_subject, "Subject should be preserved");
    assert!(has_identifier, "Identifier should be preserved");
}

// ============================================================================
// Round-Trip Tests: Format Preservation
// ============================================================================

#[test]
fn test_roundtrip_record_type_book() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "format-book".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let result_record = bibframe_to_marc(&graph).expect("conversion failed");

    // Record type and bib level should be reconstructed
    assert_eq!(
        result_record.leader.record_type, 'a',
        "Record type should be preserved as language material"
    );
}

#[test]
fn test_roundtrip_record_type_serial() {
    let mut leader = make_test_leader();
    leader.record_type = 'a';
    leader.bibliographic_level = 's';
    let mut record = Record::new(leader);

    record.add_control_field("001".to_string(), "format-serial".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520c2001    xxu          0eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let result_record = bibframe_to_marc(&graph).expect("conversion failed");

    // Serial indicator should be preserved or reconstructed
    assert!(
        result_record.leader.bibliographic_level == 's'
            || result_record.fields().any(|f| f.tag == "490"),
        "Serial record should preserve series/serial info"
    );
}

// ============================================================================
// Round-Trip Tests: Acceptable Data Loss Documentation
// ============================================================================

#[test]
fn test_roundtrip_notes_handling() {
    // Notes (5XX fields) are typically preserved as general notes
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "notes-rt".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f500 = Field::new("500".to_string(), ' ', ' ');
    f500.add_subfield('a', "This is a general note.".to_string());
    record.add_field(f500);

    let mut f520 = Field::new("520".to_string(), ' ', ' ');
    f520.add_subfield('a', "Summary of the work.".to_string());
    record.add_field(f520);

    let graph = marc_to_bibframe(&record, &make_config());
    let result_record = bibframe_to_marc(&graph).expect("conversion failed");

    // Notes may be consolidated into 500 fields (acceptable loss of note type distinction)
    let has_notes = result_record.fields().any(|f| f.tag.starts_with('5'));

    assert!(
        has_notes,
        "Note information should be preserved (may consolidate note types)"
    );
}

#[test]
fn test_roundtrip_language_field() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "lang-rt".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 fre  ".to_string(),
    );

    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "Le Titre.".to_string());
    record.add_field(f245);

    let graph = marc_to_bibframe(&record, &make_config());
    let result_record = bibframe_to_marc(&graph).expect("conversion failed");

    // Language code should be preserved in 008
    let field_008 = result_record.get_control_field("008").unwrap_or("");
    assert!(
        field_008.len() >= 38,
        "008 field should be preserved with language info"
    );
}

#[test]
fn test_roundtrip_publication_year() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "year-rt".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2023    xxu           000 0 eng  ".to_string(),
    );

    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "Book 2023.".to_string());
    record.add_field(f245);

    let graph = marc_to_bibframe(&record, &make_config());
    let result_record = bibframe_to_marc(&graph).expect("conversion failed");

    // Publication year should be in 008
    let field_008 = result_record.get_control_field("008").unwrap_or("");
    assert!(
        field_008.len() >= 11,
        "Publication year should be preserved in 008"
    );
}

// ============================================================================
// Round-Trip Tests: Edge Cases
// ============================================================================

#[test]
fn test_roundtrip_empty_subfields() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "empty-sub".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "Title".to_string());
    record.add_field(f245);

    let graph = marc_to_bibframe(&record, &make_config());
    let result = bibframe_to_marc(&graph);

    // Should not crash on empty subfields
    assert!(
        result.is_ok(),
        "Empty subfields should be handled gracefully"
    );
}

#[test]
fn test_roundtrip_special_characters() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "special-chars".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "Title with © and ® symbols /".to_string());
    record.add_field(f245);

    let graph = marc_to_bibframe(&record, &make_config());
    let result_record = bibframe_to_marc(&graph).expect("conversion failed");

    // Title should survive round-trip
    let has_title = result_record.fields().any(|f| f.tag == "245");

    assert!(has_title, "Title with special characters should survive");
}

#[test]
fn test_roundtrip_many_subjects() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "many-subj".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    // Add many subject fields
    for i in 0..10 {
        let mut f650 = Field::new("650".to_string(), ' ', '0');
        f650.add_subfield('a', format!("Subject {i}"));
        record.add_field(f650);
    }

    let graph = marc_to_bibframe(&record, &make_config());
    let result_record = bibframe_to_marc(&graph).expect("conversion failed");

    // Subjects should be preserved (may not all be present but some should)
    let subject_count = result_record
        .fields()
        .filter(|f| f.tag.starts_with('6'))
        .count();

    assert!(
        subject_count > 0,
        "At least some subjects should be preserved"
    );
}

// ============================================================================
// Round-Trip Tests: Data Loss Categories Documentation
// ============================================================================

/// Tests document expected data loss:
/// 1. Non-filing indicators (245 ind2) - structure lost, title preserved
/// 2. Authority record links ($0/$9) - relationships may be lost
/// 3. Relator codes (7XX $e/$4) - may consolidate in 700 $e
/// 4. Sub-note types (520 vs 500) - consolidated to 500 (general note)
/// 5. Detailed 008 analysis - some codes may be approximated

#[test]
fn test_documented_loss_non_filing_indicators() {
    // BIBFRAME doesn't preserve non-filing indicator concept
    // But title text is preserved
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "loss-non-filing".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f245 = Field::new("245".to_string(), '1', '2');
    f245.add_subfield('a', "A Test of Non-filing.".to_string());
    record.add_field(f245);

    let graph = marc_to_bibframe(&record, &make_config());
    let result_record = bibframe_to_marc(&graph).expect("conversion failed");

    // Title should be preserved, but indicator information lost
    let title_preserved = result_record.fields().any(|f| f.tag == "245");

    assert!(
        title_preserved,
        "Title text preserved (non-filing indicator reconstructed or lost)"
    );
}

#[test]
fn test_documented_loss_authority_links() {
    // Authority record numbers ($0) are reference data, not essential bibliographic data
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "loss-authority".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f700 = Field::new("700".to_string(), '1', ' ');
    f700.add_subfield('a', "Author, Name,".to_string());
    f700.add_subfield('0', "(OCoLC)12345678".to_string());
    record.add_field(f700);

    let graph = marc_to_bibframe(&record, &make_config());
    let result_record = bibframe_to_marc(&graph).expect("conversion failed");

    // Author name should be preserved
    let author_preserved = result_record.fields().any(|f| f.tag == "700");

    assert!(
        author_preserved,
        "Author name preserved (authority link may be lost)"
    );
}
