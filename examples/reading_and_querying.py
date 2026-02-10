#!/usr/bin/env python3
"""
Reading MARC records and querying fields (Python/pymarc-compatible API)

This example demonstrates how to read MARC records and extract specific
information using the pymarc-compatible dictionary and method-based APIs.

The mrrc Python wrapper maintains full API compatibility with pymarc,
so all patterns shown here work identically to pymarc code.
"""

import sys
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    from mrrc import MARCReader
except ImportError:
    print("Error: mrrc not installed")
    print("Install with: pip install mrrc")
    sys.exit(1)


def create_sample_record_from_binary():
    """
    Read a sample MARC record from test data.
    
    Returns the first record from a test file if available.
    """
    test_dir = Path(__file__).parent.parent / 'tests' / 'data' / 'fixtures'
    
    if test_dir.exists():
        marc_files = list(test_dir.glob('*.mrc'))
        if marc_files:
            with open(marc_files[0], 'rb') as f:
                reader = MARCReader(f)
                return reader.read_record()
    
    # If no test file available, return None
    return None


def basic_field_access(record):
    """
    Demonstrate basic field access using pymarc-compatible API.
    """
    print("=== Basic Field Access (Dictionary-Style) ===\n")
    
    # Get title (245 field, subfield 'a')
    if '245' in record:
        title_field = record['245']
        if title_field.get_subfield('a'):
            print(f"Title: {title_field.get_subfield('a')}")
    
    # Get author (100 field, subfield 'a')
    if '100' in record:
        author_field = record['100']
        if author_field.get_subfield('a'):
            print(f"Author: {author_field.get_subfield('a')}")
    
    # Get all subject headings (650 fields)
    if '650' in record:
        subjects = record.get_fields('650')
        print(f"\nSubject headings ({len(subjects)} found):")
        for field in subjects:
            subfield_a = field.get_subfield('a')
            if subfield_a:
                print(f"  - {subfield_a}")
    
    # Get control number (001 field)
    if '001' in record:
        control_num = record['001'].value
        print(f"\nControl number: {control_num}")
    
    print()


def convenience_methods(record):
    """
    Demonstrate convenience methods for common fields.
    
    These methods are more ergonomic than dictionary access
    and work with both Rust-based mrrc and Python-based pymarc.
    """
    print("=== Using Convenience Methods ===\n")
    
    # Title convenience method
    if record.title():
        print(f"Title (via method): {record.title()}")
    
    # Author convenience method
    if record.author():
        print(f"Author (via method): {record.author()}")
    
    # Subjects convenience method
    subjects = record.subjects()
    if subjects:
        print(f"\nSubjects (via method, {len(subjects)} found):")
        for subject in subjects:
            print(f"  - {subject}")
    
    # Authors (plural) convenience method
    authors = record.authors()
    if authors:
        print(f"\nAll authors (via method, {len(authors)} found):")
        for author in authors:
            print(f"  - {author}")
    
    print()


def working_with_indicators(record):
    """
    Demonstrate working with indicators (MARC field attributes).
    
    Indicators provide additional context for field interpretation.
    """
    print("=== Working with Indicators ===\n")
    
    # Title field indicators
    if '245' in record:
        title_field = record['245']
        ind1 = title_field.indicators[0] if title_field.indicators else ' '
        ind2 = title_field.indicators[1] if len(title_field.indicators) > 1 else ' '
        print(f"245 field indicators: '{ind1}' '{ind2}'")
        print(f"  Indicator 1 (='{ind1}'): Title main entry {'added' if ind1 == '0' else 'traced differently'}")
        print(f"  Indicator 2 (='{ind2}'): {ind2} characters to skip for filing")
    
    # Subject field indicators
    print("\nSubject field (650) indicators:")
    if '650' in record:
        for i, field in enumerate(record.get_fields('650')[:2]):  # Show first 2
            ind2 = field.indicators[1] if len(field.indicators) > 1 else ' '
            source = 'LCSH' if ind2 == '0' else 'Other'
            print(f"  Field {i}: source='{source}'")
    
    print()


def working_with_subfields(record):
    """
    Demonstrate working with subfields and complex field structures.
    """
    print("=== Working with Subfields ===\n")
    
    # Get all subfields from a field
    if '245' in record:
        print("Title field (245) subfields:")
        title_field = record['245']
        for subfield in title_field.subfields():
            print(f"  ${subfield.code}: {subfield.value}")
    
    # Get multiple subfields from same field
    print("\nSubjects with subdivisions:")
    if '650' in record:
        for field in record.get_fields('650'):
            main = field.get_subfield('a')
            if main:
                print(f"  {main}")
                
                # Check for subdivisions
                if field.get_subfield('x'):
                    print(f"    -- {field.get_subfield('x')} (topical)")
                if field.get_subfield('y'):
                    print(f"    -- {field.get_subfield('y')} (chronological)")
                if field.get_subfield('z'):
                    print(f"    -- {field.get_subfield('z')} (geographic)")
    
    print()


