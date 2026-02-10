# Format Selection Guide

This guide helps you choose the right serialization format for your MARC data based on your use case.

## Quick Decision Tree

```
What's your primary goal?
│
├─ Exchange with other library systems?
│  └─ ISO 2709 (.mrc) - Universal compatibility
│
├─ Build a REST API?
│  └─ JSON or MARCJSON - Human-readable, widely supported
│
├─ Linked data / semantic web?
│  └─ BIBFRAME - RDF graph model for modern linked data
│
├─ Metadata crosswalks?
│  ├─ Simple metadata? → Dublin Core (15-element standard)
│  └─ Detailed bibliographic? → MODS (Metadata Object Description Schema)
│
├─ Spreadsheet export?
│  └─ CSV
│
└─ Long-term archival?
   └─ ISO 2709 or JSON - Wide tool support
```

## Format Comparison

| Format | Speed | Size | Compatibility | Best For |
|--------|-------|------|---------------|----------|
| **ISO 2709** | Fast | Compact | Universal | Library system interchange |
| **JSON** | Medium | Large | Universal | Web APIs, debugging |
| **MARCJSON** | Medium | Large | LOC Standard | Standards-compliant JSON |
| **XML** | Slow | Very Large | Universal | MARCXML systems |
| **CSV** | Fast | Medium | Universal | Spreadsheets, simple tools |
| **Dublin Core** | Fast | Medium | Universal | Simple metadata exchange |
| **MODS** | Medium | Large | Good | Detailed metadata crosswalks |
| **BIBFRAME** | Medium | Large | Growing | Linked data, RDF applications |

## Detailed Format Profiles

### ISO 2709 (MARC Binary)

**Use when:**
- Exchanging with traditional library systems (ILS, OCLC, etc.)
- Maximum compatibility is required
- Working with existing MARC workflows

**Avoid when:**
- Building modern web APIs
- Need for human-readable output
- Integrating with linked data systems

```python
# Reading
with open("records.mrc", "rb") as f:
    for record in mrrc.MARCReader(f):
        print(record.title())

# Writing
with open("output.mrc", "wb") as f:
    with mrrc.MARCWriter(f) as writer:
        writer.write(record)
```

### JSON / MARCJSON

**Use when:**
- Building web APIs that need human-readable responses
- Debugging or inspecting records
- JavaScript frontend integration
- MARCJSON when LOC standard compliance is required

**Avoid when:**
- Bandwidth is limited
- Processing millions of records
- Need binary efficiency

```python
# Single record
json_str = record.to_json()
marcjson_str = record.to_marcjson()

# Parse back
restored = mrrc.json_to_record(json_str)
```

### BIBFRAME

**Use when:**
- Building linked data applications
- Publishing bibliographic data as RDF
- Integrating with SPARQL endpoints or triple stores
- Following Library of Congress modernization initiatives

**Avoid when:**
- Simple record exchange between library systems
- Need for compact binary format
- Systems that only support traditional MARC

```python
from mrrc import marc_to_bibframe, BibframeConfig

config = BibframeConfig()
config.set_base_uri("http://library.example.org/")
graph = marc_to_bibframe(record, config)

# Serialize to Turtle, RDF/XML, JSON-LD, or N-Triples
turtle = graph.serialize("turtle")
```

See the [BIBFRAME Conversion Guide](bibframe-conversion.md) for details.

### CSV

**Use when:**
- Exporting to spreadsheets (Excel, Google Sheets)
- Simple data analysis tools
- Quick tabular view of MARC data

**Avoid when:**
- Need to preserve full MARC structure
- Round-trip conversion is required

```python
csv_str = mrrc.records_to_csv(records)
```

### Dublin Core

**Use when:**
- Simple metadata exchange across systems
- OAI-PMH harvesting
- Need 15-element standard metadata

```python
dc_xml = record.to_dublin_core()
```

### MODS

**Use when:**
- Detailed metadata crosswalks
- Digital library applications
- Need richer metadata than Dublin Core

```python
mods_xml = record.to_mods()
```

## Use Case Recommendations

### Library System Integration

**Recommended:** ISO 2709

Traditional library systems universally support ISO 2709. Use it for:
- OCLC WorldCat submissions
- ILS data migration
- Z39.50 compatible systems

### Web Application Backend

**Recommended:** JSON or MARCJSON

For APIs serving web or mobile clients:
- JSON for generic integration
- MARCJSON for LOC-compliant systems

### Linked Data / Semantic Web

**Recommended:** BIBFRAME

BIBFRAME enables integration with:
- SPARQL endpoints
- Triple stores
- Linked data ecosystems
- Library of Congress initiatives

### Metadata Crosswalks

**Recommended:** Dublin Core or MODS

- Dublin Core for simple, widely-supported metadata
- MODS for detailed bibliographic description

### Spreadsheet Export

**Recommended:** CSV

CSV works with Excel, Google Sheets, and simple data tools.

### Archival / Preservation

**Recommended:** ISO 2709 or JSON

- ISO 2709 for maximum future compatibility with library systems
- JSON for wide tool support and human readability

## Next Steps

- [BIBFRAME Conversion Guide](bibframe-conversion.md) - BIBFRAME conversion details
- [Format Conversion Tutorial](../tutorials/python/format-conversion.md) - Python format conversion examples
- [Working with Large Files](working-with-large-files.md) - Large file handling
