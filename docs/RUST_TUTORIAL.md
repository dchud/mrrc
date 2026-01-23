# Rust Tutorial

This tutorial covers reading, writing, and converting MARC records in Rust using MRRC.

## Prerequisites

Add to your `Cargo.toml`:

```toml
[dependencies]
mrrc = "0.4"
```

## Reading MARC Records

### Basic Reading

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

### Read from Any Source

```rust
use mrrc::MarcReader;
use std::io::Cursor;

fn main() -> mrrc::Result<()> {
    // Read from bytes in memory
    let data = std::fs::read("records.mrc")?;
    let cursor = Cursor::new(data);
    let mut reader = MarcReader::new(cursor);

    while let Some(record) = reader.read_record()? {
        println!("{:?}", record.title());
    }
    Ok(())
}
```

## Working with Records

### Accessing Fields

```rust
use mrrc::Record;

fn process_record(record: &Record) {
    // Get first field by tag
    if let Some(field) = record.get_field("245") {
        // Get subfield value
        if let Some(title) = field.get_subfield('a') {
            println!("Title: {}", title);
        }
    }

    // Get all fields with a tag
    if let Some(fields) = record.get_fields("650") {
        for field in fields {
            if let Some(subject) = field.get_subfield('a') {
                println!("Subject: {}", subject);
            }
        }
    }

    // Iterate over all fields
    for field in record.fields() {
        println!("Field {}: {} subfields", field.tag, field.subfields.len());
    }
}
```

### Convenience Methods

```rust
use mrrc::Record;

fn extract_metadata(record: &Record) {
    // Built-in extraction methods
    if let Some(title) = record.title() {
        println!("Title: {}", title);
    }

    if let Some(author) = record.author() {
        println!("Author: {}", author);
    }

    if let Some(isbn) = record.isbn() {
        println!("ISBN: {}", isbn);
    }

    if let Some(year) = record.pubyear() {
        println!("Year: {}", year);
    }

    // Get all subjects
    for subject in record.subjects() {
        println!("Subject: {}", subject);
    }

    // Record type checks
    if record.is_book() {
        println!("This is a book");
    }
    if record.is_serial() {
        println!("This is a serial");
    }
}
```

### Control Fields

```rust
use mrrc::Record;

fn read_control_fields(record: &Record) {
    // Control fields (001-009)
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

### Working with Subfields

```rust
use mrrc::Field;

fn process_field(field: &Field) {
    // Get first subfield value
    if let Some(value) = field.get_subfield('a') {
        println!("Subfield a: {}", value);
    }

    // Get all values for a subfield code
    let values = field.get_subfield_values('a');
    for value in values {
        println!("Value: {}", value);
    }

    // Iterate over all subfields
    for subfield in &field.subfields {
        println!("${}: {}", subfield.code, subfield.value);
    }
}
```

### Working with Indicators

```rust
use mrrc::Field;

fn check_indicators(field: &Field) {
    // Access indicators
    println!("Indicator 1: {}", field.indicator1);
    println!("Indicator 2: {}", field.indicator2);

    // Common indicator checks
    if field.tag == "650" && field.indicator2 == '0' {
        println!("This is an LCSH subject heading");
    }
}
```

## Creating Records

### Basic Record Creation

```rust
use mrrc::{Record, Field, Leader};

fn create_record() -> mrrc::Result<Record> {
    let mut record = Record::new(Leader::default());

    // Add control fields
    record.add_control_field("001".to_string(), "12345".to_string());
    record.add_control_field(
        "008".to_string(),
        "200101s2020    xxu||||||||||||||||eng||".to_string()
    );

    // Create and add data fields
    let mut field_245 = Field::new("245".to_string(), '1', '0');
    field_245.add_subfield('a', "The Great Gatsby /".to_string());
    field_245.add_subfield('c', "F. Scott Fitzgerald.".to_string());
    record.add_field(field_245);

    // Add author
    let mut field_100 = Field::new("100".to_string(), '1', ' ');
    field_100.add_subfield('a', "Fitzgerald, F. Scott,".to_string());
    field_100.add_subfield('d', "1896-1940.".to_string());
    record.add_field(field_100);

    Ok(record)
}
```

### Builder Pattern (Recommended)

```rust
use mrrc::{Record, Field, Leader};

fn create_record_with_builder() -> Record {
    let leader = Leader {
        record_length: 0,
        record_status: 'n',
        record_type: 'a',
        bibliographic_level: 'm',
        control_record_type: ' ',
        character_coding: 'a',  // UTF-8
        indicator_count: 2,
        subfield_code_count: 2,
        data_base_address: 0,
        encoding_level: ' ',
        cataloging_form: 'a',
        multipart_level: ' ',
        reserved: "4500".to_string(),
    };

    Record::builder(leader)
        .control_field_str("001", "12345")
        .control_field_str("008", "200101s2020    xxu||||||||||||||||eng||")
        .field(
            Field::builder("245".to_string(), '1', '0')
                .subfield_str('a', "The Great Gatsby /")
                .subfield_str('c', "F. Scott Fitzgerald.")
                .build()
        )
        .field(
            Field::builder("100".to_string(), '1', ' ')
                .subfield_str('a', "Fitzgerald, F. Scott,")
                .subfield_str('d', "1896-1940.")
                .build()
        )
        .field(
            Field::builder("650".to_string(), ' ', '0')
                .subfield_str('a', "Psychological fiction.")
                .build()
        )
        .build()
}
```

### Configuring the Leader

```rust
use mrrc::Leader;

