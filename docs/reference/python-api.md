# Python API Reference

Complete Python API reference for MRRC.

## Core Classes

### Record

A MARC bibliographic record containing a leader, control fields, and data fields.

```python
from mrrc import Record, Field, Leader, Subfield

# Create a record with inline fields
record = Record(fields=[
    Field("245", indicators=["1", "0"], subfields=[
        Subfield("a", "Title"),
    ]),
])

# Or build incrementally
record = Record(Leader())
record.add_control_field("001", "123456789")
field = Field("245", "1", "0")
field.add_subfield("a", "Title")
record.add_field(field)
```

**Properties and Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `leader()` | `Leader` | Get the record's leader |
| `add_control_field(tag, value)` | `None` | Add a control field (001-009) |
| `control_field(tag)` | `str \| None` | Get a control field value |
| `control_fields()` | `list[tuple[str, str]]` | Get all control fields |
| `add_field(field)` | `None` | Add a data field |
| `fields()` | `list[Field]` | Get all data fields |
| `fields_by_tag(tag)` | `list[Field]` | Get fields matching a tag |
| `title()` | `str \| None` | Get title from 245$a |
| `author()` | `str \| None` | Get author from 100/110/111 |
| `isbn()` | `str \| None` | Get ISBN from 020$a |
| `isbns()` | `list[str]` | Get all ISBNs |

### Field

A MARC data field with tag, indicators, and subfields.

```python
from mrrc import Field, Subfield

# Create field with indicators and subfields inline
field = Field("245", indicators=["1", "0"], subfields=[
    Subfield("a", "Main title :"),
    Subfield("b", "subtitle /"),
    Subfield("c", "by Author."),
])

# Or build incrementally
field = Field("245", "1", "0")
field.add_subfield("a", "Main title :")

# Access subfields
print(field["a"])  # "Main title :"
for sf in field.subfields():
    print(f"${sf.code} {sf.value}")
```

**Properties and Methods:**

| Property/Method | Type | Description |
|-----------------|------|-------------|
| `tag` | `str` | 3-character field tag |
| `indicator1` | `str` | First indicator |
| `indicator2` | `str` | Second indicator |
| `add_subfield(code, value)` | `None` | Add a subfield |
| `subfields()` | `list[Subfield]` | Get all subfields |
| `subfields_by_code(code)` | `list[str]` | Get values for a subfield code |
| `__getitem__(code)` | `str \| None` | Get first subfield value by code |

### Subfield

A subfield within a MARC field.

```python
from mrrc import Subfield

sf = Subfield("a", "value")
print(sf.code)   # "a"
print(sf.value)  # "value"
```

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| `code` | `str` | Single-character subfield code |
| `value` | `str` | Subfield value (read/write) |

### Leader

The 24-byte MARC record header containing metadata.

```python
from mrrc import Leader

leader = Leader()
leader.record_type = "a"           # Language material
leader.bibliographic_level = "m"   # Monograph
leader.character_coding = "a"      # UTF-8
```

**Properties:**

| Property | Position | Description |
|----------|----------|-------------|
| `record_length` | 00-04 | Record length (5 digits) |
| `record_status` | 05 | Record status (n/c/d) |
| `record_type` | 06 | Type of record (a=language, c=music, etc.) |
| `bibliographic_level` | 07 | Bibliographic level (m=monograph, s=serial) |
| `control_record_type` | 08 | Type of control |
| `character_coding` | 09 | Character coding (space=MARC-8, a=UTF-8) |
| `indicator_count` | 10 | Indicator count (usually 2) |
| `subfield_code_count` | 11 | Subfield code count (usually 2) |
| `data_base_address` | 12-16 | Base address of data |
| `encoding_level` | 17 | Encoding level |
| `cataloging_form` | 18 | Descriptive cataloging form |
| `multipart_level` | 19 | Multipart resource record level |
| `reserved` | 20-23 | Entry map (usually "4500") |

## Reader/Writer Classes

### MARCReader

Reads MARC records from files with GIL-released I/O for parallelism.

```python
from mrrc import MARCReader

# From file path (recommended for performance)
for record in MARCReader("records.mrc"):
    print(record.title())

# From file object
with open("records.mrc", "rb") as f:
    for record in MARCReader(f):
        print(record.title())

# From bytes
data = open("records.mrc", "rb").read()
for record in MARCReader(data):
    print(record.title())
```

**Input Types:**

| Type | Description |
|------|-------------|
| `str` or `Path` | File path (pure Rust I/O, best performance) |
| `bytes` or `bytearray` | In-memory data |
| File object | Python file-like object |

**Thread Safety:**

- NOT thread-safe - each thread needs its own reader
- GIL released during record parsing for parallelism
- Use `ThreadPoolExecutor` with separate readers per thread

### MARCWriter

Writes MARC records to files.

```python
from mrrc import MARCWriter

with MARCWriter("output.mrc") as writer:
    writer.write(record)
```

**Methods:**

| Method | Description |
|--------|-------------|
| `write(record)` | Write a single record |
| `close()` | Close the writer (automatic with context manager) |

## Format Conversion

### Record Methods

```python
# JSON formats
json_str = record.to_json()
marcjson_str = record.to_marcjson()

# XML formats
xml_str = record.to_xml()
mods_str = record.to_mods()
dc_str = record.to_dublin_core()
```

### Module Functions

```python
import mrrc

# Parse from JSON
record = mrrc.json_to_record(json_str)

# Convert to CSV
csv_str = mrrc.record_to_csv(record)
csv_str = mrrc.records_to_csv(records)
```

## BIBFRAME Conversion

Convert MARC to BIBFRAME 2.0 RDF.

```python
from mrrc import marc_to_bibframe, BibframeConfig

# Basic conversion
config = BibframeConfig()
graph = marc_to_bibframe(record, config)

# With custom base URI
config.set_base_uri("http://library.example.org/")
graph = marc_to_bibframe(record, config)

# Serialize to different formats
turtle = graph.serialize("turtle")
rdfxml = graph.serialize("rdf-xml")
jsonld = graph.serialize("jsonld")
ntriples = graph.serialize("ntriples")
```

### BibframeConfig

| Method | Description |
|--------|-------------|
| `set_base_uri(uri)` | Set base URI for generated entities |

### BibframeGraph

| Method | Returns | Description |
|--------|---------|-------------|
| `len(graph)` | `int` | Number of triples |
| `serialize(format)` | `str` | Serialize to RDF format |

## Exceptions

```python
from mrrc import MarcError

try:
    for record in MARCReader("bad.mrc"):
        pass
except MarcError as e:
    print(f"MARC error: {e}")
```

## See Also

- [Python Quickstart](../getting-started/quickstart-python.md)
- [Migration from pymarc](../guides/migration-from-pymarc.md)
- [Format Support](formats.md)
