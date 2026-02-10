# Python Quickstart

Get started with MRRC in Python in 5 minutes.

## Install

```bash
pip install mrrc
```

## Read Records

```python
from mrrc import MARCReader

# Pass filename for best performance
for record in MARCReader("records.mrc"):
    print(record.title())
```

This approach uses pure Rust I/O and releases Python's GIL, enabling multi-threaded speedups.

## Access Fields

```python
# Get a specific field
field = record["245"]  # Title field
if field:
    print(field["a"])  # Title proper

# Get all fields with a tag
for field in record.fields_by_tag("650"):
    print(field["a"])  # Subject heading

# Use convenience methods
print(record.title())
print(record.author())
print(record.isbn())
```

## Create Records

```python
from mrrc import Record, Field, Leader, Subfield

# Build a record inline (pymarc-style)
record = Record(fields=[
    Field("245", indicators=["1", "0"], subfields=[
        Subfield("a", "My Book Title"),
        Subfield("c", "by Author Name"),
    ]),
    Field("100", "1", " ", subfields=[Subfield("a", "Author Name")]),
])

# Add a control field
record.add_control_field("001", "123456789")
```

You can also build records incrementally with `add_subfield()` and `add_field()`:

```python
field = Field("650", " ", "0")
field.add_subfield("a", "Subject heading")
record.add_field(field)
```

## Write Records

```python
from mrrc import MARCWriter

with MARCWriter("output.mrc") as writer:
    writer.write(record)
```

## Convert Formats

```python
# To JSON
json_str = record.to_json()

# To XML
xml_str = record.to_xml()

# To MARCJSON (LOC standard)
marcjson_str = record.to_marcjson()
```

## Next Steps

- [Reading Records Tutorial](../tutorials/python/reading-records.md) - Detailed reading guide
- [Writing Records Tutorial](../tutorials/python/writing-records.md) - Creating and modifying records
- [Migration from pymarc](../guides/migration-from-pymarc.md) - For existing pymarc users
- [Python API Reference](../reference/python-api.md) - Full API documentation
