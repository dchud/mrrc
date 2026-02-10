#!/usr/bin/env python3
"""
MARC-8 Encoding Support (Python/pymarc-compatible)

This example demonstrates:
- Understanding MARC-8 vs UTF-8 character encoding
- How encoding affects field interpretation
- Working with multilingual records (Hebrew, Arabic, Cyrillic, etc.)
- Proper handling of combining characters and diacritics

MARC-8 is a legacy encoding system that uses escape sequences to switch
between different character sets. Modern systems use UTF-8, but MARC-8
records are still common in legacy library systems.
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


def explain_character_encoding():
    """
    Explain the two main MARC character encodings.
    """
    print("\n" + "=" * 70)
    print("1. CHARACTER ENCODING BASICS")
    print("=" * 70 + "\n")
    
    print("MARC records support two character encodings:\n")
    
    print("MARC-8 (Legacy):")
    print("  - Indicator: Space character (' ') in Leader position 9")
    print("  - Uses ISO 2022 escape sequences")
    print("  - Supports: Latin, Greek, Cyrillic, Arabic, Hebrew, CJK")
    print("  - More compact but complex (escape sequences)")
    print("  - Example escape: ESC ) 2 = Switch to Hebrew")
    print("  - Common in: older catalog records, legacy systems")
    print()
    
    print("UTF-8 (Modern):")
    print("  - Indicator: 'a' in Leader position 9")
    print("  - Direct Unicode representation")
    print("  - Supports: All Unicode characters")
    print("  - Simpler, more flexible")
    print("  - Natively supported by modern systems")
    print("  - Common in: modern systems, web services")
    print()


def detect_encoding_from_leader():
    """
    Demonstrate how to detect character encoding from a MARC record.
    """
    print("\n" + "=" * 70)
    print("2. DETECTING ENCODING FROM LEADER")
    print("=" * 70 + "\n")
    
    # Example 1: MARC-8 encoded record
    print("Example 1: MARC-8 Encoded Record")
    leader_marc8 = Leader(
        record_type='a',
        bibliographic_level='m',
        character_coding=' ',  # Space = MARC-8
    )
    
    print(f"  Leader position 9: '{leader_marc8.character_coding}'")
    print(f"  Encoding: MARC-8 (escape sequences)")
    print(f"  How to handle: Use MARC-8 decoder to process content")
    print()
    
    # Example 2: UTF-8 encoded record
    print("Example 2: UTF-8 Encoded Record")
    leader_utf8 = Leader(
        record_type='a',
        bibliographic_level='m',
        character_coding='a',  # 'a' = UTF-8
    )
    
    print(f"  Leader position 9: '{leader_utf8.character_coding}'")
    print(f"  Encoding: UTF-8")
    print(f"  How to handle: Direct Unicode processing, no decoding needed")
    print()


def create_utf8_encoded_record():
    """
    Create a UTF-8 encoded record with multilingual content.
    """
    print("\n" + "=" * 70)
    print("3. UTF-8 ENCODED MULTILINGUAL RECORD")
    print("=" * 70 + "\n")
    
    # Create UTF-8 record
    leader = Leader(
        record_type='a',
        bibliographic_level='m',
        character_coding='a',  # UTF-8
    )
    
    record = Record(leader)
    
    record.add_control_field('001', 'ocm987654321')
    record.add_control_field('008', '210115s2021    xx||||||||||||||||eng||')
    
    # Title in English
    title = Field('245', '1', '0')
    title.add_subfield('a', 'Learning world languages /')
    title.add_subfield('c', 'Multiple authors.')
    record.add_field(title)
    
    # Author
    author = Field('100', '1', ' ')
    author.add_subfield('a', 'Smith, John,')
    author.add_subfield('e', 'author.')
    record.add_field(author)
    
    # Subjects with language info
    subject1 = Field('650', ' ', '0')
    subject1.add_subfield('a', 'Hebrew language')
    subject1.add_subfield('x', 'Study and teaching.')
    record.add_field(subject1)
    
    subject2 = Field('650', ' ', '0')
    subject2.add_subfield('a', 'Arabic language')
    subject2.add_subfield('x', 'Grammar.')
    record.add_field(subject2)
    
    # Language note field (546)
    language_note = Field('546', ' ', ' ')
    language_note.add_subfield('a', 'Text in English; includes Hebrew, Arabic, and Cyrillic examples.')
    record.add_field(language_note)
    
    # Variant title in Hebrew (880)
    # 880 fields are used to represent the same content in different scripts
    hebrew_title = Field('880', '1', '0')
    hebrew_title.add_subfield('a', 'למידת שפות עולם /')  # Hebrew text
    hebrew_title.add_subfield('6', '245-01')  # Links to 245 field
    record.add_field(hebrew_title)
    
    # Display the record
    print(f"Record Encoding: UTF-8")
    print(f"Title: {record.title()}")
    print(f"Author: {record.author()}")
    print()
    
    print("Language Information:")
    if '546' in record:
        note = record['546'].get_subfield('a')
        if note:
            print(f"  {note}")
    print()
    
    print("Variant Forms (880 fields):")
    for field in record.get_fields('880'):
        variant_a = field.get_subfield('a')
        link = field.get_subfield('6')
        if variant_a:
            print(f"  Script variant: {variant_a}")
            print(f"  Links to: field {link}")
    print()


def marc8_character_sets():
    """
    Explain MARC-8 character sets and escape sequences.
    """
    print("\n" + "=" * 70)
    print("4. MARC-8 CHARACTER SETS")
    print("=" * 70 + "\n")
    
    print("MARC-8 uses escape sequences to switch between character sets:\n")
    
    character_sets = [
        ("ASCII (G0)", "ESC ( B", "Basic Latin characters"),
        ("Extended Latin", "ESC ( S", "Accented Latin letters"),
        ("Greek", "ESC ( G", "Greek alphabet"),
        ("Arabic", "ESC ) 2", "Arabic script"),
        ("Hebrew", "ESC ) 4", "Hebrew script"),
        ("Cyrillic", "ESC ( N", "Cyrillic alphabet"),
        ("CJK", "ESC $ ) C", "Chinese, Japanese, Korean"),
    ]
    
    for name, escape, description in character_sets:
        print(f"  {name}")
        print(f"    Escape sequence: {escape}")
        print(f"    Content: {description}")
        print()
    
    print("How MARC-8 works:")
    print("  1. Default: ASCII characters (0x20-0x7E)")
    print("  2. Encounter non-ASCII: Check escape sequence prefix")
    print("  3. Switch: Change character interpretation based on set")
    print("  4. Decode: Apply character mapping for active set")
    print("  5. Switch back: ESC ( B returns to ASCII")
    print()


def practical_encoding_decisions():
    """
    Guide for deciding between MARC-8 and UTF-8.
    """
    print("\n" + "=" * 70)
    print("5. PRACTICAL ENCODING DECISIONS")
    print("=" * 70 + "\n")
    
    print("USE MARC-8 WHEN:")
    print("  ✓ Working with legacy library systems")
    print("  ✓ Preserving historical records")
    print("  ✓ Maintaining backward compatibility")
    print("  ✓ Working with Library of Congress records (historical)")
    print()
    
    print("USE UTF-8 WHEN:")
    print("  ✓ Building new systems")
    print("  ✓ Web-based MARC delivery")
    print("  ✓ Modern library platforms")
    print("  ✓ Multilingual collections (simplifies handling)")
    print("  ✓ Mobile or API-first applications")
    print()
    
    print("MIGRATION TIPS:")
    print("  1. Detect encoding from Leader position 9")
    print("  2. Use appropriate decoder (MARC-8 decoder or UTF-8)")
    print("  3. Validate round-trip conversion")
    print("  4. Test with multilingual content")
    print("  5. Preserve variant forms (880 fields) during conversion")
    print()
    
    print("HANDLING MULTILINGUAL RECORDS:")
    print("  1. Language in 546 field documents languages present")
    print("  2. 880 field variants show same content in different scripts")
    print("  3. Script code in $6 subfield links variant to main field")
    print("  4. Language code in 041 field documents languages")
    print()


def main():
    """Main example runner."""
    
    print("\n" + "=" * 70)
    print("MRRC: MARC-8 Encoding Support (Python/pymarc-compatible)")
    print("=" * 70)
    
    # Run demonstrations
    explain_character_encoding()
    detect_encoding_from_leader()
    create_utf8_encoded_record()
    marc8_character_sets()
    practical_encoding_decisions()
    
    print("=" * 70)
    print("SUMMARY")
    print("=" * 70)
    print("""
MRRC automatically detects and handles both MARC-8 and UTF-8 encoding:

1. MARC-8 records use escape sequences for non-ASCII characters
2. UTF-8 records use direct Unicode representation
3. Both are equally supported by mrrc
4. Leader position 9 indicates which encoding is used
5. Modern systems recommend UTF-8 for simplicity

When working with MARC records:
- Don't worry about encoding internally (mrrc handles it)
- Ensure your system supports the encoding before import
- For modern systems, prefer UTF-8
- Test multilingual content thoroughly
- Use 880 fields for script variants in MARC-8 records

REFERENCE:
- Library of Congress Character Encoding Guidelines:
  https://www.loc.gov/marc/specifications/spechome.html
- MARC-8 to UTF-8 Conversion:
  https://www.loc.gov/standards/marcxml/
    """)
    print()


if __name__ == '__main__':
    main()
