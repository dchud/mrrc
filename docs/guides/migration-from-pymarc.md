# Migration Guide: pymarc to mrrc

This guide helps existing pymarc users migrate to mrrc, a high-performance Rust-based MARC library with Python bindings.

## Overview

**mrrc** is a Rust-based MARC library with Python bindings, providing:

- **High performance** Rust implementation with Python convenience
- **Full pymarc API compatibility** - Drop-in replacement for existing pymarc code
- **Type-safe** design with comprehensive error handling
- **Native Python integration** through PyO3 bindings with familiar data structures
- **All standard MARC operations** including reading, writing, and format conversions

## Installation

```bash
pip install mrrc
```

## Quick Start

### Before (pymarc)
```python
import pymarc

# Reading records
with open('records.mrc', 'rb') as f:
    reader = pymarc.MARCReader(f)
    for record in reader:
        print(record['245']['a'])

# Writing records
writer = pymarc.MARCWriter(open('output.mrc', 'wb'))
field = pymarc.Field('245', ['1', '0'], [('a', 'Title')])
record = pymarc.Record(to_utf8=True)
record.add_field(field)
writer.write(record)
writer.close()
```

### After (mrrc) - pymarc-Compatible

**mrrc supports nearly identical pymarc syntax:**

```python
import mrrc

# Reading records — pass a file path for best performance.
# Path input uses Rust-native file I/O, which releases the GIL
# and enables true multi-thread parallelism.
reader = mrrc.MARCReader('records.mrc')
for record in reader:
    print(record['245']['a'])  # pymarc dictionary syntax works!
    print(record.title)        # Property access (same as pymarc)

# Writing records - inline construction (similar to pymarc)
with open('output.mrc', 'wb') as f:
     with mrrc.MARCWriter(f) as writer:
         record = mrrc.Record(fields=[
             mrrc.Field('245', indicators=['1', '0'], subfields=[
                 mrrc.Subfield('a', 'Title'),
             ]),
         ])
         writer.write(record)
```

## API Comparison

### Record Creation

| Operation | pymarc | mrrc | Same? |
|-----------|--------|------|-------|
| Create empty record | `pymarc.Record()` | `mrrc.Record()` | **Same** |
| Create with leader | `pymarc.Record(leader)` | `mrrc.Record(leader)` | **Same** |
| Add control field | `record.add_field(Field('001', data='value'))` | `record.add_control_field('001', 'value')` or `record.add_field(Field('001', data='value'))` | Similar |
| Add data field | `record.add_field(field)` | `record.add_field(field)` | **Same** |

### Field Creation

| Operation | pymarc | mrrc | Same? |
|-----------|--------|------|-------|
| Create field | `Field('245', ['1','0'], [('a', 'Title')])` | `Field('245', indicators=['1','0'], subfields=[Subfield('a', 'Title')])` | Similar |
| Create control field | `Field('001', data='12345')` | `Field('001', data='12345')` | **Same** |
| Add subfield | `field.add_subfield('a', 'value')` | `field.add_subfield('a', 'value')` | **Same** |
| Add subfield at position | `field.add_subfield('a', 'value', pos=2)` | `field.add_subfield('a', 'value', pos=2)` | **Same** |
| Get subfields | `field.get_subfields('a')` | `field.get_subfields('a')` | **Same** |
| Access subfield | `field['a']` | `field['a']` | **Same** |

### Reading/Writing

| Operation | pymarc | mrrc | Same? |
|-----------|--------|------|-------|
| Create reader | `MARCReader(file_obj)` | `MARCReader('path.mrc')` (recommended) or `MARCReader(file_obj)` | Enhanced |
| Permissive mode | `MARCReader(f, permissive=True)` | `MARCReader(f, permissive=True)` | **Same** |
| Unicode flag | `MARCReader(f, to_unicode=True)` | `MARCReader(f, to_unicode=True)` | **Same** |
| Read record | `reader.next()` or `next(reader)` | `next(reader)` | **Same** |
| Write record | `writer.write(record)` | `writer.write(record)` | **Same** |
| Iterate | `for record in reader:` | `for record in reader:` | **Same** |
| Context manager | Manual close required | `with MARCWriter(f) as w:` | Enhanced |

### Accessing Data

