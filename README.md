# MRRC: MARC Rust Crate

A Rust library for reading, writing, and manipulating MARC bibliographic records in the ISO 2709 binary format.

> **⚠️ EXPERIMENTAL**: This library is a work in progress, generated with AI coding tools (Amp) and issue tracking (Beads). APIs may change significantly. Use at your own risk in production.

## Overview

MRRC is a Rust port of [pymarc](https://gitlab.com/pymarc/pymarc), designed for developers who work with library metadata and the MARC (Machine-Readable Cataloging) standard. MARC is the primary standard for encoding bibliographic and authority data in libraries worldwide.

This library provides:

- **ISO 2709 Binary Format Support**: Read and write MARC records in the standard binary interchange format
- **Multiple Serialization Formats**: Convert records to/from JSON, XML, and MARCJSON
- **Flexible API**: Rust-friendly patterns including iterators, builders, and direct field access
- **Encoding Support**: Handle MARC-8 and UTF-8 encoded records with automatic detection

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
mrrc = "0.1"
```

## Quick Start

### Reading MARC Records

```rust,ignore
use mrrc::MarcReader;
use std::fs::File;

let file = File::open("records.mrc")?;
let mut reader = MarcReader::new(file);

// Read records one at a time
while let Some(record) = reader.read_record()? {
    println!("Record type: {}", record.leader.record_type);
    
    // Get fields by tag
    if let Some(title_fields) = record.get_fields("245") {
        if let Some(title) = title_fields[0].get_subfield('a') {
            println!("Title: {}", title);
        }
    }
}
```

### Writing MARC Records

```rust,ignore
use mrrc::{MarcWriter, Record, Field, Leader};
use std::io::Cursor;

let mut record = Record::new(Leader::default());

// Add control field
record.add_control_field("008".to_string(), "200101s2020    xxu||||||||||||||||eng||".to_string());

// Add data field with subfields
let mut field_245 = Field::new("245".to_string(), '1', '0');
field_245.add_subfield('a', "The Great Gatsby /".to_string());
field_245.add_subfield('c', "F. Scott Fitzgerald.".to_string());
record.add_field(field_245);

// Write to buffer
let mut buffer = Vec::new();
{
    let mut writer = MarcWriter::new(&mut buffer);
    writer.write_record(&record)?;
}

// Or write to file
let file = std::fs::File::create("output.mrc")?;
let mut writer = MarcWriter::new(file);
writer.write_record(&record)?;
```

### Writing MARC Records with Builder API (Recommended)

For a more fluent, idiomatic Rust experience, use the builder pattern:

```rust,ignore
use mrrc::{Record, Field, Leader};

let record = Record::builder(Leader::default())
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
            .subfield_str('d', "1896-1940")
            .build()
    )
    .field(
        Field::builder("650".to_string(), ' ', '0')
            .subfield_str('a', "Psychological fiction.")
            .build()
    )
    .field(
        Field::builder("650".to_string(), ' ', '0')
            .subfield_str('a', "United States")
            .subfield_str('x', "History")
            .subfield_str('y', "20th century")
            .build()
    )
    .build();

let mut writer = MarcWriter::new(buffer);
writer.write_record(&record)?;
```

Benefits of the builder approach:
- **Cleaner syntax** - No need for explicit `.to_string()` calls with `*_str()` methods
- **Method chaining** - Build complex records in a single expression
- **Type safety** - Compile-time checking of record structure

### Converting to Other Formats

#### JSON Format

```rust,ignore
use mrrc::json;

let json = json::record_to_json(&record)?;
println!("{}", json.to_string());

// Convert back
let restored = json::json_to_record(&json)?;
```

#### XML Format

```rust,ignore
use mrrc::xml;

let xml_string = xml::record_to_xml(&record)?;
println!("{}", xml_string);

// Convert back
let restored = xml::xml_to_record(&xml_string)?;
```

#### MARCJSON Format (standard MARC-JSON)

```rust,ignore
use mrrc::marcjson;

