//! MARC-8 Encoding Example
//!
//! This example demonstrates:
//! - Detecting MARC-8 vs UTF-8 encoding in MARC records
//! - Understanding MARC-8 character set switching with escape sequences
//! - Working with multilingual text (Hebrew, Arabic, Greek, Cyrillic)
//! - Proper handling of combining characters and diacritics

use mrrc::encoding::MarcEncoding;

fn main() {
    println!("=== MARC-8 Encoding Support ===\n");

    // Example 1: Detecting encoding from MARC leader
    detect_encoding_example();

    // Example 2: Understanding character sets in MARC-8
    character_sets_example();

    // Example 3: Handling multilingual records
    multilingual_example();
}

/// Demonstrates detecting the character encoding from a MARC record's leader.
fn detect_encoding_example() {
    println!("1. Detecting Character Encoding from Leader");
    println!("   A MARC leader's character position 9 indicates the encoding:\n");

    // Space character = MARC-8 (legacy encoding)
    let leader_char_marc8 = ' ';
    match MarcEncoding::from_leader_char(leader_char_marc8) {
        Ok(encoding) => {
            println!("   Leader position 9 = '{leader_char_marc8}'");
            println!("   Encoding: {encoding:?}\n");
        },
        Err(e) => eprintln!("   Error: {e}\n"),
    }

    // 'a' = UTF-8 (modern encoding)
    let leader_char_utf8 = 'a';
    match MarcEncoding::from_leader_char(leader_char_utf8) {
        Ok(encoding) => {
            println!("   Leader position 9 = '{leader_char_utf8}'");
            println!("   Encoding: {encoding:?}\n");
        },
        Err(e) => eprintln!("   Error: {e}\n"),
    }
}

/// Explains the character sets available in MARC-8 and their escape sequences.
fn character_sets_example() {
    println!("2. MARC-8 Character Set Overview");
    println!("   MARC-8 uses ISO 2022 escape sequences to switch between character sets.\n");

    let character_sets = vec![
        (
            "Basic Latin (ASCII)",
            "Default (G0)",
            "ESC ( B",
            "0x20-0x7E",
        ),
        (
            "ANSEL Extended Latin",
            "Default (G1)",
            "ESC ) E",
            "0xA0-0xFE (diacritics)",
        ),
        ("Basic Hebrew", "0x32", "ESC ) 2", "0xA1-0xBB (22 letters)"),
        ("Basic Arabic", "0x33", "ESC ) 3", "0xA1-0xBA (20+ letters)"),
        (
            "Extended Arabic",
            "0x34",
            "ESC ) 4",
            "Extended Arabic characters",
        ),
        (
            "Basic Cyrillic",
            "0x4E",
            "ESC ( N",
            "Russian/Slavic characters",
        ),
        ("Basic Greek", "0x53", "ESC ( S", "Greek alphabet"),
        ("Greek Symbols", "0x67", "ESC g", "α, β, γ (deprecated)"),
        ("Subscripts", "0x62", "ESC b", "₀-₉, ₊, ₋, ₍, ₎ (14 chars)"),
        (
            "Superscripts",
            "0x70",
            "ESC p",
            "⁰-⁹, ⁺, ⁻, ⁽, ⁾ (14 chars)",
        ),
        (
            "East Asian (EACC)",
            "0x31",
            "ESC $ 1",
            "Chinese, Japanese, Korean",
        ),
    ];

    println!("   Character Set          | Final Char | Escape Sequence | Examples");
    println!("   {}", "-".repeat(75));

    for (name, final_char, escape, examples) in character_sets {
        println!("   {name:<22} | {final_char:<10} | {escape:<15} | {examples}");
    }
    println!();
}

/// Demonstrates handling multilingual MARC records with different scripts.
fn multilingual_example() {
    println!("3. Working with Multilingual Records");
    println!("   MARC-8 stores data in LOGICAL order (not visual), so:");
    println!("   - RTL scripts (Arabic, Hebrew) are stored left-to-right logically");
    println!("   - Mixed LTR/RTL requires proper escape sequence handling");
    println!("   - Combining marks (diacritics) precede their base character\n");

    println!("   Example: Mixed English + Hebrew record");
    println!("   ────────────────────────────────────");
    println!("   Display:  Hello שלום !");
    println!("   Logical:  H e l l o [ESC)2] ש ל ו ם [ESC)E] !");
    println!("            ↑ ASCII (default) ↑ Hebrew set ↑ Back to ANSEL\n");

    println!("   Key points for MARC-8:");
    println!("   • Escape sequences can occur anywhere (within words, fields, subfields)");
    println!("   • Combining characters appear BEFORE the base character");
    println!("   • Must reset to ASCII (ESC s) or ANSEL (ESC ) E) before field terminator");
    println!("   • Unicode normalization (NFC) is applied to normalize combining chars\n");

    println!("   Example: Hebrew with diacritics");
    println!("   ─────────────────────────────");
    println!("   Data: [combining-mark] [hebrew-letter]");
    println!("   Display: hebrew-letter with diacritic applied\n");
}
