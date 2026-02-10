#!/usr/bin/env python3
"""
Creating MARC records (Python/pymarc-compatible API)

This example demonstrates how to create MARC records from scratch
using the pymarc-compatible API in mrrc. All patterns shown here
work identically to pymarc code.

The pymarc-compatible API is the most ergonomic for Python users.
Use the method-chaining approach for complex records.
"""

import sys
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    from mrrc import MARCReader, MARCWriter, Record, Field, Leader
except ImportError:
    print("Error: mrrc not installed")
    print("Install with: pip install mrrc")
    sys.exit(1)


def simple_record():
    """
    Create a simple bibliographic record.
    
    Demonstrates basic record creation with:
    - Control fields (001, 008)
    - Title field (245)
    - Author field (100)
    - Subject headings (650)
    """
    print("\n" + "=" * 70)
    print("1. SIMPLE BIBLIOGRAPHIC RECORD")
    print("=" * 70 + "\n")
    
    # Create a leader for a bibliographic record
    leader = Leader(
        record_type='a',           # 'a' = language material
        bibliographic_level='m',   # 'm' = monograph
        character_coding=' ',      # ' ' = MARC-8, 'a' = UTF-8
    )
    
    # Create record
    record = Record(leader)
    
    # Add control fields
    record.add_control_field('001', '9780061120084')  # Control number
    record.add_control_field('008', '051029s2005    xxu||||||||||||||||eng||')  # Fixed-length data
    
    # Add title field (245)
    title = Field('245', '1', '0')
    title.add_subfield('a', 'To Kill a Mockingbird /')
    title.add_subfield('c', 'Harper Lee.')
    record.add_field(title)
    
    # Add author field (100)
    author = Field('100', '1', ' ')
    author.add_subfield('a', 'Lee, Harper,')
    author.add_subfield('d', '1926-2016,')
    author.add_subfield('e', 'author.')
    record.add_field(author)
    
    # Add subject headings
    for subject in ['Psychological fiction.', 'Legal stories.']:
        subject_field = Field('650', ' ', '0')
        subject_field.add_subfield('a', subject)
        record.add_field(subject_field)
    
    # Display results
    print(f"Record Type:    {record.leader.record_type}")
    print(f"Control Number: {record.get_control_field('001')}")
    print(f"Title:          {record.title()}")
    print(f"Author:         {record.author()}")
    print(f"Subjects:       {', '.join(record.subjects())}")
    print()


def record_with_complex_fields():
    """
    Create a record with more complex field structures.
    
    Demonstrates:
    - Multiple subfields in one field
    - Subdivisions in subject fields (x, y, z)
    - Publication information (260)
    - Physical description (300)
    - ISBN field (020)
    """
    print("\n" + "=" * 70)
    print("2. RECORD WITH COMPLEX FIELD STRUCTURES")
    print("=" * 70 + "\n")
    
    leader = Leader(
        record_type='a',
        bibliographic_level='m',
        character_coding='a',  # UTF-8
    )
    
    record = Record(leader)
    
    # Control fields
    record.add_control_field('001', '12345678')
    record.add_control_field('005', '20051229123456.0')
    record.add_control_field('008', '051229s2005    xxu||||||||||||||||eng||')
    
    # ISBN
    isbn = Field('020', ' ', ' ')
    isbn.add_subfield('a', '9780596004957')
    record.add_field(isbn)
    
    # Title
    title = Field('245', '1', '0')
    title.add_subfield('a', 'Introduction to quantum mechanics /')
    title.add_subfield('c', 'David J. Griffiths.')
    record.add_field(title)
    
    # Author
    author = Field('100', '1', ' ')
    author.add_subfield('a', 'Griffiths, David J.,')
    author.add_subfield('d', '1942-')
    author.add_subfield('e', 'author.')
    record.add_field(author)
    
    # Publication information
    publication = Field('260', ' ', ' ')
    publication.add_subfield('a', 'Boston :')
    publication.add_subfield('b', 'Pearson,')
    publication.add_subfield('c', '2005.')
    record.add_field(publication)
    
    # Physical description
    physical = Field('300', ' ', ' ')
    physical.add_subfield('a', 'xvii, 468 pages :')
    physical.add_subfield('b', 'color illustrations ;')
    physical.add_subfield('c', '26 cm')
    record.add_field(physical)
    
    # Subjects with subdivisions
    subject1 = Field('650', ' ', '0')
    subject1.add_subfield('a', 'Quantum mechanics')
    subject1.add_subfield('v', 'Textbooks.')
    record.add_field(subject1)
    
    subject2 = Field('650', ' ', '0')
    subject2.add_subfield('a', 'Physics')
    subject2.add_subfield('x', 'Study and teaching')
    subject2.add_subfield('z', 'Higher.')
    record.add_field(subject2)
    
    # Display results
    print(f"Title:        {record.title()}")
    print(f"Author:       {record.author()}")
    print(f"ISBN:         {', '.join(record.isbns())}")
    
    if record.publication_info():
        pub = record.publication_info()
        print(f"Published:    {pub.date} in {pub.place}")
        if pub.publisher:
            print(f"Publisher:    {pub.publisher}")
    
    print(f"\nSubjects:")
    for subject in record.subjects():
        print(f"  - {subject}")
    print()


