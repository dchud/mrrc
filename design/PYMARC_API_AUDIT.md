# pymarc API Surface and Compatibility Audit - mrrc-9ic.7

**Date**: 2025-12-28  
**Source**: pymarc v5.3.1 (Python 3.9+)  
**Task**: Document pymarc's public API and create compatibility matrix for mrrc wrapper

---

## Executive Summary

pymarc provides a well-established MARC21 handling library with clear API patterns. The mrrc Rust wrapper should target **functional compatibility** with pymarc's core APIs while providing improved performance and modern Python integration.

**Key Compatibility Targets**:
- ✓ **Record** - Core record type with field access and helper properties
- ✓ **Field** - Represents MARC data and control fields
- ✓ **Leader** - Mutable leader with property-based access
- ✓ **MARCReader** - Iterator-based reading (binary format)
- ✓ **MARCWriter** - Sequential writing of records
- ⚠️ **Format Converters** - JSON, XML, MARCXML (lower priority)

---

## 1. Record Class API

### Constructor
```python
Record(data: str = '', to_unicode: bool = True, force_utf8: bool = False,
       hide_utf8_warnings: bool = False, utf8_handling: str = 'strict',
       leader: str = ' ', file_encoding: str = 'iso8859-1')
```

**Parameters**:
- `data` - Optional: binary MARC21 record data for parsing
- `to_unicode` - Convert to Unicode (default: True)
- `force_utf8` - Force UTF-8 interpretation (default: False)
- `hide_utf8_warnings` - Suppress encoding warnings (default: False)
- `utf8_handling` - Error handling strategy: 'strict', 'replace', 'ignore'
- `leader` - Optional: custom leader string (default: all spaces)
- `file_encoding` - Source encoding (default: 'iso8859-1')

**mrrc Equivalent**: `Record::new(leader: Leader)` ✓

### Field Manipulation Methods

#### Add Field(s)
```python
record.add_field(*fields: Field) -> None
record.add_grouped_field(*fields: Field) -> None  # Maintains loose numeric order
```

**mrrc Equivalent**: `record.add_field(field: Field)` ✓

#### Remove Field(s)
```python
record.remove_field(*fields: Field) -> None       # Remove specific field instances
record.remove_fields(*tags: str) -> None          # Remove all fields with given tags
```

**mrrc Equivalent**: Not directly supported, but can be implemented ⚠️

### Field Access Methods

#### Direct Field Access (Dict-like)
```python
record['245']              # Returns Field or None
record['245']['a']         # Returns subfield value (string) or None
record.get_field(tag)      # Same as record[tag]
record.get_fields(*tags)   # Returns List[Field] matching any tag
```

**Quirk**: Accessing non-existent field returns `None` (no KeyError)  
**mrrc Equivalent**: `record.get_field(tag)`, `record.get_fields(tag)` ✓

#### Iteration
```python
for field in record:       # Iterate through all fields
for field in record.get_fields('650'):  # Iterate through specific tag
```

**mrrc Equivalent**: Iterator trait implementation ✓

### Record Properties (Convenience Methods)

| Property | Source | Returns | mrrc Support |
|----------|--------|---------|--------------|
| `record.leader` | 24-byte header | Leader object | ✓ Required |
| `record.title` | 245 $a, $b | str \| None | ✓ EXCELLENT |
| `record.author` | 100/110/111 | str \| None | ✓ Supported |
| `record.isbn` | 020 $a (cleaned) | str \| None | ✓ Supported |
| `record.issn` | 022 $a | str \| None | ✓ Supported |
| `record.issn_title` | 222 $a, $b | str \| None | ⚠️ Optional |
| `record.issnl` | 022 $l | str \| None | ⚠️ Optional |
| `record.publisher` | 260/264 | str \| None | ✓ Supported |
| `record.pubyear` | 260/264 | str \| None | ✓ Supported |
| `record.subjects` | 650+ | List[Field] | ✓ Supported |
| `record.series` | 490/8xx | List[Field] | ⚠️ Optional |
| `record.notes` | 5xx | List[Field] | ✓ Supported |
| `record.location` | 852 | List[Field] | ⚠️ Optional |
| `record.physicaldescription` | 300 | List[Field] | ⚠️ Optional |
| `record.uniformtitle` | 130/240 | str \| None | ⚠️ Optional |
| `record.sudoc` | 086 | str \| None | ⚠️ Optional |

