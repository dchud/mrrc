# MRRC: MARC Rust Crate

[![Tests](https://github.com/dchud/mrrc/actions/workflows/test.yml/badge.svg)](https://github.com/dchud/mrrc/actions/workflows/test.yml)
[![Lint](https://github.com/dchud/mrrc/actions/workflows/lint.yml/badge.svg)](https://github.com/dchud/mrrc/actions/workflows/lint.yml)
[![Build](https://github.com/dchud/mrrc/actions/workflows/build.yml/badge.svg)](https://github.com/dchud/mrrc/actions/workflows/build.yml)
[![codecov](https://codecov.io/gh/dchud/mrrc/branch/main/graph/badge.svg)](https://codecov.io/gh/dchud/mrrc)

A Rust library for reading, writing, and manipulating MARC bibliographic records in the ISO 2709 binary format.

> **⚠️ EXPERIMENTAL**: This library is a work in progress. While the pymarc API is now fully compatible, other APIs may change significantly. Use at your own risk in production.

## Overview

MRRC is a high-performance Rust port of [pymarc](https://gitlab.com/pymarc/pymarc), designed for developers who work with library metadata and the MARC (Machine-Readable Cataloging) standard. MARC is the primary standard for encoding bibliographic and authority data in libraries worldwide.

**Key Features:**

- **Full pymarc API Compatibility** - Drop-in replacement for existing pymarc code (see [migration guide](docs/MIGRATION_GUIDE.md))
- **Multi-Format Support** - 10+ serialization formats for interchange, analytics, and archival
- **High Performance** - ~900k records/sec read, 7.5x faster than pymarc in Python
- **Encoding Support** - MARC-8 and UTF-8 with automatic detection
- **Flexible API** - Rust-friendly patterns with iterators, builders, and direct field access

## Format Support Matrix

MRRC supports multiple serialization formats:

| Format | Extension | Read | Write | Use Case |
|--------|-----------|------|-------|----------|
| **ISO 2709** | `.mrc` | Yes | Yes | Standard MARC interchange |
| **Protobuf** | `.pb` | Yes | Yes | APIs, cross-language IPC |
| **JSON** | `.json` | Yes | Yes | Human-readable interchange |
| **MARCJSON** | `.json` | Yes | Yes | LOC standard JSON-LD |
| **XML** | `.xml` | Yes | Yes | MARCXML standard |
| **CSV** | `.csv` | — | Yes | Spreadsheet export |
| **Arrow** ᶠ | `.arrow` | Yes | Yes | Analytics (`DuckDB`, Polars) |
| **`MessagePack`** ᶠ | `.msgpack` | Yes | Yes | Compact binary, 50+ languages |
| **`FlatBuffers`** ᶠ | `.fb` | Yes | Yes | Zero-copy, memory-efficient |

ᶠ Feature-gated formats require feature flags in Rust (`format-arrow`, `format-messagepack`, etc.) or optional dependencies in Python.

See [FORMAT_SELECTION_GUIDE.md](docs/FORMAT_SELECTION_GUIDE.md) for help choosing formats.

## Installation

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
mrrc = "0.4"
```

**With optional format support:**

```toml
[dependencies]
mrrc = { version = "0.4", features = ["format-arrow", "format-messagepack"] }

# Or enable all formats:
mrrc = { version = "0.4", features = ["all-formats"] }
```

### Python

```bash
pip install mrrc
```

**With optional analytics support:**

```bash
pip install mrrc[analytics]   # Adds DuckDB, Polars integration
pip install mrrc[all]         # All optional dependencies
```

Supported: Python 3.9+ on Linux, macOS, Windows (`x86_64`, `arm64`)

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

// Create a leader for a new bibliographic record
let leader = Leader {
    record_length: 0,  // Will be calculated during write
    record_status: 'n',
    record_type: 'a',  // 'a' = language material
    bibliographic_level: 'm',  // 'm' = monograph
    control_record_type: ' ',
    character_coding: ' ',  // ' ' = MARC-8, 'a' = UTF-8
    indicator_count: 2,
    subfield_code_count: 2,
    data_base_address: 0,  // Will be calculated during write
    encoding_level: ' ',
    cataloging_form: 'a',
    multipart_level: ' ',
    reserved: "4500".to_string(),
};

let mut record = Record::new(leader);

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
use mrrc::{Record, Field, Leader, MarcWriter};

let leader = Leader {
    record_length: 0,
    record_status: 'n',
    record_type: 'a',
    bibliographic_level: 'm',
    control_record_type: ' ',
    character_coding: ' ',
    indicator_count: 2,
    subfield_code_count: 2,
    data_base_address: 0,
    encoding_level: ' ',
    cataloging_form: 'a',
    multipart_level: ' ',
    reserved: "4500".to_string(),
};

let record = Record::builder(leader)
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

let mut buffer = Vec::new();
let mut writer = MarcWriter::new(&mut buffer);
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
use mrrc::{Record, Leader};

let leader = Leader {
    record_length: 0,
    record_status: 'n',
    record_type: 'a',
    bibliographic_level: 'm',
    control_record_type: ' ',
    character_coding: ' ',
    indicator_count: 2,
    subfield_code_count: 2,
    data_base_address: 0,
    encoding_level: ' ',
    cataloging_form: 'a',
    multipart_level: ' ',
    reserved: "4500".to_string(),
};

// Create using builder (recommended)
let record = Record::builder(leader.clone())
    .control_field_str("001", "123456")
    .field(field)
    .build();

// Or create manually
let mut record = Record::new(leader);
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

### `MarcReader` & `MarcWriter`

```rust,ignore
// Read from any source implementing Read
let mut reader = MarcReader::new(file);
while let Some(record) = reader.read_record()? { }

// Write to any destination implementing Write
let mut writer = MarcWriter::new(buffer);
writer.write_record(&record)?;
```

### Character Encoding Detection

```rust,ignore
use mrrc::encoding::MarcEncoding;

// Detect encoding from leader's character coding field (position 9)
let encoding = MarcEncoding::from_leader_char(leader_char)?;

// Use encoding for field data processing
match encoding {
    MarcEncoding::Marc8 => {
        // Handle MARC-8 with escape sequences and character set switching
    }
    MarcEncoding::Utf8 => {
        // Handle UTF-8 directly (all Unicode supported)
    }
}
```

## Character Encoding Support

MRRC provides comprehensive support for both MARC-8 and UTF-8 encodings, with special emphasis on proper handling of multilingual records.

### MARC-8 Encoding (Legacy)

MARC-8 is the historical character encoding for MARC records, still widely used in library systems. It uses ISO 2022 escape sequences to switch between different character sets:

```rust,ignore
use mrrc::encoding::MarcEncoding;

// Detect encoding from MARC leader position 9
let encoding = match leader_char {
    ' ' => MarcEncoding::Marc8,   // Space = MARC-8
    'a' => MarcEncoding::Utf8,    // 'a' = UTF-8
    _ => panic!("Unknown encoding"),
};
```

**Supported MARC-8 Character Sets:**
- **Basic Latin (ASCII)**: Standard ASCII characters
- **ANSEL Extended Latin**: Extended Latin with diacritical marks
- **Hebrew**: Full Hebrew alphabet (Escape: `ESC ) 2`)
- **Arabic**: Basic and Extended Arabic (Escape: `ESC ) 3` / `ESC ) 4`)
- **Cyrillic**: Russian and other Slavic languages (Escape: `ESC ( N`)
- **Greek**: Greek alphabet (Escape: `ESC ( S`)
- **Special Sets**: Subscripts (`ESC b`), Superscripts (`ESC p`), Greek Symbols (`ESC g`)
- **East Asian**: Chinese, Japanese, Korean (EACC) via 3-byte sequences

**Key Features:**
- Proper handling of combining characters (diacritics) that precede base characters
- Logical data ordering: Text stored left-to-right internally, regardless of display direction
- Bidirectional script support for Hebrew and Arabic records
- Unicode normalization (NFC) for combining character representation

### UTF-8 Encoding (Modern)

Modern MARC records use UTF-8, the Unicode standard. This is the recommended encoding for new systems:

```rust,ignore
// UTF-8 encoded records require 'a' in leader position 9
let encoding = MarcEncoding::Utf8;

// All Unicode characters are supported directly
```

### Handling Multilingual Records

MARC-8 records with multiple scripts require careful handling of escape sequences:

```text
// Example: Hebrew text in an otherwise English record
// MARC-8 bytes: [English text] ESC)2 [Hebrew text] ESC)E [more English]
//
// The library automatically:
// - Parses escape sequences
// - Switches character set context
// - Applies combining marks correctly
// - Normalizes to Unicode (NFC)
```

See the examples directory for detailed demonstrations:
- `examples/marc8_encoding.rs` - MARC-8 character set overview and encoding detection
- `examples/multilingual_records.rs` - Building and handling multilingual records

## Authority and Holdings Records

MRRC supports Authority (Type Z) and Holdings (Type x/y/v/u) records in addition to standard bibliographic records:

```rust,ignore
use mrrc::{AuthorityMarcReader, AuthorityMarcWriter, HoldingsMarcReader, HoldingsMarcWriter};

// Read Authority records
let file = File::open("authorities.mrc")?;
let mut reader = AuthorityMarcReader::new(file);
while let Some(record) = reader.read_record()? {
    println!("Authority type: {}", record.holdings_type());
}

// Read Holdings records
let file = File::open("holdings.mrc")?;
let mut reader = HoldingsMarcReader::new(file);
while let Some(record) = reader.read_record()? {
    println!("Holdings type: {}", record.holdings_type());
}
```

### Authority Records

Authority records use Type 'z' and organize fields by heading type:
- **Headings** (1XX): Personal, corporate, topical, and geographic names
- **Tracings** (4XX/5XX): See also references and related headings
- **Notes** (6XX, 67X): Scope notes and historical information

### Holdings Records

Holdings records use Types x, y, v, or u (single-part, serial, multipart, unknown) and track physical items:
- **Locations** (852): Library location and call numbers
- **Captions & Pattern** (853-855): Serial enumeration schemes for basic units, supplements, indexes
- **Enumeration & Chronology** (863-865): Specific enumeration data
- **Textual Holdings** (866-868): Natural language descriptions
- **Item Information** (876-878): Specific copy information

## Python Wrapper 🦀

MRRC is available as a Python package, providing Rust-backed performance with Python's ease of use.

### Installation

Install from `PyPI`:

```bash
pip install mrrc
```

Supported Python versions: **3.9+** (3.9, 3.10, 3.11, 3.12)  
Wheels available for: Linux (`x86_64`, `aarch64`), macOS (`x86_64`, `arm64`), Windows (`x64`, `arm64`)

### Quick Start (Python)

```python
from mrrc import MARCReader, MARCWriter, Record, Field, Leader
import io

# Read MARC records
with open("records.mrc", "rb") as f:
    reader = MARCReader(f)
    for record in reader:  # or use while record := reader.read_record()
        title = record.title()  # Convenience method
        print(f"Title: {title}")
        
        # Access fields
        fields_245 = record.get_fields("245")
        if fields_245:
            print(f"Field 245: {fields_245[0]}")

# Create and write records
leader = Leader(
    record_type='a',           # 'a' = language material
    bibliographic_level='m',   # 'm' = monograph
    character_coding='a',      # 'a' = UTF-8
)

record = Record(leader)
record.add_control_field("008", "200101s2020    xxu||||||||||||||||eng||")

field_245 = Field("245", '1', '0')
field_245.add_subfield('a', "The Great Gatsby /")
field_245.add_subfield('c', "F. Scott Fitzgerald.")
record.add_field(field_245)

# Write to file
with open("output.mrc", "wb") as f:
    writer = MARCWriter(f)
    writer.write_record(record)
```

### Performance

The Python wrapper achieves exceptional performance through Rust implementation with automatic GIL release:

#### Speed Comparison (Single-Threaded, Default, After Warm-Up)
- **Reading 1k records**: 3.3 ms (300,000+ rec/s)
- **Reading 10k records**: 39.1 ms (255,600 rec/s after warm-up)
- **vs pymarc**: **~4x faster** (same API, dramatically better performance)
- **vs Rust library**: ~25-30% of pure Rust speed with Python convenience
- **GIL release**: Automatic during record parsing, no code changes needed
- **Note**: Warm-up times are from pytest-benchmark; cold-start is ~20% slower

#### Multi-Threaded Parallelism (Explicit, Opt-In)
- **2-thread speedup**: 2.0x vs sequential processing (on 2-core systems)
- **4-thread speedup**: 3.74x vs sequential processing (on 4-core systems)
- **How**: Use `concurrent.futures.ThreadPoolExecutor` with separate reader per thread
- **GIL behavior**: Released during parsing in each thread, enabling true parallelism
- **Use case**: Processing multiple files concurrently

**Key Point:** Default single-threaded mode automatically benefits from GIL release (7.5x faster than pymarc). Add threading explicitly only when processing multiple files.

See [docs/benchmarks/](docs/benchmarks/) for detailed performance analysis, [docs/PERFORMANCE.md](docs/PERFORMANCE.md) for threading guidance, and [docs/THREADING.md](docs/THREADING.md) for usage patterns.

### Threading & Concurrency (Opt-In via `ThreadPoolExecutor`)

MRRC's I/O operations automatically release the Python GIL during record parsing. This means:

- **Single-threaded (default):** Records parse faster by default, no code changes needed
- **Multi-threaded (explicit):** Use `ThreadPoolExecutor` to process multiple files concurrently

**Concrete Results from Benchmarking (Multi-File Processing):**
- 2 threads: **2.0x speedup** vs sequential processing
- 4 threads: **3.74x speedup** vs sequential processing
- Each thread needs its own reader instance (not shared)
- Optimal thread count: CPU core count

**Example: Parallel Multi-File Processing (Explicit Threading)**

```python
from concurrent.futures import ThreadPoolExecutor
from mrrc import MARCReader

def process_file(filename):
    """Process a single file. Called in a thread pool."""
    count = 0
    with open(filename, 'rb') as f:
        reader = MARCReader(f)  # Each thread must have its own reader
        while record := reader.read_record():
            # Process record
            count += 1
    return count

# Sequential processing (default, uses ~1 core)
total = 0
for filename in ["file1.mrc", "file2.mrc", "file3.mrc", "file4.mrc"]:
    total += process_file(filename)

# Parallel processing on 4-core system (opt-in, uses ~4 cores)
with ThreadPoolExecutor(max_workers=4) as executor:
    futures = [executor.submit(process_file, f) 
               for f in ["file1.mrc", "file2.mrc", "file3.mrc", "file4.mrc"]]
    results = [f.result() for f in futures]
    total = sum(results)
    # Expected: 3-4x faster than sequential
```

**Important Notes:**
- **Default behavior** (single-threaded): Automatically faster via GIL release, no changes needed
- **Explicit multi-threading**: Add `ThreadPoolExecutor` only when processing multiple files
- **Not thread-safe**: Sharing a reader across threads causes undefined behavior
- **Best practice**: One reader per thread; use `ThreadPoolExecutor` or `threading.Thread`
- **GIL behavior**: Released during Phase 2 (parsing), allowing true parallelism

See [docs/THREADING.md](docs/THREADING.md) and [examples/concurrent_reading.py](examples/concurrent_reading.py) for detailed patterns and benchmarks.

### Format Conversion (Python)

```python
from mrrc import MARCReader, record_to_csv, records_to_csv

with open("records.mrc", "rb") as f:
    reader = MARCReader(f)
    record = reader.read_record()
    
    # Convert to JSON
    json_str = record.to_json()
    
    # Convert to XML
    xml_str = record.to_xml()
    
    # Convert to MARCJSON
    marcjson_str = record.to_marcjson()
    
    # Convert to CSV (single or multiple records)
    csv_str = record_to_csv(record)
    
    # Convert multiple records to CSV
    records = [reader.read_record() for _ in range(10)]
    csv_str = records_to_csv(records)
    
    # Filter specific fields when exporting to CSV
    csv_str = records_to_csv_filtered(records, lambda tag: tag in ('245', '650', '700'))
```

### Error Handling

MRRC provides typed exceptions for better error handling:

```python
from mrrc import MARCReader, MarcException, MarcEncodingError

try:
    with open("records.mrc", "rb") as f:
        reader = MARCReader(f)
        while record := reader.read_record():
            process(record)
except MarcEncodingError as e:
    print(f"Encoding issue: {e}")
except MarcException as e:
    print(f"MARC error: {e}")
```

### Migration from pymarc

MRRC provides **full API compatibility with pymarc**, making migration straightforward. All major pymarc patterns work identically:

```python
# pymarc code works nearly unchanged with mrrc
from mrrc import MARCReader

# All these patterns work exactly like pymarc:
with open("file.mrc", "rb") as f:
    reader = MARCReader(f)
    for record in reader:
        # Dictionary-style field access (identical to pymarc)
        title = record['245']['a']          # ✅ Same as pymarc
        author = record['100']['a']         # ✅ Same as pymarc
        
        # Check if field exists (identical to pymarc)
        if '650' in record:
            subjects = record.get_fields('650')
        
        # Convenience methods (better than pymarc)
        title = record.title()              # Easier than record['245']['a']
        author = record.author()            # Easier than record['100']['a']
        
        # Control field access (pymarc compatible)
        control_num = record['001'].value   # ✅ Now works like pymarc
        
        # Indicator access (pymarc compatible)
        field = record['245']
        ind1 = field.indicators[0]          # ✅ Now works like pymarc
        ind2 = field.indicators[1]          # ✅ Now works like pymarc
```

**Migration checklist:**
- ✅ Replace `import pymarc` with `import mrrc` 
- ✅ Update record creation: `mrrc.Record(mrrc.Leader())` (only change needed)
- ✅ Everything else stays the same - dictionary access, field methods, iteration all identical

See [MIGRATION_GUIDE.md](docs/MIGRATION_GUIDE.md) for detailed migration instructions and performance notes.

## Examples

The `examples/` directory contains working code demonstrations for common tasks. Examples are provided in both Rust and Python (with pymarc-compatible API):

### Quick Reference Table

| Task | Rust | Python |
|------|------|--------|
| **Basic Operations** |
| Read records | [`reading_and_querying.rs`](examples/reading_and_querying.rs) | [`reading_and_querying.py`](examples/reading_and_querying.py) |
| Create records | [`creating_records.rs`](examples/creating_records.rs) | [`creating_records.py`](examples/creating_records.py) |
| Format conversion | [`format_conversion.rs`](examples/format_conversion.rs) | [`format_conversion.py`](examples/format_conversion.py) |
| **Concurrency** |
| Parallel reading (multiple files) | [`concurrent_reading.rs`](examples/concurrent_reading.rs) (Rayon) | [`concurrent_reading.py`](examples/concurrent_reading.py) (ThreadPoolExecutor) |
| Parallel reading (single large file) | N/A | [`concurrent_reading_producer_consumer.py`](examples/concurrent_reading_producer_consumer.py) (`ProducerConsumerPipeline`) |
| Parallel writing | [`concurrent_writing.rs`](examples/concurrent_writing.rs) (Rayon) | [`concurrent_writing.py`](examples/concurrent_writing.py) (ThreadPoolExecutor) |
| **Advanced** |
| Authority records | [`authority_records.rs`](examples/authority_records.rs) | [`authority_records.py`](examples/authority_records.py) |
| MARC-8 encoding | [`marc8_encoding.rs`](examples/marc8_encoding.rs) | [`marc8_encoding.py`](examples/marc8_encoding.py) |
| Multilingual data | [`multilingual_records.rs`](examples/multilingual_records.rs) | [`multilingual_records.py`](examples/multilingual_records.py) |
| CSV conversion | [`marc_to_csv.rs`](examples/marc_to_csv.rs) | Included in [`format_conversion.py`](examples/format_conversion.py) |

### Running Examples

**Rust examples:**
```bash
cargo run --example reading_and_querying
cargo run --example creating_records
cargo run --example format_conversion
cargo run --example concurrent_reading
```

**Python examples:**
```bash
python examples/reading_and_querying.py
python examples/creating_records.py
python examples/format_conversion.py
python examples/concurrent_reading.py
python examples/concurrent_writing.py
```

### Example Highlights

- **reading_and_querying**: Demonstrates field access, filtering by indicators, working with subfields, and advanced queries
- **creating_records**: Shows how to build records from scratch using builder API (Rust) or field methods (Python)
- **format_conversion**: Converts records to JSON, MARCJSON, XML, and CSV formats
- **concurrent_reading**: Parallel processing with Rayon (Rust) or `ThreadPoolExecutor` (Python)
- **concurrent_writing**: Demonstrates safe concurrent writing with separate writer per thread (Python)

### Python Concurrency Strategies

MRRC provides two concurrency patterns optimized for different scenarios:

- **ThreadPoolExecutor** ([`concurrent_reading.py`](examples/concurrent_reading.py)): Best for processing multiple separate files. Simple API, ~3-4x speedup. Use this unless you have a specific large-file requirement.

- **ProducerConsumerPipeline** ([`concurrent_reading_producer_consumer.py`](examples/concurrent_reading_producer_consumer.py)): Specialized for maximum throughput from a single large file (100MB+). Sophisticated pipelining with producer thread, parallel batch parsing, and bounded channels. ~3.74x speedup on 4-core systems.

See [docs/CONCURRENCY.md](docs/CONCURRENCY.md) for detailed performance comparisons and decision trees.

### Python/Rust API Parity

Most examples work identically in both languages. The Python examples maintain **full pymarc API compatibility**, so you can migrate from pymarc by simply changing the import:

```python
# Before (pymarc)
from pymarc import MARCReader

# After (mrrc)
from mrrc import MARCReader

# Everything else stays the same!
```

## Testing

The library includes 239 comprehensive tests covering:

- **Unit tests**: Individual component functionality, including builder and iterator API
- **Integration tests**: End-to-end reading, writing, and format conversions
- **Authority/Holdings tests**: Specialized record type handling

Run tests with:

```bash
cargo test
```

Test data files are in `tests/data/`:
- `simple_book.mrc`: Basic monograph record
- `music_score.mrc`: Musical notation record
- `with_control_fields.mrc`: Record with 008 field
- `multi_records.mrc`: Multiple records in one file
- `simple_authority.mrc`: Sample Authority record
- `simple_holdings.mrc`: Sample Holdings record

### Code Coverage

Code coverage is automatically measured on each push and pull request via [codecov.io](https://codecov.io/gh/dchud/mrrc). 

To generate a local coverage report:

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --all --timeout 300

# Open the report in your browser
open tarpaulin-report.html
```

## Documentation

Comprehensive documentation is available in the `docs/` directory:

- **[Documentation Index](docs/README.md)** - Overview of all documentation
- **[Concurrency Guide](docs/CONCURRENCY.md)** - Parallel processing in Rust and Python
  - Rayon patterns for pure Rust library
  - `ThreadPoolExecutor` and `ProducerConsumerPipeline` for Python wrapper
  - Performance characteristics and decision trees
- **[Threading Documentation](docs/THREADING.md)** - Python-specific GIL behavior and thread safety
- **[Benchmarking Results](docs/benchmarks/RESULTS.md)** - Performance metrics and analysis
- **[Design Documents](docs/design/)** - Architecture and feature proposals
- **[Project History](docs/history/)** - Code reviews, audits, and implementation notes

## Design Principles

1. **Rust-Idiomatic**: Uses iterators, Result types, and ownership patterns naturally
2. **Zero-Copy Where Possible**: Efficient memory usage for large record sets
3. **Format Flexibility**: Support for multiple serialization formats out of the box
4. **Compatibility**: Maintains data fidelity with pymarc and standard MARC tools

## Known Limitations

None known at this time. The library now includes comprehensive MARC21 indicator validation and full MARC-8 encoding support.

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
