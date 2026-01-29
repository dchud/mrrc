# Format Support

MRRC supports multiple serialization formats for MARC records. This page provides a comprehensive reference of all supported formats.

## Format Matrix

### Core Formats (Always Available)

| Format | Read | Write | Python | Rust | Notes |
|--------|------|-------|--------|------|-------|
| ISO 2709 | Yes | Yes | Yes | Yes | Standard MARC binary interchange |
| JSON | Yes | Yes | Yes | Yes | Generic JSON representation |
| MARCJSON | Yes | Yes | Yes | Yes | LOC standard JSON-LD format |
| XML | Yes | Yes | Yes | Yes | MARCXML format |
| CSV | - | Yes | Yes | Yes | Tabular export |
| Dublin Core | - | Yes | Yes | Yes | 15-element metadata |
| MODS | - | Yes | Yes | Yes | Metadata Object Description Schema |

### Feature-Gated Formats

These formats require feature flags in Rust or are available in Python by default.

| Format | Read | Write | Feature Flag | Notes |
|--------|------|-------|--------------|-------|
| Protobuf | Yes | Yes | `format-protobuf` | Schema evolution support |
| Arrow | Yes | Yes | `format-arrow` | Analytics/columnar format |
| FlatBuffers | Yes | Yes | `format-flatbuffers` | Zero-copy access |
| MessagePack | Yes | Yes | `format-messagepack` | Compact binary |
| BIBFRAME | - | Yes | `format-bibframe` | RDF/Linked Data output |

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

### XML (MARCXML)

XML representation following MARCXML schema.

**Python**:
```python
xml_str = record.to_xml()
```

### Protobuf

Protocol Buffers format with schema evolution support.

**Feature flag**: `format-protobuf`

**Use cases**:
- Microservices and APIs
- Cross-language compatibility
- Schema evolution

**Python**:
```python
from mrrc import ProtobufReader, ProtobufWriter

# Writing
writer = ProtobufWriter("records.pb")
writer.write_record(record)
writer.close()

# Reading
for record in ProtobufReader("records.pb"):
    print(record.title())
```

### Arrow

Apache Arrow columnar format for analytics.

**Feature flag**: `format-arrow`

**Use cases**:
- DuckDB and Polars integration
- Data science pipelines
- Large-scale analytics

**Python**:
```python
from mrrc import ArrowWriter, ArrowReader

# Writing
writer = ArrowWriter("records.arrow")
writer.write_record(record)
writer.close()

# Reading
for record in ArrowReader("records.arrow"):
    print(record.title())

# DuckDB integration
import duckdb
conn = duckdb.connect()
result = conn.execute("SELECT * FROM 'records.arrow'").fetchall()
```

### FlatBuffers

Zero-copy binary format for maximum read performance.

**Feature flag**: `format-flatbuffers`

**Use cases**:
- Memory-constrained environments
- Streaming applications
- Mobile/embedded systems

**Python**:
```python
from mrrc import FlatbuffersReader, FlatbuffersWriter

writer = FlatbuffersWriter("records.fb")
writer.write_record(record)
writer.close()

for record in FlatbuffersReader("records.fb"):
    print(record.title())
```

### MessagePack

Compact binary format with wide language support.

**Feature flag**: `format-messagepack`

**Use cases**:
- REST APIs (smaller than JSON)
- Cross-language IPC
- 50+ language implementations

**Python**:
```python
from mrrc import MessagePackReader, MessagePackWriter

writer = MessagePackWriter("records.msgpack")
writer.write_record(record)
writer.close()

for record in MessagePackReader("records.msgpack"):
    print(record.title())
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

**Feature flag**: `format-bibframe`

**Python**:
```python
from mrrc import marc_to_bibframe, BibframeConfig, RdfFormat

config = BibframeConfig(
    base_uri="http://example.org/",
    output_format=RdfFormat.Turtle
)
rdf_graph = marc_to_bibframe(record, config)
turtle_str = rdf_graph.serialize(RdfFormat.Turtle)
```

## Performance Comparison

Based on benchmarks with 10k records:

| Format | Read Speed | Write Speed | File Size |
|--------|-----------|-------------|-----------|
| ISO 2709 | 900k rec/s | 800k rec/s | Baseline |
| Protobuf | 750k rec/s | 700k rec/s | 0.8x |
| Arrow | 865k rec/s | 800k rec/s | 0.04x |
| MessagePack | 750k rec/s | 700k rec/s | 0.75x |
| FlatBuffers | 1M+ rec/s | 700k rec/s | 0.64x |
| JSON | 200k rec/s | 250k rec/s | 2.5x |

## See Also

- [Format Selection Guide](../guides/format-selection.md) - Choosing the right format
- [Streaming Guide](../guides/streaming-large-files.md) - Large file handling
