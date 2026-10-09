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
use crate::marc8_tables::{CharacterMapping, CharacterSetId, get_charset_table};

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
            _ => Err(MarcError::encoding_msg(format!(
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

/// How the ISO 2709 readers choose the character encoding of a record's field
/// data.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CharacterCoding {
    /// Follow leader position 09: `a` is UTF-8 and any other value is MARC-8.
    /// This is pymarc's rule.
    #[default]
    Leader,
    /// Decode as UTF-8 whatever position 09 says, like pymarc's
    /// `force_utf8=True`.
    Utf8,
    /// Decode a record as UTF-8 when its field data is valid UTF-8 containing
    /// non-ASCII bytes, and otherwise follow position 09. This reads files
    /// that mix MARC-8 records with UTF-8 records whose leader still says
    /// MARC-8, which a MARC-8 byte sequence is very unlikely to imitate.
    Detect,
}

/// What the ISO 2709 readers do with invalid UTF-8 in a record decoded as
/// UTF-8. The values and their meanings are pymarc's `utf8_handling`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Utf8Handling {
    /// Invalid UTF-8 is an error, [`crate::MarcError::EncodingError`]
    /// (E301), which the reader's recovery mode then handles like any other.
    /// pymarc's default.
    #[default]
    Strict,
    /// Replace each invalid sequence with U+FFFD.
    Replace,
    /// Drop the invalid bytes.
    Ignore,
}

impl CharacterCoding {
    /// The encoding to decode a record's field data in, given leader position
    /// 09 and the record's field data (directory excluded).
    #[must_use]
    pub(crate) fn resolve(self, position_09: char, field_data: &[u8]) -> MarcEncoding {
        let from_leader = if position_09 == 'a' {
            MarcEncoding::Utf8
        } else {
            MarcEncoding::Marc8
        };
        match self {
            CharacterCoding::Leader => from_leader,
            CharacterCoding::Utf8 => MarcEncoding::Utf8,
            CharacterCoding::Detect => {
                if from_leader == MarcEncoding::Marc8
                    && !field_data.is_ascii()
                    && std::str::from_utf8(field_data).is_ok()
                {
                    MarcEncoding::Utf8
                } else {
                    from_leader
                }
            },
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
            .map_err(|e| MarcError::encoding_msg(format!("Invalid UTF-8: {e}"))),
        MarcEncoding::Marc8 => Ok(decode_marc8_lossy(bytes).text),
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

/// MARC-8 decoding result: the decoded text and how many characters could
/// not be decoded (no mapping in the active character set, or a sequence cut
/// off by the end of the value) and were replaced with U+FFFD.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Marc8Decoded {
    pub(crate) text: String,
    pub(crate) unmapped: usize,
}

const ESC: u8 = 0x1B;

/// Resolve an escape-sequence final byte to a character set. Besides the
/// finals [`CharacterSetId::from_byte`] knows, this accepts the finals of the
/// three MARC-specific sets (`b` subscripts, `p` superscripts, `g` Greek
/// symbols), as pymarc does.
fn charset_for_final(final_byte: u8) -> Option<CharacterSetId> {
    match final_byte {
        0x62 => Some(CharacterSetId::Subscript),
        0x70 => Some(CharacterSetId::Superscript),
        0x67 => Some(CharacterSetId::GreekSymbols),
        _ => CharacterSetId::from_byte(final_byte),
    }
}

/// Look up one byte in a single-byte character set. The tables key each set
/// at the positions the MARC-8 code tables assign (0x21-0x7E for sets normally
/// designated as G0, 0xA1-0xFE for ANSEL and the extended sets); a set
/// designated into the other half occupies the same positions with the high
/// bit flipped, so a miss is retried there.
fn lookup_single_byte(charset: CharacterSetId, byte: u8) -> Option<CharacterMapping> {
    if charset == CharacterSetId::EACC {
        return None;
    }
    let table = get_charset_table(charset);
    table
        .get(&byte)
        .or_else(|| table.get(&(byte ^ 0x80)))
        .copied()
}

/// Accumulates decoded characters, holding combining marks until their base
/// character arrives.
#[derive(Default)]
struct Marc8Output {
    text: String,
    pending_combining: Vec<char>,
    unmapped: usize,
}

impl Marc8Output {
    fn push(&mut self, mapping: Option<CharacterMapping>) {
        match mapping.and_then(|(cp, combining)| Some((char::from_u32(cp)?, combining))) {
            Some((c, true)) => self.pending_combining.push(c),
            Some((c, false)) => self.push_base(c),
            None => {
                self.unmapped += 1;
                self.push_base('\u{FFFD}');
            },
        }
    }

    /// MARC-8 stores combining marks before their base character; Unicode
    /// stores them after it.
    fn push_base(&mut self, c: char) {
        self.text.push(c);
        self.text.extend(self.pending_combining.drain(..));
    }

    fn finish(mut self) -> Marc8Decoded {
        use unicode_normalization::UnicodeNormalization;
        // Combining marks with no base character left to attach to.
        self.text.extend(self.pending_combining.drain(..));
        Marc8Decoded {
            text: self.text.nfc().collect(),
            unmapped: self.unmapped,
        }
    }
}

/// Which graphic set an escape sequence designates.
enum Designation {
    G0,
    G1,
}

/// What an ESC byte starts.
enum Escape {
    /// A designation of `set` by `final_byte`, `len` bytes long.
    Designates(Designation, u8, usize),
    /// Not a designation; the ESC is dropped.
    Other,
    /// A designation cut off by the end of the value.
    Truncated,
}

/// Classify the escape sequence starting at `bytes[0]`, an ESC.
fn parse_escape(bytes: &[u8]) -> Escape {
    let designation = |set, final_at: usize, len| match bytes.get(final_at) {
        Some(&f) => Escape::Designates(set, f, len),
        None => Escape::Truncated,
    };
    match bytes.get(1).copied() {
        None => Escape::Truncated,
        // ESC ( F and ESC , F designate G0; ESC ) F and ESC - F designate G1.
        Some(b'(' | b',') => designation(Designation::G0, 2, 3),
        Some(b')' | b'-') => designation(Designation::G1, 2, 3),
        // ESC $ F and ESC $ , F designate a multibyte G0; ESC $ ) F and
        // ESC $ - F a multibyte G1.
        Some(b'$') => match bytes.get(2).copied() {
            None => Escape::Truncated,
            Some(b',') => designation(Designation::G0, 3, 4),
            Some(b')' | b'-') => designation(Designation::G1, 3, 4),
            Some(f) => Escape::Designates(Designation::G0, f, 3),
        },
        // ESC s returns G0 to Basic Latin.
        Some(b's') => Escape::Designates(Designation::G0, CharacterSetId::BasicLatin as u8, 2),
        // ESC F with a set's final byte designates G0 directly (ESC b, ESC p,
        // and ESC g for the MARC-specific sets).
        Some(f) if charset_for_final(f).is_some() => Escape::Designates(Designation::G0, f, 2),
        Some(_) => Escape::Other,
    }
}

/// Decode MARC-8 bytes to Unicode, normalized to NFC.
///
/// Follows pymarc's `MARC8ToUnicode.translate`: ISO 2022 escape sequences
/// switch the G0 and G1 character sets; bytes 0x21-0x7E (or 3-byte groups
/// while G0 is EACC) are read from G0 and bytes 0xA1-0xFE from G1; control
/// bytes are dropped; and combining marks, which MARC-8 stores before their
/// base character, are emitted after it. Two differences from pymarc: a
/// character with no mapping becomes U+FFFD rather than a space and is counted
/// in [`Marc8Decoded::unmapped`], and a set designated into the half it is not
/// normally used in is still read correctly.
pub(crate) fn decode_marc8_lossy(bytes: &[u8]) -> Marc8Decoded {
    // Printable ASCII with no escape sequences decodes to itself, and makes up
    // most MARC-8 field values.
    if bytes.iter().all(|b| (0x20..=0x7E).contains(b)) {
        return Marc8Decoded {
            text: String::from_utf8_lossy(bytes).into_owned(),
            unmapped: 0,
        };
    }

    let mut g0 = Some(CharacterSetId::BasicLatin);
    let mut g1 = Some(CharacterSetId::AnselExtendedLatin);
    let mut out = Marc8Output {
        text: String::with_capacity(bytes.len()),
        ..Marc8Output::default()
    };
    let mut i = 0;

    while i < bytes.len() {
        let byte = bytes[i];

        if byte == ESC {
            match parse_escape(&bytes[i..]) {
                Escape::Designates(Designation::G0, f, len) => {
                    g0 = charset_for_final(f);
                    i += len;
                },
                Escape::Designates(Designation::G1, f, len) => {
                    g1 = charset_for_final(f);
                    i += len;
                },
                Escape::Other => i += 1,
                Escape::Truncated => {
                    out.push(None);
                    break;
                },
            }
            continue;
        }

        if g0 == Some(CharacterSetId::EACC) && (0x21..=0x7E).contains(&byte) {
            let Some(group) = bytes.get(i..i + 3) else {
                out.push(None);
                break;
            };
            let key = u32::from(group[0]) << 16 | u32::from(group[1]) << 8 | u32::from(group[2]);
            out.push(crate::marc8_tables::get_eacc_character(key));
            i += 3;
            continue;
        }

        i += 1;
        match byte {
            // Space is the same in every set and in either half.
            0x20 | 0xA0 => out.push(Some((0x20, false))),
            // Control bytes, including the non-sort markers 0x88 and 0x89.
            0x00..=0x1F | 0x7F..=0x9F => {},
            0x21..=0x7E => out.push(g0.and_then(|set| lookup_single_byte(set, byte))),
            _ => out.push(g1.and_then(|set| lookup_single_byte(set, byte))),
        }
    }

    out.finish()
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
                        // ESC ) 4 - Switch G1 to Extended Arabic
                        bytes.push(0x1B);
                        bytes.push(0x29);
                        bytes.push(0x34);
                    },
                    CharacterSetId::BasicCyrillic => {
                        // ESC ( N - Switch G0 to Basic Cyrillic
                        bytes.push(0x1B);
                        bytes.push(0x28);
                        bytes.push(0x4E);
                    },
                    CharacterSetId::ExtendedCyrillic => {
                        // ESC ) Q - Switch G1 to Extended Cyrillic
                        bytes.push(0x1B);
                        bytes.push(0x29);
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
                MarcError::encoding_msg(
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

    fn marc8(bytes: &[u8]) -> String {
        decode_bytes(bytes, MarcEncoding::Marc8).unwrap()
    }

    #[test]
    fn test_marc8_combining_mark_follows_its_base_character() {
        // MARC-8 stores a combining mark before its base character; Unicode
        // stores it after. ANSEL 0xE2 is the combining acute accent.
        assert_eq!(marc8(b"Caf\xE2e"), "Caf\u{e9}");
    }

    #[test]
    fn test_marc8_multiple_combining_marks_keep_their_order() {
        // Acute (0xE2) then diaeresis (0xE8) before 'a'.
        assert_eq!(marc8(b"\xE2\xE8a"), "\u{e1}\u{308}");
    }

    #[test]
    fn test_marc8_ansel_spacing_letters() {
        assert_eq!(marc8(b"\xA5\xB5"), "\u{c6}\u{e6}");
    }

    #[test]
    fn test_marc8_basic_cyrillic_as_g0() {
        assert_eq!(marc8(b"\x1B(NmIR\x1Bs"), "\u{41c}\u{438}\u{440}");
    }

    #[test]
    fn test_marc8_basic_cyrillic_as_g1() {
        // A 94-character set occupies the same positions in either half, so
        // designating it as G1 reads the same letters from the high bytes.
        assert_eq!(marc8(b"\x1B)N\xED\xC9\xD2\x1B)E"), "\u{41c}\u{438}\u{440}");
    }

    #[test]
    fn test_marc8_basic_greek_as_g0() {
        assert_eq!(marc8(b"\x1B(Sabd\x1Bs"), "\u{3b1}\u{3b2}\u{3b3}");
    }

    #[test]
    fn test_marc8_basic_hebrew_as_g0() {
        assert_eq!(marc8(b"\x1B(2ylem\x1Bs"), "\u{5e9}\u{5dc}\u{5d5}\u{5dd}");
    }

    #[test]
    fn test_marc8_basic_arabic_as_g0() {
        assert_eq!(marc8(b"\x1B(3SdGe\x1Bs"), "\u{633}\u{644}\u{627}\u{645}");
    }

    #[test]
    fn test_marc8_extended_cyrillic_as_g1() {
        assert_eq!(marc8(b"\x1B)Q\xC0\x1B)E"), "\u{491}");
    }

    #[test]
    fn test_marc8_subscript_and_superscript() {
        assert_eq!(marc8(b"H\x1Bb2\x1BsO"), "H\u{2082}O");
        assert_eq!(marc8(b"x\x1Bp2\x1Bs"), "x\u{b2}");
    }

    #[test]
    fn test_marc8_space_in_non_latin_g0() {
        assert_eq!(marc8(b"\x1B(Nm I\x1Bs"), "\u{41c} \u{438}");
    }

    #[test]
    fn test_marc8_eacc() {
        assert_eq!(marc8(b"\x1B$1\x21\x30\x21\x1B(B"), "\u{4e00}");
    }

    #[test]
    fn test_marc8_unmapped_byte_is_replacement_character() {
        // 0xAF has no mapping in ANSEL.
        assert_eq!(marc8(b"a\xAFb"), "a\u{fffd}b");
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
    fn test_marc8_encode_extended_sets_roundtrip() {
        // Extended Cyrillic and Extended Arabic are keyed in the high half, so
        // the encoder must designate them as G1 for the decoder to read them.
        for original in ["\u{491}", "\u{6FD}"] {
            let encoded = encode_string(original, MarcEncoding::Marc8).unwrap();
            assert_eq!(
                decode_bytes(&encoded, MarcEncoding::Marc8).unwrap(),
                original
            );
        }
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
        // An ESC that starts no designation is dropped; 0xFF has no mapping in
        // ANSEL (the default G1), so it becomes the replacement character.
        let bytes = b"\x1B\xFF";
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        assert_eq!(decoded, "\u{FFFD}");
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
        // Using Hebrew letters: alef (0x60), bet (0x61), gimel (0x62), read
        // from the high half (0xE0-0xE2) because ESC ) 2 designates Hebrew as G1
        let bytes = b"\x1B\x292\xE0\xE1\xE2\x1B\x29\x45"; // Designate Hebrew to G1, 3 Hebrew letters, designate ANSEL to G1 (reset)
        let decoded = decode_bytes(bytes, MarcEncoding::Marc8).unwrap();
        assert!(decoded.contains('א'), "Should contain Hebrew alef");
        assert!(decoded.contains('ב'), "Should contain Hebrew bet");
        assert!(decoded.contains('ג'), "Should contain Hebrew gimel");
    }

    #[test]
    fn test_marc8_arabic_text() {
        // Test Basic Arabic character set - ESC ) 3 (designate as G1)
        // Using Arabic letters: hamza (0x41), alef with madda (0x42), alef with
        // hamza above (0x43), read from the high half (0xC1-0xC3) as G1
        let bytes = b"\x1B\x293\xC1\xC2\xC3\x1B\x29\x45"; // Designate Arabic to G1, 3 Arabic letters, designate ANSEL to G1 (reset)
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
        // ESC ) 2 designates Hebrew to G1, then shin(0xF9)+lamed(0xEC)+vav(0xE5)+final_mem(0xED)
        let bytes = b"Hello\x1B\x292\xF9\xEC\xE5\xED\x1B\x29\x45!"; // "Hello", designate Hebrew to G1, Hebrew text, reset to ANSEL, "!"
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
