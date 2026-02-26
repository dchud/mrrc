#!/usr/bin/env python3
"""
Format conversion examples (Python/pymarc-compatible API)

This example demonstrates how to convert MARC records to various
serialization formats using the mrrc Python wrapper:
- JSON (mrrc-specific format)
- MARCJSON (standard MARC-JSON format)
- MARCXML (standard LOC MARCXML format)
- CSV (tabular format)

These conversions are useful for:
- Data interchange with other systems
- Integration with web APIs (JSON)
- Search indexing (various formats)
- Data analysis and reporting (CSV)
"""

import sys
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    from mrrc import (
        MARCReader, Record, Field, Leader,
        record_to_csv, records_to_csv, records_to_csv_filtered
    )
except ImportError:
    print("Error: mrrc not installed")
    print("Install with: pip install mrrc")
    sys.exit(1)


def create_sample_record():
    """Create a sample record for format conversion demonstrations."""
    leader = Leader(
        record_type='a',
        bibliographic_level='m',
        character_coding='a',  # UTF-8
    )
    
    record = Record(leader)
    
    # Control fields
    record.add_control_field('001', 'ocm12345678')
    record.add_control_field('008', '200101s2020    xxu||||||||||||||||eng||')
    
    # Title
    title = Field('245', '1', '0')
    title.add_subfield('a', 'Rust systems programming /')
    title.add_subfield('c', 'Jane Smith and Bob Jones.')
    record.add_field(title)
    
    # Author
    author = Field('100', '1', ' ')
    author.add_subfield('a', 'Smith, Jane,')
    author.add_subfield('d', '1975-')
    author.add_subfield('e', 'author.')
    record.add_field(author)
    
    # Additional author
    contributor = Field('700', '1', ' ')
    contributor.add_subfield('a', 'Jones, Bob,')
    contributor.add_subfield('d', '1980-')
    contributor.add_subfield('e', 'author.')
    record.add_field(contributor)
    
    # Publication
    pub = Field('260', ' ', ' ')
    pub.add_subfield('a', 'San Francisco :')
    pub.add_subfield('b', 'O\'Reilly Media,')
    pub.add_subfield('c', '2020.')
    record.add_field(pub)
    
    # Physical description
    phys = Field('300', ' ', ' ')
    phys.add_subfield('a', 'xv, 450 pages ;')
    phys.add_subfield('c', '24 cm.')
    record.add_field(phys)
    
    # ISBN
    isbn = Field('020', ' ', ' ')
    isbn.add_subfield('a', '9781491927285')
    record.add_field(isbn)
    
    # Subjects
    for subject_text in [
        'Rust (Computer program language)',
        'Systems programming (Computer science)',
        'C (Computer program language)',
    ]:
        subject = Field('650', ' ', '0')
        subject.add_subfield('a', subject_text)
        record.add_field(subject)
    
    # Genre/form
    genre = Field('655', ' ', '7')
    genre.add_subfield('a', 'Handbooks and manuals.')
    genre.add_subfield('2', 'lcgft')
    record.add_field(genre)
    
    return record


def demonstrate_record_structure(record):
    """Show basic record information."""
    print("\n" + "=" * 70)
    print("ORIGINAL RECORD")
    print("=" * 70 + "\n")
    
    print(f"Title:           {record.title()}")
    print(f"Author:          {record.author()}")
    print(f"All Authors:     {', '.join(record.authors())}")
    print(f"ISBN:            {', '.join(record.isbns())}")
    print(f"Subjects:        {', '.join(record.subjects()[:2])}... ({len(record.subjects())} total)")
    
    if record.publication_info():
        pub = record.publication_info()
        print(f"Published:       {pub.date} by {pub.publisher} in {pub.place}")
    
    print()


def convert_to_json(record):
    """
    Convert record to JSON format (mrrc-specific).
    
    This is a flat JSON representation useful for:
    - Data interchange
    - API responses
    - Document storage (MongoDB, etc.)
    """
    print("=" * 70)
    print("1. JSON FORMAT (mrrc-specific)")
    print("=" * 70 + "\n")
    
    try:
        json_str = record.to_json()
        
        # Show pretty-printed version (limit length)
        import json
        json_obj = json.loads(json_str)
        
        print("Structure:")
        print(f"  Leader: record_type={json_obj['leader']['record_type']}, "
              f"bibliographic_level={json_obj['leader']['bibliographic_level']}")
        print(f"  Fields: {len(json_obj['fields'])} fields")
        print()
        
        # Show sample fields
        print("Sample JSON output (first 300 chars):")
        print(f"  {json_str[:300]}...")
        print()
        
        # Count field types
        field_types = {}
        for field in json_obj.get('fields', []):
            field_type = 'control' if 'value' in field else 'data'
            field_types[field_type] = field_types.get(field_type, 0) + 1
        
        print("Field breakdown:")
        for field_type, count in field_types.items():
            print(f"  {field_type}: {count} field(s)")
        
    except Exception as e:
        print(f"Error: {e}")
    
    print()