| Operation | pymarc | mrrc | Same? |
|-----------|--------|------|-------|
| Get title | `record.title` | `record.title` | **Same** |
| Get field | `record['650']` | `record['650']` (first) or `record.get_fields('650')` (all) | **Same** |
| Check if field exists | `'245' in record` | `'245' in record` | **Same** |
| Get all fields | `for field in record:` | `for field in record:` | **Same** |
| Control field data | `record['001'].data` | `record['001'].data` or `record.control_field('001')` | **Same** |
| Missing field | `record['999']` raises KeyError | `record['999']` raises KeyError | **Same** |
| Safe field access | `record.get('999')` returns None | `record.get('999')` returns None | **Same** |

## API Compatibility

**mrrc provides excellent pymarc API compatibility** with support for all major operations:

### Record Field Access - Dictionary-Style (Identical to pymarc)
```python
# Dictionary-style access works exactly like pymarc
field = record['245']                      # Get first 245 field (raises KeyError if missing)
all_fields = record.get_fields('245')      # Get all 245 fields

# Safe access with .get() (returns None if missing)
field = record.get('245')                  # Get first field, None if missing
field = record.get('999', default_field)   # With default value

# Check if field exists (identical to pymarc)
if '245' in record:
    title_field = record['245']
```

### Field Subfield Access - Dictionary-Style (Identical to pymarc)
```python
# Dictionary-style access works exactly like pymarc
title = field['a']                          # Get first 'a' subfield
if 'a' in field:
    value = field['a']

# Missing subfields return None (matching pymarc behavior)
value = field['z']                          # Returns None if subfield doesn't exist
                                            # (does NOT raise KeyError)

# Get all values for a code
all_subfields = field.get_subfields('a')   # Get list of 'a' subfield values

# Iterate over all subfields
for subfield in field.subfields():
    print(f"{subfield.code}: {subfield.value}")

# Get subfields as dictionary
subfield_dict = field.subfields_as_dict()
```

### Field Operations (Identical to pymarc)
```python
field.add_subfield('a', 'value')       # Identical to pymarc
field.add_subfield('a', 'val', pos=2)  # Positional insert
field.get_subfields('a')               # Get list of values - identical to pymarc
field.delete_subfield('a')             # Delete subfield by code
field.subfields_as_dict()              # Get all subfields as dict
field.subfields()                      # Get all Subfield objects
field.is_control_field()               # False for data fields (identical to pymarc)
field.value()                          # Space-joined subfield values
field.format_field()                   # Human-readable field text
```

### Record Operations (Identical to pymarc + Extensions)
```python
# Standard pymarc operations
record.add_field(field1, field2)       # Add one or more fields
record.remove_field(field1, field2)    # Remove specific field objects
record.remove_fields('245', '650')    # Remove all fields with matching tags
record.add_ordered_field(field)        # Insert in tag-sorted position
record.add_grouped_field(field)        # Insert after same-tag group
record.add_field(field)                # Add field (accepts multiple: add_field(f1, f2, f3))
record.get_fields('650', '651')        # Get fields for multiple tags

# Record accessors (all are @property, matching pymarc)
record.title                           # Get title (245 $a)
record.author                          # Get author (100/110/111 $a)
record.isbn                            # Get ISBN (020 $a)
record.issn                            # Get ISSN (022 $a)
record.subjects                        # Get all subjects (6XX $a)
record.publisher                       # Get publisher (260 $b)
record.physical_description            # Get extent (300 $a)
record.series                          # Get series (490 $a)
record.pubyear                         # Get publication year (str, not int)
record.notes                           # Get all notes (5XX)
record.location                        # Get location (852 $a)
record.uniform_title                   # Get uniform title (130 $a)
record.sudoc                           # Get SuDoc classification (086 $a)
record.issn_title                      # Get ISSN title (222 $a)
record.issnl                           # Get ISSN-L (024 $a)
record.addedentries                    # Get added entries (7XX fields)

# Serialization (pymarc-compatible)
record.as_marc()                       # ISO 2709 bytes
record.as_json()                       # pymarc MARC-in-JSON string
record.as_dict()                       # pymarc-compatible dict
```

