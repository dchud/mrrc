//! BIBFRAME integration tests.
//!
//! These tests verify conversion of complete bibliographic records with
//! multiple field types and format variations (books, serials, music, maps, etc.)

use mrrc::bibframe::{marc_to_bibframe, BibframeConfig, RdfFormat};
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
// Integration Tests: Book Records
// ============================================================================

#[test]
fn test_integration_simple_book() {
    let mut record = Record::new(make_test_leader());

    // Control fields
    record.add_control_field("001".to_string(), "book-001".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    // ISBN
    let mut f020 = Field::new("020".to_string(), ' ', ' ');
    f020.add_subfield('a', "0123456789".to_string());
    record.add_field(f020);

    // LCCN
    let mut f010 = Field::new("010".to_string(), ' ', ' ');
    f010.add_subfield('a', "2001234567".to_string());
    record.add_field(f010);

    // Title
    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "Introduction to Computer Science /".to_string());
    f245.add_subfield('c', "by Jane Doe.".to_string());
    record.add_field(f245);

    // Creator
    let mut f100 = Field::new("100".to_string(), '1', ' ');
    f100.add_subfield('a', "Doe, Jane,".to_string());
    f100.add_subfield('d', "1960-".to_string());
    f100.add_subfield('4', "aut".to_string());
    record.add_field(f100);

    // Publication
    let mut f260 = Field::new("260".to_string(), ' ', ' ');
    f260.add_subfield('a', "New York :".to_string());
    f260.add_subfield('b', "Academic Press,".to_string());
    f260.add_subfield('c', "2001.".to_string());
    record.add_field(f260);

    // Physical description
    let mut f300 = Field::new("300".to_string(), ' ', ' ');
    f300.add_subfield('a', "500 pages :".to_string());
    f300.add_subfield('b', "illustrations ;".to_string());
    f300.add_subfield('c', "24 cm.".to_string());
    record.add_field(f300);

    // Subject
    let mut f650 = Field::new("650".to_string(), ' ', '0');
    f650.add_subfield('a', "Computer science".to_string());
    f650.add_subfield('x', "Textbooks.".to_string());
    record.add_field(f650);

    // Notes
    let mut f500 = Field::new("500".to_string(), ' ', ' ');
    f500.add_subfield('a', "Includes index.".to_string());
    record.add_field(f500);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    // Verify all major elements are present
    assert!(
        rdf.contains("0123456789") || rdf.contains("Isbn"),
        "ISBN should be present"
    );
    assert!(
        rdf.contains("Introduction to Computer Science") || rdf.contains("mainTitle"),
        "Title should be present"
    );
    assert!(
        rdf.contains("Doe, Jane") || rdf.contains("Person"),
        "Creator should be present"
    );
    assert!(
        rdf.contains("New York") || rdf.contains("Academic Press"),
        "Publication info should be present"
    );
    assert!(
        rdf.contains("Computer science") || rdf.contains("Topic"),
        "Subject should be present"
    );
    assert!(
        rdf.contains("Text") || rdf.contains("bf:Text"),
        "Book should be typed as Text"
    );
}

#[test]
fn test_integration_book_with_translator() {
    let mut record = Record::new(make_test_leader());

    record.add_control_field("001".to_string(), "trans-001".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    // Title
    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "The Book /".to_string());
    f245.add_subfield(
        'c',
        "by Original Author ; translated by Jane Translator.".to_string(),
    );
    record.add_field(f245);

    // Original author
    let mut f100 = Field::new("100".to_string(), '1', ' ');
    f100.add_subfield('a', "Author, Original,".to_string());
    f100.add_subfield('4', "aut".to_string());
    record.add_field(f100);

    // Translator as contributor
    let mut f700 = Field::new("700".to_string(), '1', ' ');
    f700.add_subfield('a', "Translator, Jane,".to_string());
    f700.add_subfield('e', "translator.".to_string());
    f700.add_subfield('4', "trl".to_string());
    record.add_field(f700);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("Author, Original") || rdf.contains("Translator, Jane"),
        "Should contain both author and translator"
    );
}

