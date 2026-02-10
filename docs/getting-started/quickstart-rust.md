# Rust Quickstart

Get started with MRRC in Rust in 5 minutes.

## Add Dependency

```toml
[dependencies]
mrrc = "0.6"
```

## Read Records

```rust
use mrrc::MarcReader;
use std::fs::File;

fn main() -> mrrc::Result<()> {
    let file = File::open("records.mrc")?;
    let mut reader = MarcReader::new(file);

    while let Some(record) = reader.read_record()? {
        if let Some(title) = record.title() {
            println!("{}", title);
        }
    }

    Ok(())
}
```

## Access Fields

```rust
use mrrc::Record;

// Get a specific field by tag
if let Some(field) = record.field("245") {
    if let Some(title) = field.subfield("a") {
        println!("Title: {}", title);
    }
}

// Get all fields with a tag
for field in record.fields_by_tag("650") {
    if let Some(subject) = field.subfield("a") {
        println!("Subject: {}", subject);
    }
}

// Use convenience methods
println!("Title: {:?}", record.title());
println!("Author: {:?}", record.author());
println!("ISBNs: {:?}", record.isbns());
```

## Create Records

```rust
use mrrc::{Record, Field, Leader};

// Create a new record with builder pattern
let record = Record::builder()
    .leader(Leader::default())
    .control_field("001", "123456789")
    .field(
        Field::builder("245", '1', '0')
            .subfield('a', "My Book Title")
            .subfield('c', "by Author Name")
            .build()
    )
    .build();
```

## Write Records

```rust
use mrrc::MarcWriter;
use std::fs::File;

fn main() -> mrrc::Result<()> {
    let file = File::create("output.mrc")?;
    let mut writer = MarcWriter::new(file);

    writer.write_record(&record)?;

    Ok(())
}
```

## Convert Formats

```rust
use mrrc::formats::{to_json, to_xml, to_marcjson};

// To JSON
let json_str = to_json(&record)?;

// To XML
let xml_str = to_xml(&record)?;

// To MARCJSON (LOC standard)
let marcjson_str = to_marcjson(&record)?;
```

## Error Handling

MRRC uses a custom `Result` type:

```rust
use mrrc::{Result, MarcError};

fn process_file(path: &str) -> Result<()> {
    let file = File::open(path)?;
    let mut reader = MarcReader::new(file);

    while let Some(record) = reader.read_record()? {
        // Process record
    }

    Ok(())
}
```

## Next Steps

- [Reading Records Tutorial](../tutorials/rust/reading-records.md) - Detailed reading guide
- [Writing Records Tutorial](../tutorials/rust/writing-records.md) - Builder pattern and traits
- [Concurrency Tutorial](../tutorials/rust/concurrency.md) - Parallel processing with Rayon
- [Rust API Reference](../reference/rust-api.md) - Full API documentation