def convert_to_marcjson(record):
    """
    Convert record to MARCJSON format (standard MARC-JSON).
    
    MARCJSON is the standard JSON representation for MARC records.
    It's useful for:
    - Standard data interchange
    - Library system integration
    - Compatibility with other MARC tools
    """
    print("=" * 70)
    print("2. MARCJSON FORMAT (Standard MARC-JSON)")
    print("=" * 70 + "\n")
    
    try:
        marcjson_str = record.to_marcjson()
        
        # Parse and analyze
        import json
        marcjson_obj = json.loads(marcjson_str)
        
        print("Structure:")
        print(f"  Records in object: {len(marcjson_obj) if isinstance(marcjson_obj, list) else 1}")
        
        # Get first record (MARCJSON is typically an array)
        if isinstance(marcjson_obj, list) and marcjson_obj:
            first = marcjson_obj[0]
        else:
            first = marcjson_obj
        
        if isinstance(first, dict):
            print(f"  Fields: {len(first) if not isinstance(first, list) else 'N/A'}")
            
            # Show structure
            for key in sorted(list(first.keys())[:5]):  # Show first 5 fields
                value = first[key]
                if isinstance(value, list) and value:
                    print(f"    {key}: array with {len(value)} item(s)")
                elif isinstance(value, dict):
                    print(f"    {key}: object")
                else:
                    print(f"    {key}: {type(value).__name__}")
        
        print()
        print("Sample MARCJSON output (first 300 chars):")
        print(f"  {marcjson_str[:300]}...")
        print()
        
        # Show use case
        print("Use cases:")
        print("  - Library system integration (standard format)")
        print("  - Linked Data / RDF conversion")
        print("  - MARC21 XML conversion tools")
        print("  - Other library software")
        
    except Exception as e:
        print(f"Error: {e}")
    
    print()


def convert_to_xml(record):
    """
    Convert record to MARCXML format.

    MARCXML is the standard XML representation for MARC records.
    Useful for:
    - XML-based processing pipelines
    - XSLT transformations
    - Document management systems
    - Web service integration
    """
    print("=" * 70)
    print("3. MARCXML FORMAT (MARCXML)")
    print("=" * 70 + "\n")
    
    try:
        xml_str = record.to_xml()
        
        print("Structure (first 500 chars):")
        print(f"  {xml_str[:500]}...")
        print()
        
        # Count elements
        import xml.etree.ElementTree as ET
        root = ET.fromstring(xml_str)
        
        # Count different element types
        element_counts = {}
        for elem in root.iter():
            tag = elem.tag.split('}')[-1]  # Remove namespace
            element_counts[tag] = element_counts.get(tag, 0) + 1
        
        print("MARCXML structure breakdown:")
        for tag in sorted(element_counts.keys()):
            print(f"  <{tag}>: {element_counts[tag]} element(s)")
        
        print()
        print("Use cases:")
        print("  - XML processing pipelines (XSLT, XPath)")
        print("  - Document management systems")
        print("  - Data warehousing")
        print("  - LOC tools and services")
        
    except Exception as e:
        print(f"Error: {e}")
    
    print()


def convert_to_csv(record):
    """
    Convert records to CSV format for tabular analysis.
    
    CSV is useful for:
    - Data analysis and reporting
    - Spreadsheet import
    - SQL database loading
    - BI tools integration
    """
    print("=" * 70)
    print("4. CSV FORMAT (Tabular)")
    print("=" * 70 + "\n")
    
    try:
        # Create multiple records for CSV output
        records = [record]
        
        # Add a second record for demonstration
        record2 = Record(record.leader)
        record2.add_control_field('001', 'test002')
        record2.add_control_field('008', '210115s2021    xxu||||||||||||||||eng||')
        
        title2 = Field('245', '1', '0')
        title2.add_subfield('a', 'Python systems programming /')
        title2.add_subfield('c', 'Alice Brown.')
        record2.add_field(title2)
        
        author2 = Field('100', '1', ' ')
        author2.add_subfield('a', 'Brown, Alice,')
        author2.add_subfield('e', 'author.')
        record2.add_field(author2)
        
        records.append(record2)
        
        # Convert to CSV - full tabular output
        print("Full CSV export (all fields):")
        csv_str = records_to_csv(records)
        lines = csv_str.split('\n')
        for line in lines[:10]:
            print(f"  {line}")
        print()
        
        # Filtered CSV export
        print("Filtered CSV export (only 245, 100, 650 fields):")
        csv_filtered = records_to_csv_filtered(
            records, 
            lambda tag: tag in ('245', '100', '650')
        )
        lines_filtered = csv_filtered.split('\n')
        for line in lines_filtered[:10]:
            print(f"  {line}")
        print()
        
        print("Use cases:")
        print("  - Data analysis in Excel/Sheets")
        print("  - SQL database import")
        print("  - Python pandas DataFrames")
        print("  - Statistical analysis tools")
        print("  - Field-level filtering for reporting")
        
    except Exception as e:
        print(f"Error: {e}")
    
    print()