For field selection beyond `get_fields(*tags)` — matching on indicators, tag
ranges, subfield presence, or a regex over subfield values — see the
[Query DSL guide](query-dsl.md). It's an mrrc extension with no pymarc
equivalent.

### Control Fields (Unified with Field)
```python
# Control fields are now Field instances (matching pymarc)
cf = Field('001', data='12345')
print(cf.data)                  # '12345'
print(cf.is_control_field())    # True
print(isinstance(cf, Field))    # True

# ControlField still works as backward-compatible alias
from mrrc import ControlField
cf = ControlField('001', '12345')
print(cf.data)                  # '12345'
```

### Leader Access - Attribute-Based and Position-Based
```python
# Attribute-based access. Note: mrrc exposes the leader as a method call,
# record.leader(), where pymarc uses a record.leader attribute.
leader = record.leader()
leader.record_status = 'c'          # Set record status
leader.record_type = 'a'            # Set record type
leader.bibliographic_level = 'd'    # Set bibliographic level

# Position-based access (also available for pymarc compatibility)
leader[5] = 'c'                     # Set record status at position 5
leader[6] = 'a'                     # Set record type at position 6

# Slice access to get multiple positions
record_length = int(leader[0:5])    # Get first 5 chars (record length)
cataloging_form = leader[18]        # Get cataloging form char at position 18

# Position and property access are automatically synchronized
leader.record_status = 'd'
assert leader[5] == 'd'             # Position-based access reflects property change
```

### Reader/Writer Interface
```python
# Reading — pass a path string or pathlib.Path for best performance.
# This uses Rust-native file I/O, which releases the Python GIL during
# parsing and enables true multi-thread parallelism.
reader = mrrc.MARCReader('records.mrc')
for record in reader:              # Standard iteration
    print(record.title)

# Python file objects and in-memory bytes also work, but hold the GIL
# during reads, so they won't benefit from multi-threading.
with open('records.mrc', 'rb') as f:
    reader = mrrc.MARCReader(f)         # Works, but slower under threading
reader = mrrc.MARCReader(marc_bytes)    # Also works for in-memory data

# Writing (identical to pymarc, with context manager support)
with mrrc.MARCWriter(f) as writer:
    writer.write(record)           # Same method name as pymarc
```

## Minimal API Differences

**mrrc is nearly 100% compatible with pymarc.** Here are the only two required changes:

### 1. Record Constructor
`Record()` now works with no arguments (leader defaults to `Leader()`):
```python
# pymarc
record = pymarc.Record()

# mrrc - both work
record = mrrc.Record()                  # Default leader
record = mrrc.Record(mrrc.Leader())     # Explicit leader

# Note: Once created, all field access works identically
print(record['245']['a'])  # Works exactly like pymarc
```

### 2. Optional: Extended Convenience Properties
mrrc extends pymarc with additional convenience properties:
```python
# All pymarc properties work:
record.title               # Get title
record.author              # Get author
record.isbn                # Get ISBN

# Plus many additional properties:
record.issn                # Get ISSN
record.issn_title          # Get ISSN title
record.sudoc               # Get SuDoc classification
record.issnl               # Get ISSN-L
record.pubyear             # Get publication year (str)
record.physical_description  # Get extent/pages
record.is_book()           # Check if book
record.is_serial()         # Check if serial
record.is_music()          # Check if music
```

## New Features Beyond pymarc

### Serialization Methods
```python
record.as_marc()           # ISO 2709 bytes
record.as_json()           # pymarc-compatible MARC-in-JSON
record.as_dict()           # pymarc-compatible dict
field.as_marc()            # Field-level binary
field.value()              # Space-joined subfield values
field.format_field()       # Human-readable text
```

### Module-Level Functions
```python
import mrrc

records = mrrc.parse_xml_to_array(xml_str)
records = mrrc.parse_json_to_array(json_str)
mrrc.map_records(func, reader)
```

### Constants
```python
from mrrc import LEADER_LEN, END_OF_FIELD, END_OF_RECORD, SUBFIELD_INDICATOR
```

### Exception Hierarchy
```python
from mrrc import MrrcException
```

See the [error handling reference](../reference/error-handling.md) for the full
hierarchy and the pymarc exception name mapping.

## Migration Checklist

**Minimal changes needed:**