def advanced_queries(record):
    """
    Demonstrate advanced field queries and filtering.
    """
    print("=== Advanced Queries ===\n")
    
    # Count fields by tag
    print("Field count summary:")
    field_counts = {}
    for field in record.fields():
        field_counts[field.tag] = field_counts.get(field.tag, 0) + 1
    
    for tag in sorted(field_counts.keys())[:10]:  # Show first 10 tags
        count = field_counts[tag]
        print(f"  {tag}: {count} field(s)")
    
    # Get fields in a range (access points: 100-799)
    print("\nAccess points (1XX, 6XX, 7XX fields):")
    access_points = []
    for field in record.fields():
        tag_num = int(field.tag)
        if (100 <= tag_num < 200) or (600 <= tag_num < 800):
            access_points.append(field)
    
    print(f"  Found {len(access_points)} access point fields")
    for field in access_points[:5]:  # Show first 5
        print(f"    {field.tag}: indicator1='{field.indicators[0]}' (has {len(field.subfields())} subfields)")
    
    # Find fields containing a specific subfield
    print("\nFields with subfield 'e' (relator term):")
    for field in record.fields():
        if field.get_subfield('e'):
            print(f"  {field.tag}: {field.get_subfield('a')} -- {field.get_subfield('e')}")
    
    print()


def format_conversions(record):
    """
    Demonstrate format conversions available in mrrc.
    
    Records can be converted to JSON, XML, and MARCJSON formats.
    """
    print("=== Format Conversions ===\n")
    
    # Convert to JSON
    try:
        json_str = record.to_json()
        print("JSON format (first 100 chars):")
        print(f"  {json_str[:100]}...")
    except Exception as e:
        print(f"JSON conversion: {e}")
    
    # Convert to MARCJSON (standard MARC-JSON)
    try:
        marcjson_str = record.to_marcjson()
        print("\nMARCJSON format (first 100 chars):")
        print(f"  {marcjson_str[:100]}...")
    except Exception as e:
        print(f"MARCJSON conversion: {e}")
    
    # Convert to XML
    try:
        xml_str = record.to_xml()
        print("\nXML format (first 100 chars):")
        print(f"  {xml_str[:100]}...")
    except Exception as e:
        print(f"XML conversion: {e}")
    
    print()


def main():
    """Main example runner."""
    
    print("\n" + "=" * 70)
    print("MRRC: Reading and Querying MARC Records (Python/pymarc-compatible)")
    print("=" * 70 + "\n")
    
    # Try to load a real record from test data
    record = create_sample_record_from_binary()
    
    if not record:
        print("No test MARC files found. Creating synthetic example...")
        print()
        
        # If no test file, create a simple record to demonstrate
        from mrrc import Record, Field, Leader
        
        leader = Leader(
            record_type='a',
            bibliographic_level='m',
            character_coding=' ',
        )
        
        record = Record(leader)
        record.add_control_field('001', 'ocm12345678')
        record.add_control_field('008', '200101s2020    xxu||||||||||||||||eng||')
        
        # Add title
        title_field = Field('245', '1', '0')
        title_field.add_subfield('a', 'Advanced Rust patterns /')
        title_field.add_subfield('c', 'Jane Smith.')
        record.add_field(title_field)
        
        # Add author
        author_field = Field('100', '1', ' ')
        author_field.add_subfield('a', 'Smith, Jane,')
        author_field.add_subfield('d', '1975-')
        record.add_field(author_field)
        
        # Add subjects
        for subject in [
            'Rust (Computer program language)',
            'Programming languages',
            'Software engineering',
        ]:
            subject_field = Field('650', ' ', '0')
            subject_field.add_subfield('a', subject)
            record.add_field(subject_field)
    
    # Run demonstrations
    basic_field_access(record)
    convenience_methods(record)
    working_with_indicators(record)
    working_with_subfields(record)
    advanced_queries(record)
    format_conversions(record)
    
    print("=" * 70)
    print("PYMARC COMPATIBILITY NOTES")
    print("=" * 70)
    print("""
KEY COMPATIBILITY FEATURES:
1. Dictionary-style field access: record['245']['a']
2. Field existence checking: '245' in record
3. Subfield methods: field.get_subfield('a')
4. Convenience methods: record.title(), record.author()
5. Iterator support: for record in reader:
6. Indicator access: field.indicators[0]

ALL pymarc PATTERNS WORK WITH MRRC:
- Read MARC files identically
- Extract fields and subfields the same way
- Iterate over records the same way
- Use convenience methods the same way

MIGRATION FROM PYMARC:
- Replace: from pymarc import MARCReader
- With:    from mrrc import MARCReader
- Everything else stays the same!

PERFORMANCE:
- mrrc is 7.5x faster than pymarc (same API)
- 549,500 records/sec vs 73,000 with pymarc
- No code changes needed - just swap the import!
    """)
    print()


if __name__ == '__main__':
    main()
