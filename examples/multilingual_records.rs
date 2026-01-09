//! Multilingual MARC Records Example
//!
//! This example shows how to work with MARC records containing:
//! - Hebrew titles and descriptions
//! - Arabic author names
//! - Cyrillic script content
//! - Mixed languages in a single record
//! - Proper diacritical marks

use mrrc::{Field, Leader, Record};

/// Create a basic leader for examples
fn create_leader() -> Leader {
    Leader {
        record_length: 0,
        record_status: 'n',
        record_type: 'a',
        bibliographic_level: 'm',
        control_record_type: ' ',
        character_coding: ' ',
        indicator_count: 2,
        subfield_code_count: 2,
        data_base_address: 0,
        encoding_level: 'I',
        cataloging_form: 'a',
        multipart_level: ' ',
        reserved: "4500".to_string(),
    }
}

fn main() {
    println!("=== Multilingual MARC Records ===\n");

    // Example 1: Hebrew-language book
    hebrew_record_example();

    // Example 2: Arabic-language resource
    arabic_record_example();

    // Example 3: Cyrillic-language resource
    cyrillic_record_example();

    // Example 4: Mixed language record
    mixed_language_record_example();
}

/// Create a record for a Hebrew-language book.
fn hebrew_record_example() {
    println!("1. Hebrew-Language Book Record");
    println!("   ──────────────────────────\n");

    // Hebrew text: "Sefer HaAlef" (Book of Alef)
    // In MARC-8: ESC ) 2 switches to Hebrew character set for high bytes
    // Hebrew letters are encoded as 0xA1-0xBB in the Basic Hebrew set
    let title = "In MARC-8, Hebrew text would be encoded with escape sequences (ESC ) 2)";
    println!("   Title field (245):");
    println!("   Hebrew title example (stored in logical order):");
    println!("   Raw bytes: [ESC)2] [hebrew-chars] [ESC)E] ");
    println!("   Display: Hebrew text displays right-to-left\n");

    // Create the record structure
    let record = Record::builder(create_leader())
        .control_field_str("008", "200101s2020    il||||||||||||||||heb||")
        .field(
            Field::builder("245".to_string(), '1', '0')
                .subfield_str('a', title)
                .subfield_str('c', "Hebrew author.")
                .build(),
        )
        .field(
            Field::builder("100".to_string(), '1', ' ')
                .subfield_str('a', "Author, Hebrew,")
                .subfield_str('d', "1970-")
                .build(),
        )
        .field(
            Field::builder("650".to_string(), ' ', '0')
                .subfield_str('a', "Hebrew language")
                .subfield_str('x', "Textbooks.")
                .build(),
        )
        .build();

    println!(
        "   Record structure created: {:?}",
        record.leader.record_type
    );
    println!("   Language code (pos 35-37 in 008): heb\n");
}

/// Create a record for an Arabic-language resource.
fn arabic_record_example() {
    println!("2. Arabic-Language Resource");
    println!("   ────────────────────────\n");

    // Arabic: Uses Basic Arabic (ESC ) 3) or Extended Arabic (ESC ) 4)
    println!("   Arabic uses character sets 3 and 4:");
    println!("   • Basic Arabic: 0xA1-0xBA (escape: ESC ) 3)");
    println!("   • Extended Arabic: Additional characters (escape: ESC ) 4)");
    println!("   • Includes diacritical marks for vowels\n");

    let _ = Record::builder(create_leader())
        .control_field_str("008", "200101s2020    xx||||||||||||||||ara||")
        .field(
            Field::builder("245".to_string(), '1', '0')
                .subfield_str('a', "Arabic title placeholder")
                .subfield_str('c', "Arabic author.")
                .build(),
        )
        .field(
            Field::builder("650".to_string(), ' ', '0')
                .subfield_str('a', "Islamic civilization")
                .subfield_str('x', "Encyclopedias.")
                .build(),
        )
        .build();

    println!("   Record created: Language code = ara\n");
}

/// Create a record for a Cyrillic-language resource.
fn cyrillic_record_example() {
    println!("3. Cyrillic-Language (Russian) Resource");
    println!("   ──────────────────────────────────\n");

    // Cyrillic: Uses Basic Cyrillic (0x4E) or Extended Cyrillic (0x51)
    println!("   Cyrillic (Russian, Serbian, etc.) uses:");
    println!("   • Basic Cyrillic: ESC ( N (escape sequence for G0)");
    println!("   • Extended Cyrillic: ESC ( Q");
    println!("   • Covers Cyrillic alphabet (A-Я, а-я) and Cyrillic punctuation\n");

    let _ = Record::builder(create_leader())
        .control_field_str("008", "200101s2020    ru||||||||||||||||rus||")
        .field(
            Field::builder("245".to_string(), '1', '0')
                .subfield_str('a', "Russian title placeholder")
                .subfield_str('c', "Russian author.")
                .build(),
        )
        .field(
            Field::builder("650".to_string(), ' ', '0')
                .subfield_str('a', "Russian literature")
                .subfield_str('y', "20th century.")
                .build(),
        )
        .build();

    println!("   Record created: Language code = rus\n");
}

/// Create a record mixing multiple languages and scripts.
fn mixed_language_record_example() {
    println!("4. Mixed Language Record (Multilingual Work)");
    println!("   ──────────────────────────────────────\n");

    println!("   MARC-8 handles mixed scripts within a single field:");
    println!("   • Data stored in LOGICAL order (left-to-right)");
    println!("   • Escape sequences switch between character sets as needed");
    println!("   • Example: English text + Hebrew text + English again\n");

    println!("   Bytes: \"English\" [ESC)2] \"Hebrew\" [ESC)E] \"English\"\n");

    let _ = Record::builder(create_leader())
        .control_field_str("008", "200101s2020    xxu||||||||||||||||eng||")
        .field(
            Field::builder("245".to_string(), '1', '0')
                .subfield_str('a', "Bilingual: English and Hebrew")
                .subfield_str('c', "Multilingual author.")
                .build(),
        )
        .field(
            Field::builder("500".to_string(), ' ', ' ')
                .subfield_str('a', "Contains text in English, Hebrew, and Arabic.")
                .build(),
        )
        .field(
            Field::builder("546".to_string(), ' ', ' ')
                .subfield_str('a', "Text in English, Hebrew, and Arabic.")
                .build(),
        )
        .field(
            Field::builder("650".to_string(), ' ', '0')
                .subfield_str('a', "Multilingual materials.")
                .build(),
        )
        .build();

    println!("   Record created for multilingual work");
    println!("   Field 546 (Language note) documents all languages present\n");
}
