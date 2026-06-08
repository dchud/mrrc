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

**Properties (read-only):**

All record accessors are properties (not methods), matching pymarc:

| Property | Type | Description |
|----------|------|-------------|
| `leader` | `Leader` | The record's leader |
| `title` | `str \| None` | Title from 245$a |
| `author` | `str \| None` | Author from 100/110/111 |
| `isbn` | `str \| None` | ISBN from 020$a |
| `issn` | `str \| None` | ISSN from 022$a |
| `publisher` | `str \| None` | Publisher from 260$b |
| `pubyear` | `str \| None` | Publication year (returns str) |
| `subjects` | `list[str]` | All subjects from 6XX$a |
| `location` | `str \| None` | Location from 852$a |
| `notes` | `list[str]` | All notes from 5XX |
| `series` | `str \| None` | Series from 490$a |
| `uniform_title` | `str \| None` | Uniform title from 130$a |
| `physical_description` | `str \| None` | Extent from 300$a |
| `sudoc` | `str \| None` | SuDoc classification from 086$a |
| `issn_title` | `str \| None` | ISSN title from 222$a |
| `issnl` | `str \| None` | ISSN-L from 024$a |
| `addedentries` | `list[Field]` | Added entries (7XX fields) |
| `physicaldescription` | `str \| None` | Alias for `physical_description` |
| `uniformtitle` | `str \| None` | Alias for `uniform_title` |

**Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `add_control_field(tag, value)` | `None` | Add a control field (001-009) |
| `control_field(tag)` | `str \| None` | Get a control field value |
| `control_fields()` | `list[tuple[str, str]]` | Get all control fields |
| `add_field(*fields)` | `None` | Add one or more data fields |
| `add_ordered_field(*fields)` | `None` | Add fields in tag-sorted order |
| `add_grouped_field(*fields)` | `None` | Add fields after same tag group |
| `remove_field(*fields)` | `None` | Remove exactly the given fields (`Field` handle or value match; raises `ValueError` if absent); a tag string removes all fields with that tag |
| `remove_fields(*tags)` | `None` | Remove all fields matching tags (control tags included) |
| `fields()` | `list[Field]` | Get all fields (control + data), as live handles; enumerates identically to no-arg `get_fields()` |
| `fields_by_tag(tag)` | `list[Field]` | Get fields matching a tag, as live handles (single-tag, data fields only; `get_fields` also covers control fields) |
| `get(tag, default=None)` | `Field \| None` | Get first field (safe, returns None) |
| `get_field(tag)` | `Field \| None` | Get first field with given tag (returns `None` if absent) |
| `get_field_or_err(tag)` | `Field` | Get first field with given tag; raises [`FieldNotFound`](#exceptions) (E105) with `field_tag` and `record_control_number` populated when absent |
| `get_fields(*tags)` | `list[Field]` | Get fields for multiple tags |
| `as_marc()` | `bytes` | Serialize to ISO 2709 binary |
| `as_marc21()` | `bytes` | Alias for `as_marc()` |
| `as_json(**kwargs)` | `str` | Serialize to pymarc-compatible MARC-in-JSON |
| `as_dict()` | `dict` | Convert to pymarc-compatible dict |

**Dictionary access:**

```python
# record['xxx'] raises KeyError if tag is missing
field = record['245']      # Returns Field or raises KeyError

# Use record.get() for safe access (returns None if missing)
field = record.get('245')  # Returns Field or None
```

**Field handles:** fields obtained from a record (`record[tag]`,
`get_field`, `get_field_or_err`, `get_fields`, `fields`,
`fields_by_tag`) are live handles — every read and write goes through
to the record, so in-place edits persist, matching pymarc:

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

**Properties and Methods:**

| Property/Method | Type | Description |
|-----------------|------|-------------|
| `tag` | `str` | 3-character field tag |
| `indicator1` | `str` | First indicator |
| `indicator2` | `str` | Second indicator |
| `data` | `str \| None` | Control field content (None for data fields) |
| `is_control_field()` | `bool` | True for control fields (001-009) |
| `add_subfield(code, value, pos=None)` | `None` | Add a subfield (optional positional insert) |
| `subfields()` | `list[Subfield]` | Get all subfields |
| `subfields_by_code(code)` | `list[str]` | Get values for a subfield code |
| `get_subfields(*codes)` | `list[str]` | Get values for one or more subfield codes |
| `value()` | `str` | Space-joined subfield values |
| `format_field()` | `str` | Human-readable text representation |
| `as_marc()` | `bytes` | Serialize to ISO 2709 binary |
| `as_marc21()` | `bytes` | Alias for `as_marc()` |
| `linkage_occurrence_num()` | `tuple[str, str] \| None` | Extract $6 linkage info |
| `convert_legacy_subfields(tag, *args)` | `Field` | Classmethod: create from flat list |
| `__getitem__(code)` | `str \| None` | Get first subfield value by code |

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

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| `leader` | `Leader` | The 24-byte record leader |
| `errors` | `list` | Non-fatal errors accumulated while parsing this record (empty in `recovery_mode="strict"`); see [`Record.errors`](#record) |

**Heading and tracing accessors:**

| Method | Returns | Description |
|--------|---------|-------------|
| `heading()` | `Field \| None` | The main heading field (1XX), or `None` if absent |
| `heading_text()` | `str \| None` | Subfield `a` of the main heading, or `None` |
| `see_from_tracings()` | `list[Field]` | See-from tracings (4XX fields) |
| `see_also_tracings()` | `list[Field]` | See-also-from tracings (5XX fields) |
| `notes()` | `list[Field]` | Note fields: 6XX fields other than the subject tracings 650/651/655 |
| `linking_entries()` | `list[Field]` | Heading linking entries (7XX fields) |

**General field accessors:**

| Method | Returns | Description |
|--------|---------|-------------|
| `record_type()` | `str` | Leader byte 06 (type of record) as a single character |
| `get_field(tag)` | `Field \| None` | First field with given tag (returns `None` if absent) |
| `get_field_or_err(tag)` | `Field` | First field with given tag; raises [`FieldNotFound`](#exceptions) (E105) with `field_tag` and `record_control_number` populated when absent |
| `get_fields(tag)` | `list[Field] \| None` | All fields matching a tag (note: returns `None` when none match, unlike `Record.get_fields` which returns `[]`) |
| `get_control_field(tag)` | `str \| None` | Value of a control field (00X) by tag, or `None` |
| `to_json()` | `str` | JSON summary of the record |

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

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| `leader` | `Leader` | The 24-byte record leader |
| `errors` | `list` | Non-fatal errors accumulated while parsing this record (empty in `recovery_mode="strict"`); see [`Record.errors`](#record) |

**Holdings-specific accessors:**

| Method | Returns | MARC tag | Description |
|--------|---------|----------|-------------|
| `locations()` | `list[Field]` | 852 | Location fields |
| `captions_basic()` | `list[Field]` | 853 | Basic caption and pattern fields |
| `captions_supplements()` | `list[Field]` | 854 | Supplementary-material caption and pattern fields |
| `captions_indexes()` | `list[Field]` | 855 | Index caption and pattern fields |
| `enumeration_basic()` | `list[Field]` | 863 | Basic enumeration and chronology fields |
| `enumeration_supplements()` | `list[Field]` | 864 | Supplementary-material enumeration and chronology fields |
| `enumeration_indexes()` | `list[Field]` | 865 | Index enumeration and chronology fields |
| `textual_holdings_basic()` | `list[Field]` | 866 | Basic textual holdings fields |
| `textual_holdings_supplements()` | `list[Field]` | 867 | Supplementary-material textual holdings fields |
| `textual_holdings_indexes()` | `list[Field]` | 868 | Index textual holdings fields |

**General field accessors:**

| Method | Returns | Description |
|--------|---------|-------------|
| `record_type()` | `str` | Leader byte 06 (type of record) as a single character |
| `get_field(tag)` | `Field \| None` | First field with given tag (returns `None` if absent) |
| `get_field_or_err(tag)` | `Field` | First field with given tag; raises [`FieldNotFound`](#exceptions) (E105) with `field_tag` and `record_control_number` populated when absent |
| `get_fields(tag)` | `list[Field] \| None` | All fields matching a tag (note: returns `None` when none match, unlike `Record.get_fields` which returns `[]`) |
| `get_control_field(tag)` | `str \| None` | Value of a control field (00X) by tag, or `None` |
| `to_json()` | `str` | JSON summary of the record |

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
MARCReader(file_obj, to_unicode=True, permissive=False, recovery_mode="strict")
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
| `recovery_mode` | `str` | `"strict"` | Controls how malformed records are handled (see below). Cannot be combined with `permissive=True`. |

**Recovery Modes:**

Instead of skipping bad records entirely (like `permissive=True`), `recovery_mode` attempts to salvage valid fields from damaged records:

| Mode | Behavior |
|------|----------|
| `"strict"` | Raise on any malformation (default). |
| `"lenient"` | Attempt to recover, salvage valid fields from damaged records. |
| `"permissive"` | Very lenient — accept partial data even from severely malformed records. |

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

**Methods:**

| Method | Description |
|--------|-------------|
| `write(record)` | Write a single record |
| `close()` | Close the writer (automatic with context manager) |

### AuthorityMARCReader

Reads MARC **authority** records, yielding [`AuthorityRecord`](#authorityrecord). Same ISO 2709 binary format and iteration protocol as [`MARCReader`](#marcreader), with a smaller keyword set.

```python
AuthorityMARCReader(source, *, recovery_mode="permissive", validation_level="structural")
```

```python
from mrrc import AuthorityMARCReader

# From a path, bytes, or file object — like MARCReader
for record in AuthorityMARCReader("authorities.mrc"):
    print(record.heading_text())
```

**Keyword Arguments:**

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `recovery_mode` | `str` | `"permissive"` | How malformed records are handled: `"strict"`, `"lenient"`, or `"permissive"` (see [MARCReader recovery modes](#marcreader)). Note the default differs from `MARCReader` (`"strict"`). |
| `validation_level` | `str` | `"structural"` | What counts as an error during parsing. |

### HoldingsMARCReader

Reads MARC **holdings** records, yielding [`HoldingsRecord`](#holdingsrecord). Same shape as `AuthorityMARCReader`.

```python
HoldingsMARCReader(source, *, recovery_mode="permissive", validation_level="structural")
```

```python
from mrrc import HoldingsMARCReader

for record in HoldingsMARCReader("holdings.mrc"):
    for location in record.locations():
        print(location)
```

**Keyword Arguments:**

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `recovery_mode` | `str` | `"permissive"` | How malformed records are handled: `"strict"`, `"lenient"`, or `"permissive"` (see [MARCReader recovery modes](#marcreader)). Note the default differs from `MARCReader` (`"strict"`). |
| `validation_level` | `str` | `"structural"` | What counts as an error during parsing. |

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
from mrrc import MrrcException, MarcError

try:
    for record in MARCReader("bad.mrc"):
        pass
except MrrcException as e:
    print(f"MRRC error: {e}")
except MarcError as e:
    print(f"MARC error: {e}")
```

The exception hierarchy:

- `MrrcException` — base exception for all mrrc errors
  - `MarcError` — MARC-specific errors (parsing, validation)
  - `StaleFieldError` — a live field handle was invalidated by field
    removal; re-fetch the field from the record and retry

## See Also

- [Python Quickstart](../getting-started/quickstart-python.md)
- [Migration from pymarc](../guides/migration-from-pymarc.md)
- [Format Support](formats.md)
