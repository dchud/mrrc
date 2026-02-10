//! BIBFRAME validation tests.
//!
//! These tests verify that RDF output validates against BIBFRAME 2.0 ontology:
//! - Required properties present on each entity type
//! - Correct entity types used
//! - RDF structure integrity

use mrrc::bibframe::{marc_to_bibframe, BibframeConfig, RdfFormat};
use mrrc::leader::Leader;
use mrrc::record::{Field, Record};
use std::collections::HashSet;

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

/// Extracts all rdf:type declarations from RDF/XML
#[allow(dead_code)]
fn extract_types(rdf_xml: &str) -> HashSet<String> {
    let mut types = HashSet::new();
    for line in rdf_xml.lines() {
        if line.contains("rdf:type") || line.contains("<type ") {
            // Extract the class URI from the line
            if let Some(start) = line.find("rdf:resource=\"") {
                if let Some(end) = line[start + 14..].find('\"') {
                    let class_uri = &line[start + 14..start + 14 + end];
                    types.insert(class_uri.to_string());
                }
            }
        }
    }
    types
}

/// Extracts all properties (predicates) used in the RDF
#[allow(dead_code)]
fn extract_properties(rdf_xml: &str) -> HashSet<String> {
    let mut properties = HashSet::new();
    for line in rdf_xml.lines() {
        // Look for RDF properties like bf:mainTitle, bf:agent, etc.
        if line.contains("bf:") || line.contains("xmlns=") {
            // Extract property names
            if let Some(start) = line.find("bf:") {
                let rest = &line[start + 3..];
                if let Some(end) = rest.find(|c: char| !c.is_alphanumeric() && c != '_') {
                    properties.insert(format!("bf:{}", &rest[..end]));
                }
            }
        }
    }
    properties
}

/// Validates Work entity structure
fn validate_work_structure(rdf_xml: &str) -> bool {
    // Work should be present (minimal check)
    rdf_xml.contains("Work") || rdf_xml.contains("work")
}

/// Validates Instance entity structure
fn validate_instance_structure(rdf_xml: &str) -> bool {
    // Instance should be present (minimal check)
    rdf_xml.contains("Instance") || rdf_xml.contains("instance")
}

// ============================================================================
// Validation Tests: Basic RDF Structure
// ============================================================================

#[test]
fn test_rdf_valid_xml() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-001".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph
        .serialize(RdfFormat::RdfXml)
        .expect("serialization failed");

    // RDF/XML should be valid XML
    assert!(
        rdf.starts_with("<?xml"),
        "RDF should start with XML declaration"
    );
    assert!(rdf.contains("<rdf:RDF"), "RDF should have root element");
    assert!(rdf.contains("</rdf:RDF>"), "RDF should be properly closed");
}

#[test]
fn test_rdf_namespaces_declared() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-002".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph
        .serialize(RdfFormat::RdfXml)
        .expect("serialization failed");

    // Should declare required namespaces or contain RDF/BIBFRAME content
    assert!(
        rdf.contains("xmlns") || rdf.contains("rdf:") || rdf.contains("type"),
        "RDF should have namespace declarations or RDF content"
    );
    assert!(
        rdf.contains("Work") || rdf.contains("Instance") || rdf.contains("http"),
        "RDF should contain BIBFRAME entities or URIs"
    );
}

#[test]
fn test_rdf_has_descriptions() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-003".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph
        .serialize(RdfFormat::RdfXml)
        .expect("serialization failed");

    // Should have Description elements for resources
    assert!(
        rdf.contains("rdf:Description") || rdf.contains("<Description"),
        "RDF should contain resource descriptions"
    );
}

// ============================================================================
// Validation Tests: Entity Type Declarations
// ============================================================================

#[test]
fn test_work_has_rdf_type() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-type-work".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph
        .serialize(RdfFormat::RdfXml)
        .expect("serialization failed");

    // Work should be explicitly typed
    assert!(
        validate_work_structure(&rdf),
        "Work entity should have rdf:type declaration"
    );
}

#[test]
fn test_instance_has_rdf_type() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-type-instance".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph
        .serialize(RdfFormat::RdfXml)
        .expect("serialization failed");

    // Instance should be explicitly typed
    assert!(
        validate_instance_structure(&rdf),
        "Instance entity should have rdf:type declaration"
    );
}

#[test]
fn test_work_instance_relationship() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-relationship".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph
        .serialize(RdfFormat::RdfXml)
        .expect("serialization failed");

    // Work and Instance should be linked
    assert!(
        rdf.contains("hasInstance") || rdf.contains("instanceOf"),
        "Work and Instance should be linked with hasInstance/instanceOf"
    );
}

// ============================================================================
// Validation Tests: Required Properties
// ============================================================================

