//! BIBFRAME conversion example: Basic BIBFRAME → MARC conversion.
//!
//! This example demonstrates:
//! - Converting a BIBFRAME RDF graph back to MARC
//! - Handling conversion results
//! - Working with the result record

use mrrc::bibframe::{bibframe_to_marc, marc_to_bibframe, BibframeConfig, RdfFormat};
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

    let mut original_record = Record::new(leader);

    // Add control fields
    original_record.add_control_field("001".to_string(), "roundtrip-001".to_string());
    original_record.add_control_field(
        "008".to_string(),
        "040520s2023    xxu           000 0 eng  ".to_string(),
    );

    // Add ISBN
    let mut f020 = Field::new("020".to_string(), ' ', ' ');
    f020.add_subfield('a', "9780123456789".to_string());
    original_record.add_field(f020);

    // Add title
    let mut f245 = Field::new("245".to_string(), '1', '0');
    f245.add_subfield('a', "MARC Roundtrip Test /".to_string());
    f245.add_subfield('c', "by Test Author.".to_string());
    original_record.add_field(f245);

    // Add author
    let mut f100 = Field::new("100".to_string(), '1', ' ');
    f100.add_subfield('a', "Author, Test,".to_string());
    f100.add_subfield('4', "aut".to_string());
    original_record.add_field(f100);

    println!("=== Original MARC Record ===");
    println!(
        "Title fields: {}",
        original_record.fields_by_tag("245").count()
    );
    println!(
        "Creator fields: {}",
        original_record.fields_by_tag("100").count()
    );
    println!(
        "Identifier fields: {}",
        original_record.fields_by_tag("020").count()
    );

    // Step 1: Convert MARC → BIBFRAME
    let config = BibframeConfig::default();
    let graph = marc_to_bibframe(&original_record, &config);
    println!("\n✓ Converted to BIBFRAME ({} triples)", graph.len());

    // Step 2: Serialize BIBFRAME to RDF/XML
    let rdf_xml = graph.serialize(RdfFormat::RdfXml)?;
    println!("✓ Serialized to RDF/XML ({} bytes)", rdf_xml.len());

    // Step 3: Convert BIBFRAME → MARC
    let recovered_record = bibframe_to_marc(&graph)?;
    println!("✓ Converted back to MARC");

    // Verify round-trip fidelity
    println!("\n=== Round-Trip Results ===");
    println!(
        "Title fields preserved: {}",
        recovered_record.fields_by_tag("245").count()
    );
    println!(
        "Creator fields preserved: {}",
        recovered_record.fields_by_tag("100").count()
    );
    println!(
        "Identifier fields preserved: {}",
        recovered_record.fields_by_tag("020").count()
    );

    // Count all fields
    let original_field_count: usize = original_record.fields().count();
    let recovered_field_count: usize = recovered_record.fields().count();

    println!("\nTotal fields:");
    println!("  Original: {original_field_count}");
    println!("  Recovered: {recovered_field_count}");

    // Show sample recovered field
    if let Some(title_field) = recovered_record.get_field("245") {
        println!("\nSample recovered 245 field:");
        println!("  Tag: {}", title_field.tag);
        println!("  Ind1: {}", title_field.indicator1);
        println!("  Ind2: {}", title_field.indicator2);
        for subfield in &title_field.subfields {
            println!("  ${}: {}", subfield.code, subfield.value);
        }
    }

    println!("\n✓ Round-trip conversion complete!");
    println!("\nNote: Some data may be lost in round-trip conversion:");
    println!("  - Non-filing indicators (245 ind2) are reconstructed");
    println!("  - Authority record links ($0) are optional");
    println!("  - Detailed 008 codes may be approximated");

    Ok(())
}
