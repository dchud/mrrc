#!/usr/bin/env python3
"""
Multilingual MARC Records (Python/pymarc-compatible)

This example demonstrates how to work with MARC records containing
content in multiple languages:
- Hebrew titles and descriptions
- Arabic author names
- Cyrillic script content
- Mixed languages in a single record
- Proper diacritical marks

Multilingual records are common in academic and research libraries,
and require special handling for script variants and transliteration.
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


def hebrew_language_record():
    """
    Create a record for a Hebrew-language book with English title.
    
    This demonstrates:
    - Hebrew original title
    - English translation/transliteration
    - Proper character encoding
    - Script variants (880 fields)
    """
    print("\n" + "=" * 70)
    print("1. HEBREW-LANGUAGE BOOK RECORD")
    print("=" * 70 + "\n")
    
    # Use UTF-8 for modern systems
    leader = Leader(
        record_type='a',
        bibliographic_level='m',
        character_coding='a',  # UTF-8
    )
    
    record = Record(leader)
    
    record.add_control_field('001', 'ocm011223344')
    
    # Fixed-length data - Language code for Hebrew
    # Position 35-37: Language code
    record.add_control_field('008', '190101s2019    is ||||||||||||||||heb||')
    
    # Language field (041) - Documents languages present
    lang_field = Field('041', '1', ' ')
    lang_field.add_subfield('a', 'heb')  # Hebrew is original language
    lang_field.add_subfield('d', 'eng')  # English translation available
    record.add_field(lang_field)
    
    # Title in English (main)
    title = Field('245', '1', '4')
    title.add_subfield('a', 'The heritage of Israel /')
    title.add_subfield('c', 'Ari Shavit.')
    title.add_subfield('6', '880-01')  # Links to Hebrew variant
    record.add_field(title)
    
    # Title in Hebrew (variant - 880 field)
    hebrew_title = Field('880', '1', '4')
    hebrew_title.add_subfield('a', 'מורשת ישראל /')  # Hebrew text
    hebrew_title.add_subfield('c', 'אריאל שביט.')  # Hebrew author
    hebrew_title.add_subfield('6', '245-01')  # Links back to 245
    record.add_field(hebrew_title)
    
    # Author
    author = Field('100', '1', ' ')
    author.add_subfield('a', 'Shavit, Ari,')
    author.add_subfield('d', '1957-')
    author.add_subfield('e', 'author.')
    author.add_subfield('6', '880-02')
    record.add_field(author)
    
    # Author in Hebrew (variant)
    hebrew_author = Field('880', '1', ' ')
    hebrew_author.add_subfield('a', 'שביט, אריאל,')
    hebrew_author.add_subfield('d', '1957-')
    hebrew_author.add_subfield('e', 'author.')
    hebrew_author.add_subfield('6', '100-02')
    record.add_field(hebrew_author)
    
    # Publication info
    pub = Field('260', ' ', ' ')
    pub.add_subfield('a', 'Tel Aviv :')
    pub.add_subfield('b', 'Sifriyat Povlim,')
    pub.add_subfield('c', '2019.')
    record.add_field(pub)
    
    # Subject in Hebrew
    subject = Field('650', ' ', '0')
    subject.add_subfield('a', 'Israel')
    subject.add_subfield('x', 'History')
    subject.add_subfield('y', 'Modern period.')
    record.add_field(subject)
    
    # Script note
    script_note = Field('546', ' ', ' ')
    script_note.add_subfield('a', 'Text in Hebrew; English translation available.')
    record.add_field(script_note)
    
    # Display
    print(f"Title (English): {record.title()}")
    print(f"Author (English): {record.author()}")
    print(f"Language: Hebrew (heb)")
    print(f"Encoding: UTF-8")
    print()
    
    print("Original Hebrew Content (880 variants):")
    for field in record.get_fields('880'):
        a_subfield = field.get_subfield('a')
        tag_link = field.get_subfield('6')
        if a_subfield and field.tag == '880':
            print(f"  {a_subfield}")
    print()


def arabic_language_record():
    """
    Create a record for an Arabic-language resource.
    
    This demonstrates:
    - Arabic script handling
    - Right-to-left text
    - Transliteration for Latin script
    - Mixed script in single record
    """
    print("\n" + "=" * 70)
    print("2. ARABIC-LANGUAGE RESOURCE")
    print("=" * 70 + "\n")
    
    leader = Leader(
        record_type='a',
        bibliographic_level='m',
        character_coding='a',  # UTF-8
    )
    
    record = Record(leader)
    
    record.add_control_field('001', 'ocm987654321')
    record.add_control_field('008', '180515s2018    xx ||||||||||||||||ara||')
    
    # Language code for Arabic
    lang = Field('041', '0', ' ')
    lang.add_subfield('a', 'ara')  # Arabic
    record.add_field(lang)
    
    # Title - Transliterated
    title = Field('245', '1', '0')
    title.add_subfield('a', 'A thousand and one nights /')
    title.add_subfield('c', 'Various authors.')
    title.add_subfield('6', '880-01')
    record.add_field(title)
    
    # Title - Arabic script variant
    arabic_title = Field('880', '1', '0')
    arabic_title.add_subfield('a', 'ألف ليلة وليلة /')  # Arabic text
    arabic_title.add_subfield('6', '245-01')
    record.add_field(arabic_title)
    
    # Added entry for translator
    trans = Field('700', '1', ' ')
    trans.add_subfield('a', 'Burton, Richard Francis,')
    trans.add_subfield('d', '1821-1890,')
    trans.add_subfield('e', 'translator.')
    record.add_field(trans)
    
    # Subject
    subject = Field('650', ' ', '0')
    subject.add_subfield('a', 'Folklore')
    subject.add_subfield('z', 'Middle East.')
    record.add_field(subject)
    
    # Note on script
    note = Field('546', ' ', ' ')
    note.add_subfield('a', 'Title transliterated from Arabic.')
    record.add_field(note)
    
    # Display
    print(f"Title (Transliterated): {record.title()}")
    print(f"Language: Arabic (ara)")
    print(f"Encoding: UTF-8")
    print()
    
    print("Script Variants (Arabic original):")
    for field in record.get_fields('880'):
        if field.tag == '880':
            title_ar = field.get_subfield('a')
            if title_ar:
                print(f"  {title_ar}")
    print()


def cyrillic_language_record():
    """
    Create a record for a Cyrillic-language resource.
    
    This demonstrates:
    - Cyrillic script handling
    - Russian language encoding
    - Transliteration to Latin
    - Diacritical marks
    """
    print("\n" + "=" * 70)
    print("3. CYRILLIC-LANGUAGE RESOURCE (Russian)")
    print("=" * 70 + "\n")
    
    leader = Leader(
        record_type='a',
        bibliographic_level='m',
        character_coding='a',  # UTF-8
    )
    
    record = Record(leader)
    
    record.add_control_field('001', 'ocm555666777')
    record.add_control_field('008', '200115s2020    ru ||||||||||||||||rus||')
    
    # Language code for Russian
    lang = Field('041', '0', ' ')
    lang.add_subfield('a', 'rus')  # Russian
    record.add_field(lang)
    
    # Title - Transliterated
    title = Field('245', '1', '0')
    title.add_subfield('a', 'War and peace /')
    title.add_subfield('c', 'Leo Tolstoy.')
    title.add_subfield('6', '880-01')
    record.add_field(title)
    
    # Title - Cyrillic variant
    cyrillic_title = Field('880', '1', '0')
    cyrillic_title.add_subfield('a', 'Война и мир /')  # Cyrillic text
    cyrillic_title.add_subfield('c', 'Лев Толстой.')
    cyrillic_title.add_subfield('6', '245-01')
    record.add_field(cyrillic_title)
    
    # Author - Transliterated
    author = Field('100', '1', ' ')
    author.add_subfield('a', 'Tolstoy, Leo,')
    author.add_subfield('d', '1828-1910,')
    author.add_subfield('e', 'author.')
    author.add_subfield('6', '880-02')
    record.add_field(author)
    
    # Author - Cyrillic variant
    cyrillic_author = Field('880', '1', ' ')
    cyrillic_author.add_subfield('a', 'Толстой, Лев,')
    cyrillic_author.add_subfield('d', '1828-1910,')
    cyrillic_author.add_subfield('e', 'author.')
    cyrillic_author.add_subfield('6', '100-02')
    record.add_field(cyrillic_author)
    
    # Subject
    subject = Field('650', ' ', '0')
    subject.add_subfield('a', 'Russian fiction')
    subject.add_subfield('y', '19th century.')
    record.add_field(subject)
    
    # Display
    print(f"Title (Transliterated): {record.title()}")
    print(f"Author (Transliterated): {record.author()}")
    print(f"Language: Russian (rus)")
    print(f"Encoding: UTF-8")
    print()
    
    print("Cyrillic Variants:")
    for field in record.get_fields('880'):
        if field.tag == '880':
            text_cyrillic = field.get_subfield('a')
            if text_cyrillic:
                print(f"  {text_cyrillic}")
    print()


def mixed_language_record():
    """
    Create a record with mixed languages in one document.
    
    This demonstrates:
    - Multiple language codes (041 field)
    - Parallel text (Hebrew and Arabic in same work)
    - Complex transliteration
    """
    print("\n" + "=" * 70)
    print("4. MIXED-LANGUAGE RECORD")
    print("=" * 70 + "\n")
    
    leader = Leader(
        record_type='a',
        bibliographic_level='m',
        character_coding='a',  # UTF-8
    )
    
    record = Record(leader)
    
    record.add_control_field('001', 'ocm444333222')
    record.add_control_field('008', '210310s2021    is ||||||||||||||||heb||')
    
    # Multiple languages
    lang = Field('041', '1', ' ')
    lang.add_subfield('a', 'heb')  # Hebrew - original
    lang.add_subfield('a', 'ara')  # Arabic
    lang.add_subfield('d', 'eng')  # English translation
    record.add_field(lang)
    
    # Title in English (neutral language)
    title = Field('245', '1', '0')
    title.add_subfield('a', 'Middle Eastern dialogs /')
    title.add_subfield('c', 'Various authors.')
    record.add_field(title)
    
    # Subject covering multiple languages
    subject = Field('650', ' ', '0')
    subject.add_subfield('a', 'Hebrew literature')
    subject.add_subfield('x', 'Translations into English.')
    record.add_field(subject)
    
    # Subject 2
    subject2 = Field('650', ' ', '0')
    subject2.add_subfield('a', 'Arabic literature')
    subject2.add_subfield('x', 'Translations into English.')
    record.add_field(subject2)
    
    # Content note
    note = Field('520', ' ', ' ')
    note.add_subfield('a', 'Parallel text in Hebrew, Arabic, and English.')
    record.add_field(note)
    
    # Language note
    lang_note = Field('546', ' ', ' ')
    lang_note.add_subfield('a', 'Hebrew and Arabic text; English translations.')
    record.add_field(lang_note)
    
    # Display
    print(f"Title: {record.title()}")
    print(f"Content: Parallel text in Hebrew, Arabic, and English")
    print()
    
    print("Language Fields:")
    if '041' in record:
        lang_field = record['041']
        langs = lang_field.get_subfield_values('a')
        if langs:
            print(f"  Original languages: {', '.join(langs)}")
        trans_langs = lang_field.get_subfield_values('d')
        if trans_langs:
            print(f"  Translations: {', '.join(trans_langs)}")
    print()


def language_handling_guide():
    """
    Guide for handling languages in MARC records.
    """
    print("\n" + "=" * 70)
    print("LANGUAGE HANDLING GUIDE")
    print("=" * 70 + "\n")
    
    print("KEY FIELDS FOR LANGUAGE INFORMATION:")
    print()
    
    print("041 - Language Code:")
    print("  $a - Language of text")
    print("  $d - Language of original (for translations)")
    print("  $e - Language of original published form")
    print("  Example: $a heb $d ara (Hebrew, translated from Arabic)")
    print()
    
    print("546 - Language Note:")
    print("  Human-readable description of language content")
    print("  Example: 'Text in Hebrew and Arabic; English summary.'")
    print()
    
    print("880 - Variant Form of Field:")
    print("  Shows same content in different script/language")
    print("  $6 subfield links variant to main field")
    print("  Example: 245-01 links to 245 field")
    print()
    
    print("LANGUAGE CODE STANDARDS:")
    print("  3-letter codes (ISO 639-2):")
    print("    heb = Hebrew")
    print("    ara = Arabic")
    print("    rus = Russian")
    print("    chi = Chinese")
    print("    kor = Korean")
    print("    jpn = Japanese")
    print("    etc.")
    print()
    
    print("BEST PRACTICES:")
    print("  1. Always include 041 field for multilingual records")
    print("  2. Use 880 fields for script variants (Hebrew, Arabic, Cyrillic)")
    print("  3. Include 546 field for human-readable language description")
    print("  4. Use correct language codes (ISO 639-2)")
    print("  5. Link 880 variants with $6 subfield")
    print("  6. Maintain transliteration accuracy")
    print("  7. Test with multilingual search systems")
    print()


def main():
    """Main example runner."""
    
    print("\n" + "=" * 70)
    print("MRRC: Multilingual MARC Records (Python/pymarc-compatible)")
    print("=" * 70)
    
    # Run demonstrations
    hebrew_language_record()
    arabic_language_record()
    cyrillic_language_record()
    mixed_language_record()
    language_handling_guide()
    
    print("=" * 70)
    print("SUMMARY")
    print("=" * 70)
    print("""
Multilingual records in MARC:

1. USE 041 FIELD for language codes
   - Documents all languages present
   - Identifies original vs translation
   - Standard ISO 639-2 codes

2. USE 880 FIELD for script variants
   - Shows content in original script
   - Links to main field with $6
   - Essential for right-to-left scripts (Arabic, Hebrew)

3. USE 546 FIELD for language notes
   - Human-readable description
   - Helps users understand language mix
   - Documents transliteration/translation status

4. CHARACTER ENCODING:
   - UTF-8 recommended for modern systems
   - Supports all scripts natively
   - MARC-8 for legacy systems (more complex)

5. SCRIPT HANDLING:
   - Hebrew, Arabic: right-to-left
   - Cyrillic: transliterate for Latin display
   - CJK: complex, requires special handling

MRRC handles all of these automatically - just focus on
creating semantically correct records!
    """)
    print()


if __name__ == '__main__':
    main()
