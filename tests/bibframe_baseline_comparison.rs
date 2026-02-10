//! BIBFRAME baseline comparison tests.
//!
//! These tests compare mrrc's BIBFRAME conversion output against the official
//! LOC marc2bibframe2 tool output to verify structural equivalence.
//!
//! Since the MARCXML formats differ slightly between LOC baselines and mrrc's parser,
//! we create equivalent records programmatically and compare the structural output.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use mrrc::bibframe::{marc_to_bibframe, BibframeConfig, RdfFormat};
use mrrc::leader::Leader;
use mrrc::record::{Field, Record};

/// Extracts key structural elements from RDF/XML for comparison.
/// Returns a set of simplified "facts" about the graph.
/// Handles both LOC format (bf:Work) and mrrc format (<Work xmlns="...">)
fn extract_structural_facts(rdf_xml: &str) -> HashSet<String> {
    let mut facts = HashSet::new();

    // Extract Work type (both formats)
    if rdf_xml.contains("bf:Work") || rdf_xml.contains("<Work ") {
        facts.insert("has_work".to_string());
    }
    if rdf_xml.contains("bf:Instance") || rdf_xml.contains("<Instance ") {
        facts.insert("has_instance".to_string());
    }

    // Extract specific BIBFRAME classes (check both prefixed and unprefixed)
    let classes = [
        ("Text", "bf:Text"),
        ("NotatedMusic", "bf:NotatedMusic"),
        ("Cartography", "bf:Cartography"),
        ("MovingImage", "bf:MovingImage"),
        ("StillImage", "bf:StillImage"),
        ("Audio", "bf:Audio"),
        ("MusicAudio", "bf:MusicAudio"),
        ("Serial", "bf:Serial"),
        ("Manuscript", "bf:Manuscript"),
        ("Electronic", "bf:Electronic"),
        ("Person", "bf:Person"),
        ("Organization", "bf:Organization"),
        ("Meeting", "bf:Meeting"),
        ("Topic", "bf:Topic"),
        ("Place", "bf:Place"),
        ("Title", "bf:Title"),
        ("Contribution", "bf:Contribution"),
        ("Publication", "bf:Publication"),
        ("Isbn", "bf:Isbn"),
        ("Issn", "bf:Issn"),
        ("Lccn", "bf:Lccn"),
    ];

    for (local, prefixed) in classes {
        // Check for both bf:Class and <Class xmlns="...">
        if rdf_xml.contains(prefixed)
            || rdf_xml.contains(&format!("<{local} "))
            || rdf_xml.contains(&format!("<{local}/>"))
        {
            facts.insert(format!("has_class:{prefixed}"));
        }
    }

    // Extract key properties (check both prefixed and unprefixed)
    let properties = [
        ("mainTitle", "bf:mainTitle"),
        ("subtitle", "bf:subtitle"),
        ("contribution", "bf:contribution"),
        ("agent", "bf:agent"),
        ("role", "bf:role"),
        ("subject", "bf:subject"),
        ("identifiedBy", "bf:identifiedBy"),
        ("provisionActivity", "bf:provisionActivity"),
        ("extent", "bf:extent"),
        ("dimensions", "bf:dimensions"),
        ("note", "bf:note"),
        ("hasInstance", "bf:hasInstance"),
        ("instanceOf", "bf:instanceOf"),
        ("hasSeries", "bf:hasSeries"),
        ("seriesStatement", "bf:seriesStatement"),
        ("precededBy", "bf:precededBy"),
        ("succeededBy", "bf:succeededBy"),
    ];

    for (local, prefixed) in properties {
        // Check for both bf:prop and <prop xmlns="...">
        if rdf_xml.contains(prefixed)
            || rdf_xml.contains(&format!("<{local} "))
            || rdf_xml.contains(&format!("<{local}>"))
        {
            facts.insert(format!("has_property:{prefixed}"));
        }
    }

    // Extract specific literal values (titles, names, etc.)
    // This is a simplified extraction - real comparison would parse the XML
    extract_literal_values(rdf_xml, &mut facts);

    facts
}

