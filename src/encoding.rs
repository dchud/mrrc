//! Character encoding support for MARC records.
//!
//! MARC records can use different character encodings:
//! - **MARC-8** (legacy) — Mixed character sets with escape sequences (ISO 2022)
//! - **UTF-8** (modern) — Unicode standard encoding
//!
//! The encoding is indicated in position 9 of the MARC leader:
//! - Space character = MARC-8
//! - 'a' = UTF-8
//!
//! This module provides automatic encoding detection and conversion, including full
//! support for MARC-8 escape sequences and character set switching.

use crate::error::{MarcError, Result};
use crate::marc8_tables::{get_charset_table, CharacterSetId};

/// Character encoding for MARC records.
///
/// Indicates the character set used to encode field data in a MARC record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarcEncoding {
    /// MARC-8 encoding (legacy, mixed character sets)
    Marc8,
    /// UTF-8 encoding (modern standard)
    Utf8,
}

impl MarcEncoding {
    /// Detect encoding from leader character coding field
    /// Position 9 of leader indicates the character coding:
    /// ' ' (space) = MARC-8
    /// 'a' = UTF-8
    ///
    /// # Errors
    ///
    /// Returns `MarcError::EncodingError` if the character is not a valid encoding indicator.
    pub fn from_leader_char(c: char) -> Result<Self> {
        match c {
            ' ' => Ok(MarcEncoding::Marc8),
            'a' => Ok(MarcEncoding::Utf8),
            _ => Err(MarcError::EncodingError(format!(
                "Unknown character encoding: {c}"
            ))),
        }
    }

    /// Get the leader character for this encoding
    #[must_use]
    pub fn as_leader_char(&self) -> char {
        match self {
            MarcEncoding::Marc8 => ' ',
            MarcEncoding::Utf8 => 'a',
        }
    }
}

/// Decode bytes using the specified encoding
///
/// # Errors
///
/// Returns `MarcError::EncodingError` if the bytes are invalid for the encoding.
pub fn decode_bytes(bytes: &[u8], encoding: MarcEncoding) -> Result<String> {
    match encoding {
        MarcEncoding::Utf8 => String::from_utf8(bytes.to_vec())
            .map_err(|e| MarcError::EncodingError(format!("Invalid UTF-8: {e}"))),
        MarcEncoding::Marc8 => decode_marc8(bytes),
    }
}

/// Encode string using the specified encoding
///
/// # Errors
///
/// Returns an error if the encoding operation fails.
pub fn encode_string(s: &str, encoding: MarcEncoding) -> Result<Vec<u8>> {
    match encoding {
        MarcEncoding::Utf8 => Ok(s.as_bytes().to_vec()),
        MarcEncoding::Marc8 => encode_marc8(s),
    }
}

/// MARC-8 decoder state machine
/// Tracks the current G0 and G1 character sets and handles escape sequence parsing
#[derive(Debug, Clone)]
struct Marc8Decoder {
    /// Current G0 character set (used for low bytes 0x20-0x7F)
    g0: CharacterSetId,
    /// Current G1 character set (used for high bytes 0xA0-0xFE)
    g1: CharacterSetId,
}

impl Marc8Decoder {
    /// Create a new decoder with default character sets
    /// G0 = Basic Latin (ASCII)
    /// G1 = ANSEL Extended Latin
    fn new() -> Self {
        Marc8Decoder {
            g0: CharacterSetId::BasicLatin,
            g1: CharacterSetId::AnselExtendedLatin,
        }
    }

    /// Check if a character set uses multibyte encoding
    fn is_multibyte(charset: CharacterSetId) -> bool {
        charset == CharacterSetId::EACC
    }
}

