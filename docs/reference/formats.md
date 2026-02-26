# Format Support

MRRC supports multiple serialization formats for MARC records. This page provides a comprehensive reference of all supported formats.

## Format Matrix

| Format | Read | Write | Python | Rust | Notes |
|--------|------|-------|--------|------|-------|
| ISO 2709 | Yes | Yes | Yes | Yes | Standard MARC binary interchange |
| JSON | Yes | Yes | Yes | Yes | Generic JSON representation |
| MARCJSON | Yes | Yes | Yes | Yes | LOC standard JSON-LD format |
| MARCXML | Yes | Yes | Yes | Yes | MARC21 XML schema |
| CSV | - | Yes | Yes | Yes | Tabular export |
| Dublin Core | - | Yes | Yes | Yes | 15-element metadata |
| MODS | Yes | Yes | Yes | Yes | Metadata Object Description Schema |
| BIBFRAME | Yes | Yes | Yes | Yes | RDF/Linked Data (bidirectional) |

All formats are available in both Python and Rust without feature flags.

## Format Details

### ISO 2709 (MARC Binary)

The standard interchange format for MARC records.

**File extension**: `.mrc`

**Use cases**:

- Library system interchange (ILS, OCLC)
- Z39.50 compatible systems
- Maximum compatibility

**Python**:
```python
from mrrc import MARCReader, MARCWriter

# Reading
for record in MARCReader("records.mrc"):
    print(record.title())

# Writing
with MARCWriter("output.mrc") as writer:
    writer.write(record)
```

**Rust**:
```rust
use mrrc::{MarcReader, MarcWriter};
use std::fs::File;

// Reading
let mut reader = MarcReader::new(File::open("records.mrc")?);
while let Some(record) = reader.read_record()? {
    println!("{:?}", record.title());
}

// Writing
let mut writer = MarcWriter::new(File::create("output.mrc")?);
writer.write_record(&record)?;
```

### JSON

Generic JSON representation of MARC records.

**Python**:
```python
json_str = record.to_json()
record = mrrc.json_to_record(json_str)
```

### MARCJSON

Library of Congress standard JSON format for MARC.

**Python**:
```python
marcjson_str = record.to_marcjson()
```

### MARCXML

MARCXML representation following the MARC21 XML schema.

**Python**:
```python
xml_str = record.to_xml()

# Parse a single record from MARCXML
record = mrrc.xml_to_record(xml_str)

# Parse a MARCXML collection (multiple records)
records = mrrc.xml_to_records(collection_xml_str)
```

### CSV

Tabular export for spreadsheet applications.

**Python**:
```python
csv_str = mrrc.record_to_csv(record)
csv_str = mrrc.records_to_csv(records)
```

### Dublin Core

Simplified 15-element metadata schema.

**Python**:
```python
dc_xml = record.to_dublin_core()
```

### MODS

Metadata Object Description Schema for detailed bibliographic metadata.

**Python**:
```python
mods_xml = record.to_mods()
```

### BIBFRAME

BIBFRAME 2.0 RDF output for linked data applications.

**Python**:
```python
from mrrc import marc_to_bibframe, BibframeConfig, RdfFormat

config = BibframeConfig()
config.set_base_uri("http://example.org/")
rdf_graph = marc_to_bibframe(record, config)
turtle_str = rdf_graph.serialize("turtle")
```

See the [BIBFRAME Conversion Guide](../guides/bibframe-conversion.md) for detailed usage.

## See Also

- [Format Selection Guide](../guides/format-selection.md) - Choosing the right format
- [Working with Large Files](../guides/working-with-large-files.md) - Large file handling