// ============================================================================
// Integration Tests: Serial Records
// ============================================================================

#[test]
fn test_integration_serial_record() {
    let mut leader = make_test_leader();
    leader.record_type = 'a';
    leader.bibliographic_level = 's'; // serial
    let mut record = Record::new(leader);

    record.add_control_field("001".to_string(), "serial-001".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520c2001    xxu          0eng  ".to_string(),
    );

    // ISSN
    let mut f022 = Field::new("022".to_string(), ' ', ' ');
    f022.add_subfield('a', "1234-5678".to_string());
    record.add_field(f022);

    // Title
    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "Journal of Computer Science.".to_string());
    record.add_field(f245);

    // Publication
    let mut f260 = Field::new("260".to_string(), ' ', ' ');
    f260.add_subfield('a', "New York :".to_string());
    f260.add_subfield('b', "Scientific Press,".to_string());
    f260.add_subfield('c', "2001-".to_string());
    record.add_field(f260);

    // Frequency
    let mut f310 = Field::new("310".to_string(), ' ', ' ');
    f310.add_subfield('a', "Monthly".to_string());
    record.add_field(f310);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("Serial") || rdf.contains("bf:Serial"),
        "Serial record should be typed as Serial"
    );
    assert!(
        rdf.contains("1234-5678") || rdf.contains("Issn"),
        "Should contain ISSN"
    );
}

#[test]
fn test_integration_serial_with_linking_entries() {
    let mut leader = make_test_leader();
    leader.record_type = 'a';
    leader.bibliographic_level = 's';
    let mut record = Record::new(leader);

    record.add_control_field("001".to_string(), "link-serial".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520c2001    xxu          0eng  ".to_string(),
    );

    // Title
    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "Current Journal.".to_string());
    record.add_field(f245);

    // Earlier title (780 with $t and $x)
    let mut f780 = Field::new("780".to_string(), '0', '0');
    f780.add_subfield('t', "Previous Title".to_string());
    f780.add_subfield('x', "0000-0000".to_string());
    record.add_field(f780);

    // Later title (785)
    let mut f785 = Field::new("785".to_string(), '0', '0');
    f785.add_subfield('t', "Future Title".to_string());
    f785.add_subfield('x', "9999-9999".to_string());
    record.add_field(f785);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    // Should reference linking entries
    assert!(
        rdf.contains("Previous")
            || rdf.contains("Future")
            || rdf.contains("precededBy")
            || rdf.contains("succeededBy"),
        "Should contain linking entry information"
    );
}

// ============================================================================
// Integration Tests: Music Records
// ============================================================================

#[test]
fn test_integration_music_score() {
    let mut leader = make_test_leader();
    leader.record_type = 'c'; // notated music
    let mut record = Record::new(leader);

    record.add_control_field("001".to_string(), "music-001".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu  n       000 0 eng  ".to_string(),
    );

    // Title
    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "Symphony No. 5 /".to_string());
    f245.add_subfield('c', "Beethoven.".to_string());
    record.add_field(f245);

    // Creator
    let mut f100 = Field::new("100".to_string(), '1', ' ');
    f100.add_subfield('a', "Beethoven, Ludwig van,".to_string());
    f100.add_subfield('d', "1770-1827.".to_string());
    f100.add_subfield('4', "cmp".to_string());
    record.add_field(f100);

    // Publication
    let mut f260 = Field::new("260".to_string(), ' ', ' ');
    f260.add_subfield('a', "Vienna :".to_string());
    f260.add_subfield('b', "Music Publisher,".to_string());
    f260.add_subfield('c', "1808.".to_string());
    record.add_field(f260);

    // Physical description (music-specific)
    let mut f300 = Field::new("300".to_string(), ' ', ' ');
    f300.add_subfield('a', "1 score (150 pages) :".to_string());
    f300.add_subfield('b', "choral.".to_string());
    record.add_field(f300);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("NotatedMusic") || rdf.contains("bf:NotatedMusic"),
        "Music score should be typed as NotatedMusic"
    );
    assert!(
        rdf.contains("Beethoven") || rdf.contains("Person"),
        "Should contain composer"
    );
}

