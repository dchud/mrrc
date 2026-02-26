# Format Conversion (Rust)

Learn to convert MARC records between different formats.

## Core Formats

All formats are available without feature flags:

```rust
use mrrc::formats::{to_json, to_marcjson, from_json};
use mrrc::marcxml::{record_to_marcxml, marcxml_to_record, marcxml_to_records};

// Convert to JSON
let json = to_json(&record)?;
let restored = from_json(&json)?;

// Convert to MARCXML
let xml = record_to_marcxml(&record)?;
let restored = marcxml_to_record(&xml)?;

// Parse a MARCXML collection (multiple records)
let records = marcxml_to_records(&collection_xml)?;

// Convert to MARCJSON (LOC standard)
let marcjson = to_marcjson(&record)?;

// MODS conversion (bidirectional)
let mods_xml = mrrc::mods::record_to_mods_xml(&record)?;
let restored = mrrc::mods::mods_xml_to_record(&mods_xml)?;
```

## BIBFRAME Conversion

Convert MARC to BIBFRAME RDF:

```rust
use mrrc::bibframe::{marc_to_bibframe, BibframeConfig, RdfFormat};

let config = BibframeConfig {
    base_uri: Some("http://example.org/".to_string()),
    ..Default::default()
};

let graph = marc_to_bibframe(&record, &config);

// Serialize to different RDF formats
let turtle = graph.serialize(RdfFormat::Turtle)?;
let rdfxml = graph.serialize(RdfFormat::RdfXml)?;
let jsonld = graph.serialize(RdfFormat::JsonLd)?;
```

## Batch Conversion

```rust
use mrrc::{MarcReader, MarcWriter};
use mrrc::formats::to_json;
use std::fs::File;
use std::io::{BufWriter, Write};

fn convert_to_json(input: &str, output: &str) -> mrrc::Result<()> {
    let input_file = File::open(input)?;
    let output_file = File::create(output)?;
    let mut writer = BufWriter::new(output_file);

    let mut reader = MarcReader::new(input_file);

    while let Some(record) = reader.read_record()? {
        let json = to_json(&record)?;
        writeln!(writer, "{}", json)?;
    }

    Ok(())
}
```

## Format Selection

| Format | Best For |
|--------|----------|
| ISO 2709 | Library system interchange |
| JSON | Web APIs, debugging |
| MARCXML | MARCXML pipelines |
| CSV | Spreadsheet export |
| Dublin Core | Simple metadata exchange |
| MODS | Detailed metadata crosswalks |
| BIBFRAME | Linked data, RDF |

## Complete Example

```rust
use mrrc::{MarcReader, Record};
use mrrc::formats::to_json;
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
        let json = to_json(&record)?;
        writeln!(json_out, "{}", json)?;

        // MARCXML
        let xml = record_to_marcxml(&record)?;
        writeln!(xml_out, "{}", xml)?;

        count += 1;
    }

    println!("Converted {} records", count);
    Ok(())
}

fn main() -> mrrc::Result<()> {
    convert_file("records.mrc")
}
```

## Next Steps

- [Reading Records](reading-records.md) - Basic record access
- [BIBFRAME Conversion](../../guides/bibframe-conversion.md) - RDF conversion
- [Format Support](../../reference/formats.md) - Full format reference
