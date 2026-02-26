# Format Conversion (Python)

Learn to convert MARC records between different formats.

## JSON Conversion

```python
import mrrc

for record in mrrc.MARCReader("records.mrc"):
    # Convert to JSON
    json_str = record.to_json()
    print(json_str)

    # Parse back from JSON
    restored = mrrc.json_to_record(json_str)
```

## MARCJSON (LOC Standard)

Library of Congress standard JSON format:

```python
# Convert to MARCJSON
marcjson_str = record.to_marcjson()

# Parse back
restored = mrrc.marcjson_to_record(marcjson_str)
```

## MARCXML Conversion

```python
# Convert to MARCXML
xml_str = record.to_xml()

# Parse back from MARCXML
restored = mrrc.xml_to_record(xml_str)

# Parse a MARCXML collection (multiple records)
records = mrrc.xml_to_records(collection_xml_str)
```

## CSV Export

```python
# Single record to CSV
csv_str = mrrc.record_to_csv(record)

# Multiple records to CSV
records = list(mrrc.MARCReader("records.mrc"))
csv_str = mrrc.records_to_csv(records)
```

## Dublin Core

```python
# Convert to Dublin Core
dc_str = record.to_dublin_core()
```

## MODS

```python
# Convert to MODS XML
mods_str = record.to_mods()

# Parse from MODS XML
record = mrrc.mods_to_record(mods_xml_str)
```

## BIBFRAME Conversion

Convert MARC to BIBFRAME RDF:

```python
from mrrc import marc_to_bibframe, BibframeConfig

# Basic conversion
config = BibframeConfig()
graph = marc_to_bibframe(record, config)

# Serialize to different RDF formats
turtle = graph.serialize("turtle")
rdfxml = graph.serialize("rdf-xml")
jsonld = graph.serialize("jsonld")
```

See [BIBFRAME Conversion Guide](../../guides/bibframe-conversion.md) for details.

## Batch Conversion

```python
#!/usr/bin/env python3
"""Convert MARC file to multiple formats."""

import mrrc

def convert_file(input_path):
    records = list(mrrc.MARCReader(input_path))

    # JSON
    with open("output.json", "w") as f:
        for record in records:
            f.write(record.to_json() + "\n")

    # CSV
    csv_str = mrrc.records_to_csv(records)
    with open("output.csv", "w") as f:
        f.write(csv_str)

    print(f"Converted {len(records)} records")

convert_file("library.mrc")
```

## Format Selection

| Format | Best For |
|--------|----------|
| ISO 2709 | Library system interchange |
| JSON | Web APIs, debugging |
| MARCJSON | LOC compatibility |
| MARCXML | MARCXML pipelines |
| CSV | Spreadsheet analysis |
| Dublin Core | Simple metadata exchange |
| MODS | Detailed metadata crosswalks |
| BIBFRAME | Linked data applications |

See [Format Selection Guide](../../guides/format-selection.md) for detailed recommendations.

## Next Steps

- [Reading Records](reading-records.md) - Basic record access
- [BIBFRAME Conversion](../../guides/bibframe-conversion.md) - RDF conversion
- [Format Support](../../reference/formats.md) - Full format reference