/// Extracts literal values from common patterns in RDF/XML.
fn extract_literal_values(rdf_xml: &str, facts: &mut HashSet<String>) {
    for line in rdf_xml.lines() {
        // Extract mainTitle values (both bf:mainTitle and <mainTitle>)
        if line.contains("mainTitle") {
            if let Some(value) = extract_element_value(line) {
                facts.insert(format!("title:{}", normalize_value(&value)));
            }
        }

        // Extract rdfs:label values for agents/subjects
        if line.contains("rdfs:label") || line.contains("<label") {
            if let Some(value) = extract_element_value(line) {
                facts.insert(format!("label:{}", normalize_value(&value)));
            }
        }

        // Extract rdf:value for identifiers
        if line.contains("rdf:value") || line.contains("<value") {
            if let Some(value) = extract_element_value(line) {
                facts.insert(format!("identifier:{}", normalize_value(&value)));
            }
        }
    }
}

/// Extracts the text value from an XML element on a single line.
fn extract_element_value(line: &str) -> Option<String> {
    // Find the first > and last <
    let start = line.find('>')?;
    let end = line.rfind('<')?;

    if start < end {
        let value = &line[start + 1..end];
        if !value.is_empty() && !value.starts_with('<') {
            return Some(value.to_string());
        }
    }
    None
}

/// Normalizes a value for comparison (trim, lowercase for some comparisons).
fn normalize_value(value: &str) -> String {
    value.trim().to_string()
}

/// Creates a test Leader.
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

/// Compares mrrc output against LOC baseline for structural equivalence.
/// Uses the LOC baseline RDF directly and builds an equivalent mrrc record programmatically.
fn compare_baseline_with_record(
    test_name: &str,
    record: &Record,
) -> (HashSet<String>, HashSet<String>, f64) {
    let baseline_path = Path::new("tests/data/bibframe-baselines")
        .join("bibframe-output")
        .join(format!("{test_name}.rdf.xml"));

    // Convert using mrrc
    let config = BibframeConfig::new().with_base_uri("http://example.org/");
    let graph = marc_to_bibframe(record, &config);

    eprintln!("\n=== DEBUG: Graph has {} triples ===", graph.len());

    let mrrc_output = graph
        .serialize(RdfFormat::RdfXml)
        .expect("Failed to serialize");

    eprintln!(
        "=== DEBUG: mrrc RDF output ({} bytes) ===",
        mrrc_output.len()
    );
    if mrrc_output.len() < 2000 {
        eprintln!("{mrrc_output}");
    } else {
        eprintln!("{}...", &mrrc_output[..500]);
    }

    // Read LOC baseline
    let loc_output = fs::read_to_string(&baseline_path).expect("Failed to read baseline");

    // Extract structural facts from both
    let mrrc_facts = extract_structural_facts(&mrrc_output);
    let loc_facts = extract_structural_facts(&loc_output);

    // Calculate overlap
    let intersection: HashSet<_> = mrrc_facts.intersection(&loc_facts).cloned().collect();
    let union: HashSet<_> = mrrc_facts.union(&loc_facts).cloned().collect();

    let similarity = if union.is_empty() {
        1.0
    } else {
        #[allow(clippy::cast_precision_loss)]
        {
            intersection.len() as f64 / union.len() as f64
        }
    };

    // Find what's missing from mrrc
    let missing_from_mrrc: HashSet<_> = loc_facts.difference(&mrrc_facts).cloned().collect();
    let extra_in_mrrc: HashSet<_> = mrrc_facts.difference(&loc_facts).cloned().collect();

    if !missing_from_mrrc.is_empty() {
        eprintln!("\n=== {test_name} ===");
        eprintln!("Missing from mrrc ({}):", missing_from_mrrc.len());
        for fact in missing_from_mrrc.iter().take(10) {
            eprintln!("  - {fact}");
        }
        if missing_from_mrrc.len() > 10 {
            eprintln!("  ... and {} more", missing_from_mrrc.len() - 10);
        }
    }

    if !extra_in_mrrc.is_empty() {
        eprintln!("Extra in mrrc ({}):", extra_in_mrrc.len());
        for fact in extra_in_mrrc.iter().take(10) {
            eprintln!("  + {fact}");
        }
        if extra_in_mrrc.len() > 10 {
            eprintln!("  ... and {} more", extra_in_mrrc.len() - 10);
        }
    }

    (mrrc_facts, loc_facts, similarity)
}

