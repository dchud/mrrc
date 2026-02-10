#!/usr/bin/env python3
"""
Create sample MARC binary records for testing the Rust MARC library.
This script generates test data files that mimic pymarc test data.
"""

import struct
import os

# Constants
FIELD_TERMINATOR = b'\x1e'
SUBFIELD_DELIMITER = b'\x1f'
RECORD_TERMINATOR = b'\x1d'


def create_simple_book_record():
    """Create a simple bibliographic record for a book."""
    # Build data fields first
    fields_data = {}

    # Field 100 - Main Entry
    field_100 = b'1 ' + SUBFIELD_DELIMITER + b'aFitzgerald, F. Scott' + FIELD_TERMINATOR
    fields_data['100'] = field_100

    # Field 245 - Title Statement
    field_245 = b'10' + SUBFIELD_DELIMITER + b'aThe Great Gatsby' + \
                SUBFIELD_DELIMITER + b'cF. Scott Fitzgerald' + FIELD_TERMINATOR
    fields_data['245'] = field_245

    # Field 650 - Subject Added Entry
    field_650 = b' 0' + SUBFIELD_DELIMITER + b'aAmerican fiction' + FIELD_TERMINATOR
    fields_data['650'] = field_650

    return build_marc_record(fields_data)


def create_music_record():
    """Create a record for a musical score."""
    fields_data = {}

    # Field 100 - Composer
    field_100 = b'1 ' + SUBFIELD_DELIMITER + b'aBeethoven, Ludwig van' + FIELD_TERMINATOR
    fields_data['100'] = field_100

    # Field 245 - Title
    field_245 = b'10' + SUBFIELD_DELIMITER + b'aBeethovens Ninth Symphony' + FIELD_TERMINATOR
    fields_data['245'] = field_245

    # Build with music type in leader
    data_area, directory = build_directory_and_data(fields_data)
    base_address = 24 + len(directory)
    record_length = base_address + len(data_area) + 1

    leader = build_leader(record_length, base_address, record_type='c')
    record = leader + directory + data_area + RECORD_TERMINATOR
    return record


def create_with_control_fields():
    """Create a record with control fields."""
    fields_data = {}

    # Field 008 - Fixed-length data elements
    field_008 = b'200101s2020    xxua   j      000 0 eng d' + FIELD_TERMINATOR
    fields_data['008'] = field_008

    # Field 245 - Title
    field_245 = b'00' + SUBFIELD_DELIMITER + b"aChildren's Book" + FIELD_TERMINATOR
    fields_data['245'] = field_245

    return build_marc_record(fields_data)


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


def build_marc_record(fields_data):
    """Build a complete MARC record from fields."""
    data_area, directory = build_directory_and_data(fields_data)
    base_address = 24 + len(directory)
    record_length = base_address + len(data_area) + 1
    
    leader = build_leader(record_length, base_address)
    record = leader + directory + data_area + RECORD_TERMINATOR
    return record


def main():
    """Generate test data files."""
    os.makedirs('tests/data', exist_ok=True)

    # Create sample records
    records = [
        ('tests/data/simple_book.mrc', create_simple_book_record()),
        ('tests/data/music_score.mrc', create_music_record()),
        ('tests/data/with_control_fields.mrc', create_with_control_fields()),
    ]

    for filename, record_data in records:
        with open(filename, 'wb') as f:
            f.write(record_data)
        print(f'Created {filename}')

    # Also create a multi-record file
    with open('tests/data/multi_records.mrc', 'wb') as f:
        for record_data in [
            create_simple_book_record(),
            create_music_record(),
            create_with_control_fields(),
        ]:
            f.write(record_data)
    print('Created tests/data/multi_records.mrc')


if __name__ == '__main__':
    main()