- [ ] Replace `import pymarc` with `import mrrc`
- [ ] Update record creation: `pymarc.Record()` to `mrrc.Record()` (or `mrrc.Record(mrrc.Leader())`)
- [ ] Update field creation to use `indicators=` and `subfields=` kwargs if desired
- [ ] Everything else works the same - dictionary access, property names, iteration all identical

**Optional enhancements:**

- [ ] Pass file paths to `MARCReader('file.mrc')` instead of file objects (releases the GIL, enables multi-thread parallelism)
- [ ] Use additional convenience properties like `record.issn`, `record.sudoc`, etc. for specialized use cases
- [ ] Update writers to use context managers: `with mrrc.MARCWriter(f) as w:` (better resource management)
- [ ] Use `record.as_marc()`, `record.as_json()`, `record.as_dict()` for serialization

## Error Handling

### Permissive Mode (pymarc-compatible)

pymarc's `permissive=True` flag yields `None` for records that fail to parse,
letting callers skip bad records and keep processing. mrrc supports the same
flag with identical behavior:

```python
# Works the same in both pymarc and mrrc
for record in mrrc.MARCReader('records.mrc', permissive=True):
    if record is None:
        continue  # skip malformed record
    print(record.title)
```

After each iteration step, two pymarc-compatible accessors carry diagnostic
information about what was just read:

- `reader.current_chunk` — the bytes of the record that was just read from
  the source. Set on every successful chunk read, whether the parse then
  succeeded or failed. The byte count matches the record's leader-declared
  length.
- `reader.current_exception` — the typed `MrrcException` swallowed by the
  permissive read (`None` on a clean read).

```python
reader = mrrc.MARCReader('records.mrc', permissive=True)
for record in reader:
    if record is None:
        log.warning(
            "skipped malformed record (%d bytes): %s",
            len(reader.current_chunk) if reader.current_chunk else 0,
            reader.current_exception,
        )
        continue
    print(record.title)
```

For pymarc-equivalent error handling, use `permissive=True`. Two
documented differences from pymarc's defaults:

- **Encoding strictness:** mrrc raises `EncodingError` (and swallows it via
  `current_exception` under `permissive=True`) on invalid UTF-8 in subfield
  values; pymarc applies lossy substitution silently. The shape of the
  iteration is unchanged (the bad record yields as `None` either way), so
  callers using `except Exception:` keep working.
- **`current_chunk` on byte-read errors:** When the underlying read of the
  next record's bytes fails before parsing begins (truncated stream, I/O
  error), `current_chunk` may be `None` even though `current_exception` is
  set. For parse failures of fully-read chunks (the common case),
  `current_chunk` carries the full record bytes as pymarc does.

### to_unicode Flag

pymarc's `to_unicode=True` (the default) converts MARC-8 encoded records to
UTF-8. mrrc always converts MARC-8 to UTF-8 automatically — the conversion
happens in the Rust parsing layer and cannot be disabled. The `to_unicode`
kwarg is accepted for compatibility so existing scripts work unchanged.
Passing `to_unicode=False` emits a warning but has no effect.

### Recovery Mode (mrrc-specific)

mrrc also offers a `recovery_mode` kwarg that goes beyond pymarc's permissive mode. Instead of skipping bad records entirely, recovery mode attempts to salvage valid fields from damaged records:

```python
# Attempt to recover partial data from malformed records
reader = mrrc.MARCReader('records.mrc', recovery_mode='lenient')
for record in reader:
    print(f"Got {len(record.get_fields())} fields")

# Even more lenient — accept partial data
reader = mrrc.MARCReader('records.mrc', recovery_mode='permissive')
```

Recovery modes:
- `"permissive"` (default for the Python user surface) — yield records with diagnostics attached on `record.errors`; yield `None` for unsalvageable records
- `"lenient"` — same shape as permissive with a tighter recovery cap; salvages valid fields
- `"strict"` — raise on any malformation

Note: `permissive=True` and `recovery_mode` other than `"strict"` cannot be combined — they represent different error-handling strategies. Use `permissive=True` for pymarc-compatible "skip bad records" behavior, or `recovery_mode` for mrrc's "salvage what you can" approach. Setting `permissive=True` without an explicit `recovery_mode` implicitly pairs it with `recovery_mode="strict"` so the pymarc-shape combo (inner raises, outer wrapper swallows) works without surprise.