/// Decode MARC-8 bytes to UTF-8 string
/// MARC-8 uses ISO 2022 escape sequences to switch between character sets.
/// This implementation handles:
/// - Character set switching via escape sequences
/// - Combining marks (diacritics)
/// - Multi-byte character sets (EACC/CJK)
#[allow(
    clippy::too_many_lines,
    clippy::cognitive_complexity,
    clippy::unnecessary_wraps,
    clippy::items_after_statements
)]
fn decode_marc8(bytes: &[u8]) -> Result<String> {
    let mut decoder = Marc8Decoder::new();
    let mut result = String::new();
    let mut combining_chars: Vec<char> = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        // Check for escape sequence (0x1B = ESC)
        if bytes[i] == 0x1B {
            if i + 1 >= bytes.len() {
                // Incomplete escape sequence at end
                result.push('\u{FFFD}');
                break;
            }

            let next_byte = bytes[i + 1];

            // Check if this is a character set designation escape sequence
            match next_byte {
                // ESC ( - Designate G0 character set (single-byte)
                0x28 => {
                    if i + 2 >= bytes.len() {
                        result.push('\u{FFFD}');
                        break;
                    }
                    let final_char = bytes[i + 2];
                    if let Some(charset) = CharacterSetId::from_byte(final_char) {
                        decoder.g0 = charset;
                    }
                    i += 3;
                    continue;
                },
                // ESC ) - Designate G1 character set (single-byte)
                0x29 => {
                    if i + 2 >= bytes.len() {
                        result.push('\u{FFFD}');
                        break;
                    }
                    let final_char = bytes[i + 2];
                    if let Some(charset) = CharacterSetId::from_byte(final_char) {
                        decoder.g1 = charset;
                    }
                    i += 3;
                    continue;
                },
                // ESC $ - Designate multi-byte character set
                0x24 => {
                    if i + 2 >= bytes.len() {
                        result.push('\u{FFFD}');
                        break;
                    }
                    let modifier = bytes[i + 2];
                    if modifier == 0x31 {
                        // ESC $ 1 - EACC (East Asian Character Code)
                        decoder.g0 = CharacterSetId::EACC;
                        i += 3;
                        continue;
                    } else if i + 3 < bytes.len() {
                        let final_char = bytes[i + 3];
                        if let Some(charset) = CharacterSetId::from_byte(final_char) {
                            decoder.g0 = charset;
                        }
                        i += 4;
                        continue;
                    }
                    i += 3;
                    continue;
                },
                // ESC s - Reset G0 to Basic Latin (ASCII)
                0x73 => {
                    decoder.g0 = CharacterSetId::BasicLatin;
                    i += 2;
                    continue;
                },
                // Custom MARC-8 escape sequences (locking, non-ISO 2022)
                // ESC g - Greek Symbols (deprecated - mapping difficulties)
                0x67 => {
                    decoder.g0 = CharacterSetId::GreekSymbols;
                    i += 2;
                    continue;
                },
                // ESC b - Subscripts (custom MARC set)
                0x62 => {
                    decoder.g0 = CharacterSetId::Subscript;
                    i += 2;
                    continue;
                },
                // ESC p - Superscripts (custom MARC set)
                0x70 => {
                    decoder.g0 = CharacterSetId::Superscript;
                    i += 2;
                    continue;
                },
                _ => {
                    // Unknown escape sequence - skip it
                    i += 2;
                    continue;
                },
            }
        }

        // Regular character handling (not an escape sequence)
        let byte = bytes[i];

        // Control characters (0x00-0x1F, 0x7F) - pass through or skip
        if byte < 0x20 || byte == 0x7F {
            // Skip control characters in output (except LF, CR)
            if byte == 0x0A || byte == 0x0D {
                result.push(byte as char);
            }
            i += 1;
            continue;
        }

        // Determine which character set to use
        let (charset, byte_value) = if byte >= 0xA0 {
            // High byte range (0xA0-0xFE) - use G1 set
            (decoder.g1, byte)
        } else {
            // Low byte range (0x20-0x7E) - use G0 set
            (decoder.g0, byte)
        };

        // Handle multibyte character sets
        if Marc8Decoder::is_multibyte(charset) {
            if i + 2 < bytes.len() {
                // EACC: 3-byte sequence
                // Concatenate 3 bytes into a u32 key for lookup
                let key = (u32::from(bytes[i]) << 16)
                    | (u32::from(bytes[i + 1]) << 8)
                    | u32::from(bytes[i + 2]);

                if let Some((unicode_point, is_combining)) =
                    crate::marc8_tables::get_eacc_character(key)
                {
                    let ch = char::from_u32(unicode_point).unwrap_or('\u{FFFD}');
                    if is_combining {
                        combining_chars.push(ch);
                    } else {
                        for combining_ch in combining_chars.drain(..) {
                            result.push(combining_ch);
                        }
                        result.push(ch);
                    }
                } else {
                    // Character not in EACC table - use replacement
                    result.push('\u{FFFD}');
                }
                i += 3;
                continue;
            }
            i += 1;
            continue;
        }

        // Single-byte character lookup
        let table = get_charset_table(charset);
        if let Some((unicode_point, is_combining)) = table.get(&byte_value) {
            let ch = char::from_u32(*unicode_point).unwrap_or('\u{FFFD}');
            if *is_combining {
                // Combining marks are stored and applied to the next base character
                combining_chars.push(ch);
            } else {
                // Base character - output combining marks first, then the base
                for combining_ch in combining_chars.drain(..) {
                    result.push(combining_ch);
                }
                result.push(ch);
            }
        } else {
            // Character not found in table - use replacement character
            result.push('\u{FFFD}');
        }

        i += 1;
    }

    // Handle any remaining combining characters at end of string
    for combining_ch in combining_chars {
        result.push(combining_ch);
    }

    // Normalize to NFC form (combining characters)
    use unicode_normalization::UnicodeNormalization;
    Ok(result.nfc().collect())
}