fn create_leader() -> Leader {
    Leader {
        record_length: 0,        // Calculated during write
        record_status: 'n',      // New record
        record_type: 'a',        // Language material
        bibliographic_level: 'm', // Monograph
        control_record_type: ' ',
        character_coding: 'a',   // UTF-8
        indicator_count: 2,
        subfield_code_count: 2,
        data_base_address: 0,    // Calculated during write
        encoding_level: ' ',     // Full level
        cataloging_form: 'a',    // AACR2
        multipart_level: ' ',
        reserved: "4500".to_string(),
    }
}
```

## Writing Records

### Basic Writing

```rust
use mrrc::{MarcWriter, Record};
use std::fs::File;

fn write_records(records: Vec<Record>) -> mrrc::Result<()> {
    let file = File::create("output.mrc")?;
    let mut writer = MarcWriter::new(file);

    for record in &records {
        writer.write_record(record)?;
    }

    Ok(())
}
```

### Write to Buffer

```rust
use mrrc::{MarcWriter, Record};

fn write_to_buffer(record: &Record) -> mrrc::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    {
        let mut writer = MarcWriter::new(&mut buffer);
        writer.write_record(record)?;
    }
    Ok(buffer)
}
```

## Format Conversion

### JSON

```rust
use mrrc::{Record, json};

fn convert_json(record: &Record) -> mrrc::Result<()> {
    // Convert to JSON
    let json = json::record_to_json(record)?;
    println!("{}", json);

    // Parse back
    let restored = json::json_to_record(&json)?;
    assert_eq!(record.title(), restored.title());

    Ok(())
}
```

### MARCJSON

```rust
use mrrc::{Record, marcjson};

fn convert_marcjson(record: &Record) -> mrrc::Result<()> {
    // Convert to standard MARC-in-JSON format
    let json = marcjson::record_to_marcjson(record)?;
    println!("{}", json);

    // Parse back
    let restored = marcjson::marcjson_to_record(&json)?;

    Ok(())
}
```

### XML

```rust
use mrrc::{Record, xml};

fn convert_xml(record: &Record) -> mrrc::Result<()> {
    // Convert to XML
    let xml_str = xml::record_to_xml(record)?;
    println!("{}", xml_str);

    // Parse back
    let restored = xml::xml_to_record(&xml_str)?;

    Ok(())
}
```

### CSV

```rust
use mrrc::{Record, csv};

fn convert_csv(records: &[Record]) -> mrrc::Result<()> {
    // Convert multiple records to CSV
    let csv_str = csv::records_to_csv(records)?;
    println!("{}", csv_str);

    Ok(())
}
```

## Using the Formats Module

The `formats` module provides unified traits for format-agnostic code.

### FormatReader and FormatWriter Traits

```rust
use mrrc::formats::{FormatReader, FormatWriter};
use mrrc::formats::iso2709::{Iso2709Reader, Iso2709Writer};
use mrrc::formats::protobuf::{ProtobufReader, ProtobufWriter};
use std::fs::File;

fn convert_format<R: FormatReader, W: FormatWriter>(
    reader: &mut R,
    writer: &mut W,
) -> mrrc::Result<usize> {
    let mut count = 0;
    while let Some(record) = reader.read_record()? {
        writer.write_record(&record)?;
        count += 1;
    }
    writer.finish()?;
    Ok(count)
}

fn example() -> mrrc::Result<()> {
    // ISO 2709 to Protobuf
    let input = File::open("input.mrc")?;
    let mut reader = Iso2709Reader::new(input);

    let mut output = Vec::new();
    let mut writer = ProtobufWriter::new(&mut output);

    let count = convert_format(&mut reader, &mut writer)?;
    println!("Converted {} records", count);

    Ok(())
}
```

### Protobuf Format

```rust
use mrrc::formats::protobuf::{ProtobufReader, ProtobufWriter};
use mrrc::formats::{FormatReader, FormatWriter};
use std::io::Cursor;

fn protobuf_example() -> mrrc::Result<()> {
    // Write
    let mut buffer = Vec::new();
    {
        let mut writer = ProtobufWriter::new(&mut buffer);
        // ... write records
        writer.finish()?;
    }

    // Read
    let cursor = Cursor::new(buffer);
    let mut reader = ProtobufReader::new(cursor);
    while let Some(record) = reader.read_record()? {
        println!("{:?}", record.title());
    }

    Ok(())
}
```

### MessagePack Format

```rust
use mrrc::messagepack::{MessagePackReader, MessagePackWriter};
use mrrc::formats::{FormatReader, FormatWriter};