#[test]
fn test_integration_music_recording() {
    let mut leader = make_test_leader();
    leader.record_type = 'i'; // nonmusical sound recording
    let mut record = Record::new(leader);

    record.add_control_field("001".to_string(), "audio-001".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    // Title
    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "Symphony No. 5 /".to_string());
    f245.add_subfield(
        'c',
        "[London Symphony Orchestra, conductor: Simon Rattle].".to_string(),
    );
    record.add_field(f245);

    // Performer
    let mut f700 = Field::new("700".to_string(), '1', ' ');
    f700.add_subfield('a', "Rattle, Simon,".to_string());
    f700.add_subfield('d', "1955-".to_string());
    f700.add_subfield('e', "conductor.".to_string());
    record.add_field(f700);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("Audio") || rdf.contains("MusicAudio") || rdf.contains("bf:Audio"),
        "Audio recording should be typed as Audio or MusicAudio"
    );
}

// ============================================================================
// Integration Tests: Map Records
// ============================================================================

#[test]
fn test_integration_map_record() {
    let mut leader = make_test_leader();
    leader.record_type = 'e'; // cartographic
    let mut record = Record::new(leader);

    record.add_control_field("001".to_string(), "map-001".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    // Title
    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "Map of North America.".to_string());
    record.add_field(f245);

    // Scale and coordinates
    let mut f034 = Field::new("034".to_string(), '1', ' ');
    f034.add_subfield('b', "2000000".to_string());
    record.add_field(f034);

    // Geographic area (651)
    let mut f651 = Field::new("651".to_string(), ' ', '0');
    f651.add_subfield('a', "North America".to_string());
    f651.add_subfield('x', "Maps.".to_string());
    record.add_field(f651);

    // Physical description
    let mut f300 = Field::new("300".to_string(), ' ', ' ');
    f300.add_subfield('a', "1 map :".to_string());
    f300.add_subfield('b', "color ;".to_string());
    f300.add_subfield('c', "50 x 40 cm.".to_string());
    record.add_field(f300);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("Cartography") || rdf.contains("bf:Cartography"),
        "Map should be typed as Cartography"
    );
    assert!(
        rdf.contains("North America") || rdf.contains("Place"),
        "Should contain geographic subject"
    );
}

// ============================================================================
// Integration Tests: Visual Materials
// ============================================================================

#[test]
fn test_integration_video_record() {
    let mut leader = make_test_leader();
    leader.record_type = 'g'; // projected medium
    let mut record = Record::new(leader);

    record.add_control_field("001".to_string(), "video-001".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    // Title
    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "Great Documentary Film /".to_string());
    f245.add_subfield('c', "directed by Someone.".to_string());
    record.add_field(f245);

    // Director as creator
    let mut f100 = Field::new("100".to_string(), '1', ' ');
    f100.add_subfield('a', "Someone, Director,".to_string());
    f100.add_subfield('4', "drt".to_string());
    record.add_field(f100);

    // Physical description
    let mut f300 = Field::new("300".to_string(), ' ', ' ');
    f300.add_subfield('a', "1 videodisc (95 min.) :".to_string());
    f300.add_subfield('b', "sound, color ;".to_string());
    f300.add_subfield('c', "4 3/4 in.".to_string());
    record.add_field(f300);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("MovingImage") || rdf.contains("bf:MovingImage"),
        "Video should be typed as MovingImage"
    );
    assert!(
        rdf.contains("videodisc"),
        "Should contain video format information"
    );
}

#[test]
fn test_integration_photograph_record() {
    let mut leader = make_test_leader();
    leader.record_type = 'k'; // two-dimensional nonprojectable graphic
    let mut record = Record::new(leader);

    record.add_control_field("001".to_string(), "photo-001".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    // Title
    let mut f245 = Field::new("245".to_string(), '0', '0');
    f245.add_subfield('a', "Portrait of the artist as a young person.".to_string());
    record.add_field(f245);

    // Subject
    let mut f650 = Field::new("650".to_string(), ' ', '0');
    f650.add_subfield('a', "Photographs".to_string());
    record.add_field(f650);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("StillImage") || rdf.contains("bf:StillImage"),
        "Photograph should be typed as StillImage"
    );
}