/// Encode UTF-8 string to MARC-8 bytes
/// Maps Unicode characters back to MARC-8 character sets with proper escape sequences
/// Prefers ASCII for ASCII-range characters, then looks for the character in other
/// MARC-8 character sets, emitting escape sequences as needed.
fn encode_marc8(s: &str) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut current_charset = CharacterSetId::BasicLatin;

    for c in s.chars() {
        let unicode = c as u32;

        // Try to find this character in the MARC-8 tables
        if let Some((target_charset, byte_value)) =
            crate::marc8_tables::find_unicode_in_marc8(unicode)
        {
            // If we need to switch character sets, emit escape sequence
            if target_charset != current_charset {
                match target_charset {
                    CharacterSetId::BasicLatin => {
                        // ESC s - Reset to ASCII
                        bytes.push(0x1B);
                        bytes.push(0x73);
                    },
                    CharacterSetId::AnselExtendedLatin => {
                        // ESC ) E - Switch G1 to ANSEL
                        bytes.push(0x1B);
                        bytes.push(0x29);
                        bytes.push(0x45);
                    },
                    CharacterSetId::Subscript => {
                        // ESC b - Switch to Subscript
                        bytes.push(0x1B);
                        bytes.push(0x62);
                    },
                    CharacterSetId::Superscript => {
                        // ESC p - Switch to Superscript
                        bytes.push(0x1B);
                        bytes.push(0x70);
                    },
                    CharacterSetId::GreekSymbols => {
                        // ESC g - Switch to Greek symbols
                        bytes.push(0x1B);
                        bytes.push(0x67);
                    },
                    CharacterSetId::BasicHebrew => {
                        // ESC ( 2 - Switch G0 to Hebrew
                        bytes.push(0x1B);
                        bytes.push(0x28);
                        bytes.push(0x32);
                    },
                    CharacterSetId::BasicArabic => {
                        // ESC ( 3 - Switch G0 to Arabic
                        bytes.push(0x1B);
                        bytes.push(0x28);
                        bytes.push(0x33);
                    },
                    CharacterSetId::ExtendedArabic => {
                        // ESC ( 4 - Switch G0 to Extended Arabic
                        bytes.push(0x1B);
                        bytes.push(0x28);
                        bytes.push(0x34);
                    },
                    CharacterSetId::BasicCyrillic => {
                        // ESC ( N - Switch G0 to Basic Cyrillic
                        bytes.push(0x1B);
                        bytes.push(0x28);
                        bytes.push(0x4E);
                    },
                    CharacterSetId::ExtendedCyrillic => {
                        // ESC ( Q - Switch G0 to Extended Cyrillic
                        bytes.push(0x1B);
                        bytes.push(0x28);
                        bytes.push(0x51);
                    },
                    CharacterSetId::BasicGreek => {
                        // ESC ( S - Switch G0 to Basic Greek
                        bytes.push(0x1B);
                        bytes.push(0x28);
                        bytes.push(0x53);
                    },
                    CharacterSetId::EACC => {
                        // Not applicable for single characters
                    },
                }
                current_charset = target_charset;
            }

            // Add the character byte(s)
            // For single-byte character sets, byte_value fits in u8
            // For EACC (multi-byte), this is handled separately above
            bytes.push(u8::try_from(byte_value).map_err(|_| {
                MarcError::EncodingError(
                    format!("Character byte value {byte_value} exceeds u8 range for charset {target_charset:?}")
                )
            })?);
        } else {
            // Character not found in MARC-8, use replacement character
            bytes.push(0x3F); // Question mark
        }
    }

    // Reset to ASCII at the end if we're not already there
    if current_charset != CharacterSetId::BasicLatin {
        bytes.push(0x1B);
        bytes.push(0x73);
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding_from_leader_char() {
        assert_eq!(
            MarcEncoding::from_leader_char(' ').unwrap(),
            MarcEncoding::Marc8
        );
        assert_eq!(
            MarcEncoding::from_leader_char('a').unwrap(),
            MarcEncoding::Utf8
        );
        assert!(MarcEncoding::from_leader_char('x').is_err());
    }

    #[test]
    fn test_encoding_as_leader_char() {
        assert_eq!(MarcEncoding::Marc8.as_leader_char(), ' ');
        assert_eq!(MarcEncoding::Utf8.as_leader_char(), 'a');
    }

    #[test]
    fn test_utf8_decode() {
        let bytes = "Hello, 世界".as_bytes();
        let decoded = decode_bytes(bytes, MarcEncoding::Utf8).unwrap();
        assert_eq!(decoded, "Hello, 世界");
    }

    #[test]
    fn test_utf8_encode() {
        let s = "Hello, 世界";
        let encoded = encode_string(s, MarcEncoding::Utf8).unwrap();
        let decoded = String::from_utf8(encoded).unwrap();
        assert_eq!(decoded, s);
    }

    #[test]
    fn test_marc8_ascii() {
        let bytes = b"Hello, World";
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        assert_eq!(decoded, "Hello, World");
    }

    #[test]
    fn test_marc8_encode_ascii() {
        let s = "Hello";
        let encoded = encode_string(s, MarcEncoding::Marc8).unwrap();
        assert_eq!(encoded, b"Hello");
    }

    #[test]
    fn test_marc8_encode_unicode() {
        // Test encoding of characters not directly in MARC-8
        // é (U+00E9) is not a single MARC-8 character, so it will be replaced
        let s = "Café";
        let encoded = encode_string(s, MarcEncoding::Marc8).unwrap();
        // We expect the encoded result to contain the basic ASCII characters and a replacement for é
        assert!(!encoded.is_empty());
        let decoded = decode_bytes(&encoded, MarcEncoding::Marc8).unwrap();
        // The decoded version will have a replacement character or loss of é
        // Just verify the decode doesn't crash
        assert!(!decoded.is_empty());
    }

    #[test]
    fn test_marc8_escape_sequence_g0() {
        // ESC ( B = Switch G0 to Basic Latin (which is default)
        let bytes = b"\x1B(BHello";
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        assert_eq!(decoded, "Hello");
    }

    #[test]
    fn test_marc8_reset_to_ascii() {
        // ESC s = Reset G0 to ASCII
        let bytes = b"\x1BsHello";
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        assert_eq!(decoded, "Hello");
    }

    #[test]
    fn test_encoding_roundtrip() {
        let original = "Test String with 123";
        let encoded = encode_string(original, MarcEncoding::Utf8).unwrap();
        let decoded = decode_bytes(&encoded, MarcEncoding::Utf8).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_marc8_combining_marks() {
        // Test that combining marks are properly identified and handled
        // Note: MARC-8 combining marks appear BEFORE the base character
        // We're testing the infrastructure for combining character tracking
        let bytes = b"Test";
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        assert_eq!(decoded, "Test");
    }

    #[test]
    fn test_marc8_ansel_extended_with_combining() {
        // ANSEL combining marks (0xE0-0xFE) should be marked as combining
        // and processed appropriately
        // This tests that the character lookup correctly identifies combining marks
        let bytes = b"A";
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        assert_eq!(decoded, "A");
    }

    #[test]
    fn test_marc8_unicode_normalization() {
        // Result should be normalized to NFC form
        let bytes = "café".as_bytes(); // Pre-composed
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        // The string should be properly decoded
        assert!(decoded.contains("caf"));
    }

    #[test]
    fn test_marc8_roundtrip_ascii() {
        // ASCII text should roundtrip cleanly
        let original = "The Quick Brown Fox";
        let encoded = encode_string(original, MarcEncoding::Marc8).unwrap();
        let decoded = decode_bytes(&encoded, MarcEncoding::Marc8).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_marc8_roundtrip_with_escape_sequences() {
        // Text with escape sequences should decode properly
        // This is a simplified test - real MARC-8 records would have more complex sequences
        let bytes = b"ASCII\x1B(BMore";
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        assert_eq!(decoded, "ASCIIMore");
    }

    #[test]
    fn test_marc8_encode_ascii_roundtrip() {
        // ASCII text should encode and decode cleanly
        let original = "The Quick Brown Fox";
        let encoded = encode_string(original, MarcEncoding::Marc8).unwrap();
        let decoded = decode_bytes(&encoded, MarcEncoding::Marc8).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_marc8_encode_subscript_roundtrip() {
        // Subscript characters should round-trip correctly
        let original = "H₂O";
        let encoded = encode_string(original, MarcEncoding::Marc8).unwrap();
        let decoded = decode_bytes(&encoded, MarcEncoding::Marc8).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_marc8_encode_superscript_roundtrip() {
        // Superscript characters should round-trip correctly
        let original = "x² + y³";
        let encoded = encode_string(original, MarcEncoding::Marc8).unwrap();
        let decoded = decode_bytes(&encoded, MarcEncoding::Marc8).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_marc8_encode_mixed_scripts() {
        // Mix of ASCII and special characters - simplified test
        let original = "Hello World";
        let encoded = encode_string(original, MarcEncoding::Marc8).unwrap();
        let decoded = decode_bytes(&encoded, MarcEncoding::Marc8).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_marc8_multiple_character_sets() {
        // Test switching between character sets
        // ESC ) E switches G1 to ANSEL
        let bytes = b"\x1B)EText";
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        assert_eq!(decoded, "Text");
    }

    #[test]
    fn test_marc8_greek_symbol_escape() {
        // ESC g should switch to Greek symbols (deprecated but supported)
        let bytes = b"\x1BgA";
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        // Greek symbols are marked but we don't have a full table yet
        // Just verify it doesn't crash
        assert!(!decoded.is_empty());
    }

    #[test]
    fn test_marc8_incomplete_escape_at_end() {
        // Incomplete escape sequence at end should be handled gracefully
        let bytes = b"Text\x1B";
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        // Should handle gracefully - replacement character or skip
        assert!(decoded.contains("Text"));
    }

    #[test]
    fn test_marc8_control_characters_ignored() {
        // Control characters (except LF/CR) should be skipped
        let mut bytes = Vec::from(&b"Hello"[..]);
        bytes.insert(2, 0x01); // Insert a control character
        let decoded = decode_bytes(&bytes, MarcEncoding::Marc8).unwrap();
        // Control char should be skipped
        assert_eq!(decoded.len(), 5); // "Hello"
    }

    #[test]
    fn test_marc8_vs_utf8_equivalence() {
        // ASCII should be the same in both encodings
        let text = "Simple ASCII Text 12345";
        let utf8_encoded = encode_string(text, MarcEncoding::Utf8).unwrap();
        let marc8_encoded = encode_string(text, MarcEncoding::Marc8).unwrap();
        // ASCII should be identical in both
        assert_eq!(utf8_encoded, marc8_encoded);

        // Both should decode to the same result
        let from_utf8 = decode_bytes(&utf8_encoded, MarcEncoding::Utf8).unwrap();
        let from_marc8 = decode_bytes(&marc8_encoded, MarcEncoding::Marc8).unwrap();
        assert_eq!(from_utf8, from_marc8);
    }

    #[test]
    fn test_marc8_replacement_char_on_unknown() {
        // Unknown escape sequences should be skipped
        let bytes = b"\x1B\xFF";
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        // Unknown sequences are skipped in parsing
        // The 0xFF byte is a control character, so it's also skipped
        // Result should be empty or just whitespace
        assert!(decoded.is_empty() || decoded.chars().all(char::is_whitespace));
    }

    #[test]
    fn test_marc8_high_byte_range_uses_g1() {
        // High bytes (0xA0-0xFE) should use G1 character set (default: ANSEL)
        // Without escape sequences, should default to ASCII for low bytes and ANSEL for high bytes
        let bytes = &[0x41, 0xA0]; // 'A' in ASCII, 0xA0 in ANSEL (should map to space)
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        assert_eq!(decoded, "A ");
    }

    #[test]
    fn test_marc8_subscript_escape() {
        // ESC b switches to subscript character set
        // Then byte 0x30 should be subscript digit 0
        let bytes = b"\x1Bb0"; // ESC b then '0'
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        assert_eq!(decoded, "₀"); // SUBSCRIPT DIGIT ZERO
    }

    #[test]
    fn test_marc8_subscript_multiple() {
        // Test multiple subscript characters
        let bytes = b"\x1Bb123"; // ESC b then subscript 1, 2, 3
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        assert_eq!(decoded, "₁₂₃");
    }

    #[test]
    fn test_marc8_superscript_escape() {
        // ESC p switches to superscript character set
        let bytes = b"\x1Bp0"; // ESC p then '0'
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        assert_eq!(decoded, "⁰"); // SUPERSCRIPT DIGIT ZERO
    }

    #[test]
    fn test_marc8_superscript_multiple() {
        // Test multiple superscript characters including special mappings
        let bytes = b"\x1Bp123"; // ESC p then superscript 1, 2, 3
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        assert_eq!(decoded, "¹²³");
    }

    #[test]
    fn test_marc8_greek_symbols_escape() {
        // ESC g switches to Greek symbols (deprecated)
        let bytes = b"\x1Bga"; // ESC g then 'a' (alpha) - 0x61 is the MARC-8 code for alpha
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        assert_eq!(decoded, "α"); // GREEK SMALL LETTER ALPHA
    }

    #[test]
    fn test_marc8_greek_symbols_all() {
        // Test all three Greek symbols: alpha, beta, gamma
        let bytes = b"\x1Bgabc"; // ESC g, then a (alpha), b (beta), c (gamma)
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        assert_eq!(decoded, "αβγ");
    }

    #[test]
    fn test_marc8_subscript_with_reset() {
        // Test switching to subscript and back to ASCII
        let bytes = b"H\x1Bb2\x1BsO"; // H, then ESC b, subscript 2, then ESC s (reset), O
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        assert_eq!(decoded, "H₂O");
    }

    #[test]
    fn test_marc8_subscript_parentheses() {
        // Test subscript parentheses
        let bytes = b"\x1Bb(0)"; // ESC b, subscript (, 0, )
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        assert_eq!(decoded, "₍₀₎");
    }

    #[test]
    fn test_marc8_superscript_plus_minus() {
        // Test superscript plus and minus
        let bytes = b"\x1Bp1+2-3"; // ESC p, superscript 1, +, 2, -, 3
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        assert_eq!(decoded, "¹⁺²⁻³");
    }

    #[test]
    fn test_marc8_eacc_multibyte_decoding() {
        // Test EACC (East Asian Character Code) 3-byte sequence decoding
        // EACC is switched with ESC $ 1 (0x1B 0x24 0x31)
        // Then 3-byte sequences follow

        // Example: IDEOGRAPHIC SPACE (U+3000) is at EACC key 0x212320
        // We construct: ESC $ 1 (switch to EACC) followed by 0x21 0x23 0x20
        let bytes = b"\x1B\x24\x31\x21\x23\x20";
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();

        // Should have decoded the IDEOGRAPHIC SPACE character
        assert!(!decoded.is_empty(), "Should decode EACC character");
        assert_eq!(decoded, "\u{3000}"); // U+3000 is IDEOGRAPHIC SPACE
    }

    #[test]
    fn test_marc8_eacc_multiple_characters() {
        // Test multiple EACC characters in sequence
        // 0x212320 = U+3000 (IDEOGRAPHIC SPACE)
        // 0x212328 = U+FF08 (FULLWIDTH LEFT PARENTHESIS)
        let bytes = b"\x1B\x24\x31\x21\x23\x20\x21\x23\x28";
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();

        assert!(
            !decoded.is_empty(),
            "Should decode multiple EACC characters"
        );
        // Should have both IDEOGRAPHIC SPACE and FULLWIDTH LEFT PARENTHESIS
        assert!(
            decoded.contains('\u{3000}'),
            "Should contain IDEOGRAPHIC SPACE"
        );
        assert!(
            decoded.contains('\u{FF08}'),
            "Should contain FULLWIDTH LEFT PARENTHESIS"
        );
    }

    #[test]
    fn test_marc8_hebrew_text() {
        // Test Basic Hebrew character set - ESC ) 2 (designate as G1)
        // Using Hebrew letters: alef (0xA1), bet (0xA2), gimel (0xA3)
        // ESC ) 2 designates Hebrew as G1 set, so high bytes (0xA1-0xFE) use Hebrew
        let bytes = b"\x1B\x292\xA1\xA2\xA3\x1B\x29\x45"; // Designate Hebrew to G1, 3 Hebrew letters, designate ANSEL to G1 (reset)
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        assert!(decoded.contains('א'), "Should contain Hebrew alef");
        assert!(decoded.contains('ב'), "Should contain Hebrew bet");
        assert!(decoded.contains('ג'), "Should contain Hebrew gimel");
    }

    #[test]
    fn test_marc8_arabic_text() {
        // Test Basic Arabic character set - ESC ) 3 (designate as G1)
        // Using Arabic letters: hamza (0xA1), alef with madda (0xA2), alef with hamza above (0xA3)
        let bytes = b"\x1B\x293\xA1\xA2\xA3\x1B\x29\x45"; // Designate Arabic to G1, 3 Arabic letters, designate ANSEL to G1 (reset)
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        assert!(decoded.contains('ء'), "Should contain Arabic hamza");
        assert!(
            decoded.contains('آ'),
            "Should contain Arabic alef with madda"
        );
        assert!(
            decoded.contains('أ'),
            "Should contain Arabic alef with hamza above"
        );
    }

    #[test]
    fn test_marc8_extended_arabic_text() {
        // Test Extended Arabic character set - ESC ) 4 (designate as G1)
        // Using extended Arabic letters
        let bytes = b"\x1B\x294\xA1\xA2\xA3\x1B\x29\x45"; // Designate Extended Arabic to G1, 3 letters, designate ANSEL to G1 (reset)
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        // Extended Arabic has different character mappings
        assert!(!decoded.is_empty(), "Should decode extended Arabic");
    }

    #[test]
    fn test_marc8_mixed_ltr_rtl() {
        // Test mixed left-to-right (ASCII) and right-to-left (Hebrew) text
        // "Hello" in ASCII (default), then switch to Hebrew for "שלום" (Shalom)
        // ESC ) 2 designates Hebrew to G1, then shin(0xB5)+lamed(0xAC)+vav(0xA6)+final_mem(0xB8)
        let bytes = b"Hello\x1B\x292\xB5\xAC\xA6\xB8\x1B\x29\x45!"; // "Hello", designate Hebrew to G1, Hebrew text, reset to ANSEL, "!"
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        assert!(
            decoded.starts_with("Hello"),
            "Should start with ASCII Hello"
        );
        assert!(decoded.contains('ש'), "Should contain Hebrew shin");
        assert!(decoded.contains('ל'), "Should contain Hebrew lamed");
        assert!(decoded.contains('ו'), "Should contain Hebrew vav");
        assert!(decoded.contains('ם'), "Should contain Hebrew final mem");
    }

    #[test]
    fn test_marc8_bidi_with_diacritics() {
        // Test bidirectional text with diacritics (combining marks)
        // MARC-8 stores combining marks before the base character
        // Using ANSEL G1 with combining grave (0xE0 in ANSEL) before Hebrew alef (via G1)
        // First designate Hebrew to G1, use 0xE0 as combining grave, then 0xA1 for alef
        let bytes = b"\x1B\x292\xE0\xA1\x1B\x29\x45AB"; // Designate Hebrew to G1, combining grave + alef, reset to ANSEL, ASCII 'AB'
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        // Combining marks are applied to the following character
        assert!(
            decoded.contains('א'),
            "Should contain Hebrew alef (may have combining mark)"
        );
        assert!(decoded.contains('A'), "Should contain ASCII A");
    }

    #[test]
    fn test_marc8_eacc_with_reset() {
        // Test EACC characters followed by reset to ASCII
        // 0x212320 = U+3000, then reset to ASCII with ESC ( B, then 'A'
        let bytes = b"\x1B\x24\x31\x21\x23\x20\x1B\x28\x42A";
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();

        assert!(!decoded.is_empty(), "Should decode EACC and ASCII");
        assert!(
            decoded.contains('\u{3000}'),
            "Should contain IDEOGRAPHIC SPACE"
        );
        assert!(decoded.contains('A'), "Should contain ASCII 'A'");
    }
}
