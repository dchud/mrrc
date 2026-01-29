# Reading Records (Rust)

Learn to read MARC records from files and work with their contents.

## Basic Reading

```rust
use mrrc::MarcReader;
use std::fs::File;

fn main() -> mrrc::Result<()> {
    let file = File::open("records.mrc")?;
    let mut reader = MarcReader::new(file);

    while let Some(record) = reader.read_record()? {
        if let Some(title) = record.title() {
            println!("Title: {}", title);
        }
    }
    Ok(())
}
```

## Reading from Memory

```rust
use mrrc::MarcReader;
use std::io::Cursor;

fn main() -> mrrc::Result<()> {
    let data = std::fs::read("records.mrc")?;
    let cursor = Cursor::new(data);
    let mut reader = MarcReader::new(cursor);

    while let Some(record) = reader.read_record()? {
        println!("{:?}", record.title());
    }
    Ok(())
}
```

## Accessing Fields

```rust
use mrrc::Record;

fn process_record(record: &Record) {
    // Get first field by tag
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

    // Iterate over all fields
    for field in record.fields() {
        println!("Field {}: {} subfields", field.tag(), field.subfields().len());
    }
}
```

## Control Fields

Control fields (001-009) contain unstructured data:

```rust
fn read_control_fields(record: &Record) {
    if let Some(control_num) = record.control_field("001") {
        println!("Control number: {}", control_num);
    }

    if let Some(fixed) = record.control_field("008") {
        // Parse fixed-length data elements
        let pub_year = &fixed[7..11];
        let language = &fixed[35..38];
        println!("Published: {}, Language: {}", pub_year, language);
    }
}
```

## Convenience Methods

```rust
fn extract_metadata(record: &Record) {
    if let Some(title) = record.title() {
        println!("Title: {}", title);
    }

    if let Some(author) = record.author() {
        println!("Author: {}", author);
    }

    // Get all ISBNs
    for isbn in record.isbns() {
        println!("ISBN: {}", isbn);
    }

    // Get all subjects
    for subject in record.subjects() {
        println!("Subject: {}", subject);
    }
}
```

## Working with Subfields

```rust
use mrrc::Field;

fn process_field(field: &Field) {
    // Get first subfield value
    if let Some(value) = field.subfield("a") {
        println!("$a: {}", value);
    }

    // Get all values for a subfield code
    let values: Vec<&str> = field.subfields()
        .iter()
        .filter(|sf| sf.code() == 'a')
        .map(|sf| sf.value())
        .collect();

    // Iterate over all subfields
    for subfield in field.subfields() {
        println!("${}: {}", subfield.code(), subfield.value());
    }
}
```

## Working with Indicators

```rust
fn check_indicators(field: &Field) {
    let ind1 = field.indicator1();
    let ind2 = field.indicator2();

    // For 245: ind2 = nonfiling characters
    if field.tag() == "245" {
        let skip = ind2.to_digit(10).unwrap_or(0) as usize;
        if let Some(title) = field.subfield("a") {
            let filing_title = &title[skip..];
            println!("Filing title: {}", filing_title);
        }
    }
}
```

## Error Handling

```rust
use mrrc::{MarcReader, MarcError, Result};

fn process_file(path: &str) -> Result<usize> {
    let file = std::fs::File::open(path)?;
    let mut reader = MarcReader::new(file);
    let mut count = 0;

    while let Some(record) = reader.read_record()? {
        count += 1;
    }

    Ok(count)
}

fn main() {
    match process_file("records.mrc") {
        Ok(count) => println!("Processed {} records", count),
        Err(MarcError::IoError(e)) => eprintln!("I/O error: {}", e),
        Err(MarcError::InvalidRecord(msg)) => eprintln!("Invalid record: {}", msg),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Complete Example

```rust
use mrrc::MarcReader;
use std::fs::File;

fn main() -> mrrc::Result<()> {
    let file = File::open("library.mrc")?;
    let mut reader = MarcReader::new(file);

    let mut books = 0;
    let mut serials = 0;

    while let Some(record) = reader.read_record()? {
        let leader = record.leader();

        match leader.bibliographic_level {
            'm' => books += 1,
            's' => serials += 1,
            _ => {}
        }

        if let Some(title) = record.title() {
            println!("{}", title);
        }
    }

    println!("\nSummary: {} books, {} serials", books, serials);
    Ok(())
}
```

## Next Steps

- [Writing Records](writing-records.md) - Create and modify records
- [Querying Fields](querying-fields.md) - Advanced field searching
- [Rust API Reference](../../reference/rust-api.md) - Full API documentation