def record_with_multiple_entries():
    """
    Create a record with multiple authors/contributors and subject fields.
    
    Demonstrates:
    - Main entry (100)
    - Added entries (700)
    - Multiple subject headings with different sources
    - Genre/form information (655)
    """
    print("\n" + "=" * 70)
    print("3. RECORD WITH MULTIPLE ENTRIES AND CONTRIBUTORS")
    print("=" * 70 + "\n")
    
    leader = Leader(
        record_type='a',
        bibliographic_level='m',
    )
    
    record = Record(leader)
    
    # Control fields
    record.add_control_field('001', 'ocm00123456')
    record.add_control_field('008', '051229s2005    xxu||||||||||||||||eng||')
    
    # Main author
    main_author = Field('100', '1', ' ')
    main_author.add_subfield('a', 'Doe, John,')
    main_author.add_subfield('d', '1950-')
    main_author.add_subfield('e', 'author.')
    record.add_field(main_author)
    
    # Title
    title = Field('245', '1', '4')
    title.add_subfield('a', 'The guide to advanced Rust programming /')
    title.add_subfield('c', 'John Doe.')
    record.add_field(title)
    
    # Added entry - editor
    editor = Field('700', '1', ' ')
    editor.add_subfield('a', 'Smith, Jane,')
    editor.add_subfield('d', '1960-')
    editor.add_subfield('e', 'editor.')
    record.add_field(editor)
    
    # Added entry - contributor
    contributor = Field('700', '1', ' ')
    contributor.add_subfield('a', 'Jones, Bob,')
    contributor.add_subfield('d', '1970-')
    contributor.add_subfield('e', 'contributor.')
    record.add_field(contributor)
    
    # Subject headings from different sources
    subject1 = Field('650', ' ', '0')  # LCSH
    subject1.add_subfield('a', 'Rust (Computer program language)')
    record.add_field(subject1)
    
    subject2 = Field('650', ' ', '0')  # LCSH
    subject2.add_subfield('a', 'Systems programming')
    record.add_field(subject2)
    
    subject3 = Field('650', ' ', '7')  # Other source
    subject3.add_subfield('a', 'Performance optimization')
    subject3.add_subfield('2', 'local')
    record.add_field(subject3)
    
    # Genre/form
    genre = Field('655', ' ', '7')
    genre.add_subfield('a', 'Handbooks and manuals.')
    genre.add_subfield('2', 'lcgft')
    record.add_field(genre)
    
    # Display results
    print(f"Main Author:     {record.author()}")
    
    print(f"\nAll Authors and Contributors:")
    all_authors = record.authors()
    for author in all_authors:
        print(f"  - {author}")
    
    print(f"\nSubjects:")
    for subject in record.subjects():
        print(f"  - {subject}")
    
    print(f"\nGenre/Form:")
    if '655' in record:
        for field in record.get_fields('655'):
            if field.get_subfield('a'):
                print(f"  - {field.get_subfield('a')}")
    print()


def writing_records():
    """
    Demonstrate writing records to a MARC file.
    
    Shows how to:
    - Create multiple records
    - Write them to a file
    - Read them back to verify
    """
    print("\n" + "=" * 70)
    print("4. WRITING RECORDS TO FILE")
    print("=" * 70 + "\n")
    
    # Create sample records
    records = []
    
    for i in range(3):
        leader = Leader(record_type='a', bibliographic_level='m')
        record = Record(leader)
        
        record.add_control_field('001', f'test{i:05d}')
        record.add_control_field('008', '200101s2020    xxu||||||||||||||||eng||')
        
        title = Field('245', '1', '0')
        title.add_subfield('a', f'Sample Record {i + 1} /')
        title.add_subfield('c', f'Author {i + 1}.')
        record.add_field(title)
        
        author = Field('100', '1', ' ')
        author.add_subfield('a', f'Author {i + 1},')
        record.add_field(author)
        
        records.append(record)
    
    # Write to a temporary file
    import tempfile
    import os
    
    with tempfile.NamedTemporaryFile(mode='wb', suffix='.mrc', delete=False) as f:
        temp_file = f.name
        writer = MARCWriter(f)
        for record in records:
            writer.write_record(record)
    
    try:
        # Read back and verify
        print(f"Wrote {len(records)} records to {temp_file}")
        
        with open(temp_file, 'rb') as f:
            reader = MARCReader(f)
            read_count = 0
            print("\nRecords read back:")
            for record in reader:
                print(f"  {read_count + 1}. {record.title()}")
                read_count += 1
        
        print(f"\nVerification: {read_count}/{len(records)} records successfully round-tripped")
        
    finally:
        # Clean up
        os.unlink(temp_file)
    
    print()


