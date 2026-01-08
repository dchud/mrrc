#!/usr/bin/env python3
"""
Working with MARC Authority and Holdings records (Python/pymarc-compatible)

This example demonstrates how to work with Authority records and Holdings records,
which are specialized record types for maintaining authority data and item holdings.

Authority records (Type 'z') are used for:
- Authorized access points (names, subjects, titles)
- Variant names and see-also references
- Authority control data

Holdings records (Types 'x', 'y', 'v', 'u') are used for:
- Item-level information (call numbers, locations, conditions)
- Copy information
- Copy-specific notes
"""

import sys
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    from mrrc import Record, Field, Leader
except ImportError:
    print("Error: mrrc not installed")
    print("Install with: pip install mrrc")
    sys.exit(1)


def create_authority_record():
    """
    Create a sample authority record for a personal name.
    
    Authority records are used for maintaining authorized access points
    and their variant forms.
    """
    print("\n" + "=" * 70)
    print("1. AUTHORITY RECORD (Personal Name)")
    print("=" * 70 + "\n")
    
    # Authority records use record type 'z'
    leader = Leader(
        record_type='z',  # 'z' = Authority data
        bibliographic_level='a',  # Authority record
        character_coding=' ',  # MARC-8
    )
    
    record = Record(leader)
    
    # Control number
    record.add_control_field('001', 'n79021850')
    
    # Fixed-length data (008 for Authority records)
    # Format: YYMMDDX1X2X3X4X5X6X7X8X9X10X11X12X13
    record.add_control_field('008', '840117n| acannaabn          |a ana')
    
    # Main heading - Personal name (100)
    heading = Field('100', '1', ' ')
    heading.add_subfield('a', 'Twain, Mark,')
    heading.add_subfield('d', '1835-1910.')
    heading.add_subfield('e', 'author.')
    record.add_field(heading)
    
    # Variant names - See from tracings (400)
    # These are alternate forms that should refer back to the authorized heading
    variant1 = Field('400', '1', ' ')
    variant1.add_subfield('a', 'Clemens, Samuel Langhorne,')
    variant1.add_subfield('d', '1835-1910.')
    record.add_field(variant1)
    
    variant2 = Field('400', '0', ' ')
    variant2.add_subfield('a', 'Samuel Clemens')
    record.add_field(variant2)
    
    # Related headings - See also from tracings (500)
    related = Field('500', '1', ' ')
    related.add_subfield('a', 'Twain, Mark,')
    related.add_subfield('d', '1835-1910.')
    related.add_subfield('x', 'Characters.')
    record.add_field(related)
    
    # Subject field (650) - topics associated with this authority
    subject = Field('650', ' ', '0')
    subject.add_subfield('a', 'American literature')
    subject.add_subfield('z', '19th century.')
    record.add_field(subject)
    
    # Display the authority record
    print(f"Record Type:        {record.leader.record_type} (Authority)")
    print(f"Control Number:     {record.get_control_field('001')}")
    print(f"\nAuthorized Heading (100):")
    if '100' in record:
        field = record['100']
        if field.get_subfield('a'):
            print(f"  Name: {field.get_subfield('a')}")
        if field.get_subfield('d'):
            print(f"  Dates: {field.get_subfield('d')}")
    
    print(f"\nVariant Names (400 - See from tracings):")
    for field in record.get_fields('400'):
        name = field.get_subfield('a')
        if name:
            print(f"  - {name}")
    
    print(f"\nRelated Headings (500 - See also tracings):")
    for field in record.get_fields('500'):
        name = field.get_subfield('a')
        if name:
            print(f"  - {name}")
    
    print()
    return record


def create_holdings_record():
    """
    Create a sample holdings record.
    
    Holdings records contain information about the physical items
    held by a library (call numbers, locations, conditions, etc.).
    """
    print("\n" + "=" * 70)
    print("2. HOLDINGS RECORD")
    print("=" * 70 + "\n")
    
    # Holdings records use record types 'x', 'y', 'v', or 'u'
    leader = Leader(
        record_type='x',  # 'x' = Physical characteristics
        bibliographic_level='y',  # Analytical or bibliographic
        character_coding=' ',  # MARC-8
    )
    
    record = Record(leader)
    
    # Bibliographic record control number (this is a pointer to the bib record)
    record.add_control_field('001', 'h001234567')  # Holdings control number
    
    # 004 field - Control number of the bibliographic record
    record.add_control_field('004', '004123456789')  # Bib record control number
    
    # Holdings statement - unformatted (852)
    # This is the main holdings field showing location and call number
    holdings = Field('852', ' ', ' ')
    holdings.add_subfield('b', 'MAIN')  # Shelving location
    holdings.add_subfield('h', 'PS1305')  # Call number classification
    holdings.add_subfield('i', '.T2')  # Call number prefix
    holdings.add_subfield('k', '1998')  # Call number suffix (year)
    record.add_field(holdings)
    
    # Holdings statement - structured (866)
    # For serial publications, showing issues held
    statement = Field('866', '1', ' ')
    statement.add_subfield('a', 'v.1 (1999) - v.10 (2008)')  # Enumeration
    record.add_field(statement)
    
    # Item information (876/877/878)
    # Individual item-level data
    item1 = Field('876', ' ', ' ')
    item1.add_subfield('a', '00000001')  # Item barcode
    item1.add_subfield('p', 'PS1305.T2 1998')  # Call number
    item1.add_subfield('j', 'IN LIBRARY')  # Item status
    record.add_field(item1)
    
    item2 = Field('876', ' ', ' ')
    item2.add_subfield('a', '00000002')  # Item barcode
    item2.add_subfield('p', 'PS1305.T2 1998')  # Call number
    item2.add_subfield('j', 'CHECKED OUT')  # Item status
    record.add_field(item2)
    
    # Display the holdings record
    print(f"Record Type:        {record.leader.record_type} (Holdings)")
    print(f"Holdings Control #: {record.get_control_field('001')}")
    print(f"Bibliographic #:    {record.get_control_field('004')}")
    
    print(f"\nLocation and Call Number (852):")
    if '852' in record:
        field = record['852']
        location = field.get_subfield('b')
        call_num = ''.join([
            field.get_subfield('h') or '',
            field.get_subfield('i') or '',
            field.get_subfield('k') or '',
        ])
        if location:
            print(f"  Location: {location}")
        if call_num:
            print(f"  Call Number: {call_num}")
    
    print(f"\nItems Held (876):")
    for field in record.get_fields('876'):
        barcode = field.get_subfield('a')
        status = field.get_subfield('j')
        if barcode:
            print(f"  - Barcode: {barcode} ({status or 'Unknown status'})")
    
    print()
    return record