fn messagepack_example() -> mrrc::Result<()> {
    let mut buffer = Vec::new();
    {
        let mut writer = MessagePackWriter::new(&mut buffer);
        // ... write records
        writer.finish()?;
    }

    Ok(())
}
```

## Parallel Processing

### Using Rayon

```rust
use mrrc::MarcReader;
use rayon::prelude::*;
use std::fs::File;

fn parallel_read_files(paths: &[&str]) -> mrrc::Result<Vec<String>> {
    let results: Vec<_> = paths
        .par_iter()
        .map(|path| {
            let file = File::open(path)?;
            let mut reader = MarcReader::new(file);
            let mut titles = Vec::new();

            while let Some(record) = reader.read_record()? {
                if let Some(title) = record.title() {
                    titles.push(title.to_string());
                }
            }
            Ok(titles)
        })
        .collect::<mrrc::Result<Vec<_>>>()?;

    Ok(results.into_iter().flatten().collect())
}
```

### Record Boundary Scanner

For parallel parsing of large files, use the boundary scanner:

```rust
use mrrc::boundary_scanner::RecordBoundaryScanner;
use std::fs::File;
use std::io::Read;

fn scan_boundaries() -> mrrc::Result<()> {
    let mut file = File::open("large.mrc")?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    let scanner = RecordBoundaryScanner::new(&data);
    let boundaries: Vec<_> = scanner.collect();

    println!("Found {} record boundaries", boundaries.len());

    Ok(())
}
```

## Authority and Holdings Records

### Authority Records

```rust
use mrrc::{AuthorityMarcReader, AuthorityRecord};
use std::fs::File;

fn read_authorities() -> mrrc::Result<()> {
    let file = File::open("authorities.mrc")?;
    let mut reader = AuthorityMarcReader::new(file);

    while let Some(record) = reader.read_record()? {
        println!("Heading type: {:?}", record.heading_type());

        // Access heading fields (1XX)
        if let Some(field) = record.get_field("100") {
            if let Some(name) = field.get_subfield('a') {
                println!("Name: {}", name);
            }
        }
    }

    Ok(())
}
```

### Holdings Records

```rust
use mrrc::{HoldingsMarcReader, HoldingsRecord};
use std::fs::File;

fn read_holdings() -> mrrc::Result<()> {
    let file = File::open("holdings.mrc")?;
    let mut reader = HoldingsMarcReader::new(file);

    while let Some(record) = reader.read_record()? {
        println!("Holdings type: {:?}", record.holdings_type());

        // Access location (852)
        if let Some(field) = record.get_field("852") {
            if let Some(location) = field.get_subfield('a') {
                println!("Location: {}", location);
            }
        }
    }

    Ok(())
}
```

## Error Handling

### Using Result

```rust
use mrrc::{MarcReader, MarcError, Result};
use std::fs::File;

fn process_with_errors() -> Result<()> {
    let file = File::open("records.mrc")?;
    let mut reader = MarcReader::new(file);

    loop {
        match reader.read_record() {
            Ok(Some(record)) => {
                // Process record
                println!("{:?}", record.title());
            }
            Ok(None) => {
                // End of file
                break;
            }
            Err(MarcError::InvalidRecord(msg)) => {
                // Skip invalid record
                eprintln!("Skipping invalid record: {}", msg);
                continue;
            }
            Err(e) => {
                // Propagate other errors
                return Err(e);
            }
        }
    }

    Ok(())
}
```

### Recovery Mode

```rust
use mrrc::{MarcReader, RecoveryMode, RecoveryContext};
use std::fs::File;

fn read_with_recovery() -> mrrc::Result<()> {
    let file = File::open("records.mrc")?;
    let mut reader = MarcReader::new(file);

    // Configure recovery for malformed records
    let mut context = RecoveryContext::new(RecoveryMode::SkipInvalid);

    while let Some(result) = reader.read_record_with_recovery(&mut context)? {
        match result {
            Ok(record) => println!("{:?}", record.title()),
            Err(e) => eprintln!("Skipped: {}", e),
        }
    }

    Ok(())
}
```

## Complete Example

```rust
//! Example: Convert MARC file to JSON Lines format.

use mrrc::{MarcReader, json};
use std::fs::File;
use std::io::{BufWriter, Write};

fn main() -> mrrc::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} input.mrc output.jsonl", args[0]);
        std::process::exit(1);
    }

    let input = File::open(&args[1])?;
    let output = File::create(&args[2])?;
    let mut writer = BufWriter::new(output);

    let mut reader = MarcReader::new(input);
    let mut count = 0;

    while let Some(record) = reader.read_record()? {
        let json = json::record_to_json(&record)?;
        writeln!(writer, "{}", json)?;
        count += 1;
    }

    println!("Converted {} records", count);
    Ok(())
}
```

## Next Steps

- [Format Selection Guide](./FORMAT_SELECTION_GUIDE.md) - Choose the right format
- [Streaming Guide](./STREAMING_GUIDE.md) - Large file handling
- [Architecture](./ARCHITECTURE.md) - System design details
