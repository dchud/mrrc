# Reading Records (Python)

Learn to read MARC records from files and work with their contents.

## Basic Reading

Pass a filename directly for best performance—this uses pure Rust I/O and fully releases Python's GIL during parsing:

```python
from mrrc import MARCReader

for record in MARCReader("records.mrc"):
    print(record.title())
```

You can also use a file object if needed (e.g., for network streams), though this holds the GIL during I/O:

```python
with open("records.mrc", "rb") as f:
    for record in MARCReader(f):
        print(record.title())
```

See [Threading in Python](../../guides/threading-python.md) for details on GIL behavior and multi-threaded performance.

## Reading from Memory

```python
# Read from bytes
data = open("records.mrc", "rb").read()
for record in mrrc.MARCReader(data):
    print(record.title())
```

## Accessing Fields

### Dictionary-Style Access

```python
# Get first field by tag, then subfield
title = record["245"]["a"]       # First 245 field, subfield 'a'
author = record["100"]["a"]      # First 100 field, subfield 'a'

# Check if field exists
if "650" in record:
    subjects = record.fields_by_tag("650")
```

### Get Multiple Fields

```python
# Get all fields with a tag
for field in record.fields_by_tag("650"):
    print(field["a"])
```

## Control Fields

Control fields (001-009) contain unstructured data:

```python
# Access control field value
control_number = record["001"]
if control_number:
    print(control_number.value)

# Or use the convenience method
control_number = record.control_field("001")
```

## Convenience Methods

MRRC provides shortcuts for common data:

```python
record.title()           # 245 $a
record.author()          # 100/110/111 $a
record.isbn()            # 020 $a (first)
record.isbns()           # 020 $a (all)
record.issn()            # 022 $a
record.publisher()       # 260 $b
record.pubyear()         # Publication year
record.subjects()        # All 650 $a values
```

## Working with Subfields

```python
field = record["245"]

# Get first subfield value
title = field["a"]

# Get all values for a subfield code
all_a_values = field.subfields_by_code("a")

# Iterate over all subfields
for subfield in field.subfields():
    print(f"${subfield.code}: {subfield.value}")
```

## Working with Indicators

```python
field = record["245"]

# Access indicators
ind1 = field.indicator1  # or field.ind1
ind2 = field.indicator2  # or field.ind2

# Indicators affect meaning:
# 245 indicator2 = number of nonfiling characters
# "4" means skip "The " when filing
```

## Error Handling

```python
from mrrc import MARCReader

try:
    for record in MARCReader("records.mrc"):
        try:
            print(record.title())
        except Exception as e:
            print(f"Error processing record: {e}")
except FileNotFoundError:
    print("File not found")
```

## Complete Example

```python
#!/usr/bin/env python3
"""Extract bibliographic data to CSV."""

import mrrc
import csv

def extract_to_csv(input_path, output_path):
    with open(output_path, 'w', newline='', encoding='utf-8') as csvfile:
        writer = csv.writer(csvfile)
        writer.writerow(['Title', 'Author', 'ISBN', 'Year', 'Subjects'])

        for record in mrrc.MARCReader(input_path):
            title = record.title() or ''
            author = record.author() or ''
            isbn = record.isbn() or ''
            year = record.pubyear() or ''
            subjects = '; '.join(record.subjects())

            writer.writerow([title, author, isbn, year, subjects])

if __name__ == '__main__':
    extract_to_csv("records.mrc", "output.csv")
```

## Next Steps

- [Writing Records](writing-records.md) - Create and modify records
- [Querying Fields](querying-fields.md) - Advanced field searching
- [Python API Reference](../../reference/python-api.md) - Full API documentation