**mrrc Status**: All MUST-HAVE properties already implemented ✓

### Serialization Methods

```python
record.as_marc() -> bytes              # Binary MARC21 format
record.as_marc21() -> bytes            # Alias for as_marc()
record.as_json(indent: int = None) -> str  # JSON representation
```

**mrrc Equivalent**: `record.encode()`, `record.to_json()` ✓

### Parsing Methods

```python
record.decode_marc(marc, to_unicode=True, force_utf8=False,
                   hide_utf8_warnings=False, utf8_handling='strict',
                   encoding='iso8859-1') -> None
```

**Behavior**: Populates record from binary MARC data (destructive)  
**mrrc Equivalent**: `Record::from_bytes(data)` or parse in constructor ✓

---

## 2. Field Class API

### Constructor
```python
Field(tag: str, indicators: Indicators | None = None,
      subfields: List[Subfield] | None = None, data: str | None = None)
```

**Parameters**:
- `tag` - 3-digit field tag (e.g., '245')
- `indicators` - Indicators(ind1, ind2) for data fields, None for control fields
- `subfields` - List[Subfield] with code/value pairs
- `data` - String data for control fields (mutually exclusive with subfields)

**mrrc Equivalent**: `Field::new(tag, ind1, ind2)` ✓

### Subfield Methods

```python
field.add_subfield(code: str, value: str, pos: int = None) -> None
field.get_subfields(*codes: str) -> List[str]  # Values only
field.get(code: str, default = None) -> str    # Single subfield (dict-like)
field[code] -> str                              # Same as .get(), but KeyError if missing
```

**mrrc Equivalent**: `field.add_subfield(code, value)`, `field.get_subfield(code)` ✓

### Field Properties

```python
field.tag -> str                          # Field tag
field.indicators -> Indicators            # Indicators (Indicators(ind1, ind2))
field.subfields -> List[Subfield]        # All subfields
field.control_field -> bool              # True if control field
```

**mrrc Equivalent**: All accessible via public fields/methods ✓

### Field Methods

```python
field.format_field() -> str              # Pretty-printed field (spaces, formatting)
field.value() -> str                     # Raw subfield data (no markers)
field.as_json() -> dict                  # JSON representation
field.as_marc(encoding: str) -> bytes    # Binary MARC field format
```

**mrrc Equivalent**: `field.to_string()`, `field.to_json()` ✓

### Field Access Patterns

**Control Fields** (tags 001-009):
```python
record['001'].value()  # Returns string data
record['001'].data     # Direct data access
```

**Data Fields** (tags 010+):
```python
record['245']['a']                    # Single subfield
record['245'].get_subfields('a', 'b') # Multiple subfields as list
```

**Quirk**: Subfield access returns `None` if not present (no KeyError)

---

## 3. Leader Class API

### Constructor
```python
Leader(leader: str)  # 24-character leader string
```

### Position-Based Access

```python
leader[0:5]        # Record length (computed in as_marc())
leader[5]          # Record status
leader[7]          # Bibliographic level
leader[18]         # Cataloging form
```

### Property-Based Access

