#!/usr/bin/env python3
"""
Generate large MARC test fixtures for benchmarking the Python wrapper.

This script creates test data files with varying numbers of records for
performance testing. Fixtures include diverse record types to simulate
realistic workloads.
"""

import struct
import os
import sys
from pathlib import Path

# MARC constants
FIELD_TERMINATOR = b'\x1e'
SUBFIELD_DELIMITER = b'\x1f'
RECORD_TERMINATOR = b'\x1d'


def build_leader(record_length, base_address, record_type='a', bib_level='m'):
    """Build a 24-byte MARC leader."""
    leader = bytearray()
    leader.extend(f'{record_length:05d}'.encode('ascii'))      # 0-4: record length
    leader.append(ord('n'))                                     # 5: status
    leader.append(ord(record_type))                             # 6: record type
    leader.append(ord(bib_level))                               # 7: bibliographic level
    leader.append(ord(' '))                                     # 8: control type
    leader.append(ord('a'))                                     # 9: character coding
    leader.append(ord('2'))                                     # 10: indicator count
    leader.append(ord('2'))                                     # 11: subfield code count
    leader.extend(f'{base_address:05d}'.encode('ascii'))        # 12-16: base address
    leader.append(ord(' '))                                     # 17: encoding level
    leader.append(ord(' '))                                     # 18: cataloging form
    leader.append(ord(' '))                                     # 19: multipart level
    leader.extend(b'4500')                                       # 20-23: reserved
    
    return bytes(leader)


def build_directory_and_data(fields_data):
    """Build directory and data area from field dict.
    
    Args:
        fields_data: Dict mapping tag -> field_bytes
    
    Returns:
        Tuple of (data_area, directory)
    """
    data_area = b''
    directory = b''
    current_pos = 0

    # Process fields in tag order
    for tag in sorted(fields_data.keys()):
        field_bytes = fields_data[tag]
        field_length = len(field_bytes)
        
        # Add directory entry: tag(3) + length(4) + offset(5)
        directory += tag.encode('ascii')
        directory += f'{field_length:04d}'.encode('ascii')
        directory += f'{current_pos:05d}'.encode('ascii')
        
        # Add field data
        data_area += field_bytes
        current_pos += field_length

    # Add directory terminator
    directory += FIELD_TERMINATOR
    return data_area, directory


def build_marc_record(fields_data):
    """Build a complete MARC record from fields."""
    data_area, directory = build_directory_and_data(fields_data)
    base_address = 24 + len(directory)
    record_length = base_address + len(data_area) + 1
    
    leader = build_leader(record_length, base_address)
    record = leader + directory + data_area + RECORD_TERMINATOR
    return record


def create_book_record(record_num):
    """Create a book record with varying content."""
    fields_data = {}
    
    # Field 008 - Fixed-length data
    field_008 = b'200101s2020    xxu||||||||||||||||eng||' + FIELD_TERMINATOR
    fields_data['008'] = field_008
    
    # Field 100 - Author
    author = f'Author, Test {record_num % 1000}'.encode('utf-8')
    field_100 = b'1 ' + SUBFIELD_DELIMITER + b'a' + author + FIELD_TERMINATOR
    fields_data['100'] = field_100
    
    # Field 245 - Title
    title = f'Test Book Number {record_num}'.encode('utf-8')
    field_245 = b'10' + SUBFIELD_DELIMITER + b'a' + title + SUBFIELD_DELIMITER + b'cTest Author' + FIELD_TERMINATOR
    fields_data['245'] = field_245
    
    # Field 260 - Publication
    pub = f'Test City : Test Publishers, {2000 + (record_num % 25)}'.encode('utf-8')
    field_260 = b' 1' + SUBFIELD_DELIMITER + b'a' + pub + FIELD_TERMINATOR
    fields_data['260'] = field_260
    
    # Field 300 - Physical Description
    field_300 = b' ' * 2 + SUBFIELD_DELIMITER + b'a' + str(100 + record_num % 400).encode('utf-8') + b' pages' + FIELD_TERMINATOR
    fields_data['300'] = field_300
    
    # Field 500 - General Note (varies)
    if record_num % 5 == 0:
        field_500 = b' ' * 2 + SUBFIELD_DELIMITER + b'aA note about this record.' + FIELD_TERMINATOR
        fields_data['500'] = field_500
    
    # Field 650 - Subject (multiple)
    subjects = ['Fiction', 'Literature', 'Novels', 'Contemporary']
    for i, subj in enumerate(subjects[:((record_num % 4) + 1)]):
        tag = f'65{i}'
        field = b' 0' + SUBFIELD_DELIMITER + b'a' + subj.encode('utf-8') + FIELD_TERMINATOR
        fields_data[tag] = field
    
    # Field 856 - Electronic location (sometimes)
    if record_num % 10 == 0:
        field_856 = b'40' + SUBFIELD_DELIMITER + b'u' + b'https://example.com/book' + str(record_num).encode('utf-8') + FIELD_TERMINATOR
        fields_data['856'] = field_856
    
    return build_marc_record(fields_data)


