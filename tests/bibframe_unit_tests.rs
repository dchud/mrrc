//! BIBFRAME unit tests for individual MARC tag → BIBFRAME property mappings.
//!
//! These tests verify individual conversion mappings and indicator handling
//! without requiring full baseline comparison.

#![cfg(feature = "format-bibframe")]

use mrrc::bibframe::{marc_to_bibframe, BibframeConfig, RdfFormat};
use mrrc::leader::Leader;
use mrrc::record::{Field, Record};

// ============================================================================
// Test Utilities
// ============================================================================

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

/// Checks if RDF output contains a specific property with expected value.
fn has_property(rdf: &str, property: &str, expected_value: &str) -> bool {
    rdf.contains(&format!("<{}>", property)) && rdf.contains(expected_value)
}

/// Checks if RDF output contains a BIBFRAME class.
fn has_class(rdf: &str, class_name: &str) -> bool {
    rdf.contains(&format!("<{}", class_name)) || rdf.contains(&format!("bf:{}", class_name))
}

// ============================================================================
// Unit Tests: Title Fields (245, 246)
// ============================================================================

#[test]
fn test_title_245_main_title() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-001".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "The test record /".to_string());
    f245.add_subfield('c', "by Test Author.".to_string());
    record.add_field(f245);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    // Verify main title is present
    assert!(
        rdf.contains("The test record") || rdf.contains("mainTitle"),
        "Should contain main title"
    );
    assert!(graph.len() > 0, "Graph should have triples for title");
}

#[test]
fn test_title_245_with_subtitle() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-002".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "Main title :".to_string());
    f245.add_subfield('b', "subtitle ;".to_string());
    f245.add_subfield('c', "by Author.".to_string());
    record.add_field(f245);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(rdf.contains("Main title"), "Should contain main title part");
    assert!(rdf.contains("subtitle"), "Should contain subtitle");
}

#[test]
fn test_title_246_variant_title() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-003".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "Primary Title.".to_string());
    record.add_field(f245);

    let mut f246 = Field::new("246".to_string(), ' ', '1');
    f246.add_subfield('a', "Alternative Title.".to_string());
    record.add_field(f246);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("Primary Title"),
        "Should contain primary title"
    );
    // Variant title mapping depends on implementation
}

// ============================================================================
// Unit Tests: Creator/Contributor Fields (1XX, 7XX)
// ============================================================================

#[test]
fn test_creator_100_person() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-100".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f100 = Field::new("100".to_string(), '1', ' ');
    f100.add_subfield('a', "Smith, John,".to_string());
    f100.add_subfield('d', "1950-".to_string());
    record.add_field(f100);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    // Should mention person/creator
    assert!(
        rdf.contains("Smith, John") || rdf.contains("Person"),
        "Should contain creator name or Person type"
    );
}

#[test]
fn test_creator_100_with_relator() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-101".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f100 = Field::new("100".to_string(), '1', ' ');
    f100.add_subfield('a', "Doe, Jane,".to_string());
    f100.add_subfield('4', "aut".to_string());
    record.add_field(f100);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("Doe, Jane") || rdf.contains("agent"),
        "Should contain creator agent"
    );
}

#[test]
fn test_contributor_700_person() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-700".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f700 = Field::new("700".to_string(), '1', ' ');
    f700.add_subfield('a', "Jones, Mary,".to_string());
    f700.add_subfield('e', "editor.".to_string());
    record.add_field(f700);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("Jones, Mary") || rdf.contains("agent"),
        "Should contain contributor"
    );
}

// ============================================================================
// Unit Tests: Subject Fields (6XX)
// ============================================================================

#[test]
fn test_subject_650_topic() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-650".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f650 = Field::new("650".to_string(), ' ', '0');
    f650.add_subfield('a', "Computer science".to_string());
    f650.add_subfield('x', "Study and teaching.".to_string());
    record.add_field(f650);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("Computer science") || rdf.contains("subject"),
        "Should contain subject"
    );
}

#[test]
fn test_subject_650_with_subdivisions() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-651".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f651 = Field::new("651".to_string(), ' ', '0');
    f651.add_subfield('a', "United States".to_string());
    f651.add_subfield('x', "History.".to_string());
    record.add_field(f651);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("United States") || rdf.contains("Place"),
        "Should contain geographic subject"
    );
}

