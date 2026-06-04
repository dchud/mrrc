# Writing Records (Python)

Learn to create MARC records from scratch and write them to files.

## Creating a Record

```python
import mrrc

# Create a new record with a leader
leader = mrrc.Leader()
record = mrrc.Record(leader)
```

## Adding Control Fields

Control fields (001-009) contain unstructured data:

```python
record.add_control_field("001", "12345")
record.add_control_field("008", "200101s2020    xxu||||||||||||||||eng||")
```

## Adding Data Fields

Data fields have a tag, two indicators, and subfields:

```python
# Create fields inline with subfields= and indicators= kwargs
title = mrrc.Field("245", indicators=["1", "0"], subfields=[
    mrrc.Subfield("a", "The Great Gatsby /"),
    mrrc.Subfield("c", "F. Scott Fitzgerald."),
])
record.add_field(title)

# Or use positional indicators
author = mrrc.Field("100", "1", " ", subfields=[
    mrrc.Subfield("a", "Fitzgerald, F. Scott,"),
    mrrc.Subfield("d", "1896-1940."),
])
record.add_field(author)
```

You can also build fields incrementally with `add_subfield()`:

```python
field = mrrc.Field("245", "1", "0")
field.add_subfield("a", "The Great Gatsby /")
field.add_subfield("c", "F. Scott Fitzgerald.")
record.add_field(field)
```

## Adding Multiple Fields

```python
# Add subject headings
subjects = ["Psychological fiction", "Long Island (N.Y.)"]
for subject in subjects:
    field = mrrc.Field("650", " ", "0")
    field.add_subfield("a", subject)
    record.add_field(field)
```

## Configuring the Leader

```python
leader = mrrc.Leader()

# Set common properties
leader.record_status = "n"           # New record
leader.record_type = "a"             # Language material
leader.bibliographic_level = "m"     # Monograph
leader.character_coding = "a"        # UTF-8

record = mrrc.Record(leader)
```

## Writing to Files

### Basic Writing

```python
with mrrc.MARCWriter("output.mrc") as writer:
    writer.write(record)
```

### Write Multiple Records

```python
with mrrc.MARCWriter("output.mrc") as writer:
    for record in records:
        writer.write(record)
```

### Read, Modify, Write

```python
# Filter records while copying
with mrrc.MARCWriter("output.mrc") as writer:
    for record in mrrc.MARCReader("input.mrc"):
        if record.title:  # Only write records with titles
            writer.write(record)
```

## Modifying Existing Records

Fields obtained from a record are live handles: edits write through to
the record, matching pymarc.

```python
for record in mrrc.MARCReader("input.mrc"):
    # Add a local note
    note = mrrc.Field("590", " ", " ")
    note.add_subfield("a", "Processed by MRRC")
    record.add_field(note)

    # Edit fields in place
    record["245"].indicator1 = "1"
    record["245"]["a"] = "Revised title /"
    for field in record.get_fields("650"):
        field.delete_subfield("9")

    # Control fields too
    record["005"].data = "20260603120000.0"
```

### Field handles: two differences from pymarc

**Handles are not identical objects.** Each lookup returns a new handle
to the same underlying field. Edits through any handle are visible
through every other, but `record["245"] is record["245"]` is `False` —
don't use `is` or `id()` to compare fields.

**Removal invalidates handles.** pymarc field objects survive record
changes; mrrc handles refuse to operate once any field has been removed
from their record, raising `mrrc.StaleFieldError` instead of guessing
which field you meant. Re-fetch and retry:

```python
subjects = record.get_fields("650")
record.remove_field("440")
subjects[0]["a"]                  # raises mrrc.StaleFieldError
record.get_fields("650")[0]["a"]  # re-fetched: works
```

## Complete Example

```python
#!/usr/bin/env python3
"""Create a MARC record from scratch."""

import mrrc

def create_book_record(title, author, isbn, subjects):
    """Create a bibliographic record for a book."""

    # Create leader
    leader = mrrc.Leader()
    leader.record_status = "n"
    leader.record_type = "a"
    leader.bibliographic_level = "m"
    leader.character_coding = "a"

    record = mrrc.Record(leader)

    # Control fields
    record.add_control_field("001", f"mrrc-{isbn}")
    record.add_control_field("008", "200101s2020    xxu||||||||||||||||eng||")

    # ISBN
    record.add_field(mrrc.Field("020", " ", " ", subfields=[
        mrrc.Subfield("a", isbn),
    ]))

    # Author
    if author:
        record.add_field(mrrc.Field("100", "1", " ", subfields=[
            mrrc.Subfield("a", author),
        ]))

    # Title
    title_subfields = [mrrc.Subfield("a", title)]
    if author:
        title_subfields.append(mrrc.Subfield("c", f"by {author}"))
    record.add_field(mrrc.Field("245", "1" if author else "0", "0",
                                subfields=title_subfields))

    # Subjects
    for subject in subjects:
        subj_field = mrrc.Field("650", " ", "0")
        subj_field.add_subfield("a", subject)
        record.add_field(subj_field)

    return record

# Create and write a record
record = create_book_record(
    title="Introduction to MARC",
    author="Smith, John",
    isbn="9780123456789",
    subjects=["MARC format", "Library science"]
)

with mrrc.MARCWriter("new_record.mrc") as writer:
    writer.write(record)

print("Record created successfully")
```

## Next Steps

- [Reading Records](reading-records.md) - Read and access records
- [Format Conversion](format-conversion.md) - Convert to other formats
- [Python API Reference](../../reference/python-api.md) - Full API documentation