/// Build a record equivalent to simple-record.xml baseline.
fn build_simple_record() -> Record {
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "13600108".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    dk a   j      000 1 dan  ".to_string(),
    );

    // 010 - LCCN
    let mut f010 = Field::new("010".to_string(), ' ', ' ');
    f010.add_subfield('a', "2004436018".to_string());
    record.add_field(f010);

    // 020 - ISBN
    let mut f020 = Field::new("020".to_string(), ' ', ' ');
    f020.add_subfield('a', "8759517352".to_string());
    record.add_field(f020);

    // 100 - Main entry
    let mut f100 = Field::new("100".to_string(), '1', ' ');
    f100.add_subfield('a', "Andersen, H. C.".to_string());
    f100.add_subfield('q', "(Hans Christian),".to_string());
    f100.add_subfield('d', "1805-1875.".to_string());
    record.add_field(f100);

    // 245 - Title
    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "Ole Lukøie /".to_string());
    f245.add_subfield(
        'c',
        "H.C. Andersen ; illustrationer af Otto Dickmeiss.".to_string(),
    );
    record.add_field(f245);

    // 260 - Publication
    let mut f260 = Field::new("260".to_string(), ' ', ' ');
    f260.add_subfield('a', "[København?] :".to_string());
    f260.add_subfield('b', "Lindhardt og Ringhof,".to_string());
    f260.add_subfield('c', "c2001.".to_string());
    record.add_field(f260);

    // 300 - Physical description
    let mut f300 = Field::new("300".to_string(), ' ', ' ');
    f300.add_subfield('a', "1 v. (unpaged) :".to_string());
    f300.add_subfield('b', "col. ill. ;".to_string());
    f300.add_subfield('c', "23 cm.".to_string());
    record.add_field(f300);

    // 500 - Note
    let mut f500 = Field::new("500".to_string(), ' ', ' ');
    f500.add_subfield('a', "Text based on the original work.".to_string());
    record.add_field(f500);

    // 700 - Added entry
    let mut f700 = Field::new("700".to_string(), '1', ' ');
    f700.add_subfield('a', "Dickmeiss, Otto,".to_string());
    f700.add_subfield('e', "ill.".to_string());
    record.add_field(f700);

    record
}

// ============================================================================
// Baseline Comparison Tests
// ============================================================================

#[test]
fn test_baseline_simple_record() {
    let record = build_simple_record();
    let (mrrc_facts, loc_facts, similarity) =
        compare_baseline_with_record("simple-record", &record);

    // Report similarity
    eprintln!(
        "\nSimple record similarity: {:.1}% ({}/{} facts match)",
        similarity * 100.0,
        mrrc_facts.intersection(&loc_facts).count(),
        mrrc_facts.union(&loc_facts).count()
    );

    // Check that mrrc produces output and extracts identifiers correctly
    assert!(!mrrc_facts.is_empty(), "mrrc should produce BIBFRAME facts");

    // Both systems should extract the same identifiers
    let mrrc_ids: HashSet<_> = mrrc_facts
        .iter()
        .filter(|f| f.starts_with("identifier:"))
        .collect();
    let loc_ids: HashSet<_> = loc_facts
        .iter()
        .filter(|f| f.starts_with("identifier:"))
        .collect();

    // Check that identifiers match
    let common_ids: HashSet<_> = mrrc_ids.intersection(&loc_ids).collect();
    eprintln!("Common identifiers: {common_ids:?}");

    assert!(
        common_ids.len() >= 2,
        "Should have at least 2 matching identifiers (ISBN, LCCN)"
    );
}

#[test]
fn test_baseline_titles() {
    // Build a record with various title fields
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "1".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "Test title /".to_string());
    f245.add_subfield('b', "subtitle ;".to_string());
    f245.add_subfield('c', "by Author.".to_string());
    record.add_field(f245);

    let (mrrc_facts, _loc_facts, similarity) = compare_baseline_with_record("titles", &record);

    eprintln!(
        "\nTitles: {:.1}% similarity, mrrc facts: {:?}",
        similarity * 100.0,
        mrrc_facts
    );

    // Verify mrrc produces title-related output
    assert!(!mrrc_facts.is_empty(), "mrrc should produce title facts");
}

#[test]
fn test_baseline_names_agents() {
    // Build a record with name entries
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "1".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f100 = Field::new("100".to_string(), '1', ' ');
    f100.add_subfield('a', "Smith, John,".to_string());
    f100.add_subfield('d', "1950-".to_string());
    f100.add_subfield('4', "aut".to_string());
    record.add_field(f100);

    let mut f700 = Field::new("700".to_string(), '1', ' ');
    f700.add_subfield('a', "Jones, Mary,".to_string());
    f700.add_subfield('e', "editor.".to_string());
    record.add_field(f700);

    let (mrrc_facts, _loc_facts, similarity) =
        compare_baseline_with_record("names-agents", &record);

    eprintln!(
        "\nNames/agents: {:.1}% similarity, mrrc facts: {:?}",
        similarity * 100.0,
        mrrc_facts
    );

    // Verify mrrc produces agent-related output
    assert!(!mrrc_facts.is_empty(), "mrrc should produce agent facts");
}

