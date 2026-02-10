//! BIBFRAME conversion example: Basic MARC → BIBFRAME conversion.
//!
//! This example demonstrates:
//! - Converting a MARC record to BIBFRAME
//! - Accessing the RDF graph
//! - Serializing to different RDF formats

use mrrc::bibframe::{marc_to_bibframe, BibframeConfig, RdfFormat};
use mrrc::leader::Leader;
use mrrc::record::{Field, Record};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a sample MARC record
    let leader = Leader {
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
    };

    let mut record = Record::new(leader);

    // Add control fields
    record.add_control_field("001".to_string(), "example-001".to_string());
    record.add_control_field(
        "008".to_string(),
        "040520s2023    xxu           000 0 eng  ".to_string(),
    );

    // Add ISBN
    let mut f020 = Field::new("020".to_string(), ' ', ' ');
    f020.add_subfield('a', "9780123456789".to_string());
    record.add_field(f020);

    // Add title
    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "Introduction to MARC /".to_string());
    f245.add_subfield('c', "by Jane Smith.".to_string());
    record.add_field(f245);

    // Add author
    let mut f100 = Field::new("100".to_string(), '1', ' ');
    f100.add_subfield('a', "Smith, Jane,".to_string());
    f100.add_subfield('d', "1970-".to_string());
    f100.add_subfield('4', "aut".to_string());
    record.add_field(f100);

    // Add publication info
    let mut f260 = Field::new("260".to_string(), ' ', ' ');
    f260.add_subfield('a', "New York :".to_string());
    f260.add_subfield('b', "Academic Press,".to_string());
    f260.add_subfield('c', "2023.".to_string());
    record.add_field(f260);

    // Add subject
    let mut f650 = Field::new("650".to_string(), ' ', '0');
    f650.add_subfield('a', "MARC (Computer record format)".to_string());
    f650.add_subfield('x', "Cataloging.".to_string());
    record.add_field(f650);

    // Convert to BIBFRAME with default configuration
    let config = BibframeConfig::default();
    let graph = marc_to_bibframe(&record, &config);

    println!("✓ Converted MARC record to BIBFRAME graph");
    println!("  Graph contains {} triples\n", graph.len());

    // Serialize to different formats
    println!("=== RDF/XML Format ===");
    let rdf_xml = graph.serialize(RdfFormat::RdfXml)?;
    println!("{}\n", &rdf_xml[..std::cmp::min(500, rdf_xml.len())]);

    println!("=== N-Triples Format (first 3 triples) ===");
    let ntriples = graph.serialize(RdfFormat::NTriples)?;
    for (i, line) in ntriples.lines().enumerate() {
        if i >= 3 {
            break;
        }
        if !line.is_empty() {
            println!("{line}");
        }
    }

    println!("\n=== JSON-LD Format ===");
    let jsonld = graph.serialize(RdfFormat::JsonLd)?;
    println!("{}\n", &jsonld[..std::cmp::min(300, jsonld.len())]);

    println!("✓ BIBFRAME conversion complete!");

    Ok(())
}