let json = marcjson::record_to_marcjson(&record)?;
println!("{}", json.to_string());

// Convert back
let restored = marcjson::marcjson_to_record(&json)?;
```

## MARC Record Structure

A MARC record consists of:

- **Leader**: 24-byte header with record metadata (length, type, encoding level, etc.)
- **Control Fields** (000-009): Fixed-length fields like the control number (001) and fixed-length data elements (008)
- **Data Fields** (010+): Variable-length fields with indicators and subfields
  - **Indicators**: Two single-character codes providing additional context
  - **Subfields**: Labeled data elements identified by a single character code

### Example: Title Field (245)

```text
245 1 0 |a The Great Gatsby / |c F. Scott Fitzgerald.
```

Breaking down:
- `245`: Field tag (Title Statement)
- `1 0`: Indicators (first = 1, second = 0)
- `|a` (or `$a`): Subfield 'a' (main title)
- `|c` (or `$c`): Subfield 'c' (statement of responsibility)

## API Overview

### Record

```rust,ignore
// Create using builder (recommended)
let record = Record::builder(Leader::default())
    .control_field_str("001", "123456")
    .field(field)
    .build();

// Or create manually
let mut record = Record::new(Leader::default());
record.add_control_field_str("001", "123456");
record.add_field(field);

// Retrieve fields
if let Some(fields) = record.get_fields("245") { }
if let Some(field) = record.get_field("245") { }

// Iterate over fields
for field in record.fields() { }
for field in record.fields_by_tag("650") { }  // By tag
for (tag, value) in record.control_fields_iter() { }  // Control fields
```

### Field

```rust,ignore
// Create using builder (recommended)
let field = Field::builder("245".to_string(), '1', '0')
    .subfield_str('a', "Title")
    .subfield_str('c', "Author")
    .build();

// Or create manually
let mut field = Field::new("245".to_string(), '1', '0');
field.add_subfield_str('a', "Title");
field.add_subfield_str('c', "Author");

// Retrieve subfields
if let Some(value) = field.get_subfield('a') { }
let values = field.get_subfield_values('a');  // Multiple occurrences

// Iterate over subfields
for subfield in field.subfields() { }
for value in field.subfields_by_code('a') { }  // By code
```

### MarcReader & MarcWriter

```rust,ignore
// Read from any source implementing Read
let mut reader = MarcReader::new(file);
while let Some(record) = reader.read_record()? { }

// Write to any destination implementing Write
let mut writer = MarcWriter::new(buffer);
writer.write_record(&record)?;
```

## Testing

The library includes 97 comprehensive tests covering:

- **Unit tests** (89): Individual component functionality, including builder and iterator API
- **Integration tests** (8): End-to-end reading, writing, and format conversions

Run tests with:

```bash
cargo test
```

Test data files are in `tests/data/`:
- `simple_book.mrc`: Basic monograph record
- `music_score.mrc`: Musical notation record
- `with_control_fields.mrc`: Record with 008 field
- `multi_records.mrc`: Multiple records in one file

## Design Principles

1. **Rust-Idiomatic**: Uses iterators, Result types, and ownership patterns naturally
2. **Zero-Copy Where Possible**: Efficient memory usage for large record sets
3. **Format Flexibility**: Support for multiple serialization formats out of the box
4. **Compatibility**: Maintains data fidelity with pymarc and standard MARC tools

## Known Limitations

- Limited validation of field indicators and indicator semantics
- No support for MARC Authority records (planned)
- No direct support for MARC Authorities or Holdings records yet

## Development Status

This library is actively under development. See the [GitHub Issues](https://github.com/dchud/mrrc) for planned features and known issues.

## License

MIT

## Contributing

Contributions welcome. Please open issues and pull requests on GitHub.

## References

- [MARC 21 Standard](https://www.loc.gov/marc/)
- [ISO 2709](https://en.wikipedia.org/wiki/MARC_standards)
- [pymarc Project](https://gitlab.com/pymarc/pymarc)