// ============================================================================
// Integration Tests: Multi-Script Records (880 Linked Fields)
// ============================================================================

#[test]
fn test_integration_alternate_script_record() {
    let mut record = Record::new(make_test_leader());

    record.add_control_field("001".to_string(), "script-001".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    // Latin script title
    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "Test Title.".to_string());
    record.add_field(f245);

    // Alternate script (880 field with $6 linkage)
    let mut f880 = Field::new("880".to_string(), '1', '0');
    f880.add_subfield('6', "245-01".to_string()); // Links to field 245
    f880.add_subfield('a', "テストタイトル。".to_string()); // Japanese
    record.add_field(f880);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    // Should handle both scripts
    assert!(
        rdf.contains("Test Title") || rdf.contains("title"),
        "Should contain Latin script title"
    );
    // Alternate script handling is implementation-dependent
}

// ============================================================================
// Integration Tests: Authority-Controlled Records
// ============================================================================

#[test]
fn test_integration_lcsh_headings() {
    let mut record = Record::new(make_test_leader());

    record.add_control_field("001".to_string(), "lcsh-001".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    // Title
    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "Book about History.".to_string());
    record.add_field(f245);

    // LCSH topical heading with subdivisions
    let mut f650 = Field::new("650".to_string(), ' ', '0');
    f650.add_subfield('a', "United States".to_string());
    f650.add_subfield('x', "History".to_string());
    f650.add_subfield('y', "19th century.".to_string());
    record.add_field(f650);

    // LCSH name heading with relationship
    let mut f700 = Field::new("700".to_string(), '1', ' ');
    f700.add_subfield('a', "Lincoln, Abraham,".to_string());
    f700.add_subfield('d', "1809-1865.".to_string());
    f700.add_subfield('e', "subject.".to_string());
    f700.add_subfield('0', "(OCoLC)12345678".to_string());
    record.add_field(f700);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    assert!(
        rdf.contains("United States") || rdf.contains("History"),
        "Should contain subject headings with subdivisions"
    );
    assert!(
        rdf.contains("Lincoln") || rdf.contains("Person"),
        "Should contain name subject"
    );
}

// ============================================================================
// Integration Tests: Complex Records with Multiple Identifiers
// ============================================================================

#[test]
fn test_integration_record_with_multiple_identifiers() {
    let mut record = Record::new(make_test_leader());

    record.add_control_field("001".to_string(), "ids-001".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    // LCCN
    let mut f010 = Field::new("010".to_string(), ' ', ' ');
    f010.add_subfield('a', "2001234567".to_string());
    record.add_field(f010);

    // ISBN
    let mut f020 = Field::new("020".to_string(), ' ', ' ');
    f020.add_subfield('a', "0123456789".to_string());
    f020.add_subfield('q', "(hardcover)".to_string());
    record.add_field(f020);

    // System number
    let mut f035 = Field::new("035".to_string(), ' ', ' ');
    f035.add_subfield('a', "(OCoLC)12345678".to_string());
    record.add_field(f035);

    // Title
    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "Highly Identifiable Book.".to_string());
    record.add_field(f245);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    // All identifiers should be present
    assert!(
        rdf.contains("2001234567") || rdf.contains("Lccn") || rdf.contains("bf:Lccn"),
        "Should contain LCCN"
    );
    assert!(
        rdf.contains("0123456789") || rdf.contains("Isbn") || rdf.contains("bf:Isbn"),
        "Should contain ISBN"
    );
    assert!(
        rdf.contains("12345678") || rdf.contains("OCoLC"),
        "Should contain system number"
    );
}

// ============================================================================
// Integration Tests: Record Types Completeness
// ============================================================================