def authority_control_example():
    """
    Demonstrate authority control - how authority records relate to bibliographic records.
    """
    print("\n" + "=" * 70)
    print("3. AUTHORITY CONTROL IN BIBLIOGRAPHIC RECORDS")
    print("=" * 70 + "\n")
    
    # Create a bibliographic record that uses authorized headings
    leader = Leader(
        record_type='a',
        bibliographic_level='m',
        character_coding=' ',
    )
    
    record = Record(leader)
    
    record.add_control_field('001', 'ocm123456789')
    record.add_control_field('008', '200101s2020    xxu||||||||||||||||eng||')
    
    # Title
    title = Field('245', '1', '4')
    title.add_subfield('a', 'The adventures of Huckleberry Finn /')
    title.add_subfield('c', 'Mark Twain.')
    record.add_field(title)
    
    # Main author entry - authority controlled
    # The '0' or '1' indicator affects how this links to authority
    author = Field('100', '1', ' ')
    author.add_subfield('a', 'Twain, Mark,')  # Must match authority record exactly
    author.add_subfield('d', '1835-1910.')
    author.add_subfield('0', 'n79021850')  # Authority record control number
    record.add_field(author)
    
    # Subject headings - authority controlled
    # Using authorized headings from Library of Congress Subject Headings (LCSH)
    subject1 = Field('650', ' ', '0')
    subject1.add_subfield('a', 'American fiction')
    subject1.add_subfield('y', '19th century.')
    subject1.add_subfield('0', 'sh85004340')  # LCSH authority number
    record.add_field(subject1)
    
    subject2 = Field('650', ' ', '0')
    subject2.add_subfield('a', 'Satire')
    subject2.add_subfield('0', 'sh85117911')  # LCSH authority number
    record.add_field(subject2)
    
    # Geographic subject - authority controlled
    geogr = Field('651', ' ', '0')
    geogr.add_subfield('a', 'Mississippi River')
    geogr.add_subfield('x', 'History.')
    geogr.add_subfield('0', 'sh85088040')  # LCSH authority number
    record.add_field(geogr)
    
    print("Bibliographic Record with Authority Control")
    print(f"Title: {record.title()}")
    print(f"Author: {record.author()}")
    
    print(f"\nAuthority-Controlled Headings:")
    print(f"  Author authority #: {record['100'].get_subfield('0') if '100' in record else 'N/A'}")
    
    print(f"\nSubject Headings (with LCSH authority numbers):")
    for field in record.get_fields('650'):
        subject = field.get_subfield('a')
        auth_num = field.get_subfield('0')
        if subject:
            print(f"  - {subject}")
            if auth_num:
                print(f"    (Authority: {auth_num})")
    
    print(f"\nGeographic Headings:")
    for field in record.get_fields('651'):
        geogr_name = field.get_subfield('a')
        if geogr_name:
            print(f"  - {geogr_name}")
    
    print()


def main():
    """Main example runner."""
    
    print("\n" + "=" * 70)
    print("MRRC: Authority and Holdings Records (Python/pymarc-compatible)")
    print("=" * 70)
    
    # Create examples
    auth_record = create_authority_record()
    holdings_record = create_holdings_record()
    authority_control_example()
    
    print("=" * 70)
    print("KEY CONCEPTS")
    print("=" * 70)
    print("""
AUTHORITY RECORDS (Type 'z'):
- Used for maintaining authorized access points
- Contain authorized heading in 1XX field
- Contain variant forms in 4XX fields (see from)
- Contain related headings in 5XX fields (see also)
- Help standardize how names/subjects are used across records

HOLDINGS RECORDS (Types 'x', 'y', 'v', 'u'):
- Contain item-level information (call numbers, locations)
- Point to bibliographic record via 004 field
- Main holdings info in 852 field
- Serial holdings in 866 field
- Individual item data in 876-878 fields

AUTHORITY CONTROL IN BIBLIOGRAPHIC RECORDS:
- $0 subfield contains authority record number
- Ensures consistent use of names, subjects, etc.
- Enables authority-based searching and linking
- Critical for library catalog quality

RECORD TYPE CODES:
- 'a' = Bibliographic (language material)
- 'z' = Authority data
- 'x', 'y', 'v', 'u' = Holdings data
- Others: 'c' (music score), 'e' (cartographic), etc.

COMMON HEADING FIELDS:
- 1XX: Main heading (100 personal, 110 corporate, 111 meeting)
- 2XX: Cross references (not used in authority)
- 4XX: See from tracings (variant forms)
- 5XX: See also tracings (related terms)
- 6XX, 7XX: Subject and name added entries (in bibliographic)
    """)
    print()


if __name__ == '__main__':
    main()
