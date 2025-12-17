//! Character encoding support for MARC records.
//!
//! MARC records can use different character encodings:
//! - **MARC-8** (legacy) — Mixed character sets with escape sequences
//! - **UTF-8** (modern) — Unicode standard encoding
//!
//! The encoding is indicated in position 9 of the MARC leader:
//! - Space character = MARC-8
//! - 'a' = UTF-8
//!
//! This module provides automatic encoding detection and conversion.

use crate::error::{MarcError, Result};

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
    pub fn from_leader_char(c: char) -> Result<Self> {
        match c {
            ' ' => Ok(MarcEncoding::Marc8),
            'a' => Ok(MarcEncoding::Utf8),
            _ => Err(MarcError::EncodingError(format!(
                "Unknown character encoding: {}",
                c
            ))),
        }
    }

    /// Get the leader character for this encoding
    pub fn as_leader_char(&self) -> char {
        match self {
            MarcEncoding::Marc8 => ' ',
            MarcEncoding::Utf8 => 'a',
        }
    }
}

/// Decode bytes using the specified encoding
pub fn decode_bytes(bytes: &[u8], encoding: MarcEncoding) -> Result<String> {
    match encoding {
        MarcEncoding::Utf8 => String::from_utf8(bytes.to_vec())
            .map_err(|e| MarcError::EncodingError(format!("Invalid UTF-8: {}", e))),
        MarcEncoding::Marc8 => decode_marc8(bytes),
    }
}

/// Encode string using the specified encoding
pub fn encode_string(s: &str, encoding: MarcEncoding) -> Result<Vec<u8>> {
    match encoding {
        MarcEncoding::Utf8 => Ok(s.as_bytes().to_vec()),
        MarcEncoding::Marc8 => encode_marc8(s),
    }
}

/// Decode MARC-8 bytes to UTF-8 string
/// MARC-8 is a complex encoding with multiple character sets and escape sequences
/// For now, we implement basic ASCII support and fallback to UTF-8 for extended chars
fn decode_marc8(bytes: &[u8]) -> Result<String> {
    let mut result = String::new();
    let mut i = 0;

    while i < bytes.len() {
        let byte = bytes[i];

        // ASCII range (0x00-0x7F) - direct mapping
        if byte < 0x80 {
            result.push(byte as char);
            i += 1;
        } else if byte >= 0x80 {
            // Non-ASCII MARC-8 characters
            // For extended characters, we try to decode as UTF-8 or use replacement char
            if i + 1 < bytes.len() {
                // Try to decode as UTF-8 sequence
                let (chars, consumed) = decode_utf8_sequence(&bytes[i..])?;
                result.push_str(&chars);
                i += consumed;
            } else {
                // Single high byte at end of string
                result.push('\u{FFFD}'); // Replacement character
                i += 1;
            }
        }
    }

    Ok(result)
}

/// Encode UTF-8 string to MARC-8 bytes
/// For simplicity, we encode ASCII as-is and convert extended Unicode to UTF-8 bytes
fn encode_marc8(s: &str) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    for c in s.chars() {
        if c.is_ascii() {
            bytes.push(c as u8);
        } else {
            // Encode non-ASCII characters as UTF-8 (multi-byte sequence)
            let mut buf = [0u8; 4];
            let encoded = c.encode_utf8(&mut buf);
            bytes.extend_from_slice(encoded.as_bytes());
        }
    }
    Ok(bytes)
}

/// Helper to decode a UTF-8 sequence from bytes
/// Returns the decoded string and number of bytes consumed
fn decode_utf8_sequence(bytes: &[u8]) -> Result<(String, usize)> {
    if bytes.is_empty() {
        return Ok((String::new(), 0));
    }

    let first = bytes[0];

    // Determine UTF-8 sequence length from first byte
    let len = if first < 0x80 {
        1
    } else if (first & 0xE0) == 0xC0 {
        2
    } else if (first & 0xF0) == 0xE0 {
        3
    } else if (first & 0xF8) == 0xF0 {
        4
    } else {
        // Invalid UTF-8 start byte
        return Ok(("\u{FFFD}".to_string(), 1));
    };

    if len > bytes.len() {
        // Incomplete sequence
        return Ok(("\u{FFFD}".to_string(), 1));
    }

    match String::from_utf8(bytes[..len].to_vec()) {
        Ok(s) => Ok((s, len)),
        Err(_) => Ok(("\u{FFFD}".to_string(), len)),
    }
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
        let s = "Café";
        let encoded = encode_string(s, MarcEncoding::Marc8).unwrap();
        // é should be encoded as UTF-8 multi-byte
        assert!(encoded.len() > 4);
    }

    #[test]
    fn test_marc8_decode_invalid_utf8() {
        // Invalid UTF-8 sequence should use replacement character
        let bytes = vec![0xFF];
        let decoded = decode_bytes(&bytes, MarcEncoding::Marc8).unwrap();
        assert!(decoded.contains('\u{FFFD}'));
    }

    #[test]
    fn test_encoding_roundtrip() {
        let original = "Test String with 123";
        let encoded = encode_string(original, MarcEncoding::Utf8).unwrap();
        let decoded = decode_bytes(&encoded, MarcEncoding::Utf8).unwrap();
        assert_eq!(original, decoded);
    }
}