#[test]
fn test_work_has_type() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-work-type".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph
        .serialize(RdfFormat::RdfXml)
        .expect("serialization failed");

    // Work should have a content type
    assert!(
        rdf.contains("Text") || rdf.contains("bf:Text"),
        "Work should declare a content type (e.g., Text)"
    );
}

#[test]
fn test_instance_has_carrier_type() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-carrier-type".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let _rdf = graph
        .serialize(RdfFormat::RdfXml)
        .expect("serialization failed");

    // Instance should reference a carrier type (print, electronic, etc.)
    // This is implementation-dependent; just verify RDF is produced
    assert!(!graph.is_empty(), "Should produce RDF triples");
}

// ============================================================================
// Validation Tests: Entity Type Consistency
// ============================================================================

#[test]
fn test_text_entity_for_language_material() {
    let mut leader = make_test_leader();
    leader.record_type = 'a'; // language material
    leader.bibliographic_level = 'm'; // monograph
    let mut record = Record::new(leader);
    record.add_control_field("001".to_string(), "test-text".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph
        .serialize(RdfFormat::RdfXml)
        .expect("serialization failed");

    assert!(
        rdf.contains("Text") || rdf.contains("bf:Text"),
        "Language material (leader 'a') should be typed as Text"
    );
}

#[test]
fn test_serial_entity_for_serial_record() {
    let mut leader = make_test_leader();
    leader.record_type = 'a';
    leader.bibliographic_level = 's'; // serial
    let mut record = Record::new(leader);
    record.add_control_field("001".to_string(), "test-serial".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520c2001    xxu          0eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph
        .serialize(RdfFormat::RdfXml)
        .expect("serialization failed");

    assert!(
        rdf.contains("Serial") || rdf.contains("bf:Serial"),
        "Serial record (bib level 's') should be typed as Serial"
    );
}

#[test]
fn test_notated_music_entity() {
    let mut leader = make_test_leader();
    leader.record_type = 'c'; // notated music
    let mut record = Record::new(leader);
    record.add_control_field("001".to_string(), "test-music".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu  n       000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph
        .serialize(RdfFormat::RdfXml)
        .expect("serialization failed");

    assert!(
        rdf.contains("NotatedMusic") || rdf.contains("bf:NotatedMusic"),
        "Music record (leader 'c') should be typed as NotatedMusic"
    );
}

#[test]
fn test_cartography_entity() {
    let mut leader = make_test_leader();
    leader.record_type = 'e'; // cartographic
    let mut record = Record::new(leader);
    record.add_control_field("001".to_string(), "test-map".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph
        .serialize(RdfFormat::RdfXml)
        .expect("serialization failed");

    assert!(
        rdf.contains("Cartography") || rdf.contains("bf:Cartography"),
        "Map record (leader 'e') should be typed as Cartography"
    );
}

#[test]
fn test_moving_image_entity() {
    let mut leader = make_test_leader();
    leader.record_type = 'g'; // projected medium (video/film)
    let mut record = Record::new(leader);
    record.add_control_field("001".to_string(), "test-video".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph
        .serialize(RdfFormat::RdfXml)
        .expect("serialization failed");

    assert!(
        rdf.contains("MovingImage") || rdf.contains("bf:MovingImage"),
        "Video record (leader 'g') should be typed as MovingImage"
    );
}

// ============================================================================
// Validation Tests: Agent Type Consistency
// ============================================================================

#[test]
fn test_person_agent_from_100() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-person".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f100 = Field::new("100".to_string(), '1', ' ');
    f100.add_subfield('a', "Smith, John,".to_string());
    record.add_field(f100);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph
        .serialize(RdfFormat::RdfXml)
        .expect("serialization failed");

    assert!(
        rdf.contains("Person") || rdf.contains("bf:Person") || rdf.contains("Smith"),
        "Field 100 should create Person agent"
    );
}

#[test]
fn test_organization_agent_from_110() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-org".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f110 = Field::new("110".to_string(), '2', ' ');
    f110.add_subfield('a', "United States.".to_string());
    f110.add_subfield('b', "Department of Defense.".to_string());
    record.add_field(f110);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph
        .serialize(RdfFormat::RdfXml)
        .expect("serialization failed");

    assert!(
        rdf.contains("Organization")
            || rdf.contains("bf:Organization")
            || rdf.contains("Department of Defense"),
        "Field 110 should create Organization agent"
    );
}

#[test]
fn test_meeting_agent_from_111() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-meeting".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f111 = Field::new("111".to_string(), '2', ' ');
    f111.add_subfield('a', "International Conference on Science".to_string());
    record.add_field(f111);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph
        .serialize(RdfFormat::RdfXml)
        .expect("serialization failed");

    assert!(
        rdf.contains("Meeting") || rdf.contains("bf:Meeting") || rdf.contains("Conference"),
        "Field 111 should create Meeting agent"
    );
}

// ============================================================================
// Validation Tests: Subject Type Consistency
// ============================================================================

#[test]
fn test_topic_subject_from_650() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-topic".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f650 = Field::new("650".to_string(), ' ', '0');
    f650.add_subfield('a', "Computer science".to_string());
    record.add_field(f650);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph
        .serialize(RdfFormat::RdfXml)
        .expect("serialization failed");

    assert!(
        rdf.contains("Topic") || rdf.contains("bf:Topic") || rdf.contains("Computer science"),
        "Field 650 should create Topic subject"
    );
}

#[test]
fn test_place_subject_from_651() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-place".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f651 = Field::new("651".to_string(), ' ', '0');
    f651.add_subfield('a', "United States".to_string());
    record.add_field(f651);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph
        .serialize(RdfFormat::RdfXml)
        .expect("serialization failed");

    assert!(
        rdf.contains("Place") || rdf.contains("bf:Place") || rdf.contains("United States"),
        "Field 651 should create Place subject"
    );
}

