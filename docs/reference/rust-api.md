# Rust API Reference

Rust API reference for MRRC. See [docs.rs/mrrc](https://docs.rs/mrrc) for full auto-generated documentation.

## Core Types

### Record

A MARC bibliographic record.

```rust
use mrrc::{Record, Field, Leader, RecordHelpers};

// Create with builder
let record = Record::builder(Leader::from_bytes(b"00000nam a2200000 i 4500")?)
    .control_field_str("001", "123456789")
    .field(
        Field::builder("245".to_string(), '1', '0')
            .subfield_str('a', "Title")
            .build()
    )
    .build();

// Access fields
if let Some(title) = record.title() {
    println!("{}", title);
}

for field in record.fields_by_tag("650") {
    if let Some(subject) = field.get_subfield('a') {
        println!("Subject: {}", subject);
    }
}
```

**Key members** (fields are accessed without parentheses; the `title`/`author`/`isbns` helpers require `use mrrc::RecordHelpers`):

| Member | Type / Returns | Description |
|--------|----------------|-------------|
| `leader` *(field)* | `Leader` | The record's leader |
| `get_control_field(tag)` | `Option<&str>` | Get a control field value |
| `get_field(tag)` | `Option<&Field>` | Get first field with tag |
| `fields_by_tag(tag)` | `impl Iterator<Item = &Field>` | Iterate fields with tag |
| `fields()` | `impl Iterator<Item = &Field>` | Iterate all data fields |
| `title()` | `Option<&str>` | Title from 245$a (`RecordHelpers`) |
| `author()` | `Option<&str>` | Author from 100/110/111 (`RecordHelpers`) |
| `isbns()` | `Vec<&str>` | All ISBNs (`RecordHelpers`) |

### RecordBuilder

Builder pattern for creating records.

```rust
use mrrc::Record;

let record = Record::builder(leader)
    .control_field_str("001", "12345")
    .control_field_str("008", "040520s2023    xxu")
    .field(title_field)
    .field(author_field)
    .build();
```

### Field

A MARC data field with tag, indicators, and subfields.

```rust
use mrrc::Field;

// Create with builder
let field = Field::builder("245".to_string(), '1', '0')
    .subfield_str('a', "Main title :")
    .subfield_str('b', "subtitle /")
    .subfield_str('c', "by Author.")
    .build();

// Create directly
let mut field = Field::new("650".to_string(), ' ', '0');
field.add_subfield('a', "Subject heading".to_string());

// Access data
println!("Tag: {}", field.tag);
println!("Ind1: {}", field.indicator1);
if let Some(value) = field.get_subfield('a') {
    println!("$a: {}", value);
}
```

**Key members** (`tag`, `indicator1`, and `indicator2` are public fields, accessed without parentheses):

| Member | Type / Returns | Description |
|--------|----------------|-------------|
| `tag` *(field)* | `String` | 3-character field tag |
| `indicator1` *(field)* | `char` | First indicator |
| `indicator2` *(field)* | `char` | Second indicator |
| `get_subfield(code)` | `Option<&str>` | Get first subfield value |
| `subfields()` | `impl Iterator<Item = &Subfield>` | Iterate subfields |
| `add_subfield(code, value)` | `()` | Add a subfield (`code: char`, `value: String`) |

### Subfield

A subfield within a field.

```rust
use mrrc::Subfield;

let sf = Subfield { code: 'a', value: "value".to_string() };
println!("Code: {}", sf.code);
println!("Value: {}", sf.value);
```

### Leader

The 24-byte MARC record header.

```rust
use mrrc::Leader;

let mut leader = Leader::from_bytes(b"00000nam a2200000 i 4500")?;
leader.record_type = 'a';           // Language material
leader.bibliographic_level = 'm';   // Monograph
leader.character_coding = 'a';      // UTF-8
```

**Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `record_length` | `u32` | Record length (positions 00-04) |
| `record_status` | `char` | Status: n/c/d (position 05) |
| `record_type` | `char` | Type: a=language, c=music, etc. (position 06) |
| `bibliographic_level` | `char` | Level: m=monograph, s=serial (position 07) |
| `character_coding` | `char` | Encoding: space=MARC-8, a=UTF-8 (position 09) |

## Reader/Writer

### MarcReader

Reads MARC records from any `Read` source.

```rust
use mrrc::MarcReader;
use std::fs::File;

let file = File::open("records.mrc")?;
let mut reader = MarcReader::new(file);

while let Some(record) = reader.read_record()? {
    println!("{:?}", record.title());
}
```

**Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `new(reader)` | `MarcReader<R>` | Create from any `Read` |
| `read_record()` | `Result<Option<Record>>` | Read next record |

### MarcWriter

Writes MARC records to any `Write` sink.

```rust
use mrrc::MarcWriter;
use std::fs::File;

let file = File::create("output.mrc")?;
let mut writer = MarcWriter::new(file);

writer.write_record(&record)?;
```

**Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `new(writer)` | `MarcWriter<W>` | Create from any `Write` |
| `write_record(record)` | `Result<()>` | Write a record |

## Specialized Record Types

### AuthorityRecord

MARC authority records for controlled vocabularies.

```rust
use mrrc::{AuthorityRecord, Field, Leader};

let record = AuthorityRecord::builder(Leader::from_bytes(b"00000nz  a2200000n  4500")?)
    .control_field("001".to_string(), "n12345678".to_string())
    .heading(
        Field::builder("100".to_string(), '1', ' ')
            .subfield_str('a', "Smith, John")
            .build(),
    )
    .build();
```

### HoldingsRecord

MARC holdings records for library holdings data.

```rust
use mrrc::{HoldingsRecord, Field, Leader};

let record = HoldingsRecord::builder(Leader::from_bytes(b"00000nx  a2200000   4500")?)
    .control_field("001".to_string(), "h12345".to_string())
    .location(
        Field::builder("852".to_string(), ' ', ' ')
            .subfield_str('a', "Main Library")
            .build(),
    )
    .build();
```

## Query DSL

Query records using a fluent API.

```rust
use mrrc::{FieldQuery, SubfieldValueQuery};

// Find fields by tag
let query = FieldQuery::new().tag("245");
for field in record.fields_matching(&query) {
    println!("{}", field.tag);
}

// Find by tag range
let query = FieldQuery::new().tag_range("600", "699");
let subject_fields = record.fields_matching_range(&query);

// Find by subfield value
let query = SubfieldValueQuery::new("100", 'a', "Smith");
let matches = record.fields_matching_value(&query);

// Find by subfield presence only
let query = FieldQuery::new().tag("100").has_subfield('a');
let with_name = record.fields_matching(&query);
```

## Format Conversion

### Core Formats (Always Available)

```rust
use mrrc::marcxml::{record_to_marcxml, marcxml_to_record, marcxml_to_records};

let json = mrrc::json::record_to_json(&record)?;
let marcjson = mrrc::marcjson::record_to_marcjson(&record)?;

// MARCXML conversion (bidirectional)
let xml = record_to_marcxml(&record)?;
let restored = marcxml_to_record(&xml)?;

// Parse a MARCXML collection (multiple records)
let records = marcxml_to_records(&collection_xml)?;

// MODS conversion (bidirectional)
let mods = mrrc::mods::record_to_mods_xml(&record)?;
let restored = mrrc::mods::mods_xml_to_record(&mods)?;
```

## BIBFRAME Conversion

Convert MARC to BIBFRAME 2.0 RDF:

```rust
use mrrc::bibframe::{marc_to_bibframe, BibframeConfig, RdfFormat};

let config = BibframeConfig {
    base_uri: Some("http://example.org/".to_string()),
    ..Default::default()
};

let graph = marc_to_bibframe(&record, &config);

// Serialize to various RDF formats
let turtle = graph.serialize(RdfFormat::Turtle)?;
let rdfxml = graph.serialize(RdfFormat::RdfXml)?;
let jsonld = graph.serialize(RdfFormat::JsonLd)?;
```

## Character Encoding

```rust
use mrrc::encoding::{MarcEncoding, decode_bytes, encode_string};

// Detect encoding from leader
let encoding = MarcEncoding::from_leader_char(leader.character_coding)?;

// Decode bytes
let text = decode_bytes(bytes, encoding)?;

// Encode string
let bytes = encode_string(&text, encoding)?;
```

## Error Handling

```rust
use mrrc::{Result, MarcError};

fn process(path: &str) -> Result<()> {
    let file = File::open(path)?;
    let mut reader = MarcReader::new(file);

    while let Some(record) = reader.read_record()? {
        // Process record
    }

    Ok(())
}
```

**Error Types:**

`MarcError` carries one variant per stable error code (leader, directory,
field, encoding, serialization, and I/O failures), each with structured
positional context. See the [error codes reference](error-codes.md) for the
complete per-variant documentation, and
[docs.rs](https://docs.rs/mrrc/latest/mrrc/enum.MarcError.html) for the enum
definition.

## Parallel Processing

Use Rayon for parallel processing:

```rust
use mrrc::boundary_scanner::RecordBoundaryScanner;
use mrrc::rayon_parser_pool::parse_batch_parallel;
use mrrc::RecordHelpers;
use rayon::prelude::*;

// Read the whole file into one buffer
let data = std::fs::read("records.mrc")?;

// Locate each record as an (offset, length) pair
let mut scanner = RecordBoundaryScanner::new();
let boundaries = scanner.scan(&data)?;

// Parse every record in parallel against the shared buffer
let records = parse_batch_parallel(&boundaries, &data)?;

// Process records in parallel
records.par_iter()
    .filter(|r| r.title().is_some())
    .for_each(|r| println!("{:?}", r.title()));
```

See the [Rust concurrency tutorial](../tutorials/rust/concurrency.md) for the
full worked example.

## See Also

- [Rust Quickstart](../getting-started/quickstart-rust.md)
- [Format Support](formats.md)
- [docs.rs/mrrc](https://docs.rs/mrrc) - Full API documentation
