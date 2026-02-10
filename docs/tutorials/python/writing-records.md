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
# Create a title field (245)
field = mrrc.Field("245", "1", "0")
field.add_subfield("a", "The Great Gatsby /")
field.add_subfield("c", "F. Scott Fitzgerald.")
record.add_field(field)

# Create an author field (100)
author = mrrc.Field("100", "1", " ")
author.add_subfield("a", "Fitzgerald, F. Scott,")
author.add_subfield("d", "1896-1940.")
record.add_field(author)
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
        if record.title():  # Only write records with titles
            writer.write(record)
```

## Modifying Existing Records

```python
for record in mrrc.MARCReader("input.mrc"):
    # Add a local note
    note = mrrc.Field("590", " ", " ")
    note.add_subfield("a", "Processed by MRRC")
    record.add_field(note)

    # Modify an existing field
    for field in record.fields_by_tag("245"):
        # Change indicator
        field.indicator1 = "1"
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
    isbn_field = mrrc.Field("020", " ", " ")
    isbn_field.add_subfield("a", isbn)
    record.add_field(isbn_field)

    # Author
    if author:
        author_field = mrrc.Field("100", "1", " ")
        author_field.add_subfield("a", author)
        record.add_field(author_field)

    # Title
    title_field = mrrc.Field("245", "1" if author else "0", "0")
    title_field.add_subfield("a", title)
    if author:
        title_field.add_subfield("c", f"by {author}")
    record.add_field(title_field)

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