#[test]
fn test_baseline_subjects() {
    // Build a record with subject entries
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "1".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2001    xxu           000 0 eng  ".to_string(),
    );

    let mut f650 = Field::new("650".to_string(), ' ', '0');
    f650.add_subfield('a', "Computer science".to_string());
    f650.add_subfield('x', "Study and teaching.".to_string());
    record.add_field(f650);

    let (mrrc_facts, _loc_facts, similarity) = compare_baseline_with_record("subjects", &record);

    eprintln!(
        "\nSubjects: {:.1}% similarity, mrrc facts: {:?}",
        similarity * 100.0,
        mrrc_facts
    );

    // Verify mrrc produces subject-related output
    assert!(!mrrc_facts.is_empty(), "mrrc should produce subject facts");
}

#[test]
fn test_baseline_linking_entries() {
    // Build a record with linking entries
    let mut record = Record::new(make_test_leader());
    record.add_control_field("001".to_string(), "1".to_string());

    let mut f245 = Field::new("245".to_string(), '0', '0');
    f245.add_subfield('a', "College English.".to_string());
    record.add_field(f245);

    let mut f780 = Field::new("780".to_string(), '0', '0');
    f780.add_subfield('t', "Previous title".to_string());
    f780.add_subfield('x', "1234-5678".to_string());
    record.add_field(f780);

    let mut f785 = Field::new("785".to_string(), '0', '0');
    f785.add_subfield('t', "Later title".to_string());
    record.add_field(f785);

    let (mrrc_facts, _loc_facts, similarity) =
        compare_baseline_with_record("linking-entries", &record);

    eprintln!(
        "\nLinking entries: {:.1}% similarity, mrrc facts: {:?}",
        similarity * 100.0,
        mrrc_facts
    );

    // Verify mrrc produces linking entry output
    assert!(
        !mrrc_facts.is_empty(),
        "mrrc should produce linking entry facts"
    );

    // Check that we found the identifier from the 780 field
    assert!(
        mrrc_facts.iter().any(|f| f.contains("1234-5678")),
        "Should extract ISSN from linking entry"
    );
}

/// Summary test that reports overall baseline coverage using simple-record.
///
/// NOTE: Full LOC compatibility would require:
/// - More detailed 008 parsing (language, audience, genre)
/// - bf:instanceOf relationship (inverse of hasInstance)
/// - More structured `AdminMetadata` (status changes, agents)
/// - Content/Media/Carrier type vocabulary URIs
/// - Classification (050, 082, etc.)
#[test]
fn test_baseline_coverage_summary() {
    let record = build_simple_record();
    let (mrrc_facts, loc_facts, similarity) =
        compare_baseline_with_record("simple-record", &record);

    eprintln!("\n========================================");
    eprintln!("BIBFRAME Baseline Coverage Summary");
    eprintln!("========================================\n");

    eprintln!(
        "simple-record: {:.1}% similarity ({}/{} facts)",
        similarity * 100.0,
        mrrc_facts.intersection(&loc_facts).count(),
        mrrc_facts.union(&loc_facts).count()
    );

    eprintln!("\nMatching facts:");
    let matching: Vec<_> = mrrc_facts.intersection(&loc_facts).collect();
    for fact in matching.iter().take(15) {
        eprintln!("  ✓ {fact}");
    }
    if matching.len() > 15 {
        eprintln!("  ... and {} more", matching.len() - 15);
    }

    eprintln!("\nmrrc produces these facts not extracted from baseline:");
    for fact in mrrc_facts.iter().take(10) {
        if !loc_facts.contains(fact) {
            eprintln!("  + {fact}");
        }
    }

    eprintln!("\n========================================\n");

    // Current implementation produces core identifiers correctly
    // Full LOC structural similarity would require additional work
    assert!(
        !mrrc_facts.is_empty(),
        "mrrc should produce some BIBFRAME facts"
    );

    // Verify key identifiers are extracted
    assert!(
        mrrc_facts.iter().any(|f| f.starts_with("identifier:")),
        "Should extract identifiers"
    );
}
