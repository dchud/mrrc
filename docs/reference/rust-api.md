# Rust API Reference

Rust API reference for MRRC. See [docs.rs/mrrc](https://docs.rs/mrrc) for full auto-generated documentation.

## Core Types

### Record

A MARC bibliographic record.

```rust
use mrrc::{Record, Field, Leader};

// Create with builder
let record = Record::builder()
    .leader(Leader::default())
    .control_field("001", "123456789")
    .field(
        Field::builder("245", '1', '0')
            .subfield('a', "Title")
            .build()
    )
    .build();

// Access fields
if let Some(title) = record.title() {
    println!("{}", title);
}

for field in record.fields_by_tag("650") {
    if let Some(subject) = field.subfield("a") {
        println!("Subject: {}", subject);
    }
}
```

**Key Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `leader()` | `&Leader` | Get the record's leader |
| `control_field(tag)` | `Option<&str>` | Get control field value |
| `field(tag)` | `Option<&Field>` | Get first field with tag |
| `fields_by_tag(tag)` | `Vec<&Field>` | Get all fields with tag |
| `fields()` | `&[Field]` | Get all data fields |
| `title()` | `Option<String>` | Get title from 245$a |
| `author()` | `Option<String>` | Get author from 100/110/111 |
| `isbns()` | `Vec<String>` | Get all ISBNs |

### RecordBuilder

Builder pattern for creating records.

```rust
use mrrc::Record;

let record = Record::builder()
    .leader(leader)
    .control_field("001", "12345")
    .control_field("008", "040520s2023    xxu")
    .field(title_field)
    .field(author_field)
    .build();
```

### Field

A MARC data field with tag, indicators, and subfields.

```rust
use mrrc::Field;

// Create with builder
let field = Field::builder("245", '1', '0')
    .subfield('a', "Main title :")
    .subfield('b', "subtitle /")
    .subfield('c', "by Author.")
    .build();

// Create directly
let mut field = Field::new("650".to_string(), ' ', '0');
field.add_subfield('a', "Subject heading".to_string());

// Access data
println!("Tag: {}", field.tag());
println!("Ind1: {}", field.indicator1());
if let Some(value) = field.subfield("a") {
    println!("$a: {}", value);
}
```

**Key Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `tag()` | `&str` | 3-character field tag |
| `indicator1()` | `char` | First indicator |
| `indicator2()` | `char` | Second indicator |
| `subfield(code)` | `Option<&str>` | Get first subfield value |
| `subfields()` | `&[Subfield]` | Get all subfields |
| `add_subfield(code, value)` | `()` | Add a subfield |

### Subfield

A subfield within a field.

```rust
use mrrc::Subfield;

let sf = Subfield::new('a', "value".to_string());
println!("Code: {}", sf.code());
println!("Value: {}", sf.value());
```

### Leader

The 24-byte MARC record header.

```rust
use mrrc::Leader;

let mut leader = Leader::default();
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
use mrrc::{AuthorityRecord, AuthorityRecordBuilder};

let record = AuthorityRecordBuilder::new()
    .control_number("n12345678")
    .heading_type(HeadingType::PersonalName)
    .build()?;
```

### HoldingsRecord

MARC holdings records for library holdings data.

```rust
use mrrc::{HoldingsRecord, HoldingsRecordBuilder};

let record = HoldingsRecordBuilder::new()
    .control_number("h12345")
    .holdings_type(HoldingsType::SinglePart)
    .build()?;
```

## Query DSL

Query records using a fluent API.

```rust
use mrrc::FieldQuery;

// Find fields by tag
let query = FieldQuery::tag("245");
let fields = query.find_all(&record);

// Find by tag range
let query = FieldQuery::tag_range("600", "699");
let subject_fields = query.find_all(&record);

// Find by subfield pattern
let query = FieldQuery::tag("100")
    .with_subfield("a", "Smith");
let matches = query.find_all(&record);
```

## Format Conversion

### Core Formats (Always Available)

```rust
use mrrc::formats::{to_json, to_xml, to_marcjson};

let json = to_json(&record)?;
let xml = to_xml(&record)?;
let marcjson = to_marcjson(&record)?;
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

| Variant | Description |
|---------|-------------|
| `InvalidLeader` | Leader parsing error |
| `InvalidRecord` | Record structure error |
| `InvalidField` | Field parsing error |
| `IoError` | I/O error |
| `EncodingError` | Character encoding error |

## Parallel Processing

Use Rayon for parallel processing:

```rust
use mrrc::rayon_parser_pool::parse_batch_parallel;
use rayon::prelude::*;

// Parse multiple record byte slices in parallel
let records = parse_batch_parallel(&record_bytes)?;

// Process records in parallel
records.par_iter()
    .filter(|r| r.title().is_some())
    .for_each(|r| println!("{:?}", r.title()));
```

## See Also

- [Rust Quickstart](../getting-started/quickstart-rust.md)
- [Format Support](formats.md)
- [docs.rs/mrrc](https://docs.rs/mrrc) - Full API documentation
