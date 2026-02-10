#!/usr/bin/env python3
"""
Create the 100-record fidelity test set for binary format evaluations.

This script:
1. Extracts diverse real-world records from the 10k fixture file
2. Creates synthetic edge case records
3. Assembles them into tests/data/fixtures/fidelity_test_100.mrc
"""

import sys
from pathlib import Path

# Add src-python to path for pymarc
sys.path.insert(0, str(Path(__file__).parent.parent / 'src-python'))

from pymarc import Record, Field, Indicators, MARCReader, MARCWriter


def create_field(tag: str, ind1: str, ind2: str, subfields: list) -> Field:
    """Helper to create a field with subfields."""
    f = Field(tag, Indicators(ind1, ind2))
    for code, value in subfields:
        f.add_subfield(code, value)
    return f


def create_edge_case_cjk() -> Record:
    """Create record with CJK (Chinese) characters."""
    record = Record()
    record.leader = '00000cam a2200000   4500'
    record.add_field(create_field('245', ' ', '1', [
        ('a', '中文标题'),  # Chinese: "Chinese Title"
        ('c', '作者')       # Chinese: "Author"
    ]))
    record.add_field(create_field('650', ' ', '0', [
        ('a', '中国')       # Chinese: "China"
    ]))
    return record


def create_edge_case_rtl() -> Record:
    """Create record with RTL (Arabic/Hebrew) text."""
    record = Record()
    record.leader = '00000cam a2200000   4500'
    record.add_field(create_field('245', ' ', '1', [
        ('a', 'العنوان العربي'),  # Arabic: "The Arabic Title"
        ('c', 'المؤلف')           # Arabic: "Author"
    ]))
    record.add_field(create_field('650', ' ', '0', [
        ('a', 'مصر')               # Arabic: "Egypt"
    ]))
    return record


def create_edge_case_combining_marks() -> Record:
    """Create record with combining diacritical marks."""
    record = Record()
    record.leader = '00000cam a2200000   4500'
    # Combining marks: e + combining acute accent (not precomposed é)
    combining_e = 'e\u0301'  # e with combining acute
    combining_multi = 'a\u0300\u0301\u0302'  # a with multiple combining marks
    
    record.add_field(create_field('245', ' ', '1', [
        ('a', f'Title with {combining_e}'),
        ('c', f'Aut{combining_multi}r')
    ]))
    return record


def create_edge_case_max_field_length() -> Record:
    """Create record with a field approaching 9999 byte limit."""
    record = Record()
    record.leader = '00000cam a2200000   4500'
    # Create a field with ~9000 bytes of data
    large_data = 'x' * 9000
    record.add_field(create_field('520', ' ', ' ', [
        ('a', large_data)
    ]))
    return record


def create_edge_case_many_fields() -> Record:
    """Create record with 100+ fields."""
    record = Record()
    record.leader = '00000cam a2200000   4500'
    
    # Add control field
    f = Field('001', data='12345678901234')
    record.add_field(f)
    
    # Add many variable fields (650 = subject headings, can repeat)
    for i in range(100):
        record.add_field(create_field('650', ' ', '0', [
            ('a', f'Subject {i:03d}'),
            ('x', f'Subdivision {i:03d}')
        ]))
    
    return record


def create_edge_case_many_subfields() -> Record:
    """Create record with field containing 50+ subfields."""
    record = Record()
    record.leader = '00000cam a2200000   4500'
    
    # Create a field with many subfields
    f = Field('653', Indicators(' ', ' '))
    for i in range(50):
        f.add_subfield(chr(97 + (i % 26)), f'Subfield value {i:02d}')
    
    record.add_field(f)
    return record


def create_edge_case_empty_subfield() -> Record:
    """Create record with empty subfield value."""
    record = Record()
    record.leader = '00000cam a2200000   4500'
    record.add_field(create_field('245', ' ', '1', [
        ('a', 'Title'),
        ('b', ''),  # Empty subfield
        ('c', 'Author')
    ]))
    return record


def create_edge_case_repeating_subfields() -> Record:
    """Create record with repeating subfields."""
    record = Record()
    record.leader = '00000cam a2200000   4500'
    record.add_field(create_field('650', ' ', '0', [
        ('a', 'First subject'),
        ('a', 'Second subject'),  # Repeating $a
        ('a', 'Third subject'),
        ('x', 'Subdivision')
    ]))
    return record


def create_edge_case_whitespace_preservation() -> Record:
    """Create record with leading/trailing whitespace in subfields."""
    record = Record()
    record.leader = '00000cam a2200000   4500'
    record.add_field(create_field('245', ' ', '1', [
        ('a', '  leading and trailing spaces  '),
        ('c', '  Author Name  ')
    ]))
    return record


def create_edge_case_multiple_245() -> Record:
    """Create record with multiple 245 fields."""
    record = Record()
    record.leader = '00000cam a2200000   4500'
    
    # Add first 245
    record.add_field(create_field('245', ' ', '1', [
        ('a', 'First title'),
        ('c', 'Author 1')
    ]))
    
    # Add second 245 (semantically wrong but structurally valid)
    record.add_field(create_field('245', ' ', '1', [
        ('a', 'Second title'),
        ('c', 'Author 2')
    ]))
    
    return record


