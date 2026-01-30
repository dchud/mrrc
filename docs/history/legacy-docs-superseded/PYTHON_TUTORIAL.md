# Python Tutorial

This tutorial covers reading, writing, and converting MARC records in Python using MRRC.

## Prerequisites

```bash
pip install mrrc
```

## Reading MARC Records

### Basic Reading

```python
import mrrc

# Read from a file
with open("records.mrc", "rb") as f:
    reader = mrrc.MARCReader(f)
    for record in reader:
        print(record.title())
```

### Read with Iterator Protocol

```python
# Using next() directly
with open("records.mrc", "rb") as f:
    reader = mrrc.MARCReader(f)
    while record := reader.read_record():
        print(record.title())
```

### Read from Bytes

```python
# Read from bytes in memory
data = open("records.mrc", "rb").read()
import io
reader = mrrc.MARCReader(io.BytesIO(data))
for record in reader:
    print(record.title())
```

## Working with Records

### Accessing Fields

```python
# Dictionary-style access (pymarc compatible)
title = record['245']['a']       # First 245 field, subfield 'a'
author = record['100']['a']      # First 100 field, subfield 'a'

# Check if field exists
if '650' in record:
    subjects = record.get_fields('650')

# Get all fields with a tag
for field in record.get_fields('650'):
    print(field['a'])

# Get multiple tags at once
for field in record.get_fields('600', '650', '651'):
    print(f"{field.tag}: {field['a']}")
```

### Convenience Methods

```python
# These methods extract common data
record.title()           # 245 $a
record.author()          # 100/110/111 $a
record.isbn()            # 020 $a
record.issn()            # 022 $a
record.publisher()       # 260 $b
record.pubyear()         # Publication year
record.subjects()        # All 650 $a values
record.notes()           # All 5xx notes
record.series()          # 490 $a
record.physical_description()  # 300 $a

# Record type checks
record.is_book()         # True if language material monograph
record.is_serial()       # True if serial
record.is_music()        # True if music
```

### Control Fields

```python
# Control fields (001-009) have a .value property
control_number = record['001'].value
fixed_data = record['008'].value

# Or use the convenience method
control_number = record.control_field('001')
```

### Working with Subfields

```python
field = record['245']

# Get first subfield value
title = field['a']

# Get all values for a subfield code
all_a_values = field.get_subfields('a')

# Get multiple subfield codes
values = field.get_subfields('a', 'b', 'c')

# Check if subfield exists
if 'c' in field:
    responsibility = field['c']

# Iterate over all subfields
for subfield in field.subfields():
    print(f"${subfield.code}: {subfield.value}")

# Get subfields as dictionary
subfield_dict = field.subfields_as_dict()
# {'a': ['Title'], 'c': ['Author']}
```

### Working with Indicators

```python
field = record['245']

# Access indicators
ind1 = field.indicator1
ind2 = field.indicator2

# Or via tuple-like interface
ind1 = field.indicators[0]
ind2 = field.indicators[1]

# Set indicators
field.indicator1 = '1'
field.indicator2 = '0'
```

## Creating Records

### Basic Record Creation

```python
# Create a new record with a leader
leader = mrrc.Leader()
record = mrrc.Record(leader)

# Add control fields
record.add_control_field('001', '12345')
record.add_control_field('008', '200101s2020    xxu||||||||||||||||eng||')

# Create and add data fields
field = mrrc.Field('245', '1', '0')
field.add_subfield('a', 'The Great Gatsby /')
field.add_subfield('c', 'F. Scott Fitzgerald.')
record.add_field(field)

# Add author
author = mrrc.Field('100', '1', ' ')
author.add_subfield('a', 'Fitzgerald, F. Scott,')
author.add_subfield('d', '1896-1940.')
record.add_field(author)

# Add subjects
for subject in ['Psychological fiction', 'Long Island (N.Y.)']:
    subj = mrrc.Field('650', ' ', '0')
    subj.add_subfield('a', subject)
    record.add_field(subj)
```

### Configuring the Leader

```python
leader = mrrc.Leader()

# Set common properties
leader.record_status = 'n'           # New record
leader.record_type = 'a'             # Language material
leader.bibliographic_level = 'm'     # Monograph
leader.character_coding = 'a'        # UTF-8

# Or use position-based access
leader[5] = 'n'   # Record status
leader[6] = 'a'   # Record type
leader[7] = 'm'   # Bibliographic level
leader[9] = 'a'   # Character coding
```

## Writing Records

### Basic Writing

```python
with open("output.mrc", "wb") as f:
    with mrrc.MARCWriter(f) as writer:
        writer.write(record)
```

### Write Multiple Records

```python
# Read, modify, and write
with open("input.mrc", "rb") as infile:
    with open("output.mrc", "wb") as outfile:
        reader = mrrc.MARCReader(infile)
        with mrrc.MARCWriter(outfile) as writer:
            for record in reader:
                # Modify record if needed
                if record.title():
                    writer.write(record)
```

## Format Conversion

### Convert to JSON

```python
# Single record
json_str = record.to_json()

# Parse back
restored = mrrc.json_to_record(json_str)
```

### Convert to MARCJSON

```python
# LOC standard MARC-in-JSON format
marcjson_str = record.to_marcjson()
restored = mrrc.marcjson_to_record(marcjson_str)
```

### Convert to XML

```python
xml_str = record.to_xml()
restored = mrrc.xml_to_record(xml_str)
```

### Convert to CSV

```python
# Single record
csv_str = mrrc.record_to_csv(record)

# Multiple records
records = list(mrrc.MARCReader(f))
csv_str = mrrc.records_to_csv(records)

# Filter specific fields
csv_str = mrrc.records_to_csv_filtered(
    records,
    lambda tag: tag in ('245', '100', '650')
)
```

