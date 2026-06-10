# Python API Reference

Complete Python API reference for MRRC.

!!! tip "Canonical Type Stubs"
    The file `mrrc/_mrrc.pyi` is the ground-truth type reference for the
    Python extension module. IDEs use it for autocompletion and type checking.
    If this page and the stub file disagree, the stub file is authoritative.

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

All record accessors (`title`, `author`, `isbn`, …) are read-only
properties, matching pymarc.

::: mrrc.Record

**Dictionary access:**

```python
# record['xxx'] raises KeyError if tag is missing
field = record['245']      # Returns Field or raises KeyError

# Use record.get() for safe access (returns None if missing)
field = record.get('245')  # Returns Field or None
```

**Field handles:** fields obtained from a record (`record[tag]`,
`get_field`, `get_field_or_err`, `get_fields`, `fields`) are live
handles — every read and write goes through to the record, so in-place
edits persist, matching pymarc:

```python
record["245"].indicator1 = "1"        # persists
record["245"]["a"] = "New title /"    # persists
record["005"].data = "20260603120000.0"  # control fields too
```

Two caveats. Handles to the same field are distinct objects
(`record["245"] is record["245"]` is `False`) that share underlying
state — don't compare fields with `is` or `id()`. And removing fields
invalidates all outstanding handles for that record: any later use
raises [`StaleFieldError`](#exceptions); re-fetch the field and retry.
Query results (`fields_matching`, `fields_by_indicator`,
`fields_in_range`, and related) are snapshots, not handles — edits to
them do not write back.

### Field

A MARC field — both control fields and data fields use this class.

```python
from mrrc import Field, Subfield

# Create a data field with indicators and subfields inline
field = Field("245", indicators=["1", "0"], subfields=[
    Subfield("a", "Main title :"),
    Subfield("b", "subtitle /"),
    Subfield("c", "by Author."),
])

# Create a control field
field = Field("001", data="12345")

# Or build incrementally
field = Field("245", "1", "0")
field.add_subfield("a", "Main title :")

# Access subfields
print(field["a"])  # "Main title :"
for sf in field.subfields():
    print(f"${sf.code} {sf.value}")
```

::: mrrc.Field

### ControlField

A backward-compatible subclass of `Field` for control fields. Prefer `Field(tag, data=value)` for new code.

```python
from mrrc import ControlField

# Still works for backward compatibility
cf = ControlField("001", "12345")
print(cf.data)              # "12345"
print(cf.is_control_field())  # True
print(isinstance(cf, Field))  # True
```

### Subfield

A subfield within a MARC field.

```python
from mrrc import Subfield

sf = Subfield("a", "value")
print(sf.code)   # "a"
print(sf.value)  # "value"
```

::: mrrc.Subfield

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

### AuthorityRecord

A MARC authority record. Returned by [`AuthorityMARCReader`](#authoritymarcreader).

```python
from mrrc import AuthorityMARCReader, FieldNotFound

reader = AuthorityMARCReader("authorities.mrc")
record = next(reader)

print(record.heading_text())      # main heading string, or None
for tracing in record.see_from_tracings():
    print("see from:", tracing)
```

Note: `get_fields(tag)` returns `None` when no fields match (unlike
`Record.get_fields`, which returns `[]`).

::: mrrc.AuthorityRecord

```python
try:
    main = record.get_field_or_err("100")
except FieldNotFound as e:
    print(f"missing {e.field_tag} on record {e.record_control_number}")
```

### HoldingsRecord

A MARC holdings record. Returned by [`HoldingsMARCReader`](#holdingsmarcreader).

```python
from mrrc import HoldingsMARCReader

reader = HoldingsMARCReader("holdings.mrc")
record = next(reader)

for location in record.locations():       # 852 fields
    print(location)
for caption in record.captions_basic():   # 853 fields
    print(caption)
```

Note: `get_fields(tag)` returns `None` when no fields match (unlike
`Record.get_fields`, which returns `[]`).

::: mrrc.HoldingsRecord

## Reader/Writer Classes

mrrc provides a dedicated reader per MARC record type. All three share the
ISO 2709 binary format and the iteration protocol; they differ in the record
type they yield and their constructor keywords.

| Reader | Yields | Use for |
|--------|--------|---------|
| [`MARCReader`](#marcreader) | `Record` | Bibliographic records |
| [`AuthorityMARCReader`](#authoritymarcreader) | `AuthorityRecord` | Authority records |
| [`HoldingsMARCReader`](#holdingsmarcreader) | `HoldingsRecord` | Holdings records |

There are no dedicated authority/holdings *writers*; [`MARCWriter`](#marcwriter)
serializes any record's fields to ISO 2709.

### MARCReader

Reads MARC records from files with GIL-released I/O for parallelism.

```python
MARCReader(file_obj, to_unicode=True, permissive=False, recovery_mode=None,
           validation_level="structural", max_errors=None)
```

```python
from mrrc import MARCReader

# From file path (recommended for performance)
for record in MARCReader("records.mrc"):
    print(record.title)

# From file object
with open("records.mrc", "rb") as f:
    for record in MARCReader(f):
        print(record.title)

# From bytes
data = open("records.mrc", "rb").read()
for record in MARCReader(data):
    print(record.title)
```

**Input Types:**

| Type | Description |
|------|-------------|
| `str` or `Path` | File path (pure Rust I/O, best performance) |
| `bytes` or `bytearray` | In-memory data |
| File object | Python file-like object |

**Keyword Arguments:**

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `to_unicode` | `bool` | `True` | Accepted for pymarc compatibility. mrrc always converts MARC-8 to UTF-8; passing `False` emits a warning but has no effect. |
| `permissive` | `bool` | `False` | When `True`, yields `None` for records that fail to parse instead of raising, matching pymarc's permissive behavior. |
| `recovery_mode` | `str` | `None` | Controls how malformed records are handled (see below). When not given, resolves to `"permissive"` — or `"strict"` if `permissive=True` (the inner reader raises so the outer wrapper can swallow). Cannot be combined explicitly with `permissive=True` unless `"strict"`. |
| `validation_level` | `str` | `"structural"` | What counts as an error during parsing, orthogonal to `recovery_mode`. `"structural"` fires only ISO 2709 structural errors; `"strict_marc"` adds byte-level MARC 21 checks (indicators, subfield codes, strict UTF-8). See [Validation level vs recovery mode](error-handling.md#validation-level-vs-recovery-mode). |
| `max_errors` | `int` | `None` | Cap on accumulated recovered errors in lenient/permissive mode; exceeding it raises `FatalReaderError`. `None` or `0` disables the cap. See [Capping recovered errors](error-handling.md#capping-recovered-errors-with-max_errors). |

**Recovery Modes:**

Instead of skipping bad records entirely (like `permissive=True`), `recovery_mode` attempts to salvage valid fields from damaged records:

| Mode | Behavior |
|------|----------|
| `"strict"` | Raise on any malformation. |
| `"lenient"` | Attempt to recover, salvage valid fields from damaged records; diagnostics attach to `record.errors`. |
| `"permissive"` | Very lenient — accept partial data even from severely malformed records (default when `recovery_mode` is not given and `permissive=False`). |

See [Recovery modes and errors](error-handling.md#recovery-modes-and-errors) for the full
default story and guidance on choosing a mode.

```python
# Skip bad records (pymarc-compatible)
for record in MARCReader("bad.mrc", permissive=True):
    if record is None:
        continue
    print(record.title)

# Salvage partial data from malformed records
for record in MARCReader("bad.mrc", recovery_mode="lenient"):
    print(f"Got {len(record.get_fields())} fields")
```

Note: `permissive=True` and `recovery_mode` other than `"strict"` cannot be
combined — they represent different error-handling strategies. Use `permissive=True`
for pymarc-compatible "skip bad records" behavior, or `recovery_mode` for mrrc's
"salvage what you can" approach.

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

::: mrrc.MARCWriter

### AuthorityMARCReader

Reads MARC **authority** records, yielding [`AuthorityRecord`](#authorityrecord). Same ISO 2709 binary format and iteration protocol as [`MARCReader`](#marcreader), with a smaller keyword set. The record source (path, bytes, or file object) is positional; `recovery_mode` and `validation_level` are keyword-only and follow the [MARCReader recovery modes](#marcreader), except `recovery_mode` defaults to `"permissive"` here rather than `"strict"`.

```python
from mrrc import AuthorityMARCReader

# From a path, bytes, or file object — like MARCReader
for record in AuthorityMARCReader("authorities.mrc"):
    print(record.heading_text())
```

::: mrrc.AuthorityMARCReader

### HoldingsMARCReader

Reads MARC **holdings** records, yielding [`HoldingsRecord`](#holdingsrecord). Same shape and keyword semantics as `AuthorityMARCReader` (`recovery_mode` defaults to `"permissive"`).

```python
from mrrc import HoldingsMARCReader

for record in HoldingsMARCReader("holdings.mrc"):
    for location in record.locations():
        print(location)
```

::: mrrc.HoldingsMARCReader

## Query DSL

Composable query objects passed to the `Record.fields_matching*` methods to
select fields by tag, indicators, tag range, or subfield value/pattern.

```python
from mrrc import FieldQuery, SubfieldValueQuery

# Fields with tag 650 and indicator2 = "0"
q = FieldQuery().tag("650").indicator2("0")
for field in record.fields_matching(q):
    print(field)

# Fields whose subfield $a equals "History"
for field in record.fields_matching_value(
    SubfieldValueQuery("650", "a", "History")
):
    print(field)
```

### FieldQuery

::: mrrc.FieldQuery

### TagRangeQuery

::: mrrc.TagRangeQuery

### SubfieldPatternQuery

::: mrrc.SubfieldPatternQuery

### SubfieldValueQuery

::: mrrc.SubfieldValueQuery

## Format Conversion

### Record Methods

```python
# JSON formats
json_str = record.to_json()
marcjson_str = record.to_marcjson()

# pymarc-compatible serialization
json_str = record.as_json()     # pymarc MARC-in-JSON format
record_dict = record.as_dict()  # pymarc-compatible dict

# MARCXML
xml_str = record.to_xml()

# Other XML-based formats
mods_str = record.to_mods()
dc_str = record.to_dublin_core()

# Binary (ISO 2709)
marc_bytes = record.as_marc()   # returns bytes
marc_bytes = record.as_marc21() # alias
```

### Module Functions

```python
import mrrc

# Parse from JSON
record = mrrc.json_to_record(json_str)

# Parse from MARCXML
record = mrrc.xml_to_record(xml_str)

# Parse MARCXML collection (multiple records)
records = mrrc.xml_to_records(collection_xml_str)

# Parse from MODS XML
record = mrrc.mods_to_record(mods_xml)

# Parse MODS collection (multiple records)
records = mrrc.mods_collection_to_records(mods_collection_xml)

# Convert to CSV
csv_str = mrrc.record_to_csv(record)
csv_str = mrrc.records_to_csv(records)

# Convenience functions
records = mrrc.parse_xml_to_array(xml_str)
records = mrrc.parse_json_to_array(json_str)
mrrc.map_records(func, reader)
```

## Constants

```python
from mrrc import (
    LEADER_LEN,           # 24
    DIRECTORY_ENTRY_LEN,  # 12
    END_OF_FIELD,         # '\x1e'
    END_OF_RECORD,        # '\x1d'
    SUBFIELD_INDICATOR,   # '\x1f'
    MARC_XML_NS,          # MARCXML namespace URI
    MARC_XML_SCHEMA,      # MARCXML schema URI
)
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

::: mrrc.BibframeConfig

### RdfGraph

The RDF graph produced by [`marc_to_bibframe`](#bibframe-conversion). Use `len(graph)` for the triple count and `serialize(format)` to emit RDF in `"turtle"`, `"rdf-xml"`, `"jsonld"`, or `"ntriples"`.

::: mrrc.RdfGraph

## Parallel Processing

Lower-level utilities for splitting and parsing large MARC files across threads.
Most callers should use [`MARCReader`](#marcreader) with a `ThreadPoolExecutor`;
these are the building blocks underneath.

### RecordBoundaryScanner

::: mrrc.RecordBoundaryScanner

### ProducerConsumerPipeline

::: mrrc.ProducerConsumerPipeline

## Exceptions

```python
from mrrc import MrrcException

try:
    for record in MARCReader("bad.mrc", recovery_mode="strict"):
        pass
except MrrcException as e:
    print(f"MRRC error: {e}")
```

`MrrcException` is the base class for all mrrc errors; per-error-code subclasses
(`RecordLeaderInvalid`, `TruncatedRecord`, `FatalReaderError`, …) let you catch
specific failures, and `StaleFieldError` signals that a live field handle was
invalidated by field removal (re-fetch the field from the record and retry).

See the [error handling reference](error-handling.md) for the full exception
hierarchy, pymarc name mapping, and guidance on choosing what to catch.

## See Also

- [Python Quickstart](../getting-started/quickstart-python.md)
- [Migration from pymarc](../guides/migration-from-pymarc.md)
- [Format Support](formats.md)
