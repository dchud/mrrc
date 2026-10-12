//! Code for `docs/tutorials/rust/format-conversion.md`.

use mrrc::Record;

fn core_formats(record: Record, collection_xml: String) -> mrrc::Result<()> {
    // --8<-- [start:core-formats]
    use mrrc::marcxml::{marcxml_to_record, marcxml_to_records, record_to_marcxml};

    // Convert to JSON
    let json = mrrc::json::record_to_json(&record)?;
    let restored = mrrc::json::json_to_record(&json)?;

    // Convert to MARCXML
    let xml = record_to_marcxml(&record)?;
    let restored = marcxml_to_record(&xml)?;

    // Parse a MARCXML collection (multiple records)
    let records = marcxml_to_records(&collection_xml)?;

    // Convert to MARCJSON (LOC standard)
    let marcjson = mrrc::marcjson::record_to_marcjson(&record)?;

    // MODS conversion (bidirectional)
    let mods_xml = mrrc::mods::record_to_mods_xml(&record)?;
    let restored = mrrc::mods::mods_xml_to_record(&mods_xml)?;
    // --8<-- [end:core-formats]
    Ok(())
}

fn bibframe_conversion(record: Record) -> mrrc::Result<()> {
    // --8<-- [start:bibframe-conversion]
    use mrrc::bibframe::{BibframeConfig, RdfFormat, marc_to_bibframe};

    let config = BibframeConfig {
        base_uri: Some("http://example.org/".to_string()),
        ..Default::default()
    };

    let graph = marc_to_bibframe(&record, &config);

    // Serialize to different RDF formats
    let turtle = graph.serialize(RdfFormat::Turtle)?;
    let rdfxml = graph.serialize(RdfFormat::RdfXml)?;
    let jsonld = graph.serialize(RdfFormat::JsonLd)?;
    // --8<-- [end:bibframe-conversion]
    Ok(())
}

mod batch_conversion {
    // --8<-- [start:batch-conversion]
    use mrrc::MarcReader;
    use std::fs::File;
    use std::io::{BufWriter, Write};

    fn convert_to_json(input: &str, output: &str) -> mrrc::Result<()> {
        let input_file = File::open(input)?;
        let output_file = File::create(output)?;
        let mut writer = BufWriter::new(output_file);

        let mut reader = MarcReader::new(input_file);

        while let Some(record) = reader.read_record()? {
            let json = mrrc::json::record_to_json(&record)?;
            writeln!(writer, "{json}")?;
        }

        Ok(())
    }
    // --8<-- [end:batch-conversion]
}

mod complete_example {
    // --8<-- [start:complete-example]
    use mrrc::MarcReader;
    use mrrc::marcxml::record_to_marcxml;
    use std::fs::File;
    use std::io::Write;

    fn convert_file(input: &str) -> mrrc::Result<()> {
        let file = File::open(input)?;
        let mut reader = MarcReader::new(file);

        let mut json_out = File::create("output.json")?;
        let mut xml_out = File::create("output.xml")?;

        let mut count = 0;

        while let Some(record) = reader.read_record()? {
            // JSON
            let json = mrrc::json::record_to_json(&record)?;
            writeln!(json_out, "{json}")?;

            // MARCXML
            let xml = record_to_marcxml(&record)?;
            writeln!(xml_out, "{xml}")?;

            count += 1;
        }

        println!("Converted {count} records");
        Ok(())
    }

    fn main() -> mrrc::Result<()> {
        convert_file("records.mrc")
    }
    // --8<-- [end:complete-example]
}

#[test]
fn fragments_run_against_fixtures() {
    let collection_xml =
        std::fs::read_to_string("tests/data/loc_collection.marcxml").expect("read fixture");
    for record in crate::fixtures::records() {
        core_formats(record.clone(), collection_xml.clone()).unwrap();
        bibframe_conversion(record).unwrap();
    }
}