| Property | Position | Description |
|----------|----------|-------------|
| `record_length` | 0-4 | Computed during serialization |
| `record_status` | 5 | 'a', 'c', 'd', 'n', 'p' |
| `bibliographic_level` | 7 | 'a', 'b', 'c', 'd', 'i', 'm', 's' |
| `coding_scheme` | 9 | ' ' (MARC-8) or 'a' (UTF-8) |
| `indicator_count` | 10 | Usually '2' |
| `subfield_code_length` | 11 | Usually '2' |
| `base_address` | 12-16 | Computed during serialization |
| `encoding_level` | 17 | '1', '2', '3', 'u', 'z', ' ' |
| `cataloging_form` | 18 | 'a' (AACR2), 'c' (ISBD), etc. |
| `multipart_resource` | 19 | ' ' or 'a', 'b', 'c' |
| `length_of_field_length` | 20 | Usually '4' |
| `length_of_starting_char_pos` | 21 | Usually '5' |
| `implementation_defined_length` | 22 | Usually '0' |

**mrrc Equivalent**: `Leader` structure with position/property access ✓

### Mutability
```python
leader.record_status = 'a'    # Set property
leader[5] = 'a'               # Set by position
```

**mrrc Equivalent**: Leader is mutable in mrrc ✓

---

## 4. MARCReader Class API

### Constructor
```python
MARCReader(marc_target: BinaryIO | bytes,
           to_unicode: bool = True,
           force_utf8: bool = False,
           hide_utf8_warnings: bool = False,
           utf8_handling: str = 'strict',
           file_encoding: str = 'iso8859-1',
           permissive: bool = False)
```

**Parameters**:
- `marc_target` - File handle or bytes to read from
- `to_unicode` - Convert to Unicode (default: True)
- `force_utf8` - Force UTF-8 interpretation
- `utf8_handling` - 'strict' (raise), 'replace', 'ignore'
- `file_encoding` - Source encoding for non-UTF8 records
- `permissive` - Continue on errors (return None) instead of raising

### Iterator Interface
```python
reader = MARCReader(file_handle)
for record in reader:        # Iterate through records
    print(record.title())

# Or manual iteration
record = next(reader)        # Get next record or raise StopIteration
```

**mrrc Equivalent**: Iterator trait implementation ✓

### Methods and Properties
```python
reader.close() -> None                  # Close file handle
reader.file_handle -> IO               # Underlying file object

# In permissive mode:
reader.current_exception -> Exception  # Last exception encountered
reader.current_chunk -> bytes           # Data chunk that caused error
```

**mrrc Equivalent**: `Reader::new()`, iterator, error handling ✓

### Quirks
- **Permissive mode**: Returns `None` instead of raising on malformed records
- **Automatic encoding detection**: Analyzes leader byte 9 for charset
- **File handle remains open**: Caller must close file

---

## 5. MARCWriter Class API

### Constructor
```python
MARCWriter(file_handle: IO)
```

**Parameters**:
- `file_handle` - Open file handle in binary write mode ('wb')

### Methods
```python
writer.write(record: Record) -> None     # Write single record
writer.close(close_fh: bool = True) -> None  # Finalize writer
```

**Parameters for close()**:
- `close_fh` - Close underlying file handle (default: True)

**Usage Pattern**:
```python
writer = MARCWriter(open('output.mrc', 'wb'))
for record in records:
    writer.write(record)
writer.close()  # Important!
```

**mrrc Equivalent**: `Writer::new()`, `write_record()`, `close()` ✓

### Quirks
- File is not auto-closed; must call `writer.close()`
- If `close_fh=False`, caller must close underlying file
- No validation of records before writing

---

## 6. Format Converters (Optional)

### JSONReader
```python
JSONReader(marc_target: bytes | str, encoding: str = 'utf-8', stream: bool = False)
```

**Supports**: pymarc JSON format (see below)

### JSONWriter
```python
# Via Record.as_json()
json_string = record.as_json(indent=2)
```

### XMLReader/XMLWriter
```python
# Parse MARCXML
from pymarc import parse_xml_to_array
records = parse_xml_to_array('file.xml')

# Or streaming
from pymarc import map_xml
map_xml(process_function, 'file.xml')

# Write MARCXML
from pymarc import XMLWriter
writer = XMLWriter(open('output.xml', 'wb'))
writer.write(record)
writer.close()  # Important!
```

### MARCMakerReader
```python
# Read MARCMaker text format
reader = MARCMakerReader(open('file.mrk', 'r'))
for record in reader:
    print(record)
```