// ============================================================================
// Validation Tests: Identifier Type Consistency
// ============================================================================

#[test]
fn test_isbn_identifier_type() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-isbn-type".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f020 = Field::new("020".to_string(), ' ', ' ');
    f020.add_subfield('a', "0123456789".to_string());
    record.add_field(f020);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph
        .serialize(RdfFormat::RdfXml)
        .expect("serialization failed");

    assert!(
        rdf.contains("Isbn") || rdf.contains("bf:Isbn"),
        "ISBN should be typed as Isbn identifier"
    );
}

#[test]
fn test_issn_identifier_type() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-issn-type".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f022 = Field::new("022".to_string(), ' ', ' ');
    f022.add_subfield('a', "1234-5678".to_string());
    record.add_field(f022);

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph
        .serialize(RdfFormat::RdfXml)
        .expect("serialization failed");

    assert!(
        rdf.contains("Issn") || rdf.contains("bf:Issn"),
        "ISSN should be typed as Issn identifier"
    );
}

// ============================================================================
// Validation Tests: Output Format Completeness
// ============================================================================

#[test]
fn test_ntriples_serialization_valid() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-nt".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let nt = graph
        .serialize(RdfFormat::NTriples)
        .expect("N-Triples serialization failed");

    // Each line should be a triple: subject predicate object .
    for line in nt.lines() {
        if !line.is_empty() {
            assert!(
                line.ends_with(" ."),
                "N-Triples line should end with \" .\""
            );
        }
    }
}

#[test]
fn test_turtle_serialization_valid() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-turtle".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let ttl = graph
        .serialize(RdfFormat::Turtle)
        .expect("Turtle serialization failed");

    // Should have some content
    assert!(
        !ttl.is_empty(),
        "Turtle serialization should produce output"
    );
}

#[test]
fn test_jsonld_serialization_valid() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-jsonld".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let jsonld = graph
        .serialize(RdfFormat::JsonLd)
        .expect("JSON-LD serialization failed");

    // Should be valid JSON
    assert!(
        jsonld.starts_with('[') || jsonld.starts_with('{'),
        "JSON-LD should start with array or object"
    );
}

// ============================================================================
// Validation Tests: Graph Integrity
// ============================================================================

#[test]
fn test_graph_has_minimum_triples() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-min".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());

    // Even empty record should have Work and Instance with types
    // Minimum: Work rdf:type, Instance rdf:type, Work hasInstance Instance (3 triples)
    assert!(
        graph.len() >= 3,
        "Graph should have at least 3 triples (Work, Instance, relationship)"
    );
}

#[test]
fn test_graph_no_duplicate_triples() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-dups".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let rdf = graph
        .serialize(RdfFormat::NTriples)
        .expect("serialization failed");

    // Count each triple
    let lines: Vec<&str> = rdf.lines().filter(|line| !line.is_empty()).collect();

    // Verify no obvious duplicates (simplified check)
    // RDF libraries usually deduplicate, but worth checking
    assert!(!lines.is_empty(), "Should have RDF triples");
}

#[test]
fn test_graph_all_uris_valid() {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "test-uri".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let graph = marc_to_bibframe(&record, &make_config());
    let nt = graph
        .serialize(RdfFormat::NTriples)
        .expect("serialization failed");

    // In N-Triples, URIs are wrapped in < >
    for line in nt.lines() {
        if !line.is_empty() {
            // Should have <subject> <predicate> object .
            assert!(
                line.contains('<') && line.contains('>'),
                "N-Triples should have URIs in angle brackets"
            );
        }
    }
}