def create_authority_record(record_num):
    """Create an authority record."""
    fields_data = {}
    
    # Field 008 - Authority control
    field_008 = b'200101n  azznnaabn          |a aaa      ' + FIELD_TERMINATOR
    fields_data['008'] = field_008
    
    # Field 150 - Topical Term
    term = f'Test Term {record_num}'.encode('utf-8')
    field_150 = b' ' * 2 + SUBFIELD_DELIMITER + b'a' + term + FIELD_TERMINATOR
    fields_data['150'] = field_150
    
    # Field 450 - See From
    if record_num % 3 == 0:
        field_450 = b' ' * 2 + SUBFIELD_DELIMITER + b'a' + b'Variant Term' + FIELD_TERMINATOR
        fields_data['450'] = field_450
    
    return build_marc_record(fields_data)


def generate_fixture(output_path, num_records, progress=True):
    """Generate a fixture file with specified number of records.
    
    Args:
        output_path: Path to write the fixture file
        num_records: Number of records to generate
        progress: Whether to print progress
    """
    output_path = Path(output_path)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    with open(output_path, 'wb') as f:
        for i in range(num_records):
            # Alternate between book and authority records
            if i % 5 == 0:
                record = create_authority_record(i)
            else:
                record = create_book_record(i)
            
            f.write(record)
            
            if progress and (i + 1) % 10000 == 0:
                print(f"  {i + 1:,} records written...", file=sys.stderr)
    
    file_size = output_path.stat().st_size
    size_mb = file_size / (1024 * 1024)
    print(f"Created {output_path} with {num_records:,} records ({size_mb:.2f} MB)")


def main():
    """Generate benchmark fixtures."""
    fixtures_dir = Path('tests/data/fixtures')
    fixtures_dir.mkdir(parents=True, exist_ok=True)
    
    print("Generating MARC benchmark fixtures...")
    print()
    
    # Small fixture for quick tests
    print("Small fixture (1k records):")
    generate_fixture(fixtures_dir / '1k_records.mrc', 1000)
    print()
    
    # Medium fixture for standard benchmarks
    print("Medium fixture (10k records):")
    generate_fixture(fixtures_dir / '10k_records.mrc', 10000)
    print()
    
    print("✓ All fixtures generated successfully!")
    print()
    print("Available fixtures:")
    for fixture in sorted(fixtures_dir.glob('*.mrc')):
        size_mb = fixture.stat().st_size / (1024 * 1024)
        with open(fixture, 'rb') as f:
            content = f.read()
            # Count records by counting record terminators
            count = content.count(RECORD_TERMINATOR)
        print(f"  {fixture.name}: {count:,} records ({size_mb:.2f} MB)")


if __name__ == '__main__':
    main()