def create_edge_case_field_reordering() -> Record:
    """Create record with non-sequential field ordering."""
    record = Record()
    record.leader = '00000cam a2200000   4500'
    
    # Add fields in unusual order
    f = Field('001', data='001value')
    record.add_field(f)
    
    record.add_field(create_field('650', ' ', '0', [('a', 'Subject')]))
    record.add_field(create_field('245', ' ', '1', [('a', 'Title'), ('c', 'Author')]))
    record.add_field(create_field('260', ' ', ' ', [('a', 'Place'), ('b', 'Publisher'), ('c', 'Date')]))
    
    return record


def create_edge_case_subfield_reordering() -> Record:
    """Create record with non-standard subfield order."""
    record = Record()
    record.leader = '00000cam a2200000   4500'
    
    # Create field with subfields in non-standard order: d, c, a (not a, c, d)
    record.add_field(create_field('700', '1', ' ', [
        ('d', '1950-'),         # Birth year (normally would be after name)
        ('c', 'Musician'),      # Title (normally would be after name)
        ('a', 'Name, First'),   # Name (normally first)
        ('q', '(Full name)')    # Fuller form (normally last)
    ]))
    
    return record


def create_edge_case_mixed_script() -> Record:
    """Create single field with multiple scripts."""
    record = Record()
    record.leader = '00000cam a2200000   4500'
    
    mixed_text = 'English مصر English עברית English'  # English + Arabic + Hebrew mixed
    record.add_field(create_field('650', ' ', '0', [
        ('a', mixed_text)
    ]))
    return record


def create_edge_case_control_field_validity() -> Record:
    """Create record with control field (001) containing exactly 12 chars."""
    record = Record()
    record.leader = '00000cam a2200000   4500'
    
    # 001 should be exactly 12 characters
    f = Field('001', data='123456789012')
    record.add_field(f)
    
    record.add_field(create_field('245', ' ', '1', [('a', 'Test Title')]))
    
    return record


def create_edge_case_blank_indicators() -> Record:
    """Create record with all blank indicators vs filled indicators."""
    record = Record()
    record.leader = '00000cam a2200000   4500'
    
    # Field with blank indicators
    record.add_field(create_field('245', ' ', ' ', [('a', 'Title with blanks')]))
    
    # Field with non-blank indicators
    record.add_field(create_field('650', '1', '0', [('a', 'Subject'), ('x', 'Subdivision')]))
    
    return record


def create_edge_case_control_chars() -> Record:
    """Create record with control characters in data."""
    record = Record()
    record.leader = '00000cam a2200000   4500'
    
    # Include tab character (0x09)
    control_data = 'Data\twith\ttabs'
    
    record.add_field(create_field('520', ' ', ' ', [
        ('a', control_data)
    ]))
    return record


def main():
    fixture_10k = Path(__file__).parent.parent / 'tests' / 'data' / 'fixtures' / '10k_records.mrc'
    output_path = Path(__file__).parent.parent / 'tests' / 'data' / 'fixtures' / 'fidelity_test_100.mrc'
    
    if not fixture_10k.exists():
        print(f"Error: {fixture_10k} not found")
        sys.exit(1)
    
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    print(f"Extracting records from {fixture_10k.name}...")
    records_to_write = []
    
    # Extract diverse records from 10k fixture
    with open(fixture_10k, 'rb') as f:
        reader = MARCReader(f)
        all_records = list(reader)
    
    print(f"Total records in 10k fixture: {len(all_records)}")
    
    # Select first ~75 diverse records
    records_to_write.extend(all_records[:75])
    
    print(f"Extracted {len(records_to_write)} real records")
    
    # Create 15 edge case records
    edge_cases = [
        create_edge_case_cjk(),
        create_edge_case_rtl(),
        create_edge_case_combining_marks(),
        create_edge_case_max_field_length(),
        create_edge_case_many_fields(),
        create_edge_case_many_subfields(),
        create_edge_case_empty_subfield(),
        create_edge_case_repeating_subfields(),
        create_edge_case_whitespace_preservation(),
        create_edge_case_multiple_245(),
        create_edge_case_field_reordering(),
        create_edge_case_subfield_reordering(),
        create_edge_case_mixed_script(),
        create_edge_case_control_field_validity(),
        create_edge_case_blank_indicators(),
    ]
    
    print(f"Created {len(edge_cases)} edge case records")
    
    # Add remaining real records
    records_to_write.extend(all_records[75:90])
    records_to_write.extend(edge_cases)
    
    # Ensure we have exactly 105 records
    records_to_write = records_to_write[:105]
    
    print(f"Writing {len(records_to_write)} records to {output_path}...")
    
    # Write to MRC file
    with open(output_path, 'wb') as f:
        writer = MARCWriter(f)
        for record in records_to_write:
            writer.write(record)
    
    print(f"✓ Fidelity test set created: {output_path}")
    print(f"  Total records: {len(records_to_write)}")
    print(f"  File size: {output_path.stat().st_size:,} bytes")
    
    # Verify by reading back
    with open(output_path, 'rb') as f:
        reader = MARCReader(f)
        verified = list(reader)
    
    print(f"  Verified: {len(verified)} records read back")
    if len(verified) == len(records_to_write):
        print("✓ Validation PASSED: Record count matches")
    else:
        print(f"✗ Validation FAILED: Expected {len(records_to_write)}, got {len(verified)}")
        sys.exit(1)


if __name__ == '__main__':
    main()