mrrc's Python default matches the pymarc / marc4j / libmarc convention: a fresh `mrrc.MARCReader(file)` iterates past per-record defects rather than aborting on the first one. The trade-off is real: permissive mode can hand you `None` for unsalvageable records, and per-record defects live on `record.errors` rather than raising. If you control the input and quality matters more than throughput, pass `recovery_mode="strict"` explicitly to make defects loud. See [A gentle case for choosing strict when feasible](../reference/error-handling.md#a-gentle-case-for-choosing-strict-when-feasible).

### Exception class names

mrrc keeps the same class names pymarc uses, so most `except` clauses
work after a port with only the import line changing:

```python
# pymarc
from pymarc import MARCReader, RecordDirectoryInvalid
# mrrc — same names, different package
from mrrc import MARCReader, RecordDirectoryInvalid
```

The full pymarc↔mrrc class-name mapping, the names mrrc deliberately
omits (and why), and the per-variant attribute reference live in the
[Error handling reference](../reference/error-handling.md). Two
porting-specific notes worth inlining here:

**Base class rename.** `from pymarc import PymarcException` fails at
import; replace with `from mrrc import MrrcException`, or alias on
import:

```python
from mrrc import MrrcException as PymarcException
```

**`FatalReaderError` catches different things.** mrrc keeps the
fatal record-level classes (`RecordLengthInvalid`, `TruncatedRecord`,
`EndOfRecordNotFound`) as siblings under `MrrcException`, not as
children of `FatalReaderError` (as in pymarc). A port writing
`except FatalReaderError:` to catch a malformed-record error won't
catch what it expects. Two pymarc-compatible recipes:

```python
# Enumerate the four classes by name (matches what pymarc's
# `except FatalReaderError:` would have caught)
try:
    record = next(reader)
except (RecordLengthInvalid, TruncatedRecord, EndOfRecordNotFound,
        FatalReaderError):
    ...

# Or catch the mrrc base (broader — catches every typed mrrc error)
try:
    record = next(reader)
except MrrcException:
    ...
```

See [Known hierarchy divergences from pymarc](../reference/error-handling.md#known-hierarchy-divergences-from-pymarc)
in the reference for the rationale.

**Capping recovered errors with `max_errors`.** mrrc's `MARCReader` accepts a `max_errors=N` kwarg that caps the total number of `record.errors` entries accumulated across a `lenient` / `permissive` stream. Once the (N+1)-th recovered error lands, the next read raises `mrrc.FatalReaderError` (E099). pymarc has no equivalent. Pass `max_errors=0` (or omit the kwarg) to disable the cap. See [Capping recovered errors with `max_errors`](../reference/error-handling.md#capping-recovered-errors-with-max_errors).

## Known Differences from pymarc

1. **Record constructor**: `mrrc.Record()` works (defaults to `Leader()`), or pass explicit `mrrc.Record(mrrc.Leader())`
2. **UTF-8 encoding**: Set `leader.character_coding = 'a'` for UTF-8 (mrrc uses UTF-8 by default internally)
3. **No field removal during iteration**: Use list comprehension or separate pass if modifying records during iteration
4. **Type safety**: All data is validated at Rust layer (this is a feature, prevents data corruption)
5. **Field handles, not shared objects**: fields obtained from a record are live handles — in-place edits persist exactly as in pymarc, but each lookup returns a distinct handle object (`record['245'] is record['245']` is `False`), so don't compare fields with `is` or `id()`
6. **Removal invalidates field handles**: after any `remove_field`/`remove_fields` call, outstanding handles raise `mrrc.StaleFieldError` on use instead of silently targeting the wrong field — re-fetch the field and retry (pymarc object references survive removals)

## Getting Help

- **Documentation**: See class docstrings in Python (IDE autocomplete available)
- **Type hints**: Full `.pyi` stub file provides IDE support
- **Examples**: See test files for comprehensive examples
- **Issues**: Report bugs at https://github.com/dchud/mrrc/issues

## Contributing

We welcome contributions! The project is structured as:
- `src/`: Core Rust MARC library
- `src-python/`: Python wrapper with PyO3
- `tests/`: Integration tests

To build locally:
```bash
uv sync
uv run maturin develop
```
