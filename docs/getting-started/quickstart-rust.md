# Rust Quickstart

Get started with MRRC in Rust in 5 minutes.

## Add Dependency

```toml
[dependencies]
mrrc = "0.8"
```

## Read Records

```rust
use mrrc::{MarcReader, RecordHelpers};
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
use mrrc::{Record, RecordHelpers};

// Get a specific field by tag
if let Some(field) = record.get_field("245") {
    if let Some(title) = field.get_subfield('a') {
        println!("Title: {}", title);
    }
}

// Get all fields with a tag
for field in record.fields_by_tag("650") {
    if let Some(subject) = field.get_subfield('a') {
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
let record = Record::builder(Leader::from_bytes(b"00000nam a2200000 i 4500").unwrap())
    .control_field_str("001", "123456789")
    .field(
        Field::builder("245".to_string(), '1', '0')
            .subfield_str('a', "My Book Title")
            .subfield_str('c', "by Author Name")
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
// To JSON
let json = mrrc::json::record_to_json(&record)?;

// To MARCXML
let xml_str = mrrc::marcxml::record_to_marcxml(&record)?;

// To MARCJSON (LOC standard)
let marcjson = mrrc::marcjson::record_to_marcjson(&record)?;
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