def round_trip_conversion(record):
    """
    Demonstrate round-trip conversion (JSON → Record → JSON).
    
    Shows that conversions preserve data fidelity.
    """
    print("=" * 70)
    print("5. ROUND-TRIP CONVERSION (Fidelity Check)")
    print("=" * 70 + "\n")
    
    try:
        # Convert to JSON
        json1 = record.to_json()
        
        # Parse and convert back
        import json as json_module
        json_obj = json_module.loads(json1)
        
        # Convert back to Record (if supported)
        record2 = __import__('mrrc').json.json_to_record(json_obj)
        
        # Convert again to compare
        json2 = record2.to_json()
        
        # Compare
        match = json1 == json2
        print(f"Original JSON → Record → JSON: {'✓ Match' if match else '✗ Differs'}")
        print(f"  Original size: {len(json1)} chars")
        print(f"  Round-trip size: {len(json2)} chars")
        print()
        
        # Verify field counts
        original_title = record.title()
        restored_title = record2.title()
        
        print(f"Fidelity verification:")
        print(f"  Title match: {original_title == restored_title}")
        print(f"  Author match: {record.author() == record2.author()}")
        print(f"  Subject count: {len(record.subjects())} → {len(record2.subjects())}")
        
    except Exception as e:
        print(f"Round-trip conversion not fully supported: {e}")
        print("(This is expected - not all conversions support round-trip.)")
    
    print()


def comparison_table():
    """Show a comparison of different formats."""
    print("=" * 70)
    print("FORMAT COMPARISON")
    print("=" * 70 + "\n")
    
    print("""
FORMAT       | NATIVE | SIZE    | USE CASE
─────────────┼────────┼─────────┼──────────────────────────────────
JSON         | mrrc   | Medium  | API responses, document stores
MARCJSON     | Std    | Medium  | Library system integration
MARCXML      | Std    | Large   | XML pipelines, XSLT transforms
CSV          | N/A    | Varies  | Spreadsheets, data analysis
ISO 2709     | Binary | Small   | Archive storage, interchange
─────────────┼────────┼─────────┼──────────────────────────────────

JSON:
  Pros: Compact, language-neutral, easy to parse
  Cons: Not standard for MARC
  
MARCJSON:
  Pros: Standard MARC JSON format, widely supported
  Cons: Slightly larger than binary
  
MARCXML:
  Pros: Standard format, XSLT support, widely supported
  Cons: Larger file size, slower parsing
  
CSV:
  Pros: Spreadsheet-friendly, simple structure
  Cons: Loses MARC structure, subfield complexity

ISO 2709:
  Pros: Compact binary, original MARC format
  Cons: Binary (not human-readable), requires MARC parser
    """)
    print()


def main():
    """Main example runner."""
    
    print("\n" + "=" * 70)
    print("MRRC: Format Conversion Examples (Python/pymarc-compatible)")
    print("=" * 70)
    
    # Create sample record
    record = create_sample_record()
    
    # Show original
    demonstrate_record_structure(record)
    
    # Show all conversions
    convert_to_json(record)
    convert_to_marcjson(record)
    convert_to_xml(record)
    convert_to_csv(record)
    round_trip_conversion(record)
    comparison_table()
    
    print("=" * 70)
    print("RECOMMENDATIONS")
    print("=" * 70)
    print("""
CHOOSE YOUR FORMAT BASED ON YOUR USE CASE:

1. Archival/Storage:
   → ISO 2709 binary (most compact, original format)

2. Web APIs / JSON Services:
   → JSON (mrrc format - compact and clean)
   → MARCJSON (if interoperability needed)

3. Data Analysis / Reporting:
   → CSV (simple, spreadsheet-friendly)

4. XML Processing / XSLT:
   → MARCXML (standard, tool support)

5. Linked Data / RDF:
   → MARCJSON or custom RDF conversion

PERFORMANCE TIPS:
- JSON conversions are fastest
- MARCXML is slower due to parsing overhead
- CSV generation can be done incrementally
- Keep ISO 2709 for long-term storage

MIGRATION FROM PYMARC:
All these methods work identically in pymarc:
  record.to_json()
  record.to_marcjson()
  record.to_xml()
Just swap the import and you're ready to go!
    """)
    print()


if __name__ == '__main__':
    main()