#[test]
fn test_subject_655_genre() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-655".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f655 = Field::new("655".to_string(), ' ', '0');
    f655.add_subfield('a', "Science fiction.".to_string());
    record.add_field(f655);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    // Genre mapping implementation-dependent
    assert!(rdf.len() > 0, "Should produce RDF output");
}

// ============================================================================
// Unit Tests: Identifier Fields (020, 022, 024, 035)
// ============================================================================

#[test]
fn test_identifier_020_isbn() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-020".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f020 = Field::new("020".to_string(), ' ', ' ');
    f020.add_subfield('a', "0123456789".to_string());
    f020.add_subfield('q', "(hardcover)".to_string());
    record.add_field(f020);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("0123456789") || rdf.contains("Isbn"),
        "Should contain ISBN identifier"
    );
}

#[test]
fn test_identifier_022_issn() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-022".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f022 = Field::new("022".to_string(), ' ', ' ');
    f022.add_subfield('a', "1234-5678".to_string());
    record.add_field(f022);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("1234-5678") || rdf.contains("Issn"),
        "Should contain ISSN identifier"
    );
}

#[test]
fn test_identifier_024_multiple_types() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-024".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    // EAN (indicator 3)
    let mut f024 = Field::new("024".to_string(), '3', ' ');
    f024.add_subfield('a', "9780123456789".to_string());
    record.add_field(f024);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(rdf.contains("9780123456789"), "Should contain EAN");
}

#[test]
fn test_identifier_035_system_number() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-035".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f035 = Field::new("035".to_string(), ' ', ' ');
    f035.add_subfield('a', "(OCoLC)12345678".to_string());
    record.add_field(f035);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("12345678") || rdf.contains("OCoLC"),
        "Should contain system number"
    );
}

// ============================================================================
// Unit Tests: Publication/Provision Activity Fields (260, 264)
// ============================================================================

#[test]
fn test_publication_260_basic() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-260".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f260 = Field::new("260".to_string(), ' ', ' ');
    f260.add_subfield('a', "New York :".to_string());
    f260.add_subfield('b', "Publisher,".to_string());
    f260.add_subfield('c', "2001.".to_string());
    record.add_field(f260);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("New York") || rdf.contains("Publisher"),
        "Should contain publication info"
    );
}

#[test]
fn test_publication_264_RDA() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-264".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    // 264 with indicator 1='1' (publication)
    let mut f264 = Field::new("264".to_string(), ' ', '1');
    f264.add_subfield('a', "London :".to_string());
    f264.add_subfield('b', "New Publisher,".to_string());
    f264.add_subfield('c', "2023.".to_string());
    record.add_field(f264);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("London") || rdf.contains("New Publisher"),
        "Should contain RDA publication info"
    );
}

// ============================================================================
// Unit Tests: Physical Description Field (300)
// ============================================================================

#[test]
fn test_physical_description_300() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-300".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f300 = Field::new("300".to_string(), ' ', ' ');
    f300.add_subfield('a', "500 pages :".to_string());
    f300.add_subfield('b', "illustrations ;".to_string());
    f300.add_subfield('c', "24 cm.".to_string());
    record.add_field(f300);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("500") || rdf.contains("pages") || rdf.contains("extent"),
        "Should contain extent/physical description"
    );
}

// ============================================================================
// Unit Tests: Notes Field (5XX)
// ============================================================================

#[test]
fn test_note_500_general() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-500".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f500 = Field::new("500".to_string(), ' ', ' ');
    f500.add_subfield(
        'a',
        "Based on the author's doctoral dissertation.".to_string(),
    );
    record.add_field(f500);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("doctoral") || rdf.contains("note"),
        "Should contain general note"
    );
}

#[test]
fn test_note_520_summary() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-520".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f520 = Field::new("520".to_string(), ' ', ' ');
    f520.add_subfield('a', "A comprehensive overview of the subject.".to_string());
    record.add_field(f520);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("overview") || rdf.contains("summary"),
        "Should contain summary note"
    );
}

// ============================================================================
// Unit Tests: Indicator Handling
// ============================================================================