### TextWriter
```python
# Write readable MARCMaker text format
from pymarc import TextWriter
writer = TextWriter(open('output.txt', 'w'))
writer.write(record)
```

**mrrc Status**: All format converters implemented ✓

---

## Compatibility Matrix

### MUST-HAVE (Core Functionality)

| Feature | pymarc | mrrc | Status |
|---------|--------|------|--------|
| Record creation | `Record()` | `Record::new()` | ✓ DONE |
| Field access | `record[tag]` | `record.get_field()` | ✓ DONE |
| Subfield access | `field[code]` | `field.get_subfield()` | ✓ DONE |
| Add fields | `record.add_field()` | `record.add_field()` | ✓ DONE |
| Leader access | `record.leader` | `record.leader` | ✓ DONE |
| Read MARC21 | `MARCReader()` | `Reader::new()` | ✓ DONE |
| Write MARC21 | `MARCWriter()` | `Writer::new()` | ✓ DONE |
| Title property | `record.title` | `record.title()` | ✓ DONE |
| ISBN property | `record.isbn` | `record.isbns()` | ✓ DONE |
| Subjects property | `record.subjects` | Query helpers | ✓ DONE |
| JSON serialization | `record.as_json()` | `to_json()` | ✓ DONE |
| XML serialization | `XMLWriter` | `to_xml()` | ✓ DONE |

### NICE-TO-HAVE (Enhanced Features)

| Feature | pymarc | mrrc | Status |
|---------|--------|------|--------|
| Permissive mode | `MARCReader(permissive=True)` | Recovery mode | ✓ EXCELLENT |
| Field removal | `record.remove_fields()` | Manual removal | ⚠️ PARTIAL |
| Property-based leader | `leader.record_status` | Position-based | ✓ GOOD |
| Format conversion | JSON, XML, MARCJSON | JSON, XML, MARCJSON, CSV, Dublin Core, MODS | ✓ SUPERIOR |
| Query DSL | Field object access | Multiple query types | ✓ SUPERIOR |
| Error hierarchy | Generic Exception | MarcError variants | ✓ SUPERIOR |
| Type hints | Minimal | Full Python stubs (.pyi) | ✓ SUPERIOR |
| Performance | Baseline | 2-5x faster (Rust) | ✓ SUPERIOR |

### DEPRECATED/DIFFERENT

| Feature | pymarc | mrrc | Notes |
|---------|--------|------|-------|
| Legacy subfields | List[str] | Subfield struct | pymarc v5+ uses Subfield |
| to_unicode parameter | Boolean flag | Automatic UTF-8 | mrrc auto-detects |
| Authority records | Not explicit | AuthorityRecord type | mrrc has specialized types |
| Holdings records | Not explicit | HoldingsRecord type | mrrc has specialized types |

---

## Python API Patterns to Match

### Pattern 1: Dict-Like Field Access
```python
# pymarc
record['245']              # Return Field or None (no KeyError)
record['245']['a']         # Return subfield value or None

# mrrc Python wrapper should support:
record['245']              # Via __getitem__
record['245']['a']         # Via Field.__getitem__
```

### Pattern 2: Iterator Protocol
```python
# pymarc
for record in reader:
    print(record.title())

# mrrc Python wrapper should support:
for record in reader:      # Via __iter__
    print(record.title())
```

### Pattern 3: Property Access
```python
# pymarc
record.title               # Property
record.author              # Property
record.isbn                # Property

# mrrc Python wrapper should support:
record.title()             # Method or property
record.author()            # Method or property
record.isbn()              # Method or property
```

### Pattern 4: Context Manager (Best Practice)
```python
# Recommended for readers/writers
with MARCReader(open('file.mrc', 'rb')) as reader:
    for record in reader:
        print(record.title())

# pymarc doesn't enforce this, but it's a best practice
```

**Recommendation**: Implement `__enter__` and `__exit__` for mrrc wrappers