### Convert to Dublin Core

```python
dc_str = record.to_dublin_core()
dc_xml = mrrc.dublin_core_to_xml(dc_str)
```

### Convert to Other Formats

```python
# Protobuf
writer = mrrc.ProtobufWriter("records.pb")
writer.write_record(record)
writer.close()

for record in mrrc.ProtobufReader("records.pb"):
    print(record.title())

# Arrow
writer = mrrc.ArrowWriter("records.arrow")
writer.write_record(record)
writer.close()

# MessagePack
writer = mrrc.MessagePackWriter("records.msgpack")
writer.write_record(record)
writer.close()
```

### Format-Agnostic I/O

```python
# Auto-detect format from extension
for record in mrrc.read("data.mrc"):
    print(record.title())

for record in mrrc.read("data.pb"):
    print(record.title())

# Write with auto-detection
records = list(mrrc.read("input.mrc"))
mrrc.write(records, "output.pb")
mrrc.write(records, "output.arrow")
```

## Query DSL

MRRC provides a powerful Query DSL for advanced field searching.

### Filter by Indicators

```python
# Find all 650 fields with indicator2='0' (LCSH)
lcsh_subjects = record.fields_by_indicator('650', indicator2='0')

for field in lcsh_subjects:
    print(field['a'])
```

### Filter by Tag Range

```python
# Find all subject fields (600-699)
subjects = record.fields_in_range('600', '699')

for field in subjects:
    print(f"{field.tag}: {field['a']}")
```

### FieldQuery (Complex Matching)

```python
# Build a query
query = mrrc.FieldQuery()
query.tag('650')
query.indicator2('0')
query.has_subfield('a')

# Execute
for field in record.fields_matching(query):
    print(field['a'])
```

### Pattern Matching

```python
# Find ISBN-13s (start with 978 or 979)
query = mrrc.SubfieldPatternQuery('020', 'a', r'^97[89]')
for field in record.fields_matching_pattern(query):
    print(field['a'])
```

### Value Matching

```python
# Exact match
query = mrrc.SubfieldValueQuery('650', 'a', 'History')
exact_matches = record.fields_matching_value(query)

# Partial match
query = mrrc.SubfieldValueQuery('650', 'a', 'History', partial=True)
partial_matches = record.fields_matching_value(query)
```

## Authority and Holdings Records

### Authority Records

```python
with open("authorities.mrc", "rb") as f:
    reader = mrrc.AuthorityMARCReader(f)
    for record in reader:
        # Access heading fields (1XX)
        heading = record['100']
        if heading:
            print(f"Name: {heading['a']}")
```

### Holdings Records

```python
with open("holdings.mrc", "rb") as f:
    reader = mrrc.HoldingsMARCReader(f)
    for record in reader:
        # Access location (852)
        location = record['852']
        if location:
            print(f"Location: {location['a']} {location['b']}")
```

## Performance Tips

### Use File Paths When Possible

```python
# Faster: file path (zero-GIL overhead)
for record in mrrc.read("records.mrc"):
    print(record.title())

# Slower: file object (GIL for read calls)
with open("records.mrc", "rb") as f:
    for record in mrrc.MARCReader(f):
        print(record.title())
```

### Parallel Processing

```python
from concurrent.futures import ThreadPoolExecutor

def process_file(path):
    count = 0
    for record in mrrc.read(path):
        count += 1
    return count

# Process multiple files in parallel
files = ["file1.mrc", "file2.mrc", "file3.mrc", "file4.mrc"]
with ThreadPoolExecutor(max_workers=4) as executor:
    results = list(executor.map(process_file, files))
print(f"Total: {sum(results)}")
```

### High-Throughput Single File

```python
from mrrc import ProducerConsumerPipeline

# For large files, use the pipeline (3.74x speedup on 4 cores)
pipeline = ProducerConsumerPipeline.from_file('large_file.mrc')

for record in pipeline:
    process(record)
```

## Error Handling

```python
from mrrc import MARCReader

try:
    with open("records.mrc", "rb") as f:
        reader = MARCReader(f)
        for record in reader:
            try:
                print(record.title())
            except Exception as e:
                print(f"Error processing record: {e}")
except FileNotFoundError:
    print("File not found")
except Exception as e:
    print(f"Error opening file: {e}")
```

## Complete Example

```python
#!/usr/bin/env python3
"""Example: Extract bibliographic data to CSV."""

import mrrc
import csv
import sys

def extract_to_csv(input_path, output_path):
    """Extract titles and authors from MARC file to CSV."""

    with open(output_path, 'w', newline='', encoding='utf-8') as csvfile:
        writer = csv.writer(csvfile)
        writer.writerow(['Title', 'Author', 'ISBN', 'Year', 'Subjects'])

        for record in mrrc.read(input_path):
            title = record.title() or ''
            author = record.author() or ''
            isbn = record.isbn() or ''
            year = record.pubyear() or ''
            subjects = '; '.join(record.subjects())

            writer.writerow([title, author, isbn, year, subjects])

if __name__ == '__main__':
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} input.mrc output.csv")
        sys.exit(1)

    extract_to_csv(sys.argv[1], sys.argv[2])
    print("Done!")
```

## Next Steps

- [Migration Guide](./MIGRATION_GUIDE.md) - Migrate from pymarc
- [Streaming Guide](./STREAMING_GUIDE.md) - Large file handling
- [Threading Documentation](./THREADING.md) - Parallel processing patterns
- [Query DSL Guide](./QUERY_DSL.md) - Advanced field searching
