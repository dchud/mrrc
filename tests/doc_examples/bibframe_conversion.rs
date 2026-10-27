//! Code for `docs/guides/bibframe-conversion.md`.

use mrrc::bibframe::RdfGraph;

fn marc_to_bibframe() -> mrrc::Result<()> {
    // --8<-- [start:marc-to-bibframe]
    use mrrc::MarcReader;
    use mrrc::bibframe::{BibframeConfig, RdfFormat, marc_to_bibframe};
    use std::fs::File;

    let mut reader = MarcReader::new(File::open("records.mrc")?);
    while let Some(record) = reader.read_record()? {
        let config = BibframeConfig::default();
        let graph = marc_to_bibframe(&record, &config);

        // Serialize to Turtle format
        let turtle = graph.serialize(RdfFormat::Turtle)?;
        println!("{turtle}");
    }
    // --8<-- [end:marc-to-bibframe]
    Ok(())
}

fn bibframe_to_marc(turtle_string: String) -> mrrc::Result<()> {
    // --8<-- [start:bibframe-to-marc]
    use mrrc::bibframe::{RdfFormat, RdfGraph, bibframe_to_marc};

    let graph = RdfGraph::parse(&turtle_string, RdfFormat::Turtle)?;
    let record = bibframe_to_marc(&graph)?;
    // --8<-- [end:bibframe-to-marc]
    Ok(())
}

fn base_uri() {
    // --8<-- [start:base-uri]
    use mrrc::bibframe::BibframeConfig;

    let config = BibframeConfig {
        base_uri: Some("http://library.example.org/bibframe/".to_string()),
        ..Default::default()
    };
    // --8<-- [end:base-uri]
}

fn serialization_formats(graph: RdfGraph) -> mrrc::Result<()> {
    // --8<-- [start:serialization-formats]
    use mrrc::bibframe::RdfFormat;

    let rdf_xml = graph.serialize(RdfFormat::RdfXml)?;
    let turtle = graph.serialize(RdfFormat::Turtle)?;
    let jsonld = graph.serialize(RdfFormat::JsonLd)?;
    let ntriples = graph.serialize(RdfFormat::NTriples)?;
    // --8<-- [end:serialization-formats]
    Ok(())
}

fn batch_conversion() -> mrrc::Result<()> {
    // --8<-- [start:batch-conversion]
    use mrrc::MarcReader;
    use mrrc::bibframe::{BibframeConfig, RdfFormat, marc_to_bibframe};
    use std::fs::File;
    use std::io::Write;

    let config = BibframeConfig {
        base_uri: Some("http://library.example.org/".to_string()),
        ..Default::default()
    };

    let mut reader = MarcReader::new(File::open("large_file.mrc")?);
    let mut out = File::create("output.ttl")?;

    while let Some(record) = reader.read_record()? {
        let graph = marc_to_bibframe(&record, &config);
        writeln!(out, "{}", graph.serialize(RdfFormat::Turtle)?)?;
    }
    // --8<-- [end:batch-conversion]
    Ok(())
}

#[test]
fn fragments_run_against_fixtures() {
    use mrrc::bibframe::{BibframeConfig, RdfFormat, marc_to_bibframe};

    base_uri();
    for record in crate::fixtures::records() {
        let graph = marc_to_bibframe(&record, &BibframeConfig::default());
        let turtle = graph.serialize(RdfFormat::Turtle).unwrap();
        bibframe_to_marc(turtle).unwrap();
        serialization_formats(graph).unwrap();
    }
}