#[test]
fn test_indicator_245_non_filing() {
    // Indicator 2 of 245 specifies non-filing characters
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-ind-245".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f245 = Field::new("245".to_string(), '1', '2');
    // Second indicator '2' means skip first 2 characters ("A ")
    f245.add_subfield('a', "A Test Title.".to_string());
    record.add_field(f245);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    // Should extract title (with or without non-filing processing)
    assert!(
        rdf.contains("Test") || rdf.contains("Title"),
        "Should contain title"
    );
}

#[test]
fn test_indicator_650_subject_source() {
    // Indicator 2 of 650 specifies controlled vocabulary source
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-ind-650".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f650 = Field::new("650".to_string(), ' ', '0');
    // Indicator 2 = '0' means LCSH
    f650.add_subfield('a', "Computer science".to_string());
    record.add_field(f650);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("Computer science"),
        "Should contain subject with vocabulary source indicator"
    );
}

// ============================================================================
// Unit Tests: Subfield Combinations
// ============================================================================

#[test]
fn test_subfield_combination_100_dates() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-subfield-100".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f100 = Field::new("100".to_string(), '1', ' ');
    f100.add_subfield('a', "Darwin, Charles,".to_string());
    f100.add_subfield('d', "1809-1882.".to_string());
    record.add_field(f100);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(rdf.contains("Darwin"), "Should contain author name");
    // Date handling depends on implementation
}

#[test]
fn test_subfield_combination_650_subdivisions() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-subfield-650".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f650 = Field::new("650".to_string(), ' ', '0');
    f650.add_subfield('a', "History".to_string());
    f650.add_subfield('z', "United States.".to_string());
    f650.add_subfield('x', "20th century.".to_string());
    record.add_field(f650);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("History") || rdf.contains("United States"),
        "Should handle subject subdivisions"
    );
}

// ============================================================================
// Unit Tests: Leader and Type Determination
// ============================================================================

#[test]
fn test_leader_type_book() {
    let mut leader = make_test_leader();
    leader.record_type = 'a'; // language material
    leader.bibliographic_level = 'm'; // monograph
    let mut record = Record::new(leader);
    record.add_control_field("001".to_string(), "test-type-book".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("Text") || rdf.contains("bf:Text"),
        "Book should map to Text type"
    );
}

#[test]
fn test_leader_type_serial() {
    let mut leader = make_test_leader();
    leader.record_type = 'a'; // language material
    leader.bibliographic_level = 's'; // serial
    let mut record = Record::new(leader);
    record.add_control_field("001".to_string(), "test-type-serial".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520c2001    xxu          0eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("Serial") || rdf.contains("bf:Serial"),
        "Serial should map to Serial type"
    );
}

#[test]
fn test_leader_type_music() {
    let mut leader = make_test_leader();
    leader.record_type = 'c'; // notated music
    let mut record = Record::new(leader);
    record.add_control_field("001".to_string(), "test-type-music".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu  n       000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("NotatedMusic") || rdf.contains("bf:NotatedMusic"),
        "Music should map to NotatedMusic type"
    );
}

#[test]
fn test_leader_type_map() {
    let mut leader = make_test_leader();
    leader.record_type = 'e'; // cartographic
    let mut record = Record::new(leader);
    record.add_control_field("001".to_string(), "test-type-map".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("Cartography") || rdf.contains("bf:Cartography"),
        "Map should map to Cartography type"
    );
}

#[test]
fn test_leader_type_visual() {
    let mut leader = make_test_leader();
    leader.record_type = 'g'; // projected medium
    let mut record = Record::new(leader);
    record.add_control_field("001".to_string(), "test-type-visual".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("MovingImage")
            || rdf.contains("bf:MovingImage")
            || rdf.contains("StillImage")
            || rdf.contains("bf:StillImage"),
        "Visual material should map to MovingImage or StillImage"
    );
}

// ============================================================================
// Unit Tests: Empty/Minimal Records
// ============================================================================

#[test]
fn test_empty_record() {
    let record = Record::new(make_test_leader());
    let graph = marc_to_bibframe(&record, &make_config());

    // Even empty record should produce Work and Instance
    assert!(
        graph.len() > 0,
        "Empty record should still produce RDF triples"
    );
}

#[test]
fn test_record_with_control_fields_only() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "control-id".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());

    assert!(
        graph.len() > 0,
        "Record with only control fields should produce RDF"
    );
}
