# Format Conversion (Rust)

Learn to convert MARC records between different formats.

## Core Formats

Core formats are always available without feature flags:

```rust
use mrrc::formats::{to_json, to_xml, to_marcjson, from_json, from_xml};

// Convert to JSON
let json = to_json(&record)?;
let restored = from_json(&json)?;

// Convert to XML
let xml = to_xml(&record)?;
let restored = from_xml(&xml)?;

// Convert to MARCJSON (LOC standard)
let marcjson = to_marcjson(&record)?;
```

## Feature-Gated Formats

Enable with Cargo features:

```toml
[dependencies]
mrrc = { version = "0.6", features = ["format-protobuf", "format-arrow"] }
```

### Protobuf

```rust
use mrrc::protobuf::{ProtobufWriter, ProtobufReader};
use std::fs::File;

// Write
let file = File::create("records.pb")?;
let mut writer = ProtobufWriter::new(file);
writer.write_record(&record)?;

// Read
let file = File::open("records.pb")?;
let mut reader = ProtobufReader::new(file);
while let Some(record) = reader.read_record()? {
    println!("{:?}", record.title());
}
```

### Arrow

```rust
use mrrc::arrow::{ArrowWriter, ArrowReader};
use std::fs::File;

// Write
let file = File::create("records.arrow")?;
let mut writer = ArrowWriter::new(file);
writer.write_record(&record)?;

// Read
let file = File::open("records.arrow")?;
let mut reader = ArrowReader::new(file);
while let Some(record) = reader.read_record()? {
    println!("{:?}", record.title());
}
```

### FlatBuffers

```rust
use mrrc::flatbuffers::{FlatbuffersWriter, FlatbuffersReader};
use std::fs::File;

// Write
let file = File::create("records.fb")?;
let mut writer = FlatbuffersWriter::new(file);
writer.write_record(&record)?;

// Read
let file = File::open("records.fb")?;
let mut reader = FlatbuffersReader::new(file);
while let Some(record) = reader.read_record()? {
    println!("{:?}", record.title());
}
```

### MessagePack

```rust
use mrrc::messagepack::{MessagePackWriter, MessagePackReader};
use std::fs::File;

// Write
let file = File::create("records.msgpack")?;
let mut writer = MessagePackWriter::new(file);
writer.write_record(&record)?;

// Read
let file = File::open("records.msgpack")?;
let mut reader = MessagePackReader::new(file);
while let Some(record) = reader.read_record()? {
    println!("{:?}", record.title());
}
```

## BIBFRAME Conversion

Convert MARC to BIBFRAME RDF (requires `format-bibframe` feature):

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

| Format | Feature Flag | Best For |
|--------|--------------|----------|
| ISO 2709 | (none) | Library system interchange |
| JSON | (none) | Web APIs, debugging |
| XML | (none) | MARCXML pipelines |
| Protobuf | `format-protobuf` | Microservices, cross-language |
| Arrow | `format-arrow` | Analytics (DuckDB, Polars) |
| FlatBuffers | `format-flatbuffers` | Zero-copy, memory-efficient |
| MessagePack | `format-messagepack` | Compact binary, 50+ languages |
| BIBFRAME | `format-bibframe` | Linked data, RDF |

## Complete Example

```rust
use mrrc::{MarcReader, Record};
use mrrc::formats::{to_json, to_xml};
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

        // XML
        let xml = to_xml(&record)?;
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
