# Migration Guide: pymarc to mrrc

This guide helps existing pymarc users migrate to mrrc (MARC Rust Crate), a high-performance Rust-based MARC library with Python bindings.

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
record.append(field)
writer.write(record)
writer.close()
```

### After (mrrc) - pymarc-Compatible

**mrrc supports nearly identical pymarc syntax:**

```python
import mrrc

# Reading records - 100% compatible with pymarc syntax
with open('records.mrc', 'rb') as f:
    reader = mrrc.MARCReader(f)
    for record in reader:
        # Access fields using pymarc dictionary syntax
        print(record['245']['a'])  # Works!
        print(record.title())      # Also available as convenience method

# Writing records
with open('output.mrc', 'wb') as f:
     with mrrc.MARCWriter(f) as writer:
         field = mrrc.Field('245', '1', '0')
         field.add_subfield('a', 'Title')
         record = mrrc.Record(mrrc.Leader())
         record.add_field(field)
         writer.write(record)  # write() works like pymarc
```

## API Comparison

### Record Creation

| Operation | pymarc | mrrc | Same? |
|-----------|--------|------|-------|
| Create empty record | `pymarc.Record()` | `mrrc.Record(mrrc.Leader())` | Different |
| Create with leader | `pymarc.Record(leader)` | `mrrc.Record(leader)` | **Same** |
| Add control field | `record.add_field(Field('001', data='value'))` | `record.add_control_field('001', 'value')` | Different |
| Add data field | `record.append(field)` | `record.add_field(field)` | Different |

### Field Creation

| Operation | pymarc | mrrc | Same? |
|-----------|--------|------|-------|
| Create field | `Field('245', ['1','0'], [('a', 'Title')])` | `Field('245', '1', '0'); field.add_subfield('a', 'Title')` | Different |
| Add subfield | `field.add_subfield('a', 'value')` | `field.add_subfield('a', 'value')` | **Same** |
| Get subfields | `field.get_subfields('a')` | `field.get_subfields('a')` | **Same** |
| Access subfield | `field['a']` | `field['a']` | **Same** |

### Reading/Writing

| Operation | pymarc | mrrc | Same? |
|-----------|--------|------|-------|
| Read record | `reader.next()` or `next(reader)` | `next(reader)` | **Same** |
| Write record | `writer.write(record)` | `writer.write(record)` | **Same** |
| Iterate | `for record in reader:` | `for record in reader:` | **Same** |
| Context manager | Manual close required | `with MARCWriter(f) as w:` | Enhanced |

### Accessing Data

| Operation | pymarc | mrrc | Same? |
|-----------|--------|------|-------|
| Get title | `record['245']['a']` | `record['245']['a']` or `record.title()` | **Same** |
| Get field | `record['650']` | `record['650']` or `record.fields_by_tag('650')` | **Same** |
| Check if field exists | `'245' in record` | `'245' in record` | **Same** |
| Get all fields | `for field in record:` | `for field in record:` | **Same** |
| Control field | `record['001'].value` | `record['001'].value` or `record.control_field('001')` | **Same** |

## API Compatibility

**mrrc provides excellent pymarc API compatibility** with support for all major operations:

### Record Field Access - Dictionary-Style (Identical to pymarc)
```python
# Dictionary-style access works exactly like pymarc
field = record['245']                      # Get first 245 field (or None if missing)
all_fields = record.fields_by_tag('245')   # Get all 245 fields

# Missing fields return None (matching pymarc behavior)
field = record['999']                      # Returns None if field doesn't exist
                                           # (does NOT raise KeyError)

# Check if field exists (identical to pymarc)
if '245' in record:
    title_field = record['245']

# Alternative method-based access also available
field = record.get_field('245')            # Get first field
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
field.add_subfield('a', 'value')   # Identical to pymarc
field.get_subfields('a')           # Get list of values - identical to pymarc
field.delete_subfield('a')         # Delete subfield by code
field.subfields_as_dict()          # Get all subfields as dict
field.subfields()                  # Get all Subfield objects
```

### Record Operations (Identical to pymarc + Extensions)
```python
# Standard pymarc operations
record.remove_field('245')         # Remove field(s) by tag
record.append(field)               # Add field (same as add_field for compatibility)
record.get_fields('650', '651')    # Get fields for multiple tags

# Convenience methods (identical to pymarc)
record.title()                     # Get title (245 $a)
record.author()                    # Get author (100/110/111 $a)
record.isbn()                      # Get ISBN (020 $a)
record.issn()                      # Get ISSN (022 $a)
record.subjects()                  # Get all subjects (650 $a)
record.publisher()                 # Get publisher (260 $b)
record.physical_description()      # Get extent (300 $a)
record.series()                    # Get series (490 $a)
```

### Leader Access - Property-Based and Position-Based
```python
# Property-based access (recommended for clarity)
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

### Reader/Writer Interface (Identical to pymarc)
```python
# Reading (identical to pymarc)
reader = mrrc.MARCReader(f)
for record in reader:              # Standard iteration
    print(record.title())

# Writing (identical to pymarc, with context manager support)
with mrrc.MARCWriter(f) as writer:
    writer.write(record)           # Same method name as pymarc
```

## Minimal API Differences

**mrrc is nearly 100% compatible with pymarc.** Here are the only two required changes:

### 1. Record Constructor Requires Explicit Leader
**The only required change** - needed once per record:
```python
# pymarc (implicit default leader)
record = pymarc.Record()

# mrrc (explicit leader required)
record = mrrc.Record(mrrc.Leader())

# Note: Once created, all field access works identically
print(record['245']['a'])  # Works exactly like pymarc
```

### 2. Optional: Extended Convenience Methods
mrrc extends pymarc with additional convenience methods:
```python
# All pymarc methods work:
record.title()             # Get title
record.author()            # Get author
record.isbn()              # Get ISBN

# Plus many additional methods:
record.issn()              # Get ISSN
record.issn_title()        # Get ISSN title
record.sudoc()             # Get SuDoc classification
record.issnl()             # Get ISSN-L
record.pubyear()           # Get publication year
record.physical_description()  # Get extent/pages
record.is_book()           # Check if book
record.is_serial()         # Check if serial
record.is_music()          # Check if music
```

## Migration Checklist

**Minimal changes needed:**

- [ ] Replace `import pymarc` with `import mrrc`
- [ ] Update record creation: `pymarc.Record()` → `mrrc.Record(mrrc.Leader())`
- [ ] Everything else works the same - dictionary access, method names, iteration all identical

**Optional enhancements:**

- [ ] Use additional convenience methods like `record.issn()`, `record.sudoc()`, etc. for specialized use cases
- [ ] Update writers to use context managers: `with mrrc.MARCWriter(f) as w:` (better resource management)

## Known Differences from pymarc

1. **Record constructor requires explicit Leader**: `mrrc.Record(mrrc.Leader())` instead of `pymarc.Record()`
2. **UTF-8 encoding**: Set `leader.character_coding = 'a'` for UTF-8 (mrrc uses UTF-8 by default internally)
3. **No field removal during iteration**: Use list comprehension or separate pass if modifying records during iteration
4. **Type safety**: All data is validated at Rust layer (this is a feature, prevents data corruption)

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
cd src-python
python -m venv venv
source venv/bin/activate
maturin develop
```