def format_conversions():
    """
    Demonstrate format conversion when creating records.
    
    Shows how to convert a newly created record to various formats.
    """
    print("\n" + "=" * 70)
    print("5. FORMAT CONVERSIONS")
    print("=" * 70 + "\n")
    
    leader = Leader(record_type='a', bibliographic_level='m')
    record = Record(leader)
    
    record.add_control_field('001', 'test123')
    record.add_control_field('008', '200101s2020    xxu||||||||||||||||eng||')
    
    title = Field('245', '1', '0')
    title.add_subfield('a', 'Test Record /')
    title.add_subfield('c', 'Test Author.')
    record.add_field(title)
    
    author = Field('100', '1', ' ')
    author.add_subfield('a', 'Author, Test')
    record.add_field(author)
    
    subject = Field('650', ' ', '0')
    subject.add_subfield('a', 'Test subject')
    record.add_field(subject)
    
    # Convert to various formats
    print("Original record:")
    print(f"  Title: {record.title()}")
    print(f"  Author: {record.author()}")
    print()
    
    # JSON
    try:
        json_str = record.to_json()
        print("JSON (first 150 chars):")
        print(f"  {json_str[:150]}...")
    except Exception as e:
        print(f"JSON conversion failed: {e}")
    
    print()
    
    # MARCJSON
    try:
        marcjson_str = record.to_marcjson()
        print("MARCJSON (first 150 chars):")
        print(f"  {marcjson_str[:150]}...")
    except Exception as e:
        print(f"MARCJSON conversion failed: {e}")
    
    print()
    
    # XML
    try:
        xml_str = record.to_xml()
        print("XML (first 150 chars):")
        print(f"  {xml_str[:150]}...")
    except Exception as e:
        print(f"XML conversion failed: {e}")
    
    print()


def main():
    """Main example runner."""
    
    print("\n" + "=" * 70)
    print("MRRC: Creating MARC Records (Python/pymarc-compatible)")
    print("=" * 70)
    
    simple_record()
    record_with_complex_fields()
    record_with_multiple_entries()
    writing_records()
    format_conversions()
    
    print("=" * 70)
    print("TIPS FOR CREATING RECORDS")
    print("=" * 70)
    print("""
1. FIELD TAGS AND INDICATORS:
   - First indicator: context-specific (check MARC21 standards)
   - Second indicator: also context-specific
   - Examples:
     * '245' (title): indicators are '1', '0' for normal title
     * '100' (author): indicator '1' for personal name
     * '650' (subject): indicators ' ', '0' for LCSH

2. COMMON CONTROL FIELDS:
   - '001': Control number (ISBN, OCLC number, etc.)
   - '005': Date/time of latest transaction
   - '008': Fixed-length data elements (publication date, language, etc.)

3. SUBFIELD CODES:
   - 'a': Main part of heading
   - 'x', 'y', 'z': Subdivisions (topical, chronological, geographic)
   - 'e': Relator term (author, editor, translator, etc.)
   - 'd': Date (birth/death dates, publication date)

4. RECOMMENDED PATTERN:
   ```python
   leader = Leader(record_type='a', bibliographic_level='m')
   record = Record(leader)
   record.add_control_field('001', 'my-control-number')
   
   # Add fields
   field = Field('tag', 'ind1', 'ind2')
   field.add_subfield('a', 'main text')
   record.add_field(field)
   
   # Write
   with open('output.mrc', 'wb') as f:
       writer = MARCWriter(f)
       writer.write_record(record)
   ```

5. PYMARC COMPATIBILITY:
   All patterns work identically in pymarc and mrrc.
   Just swap the import and you're done!
    """)
    print()


if __name__ == '__main__':
    main()