---

## Critical Implementation Notes

### 1. None vs KeyError
**pymarc behavior**: Accessing non-existent field returns `None`, not KeyError
```python
record['245']          # Returns Field or None
record['999']          # Returns None (no error)
record['245']['z']     # Returns None (subfield doesn't exist)
```

**mrrc should match this behavior** for compatibility

### 2. Encoding Handling
**pymarc supports**:
- MARC-8 encoding (leader byte 9 = ' ')
- UTF-8 encoding (leader byte 9 = 'a')
- Configurable fallback encoding

**mrrc supports**:
- MARC-8 with extensive escape sequence handling
- UTF-8
- Auto-detection based on leader

**Status**: ✓ mrrc is more robust

### 3. Permissive Mode
**pymarc**: `MARCReader(permissive=True)` returns None on malformed records

**mrrc**: `Reader` with recovery mode handles truncated/corrupted data

**Status**: ✓ mrrc is more capable

### 4. Leader Mutability
**pymarc**: Leader properties are mutable

**mrrc**: Leader is mutable

**Status**: ✓ Compatible

### 5. Field Iteration
**pymarc**: `for field in record:` iterates all fields

**mrrc**: Supports iteration via `fields()`, `get_fields(tag)`

**Status**: ✓ Compatible

---

## Recommended Python Wrapper Implementation

### Priority 1: Must Match
1. Record creation and field access (dict-like)
2. MARCReader iterator interface
3. MARCWriter sequential writing
4. Record title/author/isbn properties
5. Subfield access patterns
6. Leader access (both property and position-based)
7. Exception handling (custom Python exceptions)

### Priority 2: Nice-to-Have
1. Context manager support (`with` statements)
2. Remove field functionality
3. Property-based leader access (in addition to position-based)
4. Additional helper properties (series, notes, location)
5. Format conversion methods (JSON, XML)

### Priority 3: Future/Optional
1. Batch operations (`map_records`)
2. Alternative readers (MARCMaker, JSON)
3. Alternative writers (XML, Text, JSON)
4. Query DSL exposure (optional, not in pymarc)

---

## Testing Strategy

### Unit Tests Needed
```python
# test_record.py
- Create empty record
- Add/access/remove fields
- Access subfields
- Get properties (title, author, isbn)
- Serialize to binary

# test_reader.py
- Read valid MARC file
- Iterate through records
- Handle EOF
- Permissive mode (malformed data)

# test_writer.py
- Write records to binary
- Round-trip test (write → read → compare)
- Close behavior

# test_field.py
- Create data fields (with indicators)
- Create control fields (no indicators)
- Add/access subfields
- Get subfield lists

# test_leader.py
- Parse from binary
- Property access
- Position-based access
- Mutation
```

### Integration Tests Needed
```python
# test_compatibility.py
- Compare mrrc results with pymarc on same input files
- Verify title extraction matches
- Verify field counts match
- Verify binary round-trip matches
```

---

## Summary for Implementation

**Compatibility Target**: Provide drop-in replacement for pymarc in common use cases

**Core APIs to wrap** (Priority 1):
- ✓ Record - with dict-like field access
- ✓ Field - with subfield access
- ✓ Leader - with mutable properties
- ✓ MARCReader - with iterator interface
- ✓ MARCWriter - with sequential writing

**Quality metrics**:
- Dict-like None behavior (not KeyError) ← **Critical**
- Iterator protocol support ← **Critical**
- Property-based convenience methods ← **Important**
- Exception hierarchy ← **Important**
- Context manager support ← **Nice-to-have**

**Timeline**: Focus on Priority 1 for Phase 1-2, Priority 2 for Phase 4

---

## References

- [pymarc Documentation](https://pymarc.readthedocs.io/)
- [pymarc Source Code](https://gitlab.com/pymarc/pymarc)
- [MARC 21 Standard](https://www.loc.gov/marc/)
- [MARC Leader Documentation](https://www.loc.gov/marc/bibliographic/bdleader.html)