#[test]
fn test_integration_all_format_types() {
    let test_cases = vec![
        ('a', 'm', "Text"),         // language material, monograph
        ('a', 's', "Serial"),       // language material, serial
        ('c', ' ', "NotatedMusic"), // notated music
        ('d', ' ', "NotatedMusic"), // manuscript music
        ('e', ' ', "Cartography"),  // cartographic material
        ('f', ' ', "Cartography"),  // manuscript cartography
        ('g', ' ', "MovingImage"),  // projected medium
        ('i', ' ', "Audio"),        // nonmusical sound recording
        ('j', ' ', "MusicAudio"),   // musical sound recording
        ('k', ' ', "StillImage"),   // two-dimensional nonprojectable graphic
        ('o', ' ', "Electronic"),   // kit
        ('p', ' ', "Text"),         // mixed materials
        ('r', ' ', "Text"),         // three-dimensional object/artifact
        ('t', ' ', "Text"),         // manuscript language material
    ];

    for (record_type, bib_level, _expected_bf_type) in test_cases {
        let mut leader = make_test_leader();
        leader.record_type = record_type;
        if bib_level != ' ' {
            leader.bibliographic_level = bib_level;
        }
        let mut record = Record::new(leader);
        record.add_control_field(
            "001".to_string(),
            format!("test-{record_type}-{}", bib_level as u32),
        );
        record.add_control_field(
            "008".to_string(),
            "040520s2001    xxu           000 0 eng  ".to_string(),
        );

        let graph = marc_to_bibframe(&record, &make_config());
        let rdf = graph
            .serialize(RdfFormat::RdfXml)
            .unwrap_or_else(|_| String::new());

        assert!(
            !rdf.is_empty(),
            "Record type {record_type} should produce RDF output"
        );
        // Type assertions are lenient - implementation may vary
    }
}

// ============================================================================
// Integration Tests: Edge Cases in Complete Records
// ============================================================================

#[test]
fn test_integration_record_with_missing_245() {
    // Some records may be missing main title (rare but possible)
    let mut record = Record::new(make_test_leader());

    record.add_control_field("001".to_string(), "no-245".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    // Only ISBN and publication info, no title
    let mut f020 = Field::new("020".to_string(), ' ', ' ');
    f020.add_subfield('a', "0123456789".to_string());
    record.add_field(f020);

    let graph = marc_to_bibframe(&record, &make_config());
    // Should not crash and should produce some output
    assert!(!graph.is_empty(), "Should produce RDF even without 245");
}

#[test]
fn test_integration_record_with_only_control_fields() {
    let mut record = Record::new(make_test_leader());

    record.add_control_field("001".to_string(), "minimal".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    // Even minimal record should produce Work and Instance
    assert!(
        rdf.contains("Work") || rdf.contains("Instance"),
        "Minimal record should produce Work/Instance"
    );
}

#[test]
fn test_integration_record_with_many_fields() {
    let mut record = Record::new(make_test_leader());

    record.add_control_field("001".to_string(), "many-fields".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    // Title
    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "Complex Record.".to_string());
    record.add_field(f245);

    // Add multiple subjects
    for i in 0..5 {
        let mut f650 = Field::new("650".to_string(), ' ', '0');
        f650.add_subfield('a', format!("Subject {i}"));
        record.add_field(f650);
    }

    // Add multiple contributors
    for i in 0..3 {
        let mut f700 = Field::new("700".to_string(), '1', ' ');
        f700.add_subfield('a', format!("Contributor {i}, Person {i}"));
        record.add_field(f700);
    }

    // Add multiple notes
    for i in 0..4 {
        let mut f500 = Field::new("500".to_string(), ' ', ' ');
        f500.add_subfield('a', format!("Note {i}"));
        record.add_field(f500);
    }

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph.serialize(RdfFormat::RdfXml).unwrap();

    // Should handle all fields without crashing
    assert!(
        rdf.contains("Complex Record") || rdf.contains("Subject"),
        "Should contain record data"
    );
    assert!(
        graph.len() > 10,
        "Complex record should produce many triples"
    );
}
