//! Encoding detection and validation for MARC records.
//!
//! This module provides tools for detecting and validating character encodings
//! in MARC records, including support for mixed-encoding records and encoding
//! consistency checks.

use crate::encoding::MarcEncoding;
use crate::error::{MarcError, Result};
use crate::record::Record;

/// Result of encoding validation analysis
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodingAnalysis {
    /// Record uses a single consistent encoding
    Consistent(MarcEncoding),
    /// Record appears to have mixed encodings
    Mixed {
        /// Primary encoding (from leader)
        primary: MarcEncoding,
        /// Secondary encodings detected in data
        secondary: Vec<MarcEncoding>,
        /// Number of fields with inconsistent encoding
        field_count: usize,
    },
    /// Unable to determine encoding from data
    Undetermined,
}

/// Validator for MARC record encodings
#[derive(Debug)]
pub struct EncodingValidator;

impl EncodingValidator {
    /// Analyze the encoding of a MARC record
    ///
    /// The primary encoding is the one leader position 09 declares, by the
    /// readers' rule ([`MarcEncoding::declared_by_leader`]). The record's
    /// values have already been decoded, so the only mismatch visible in
    /// them is a MARC-8 escape sequence (ESC, 0x1B) left in a record
    /// decoded as UTF-8: MARC-8 data whose leader says UTF-8. UTF-8 data
    /// whose leader says MARC-8 has been decoded as MARC-8 by the time it
    /// reaches here; read such files with [`crate::CharacterCoding::Detect`].
    ///
    /// # Errors
    ///
    /// This function does not return an error.
    pub fn analyze_encoding(record: &Record) -> Result<EncodingAnalysis> {
        let primary_encoding = MarcEncoding::declared_by_leader(record.leader.character_coding);

        let mut mixed_encodings = Vec::new();
        let mut inconsistent_field_count = 0usize;

        // Check control fields
        for values in record.control_fields.values() {
            for value in values {
                if Self::is_likely_different_encoding(value, primary_encoding) {
                    inconsistent_field_count += 1;
                    let detected = Self::detect_encoding_from_string(value);
                    if let Some(enc) = detected
                        && enc != primary_encoding
                        && !mixed_encodings.contains(&enc)
                    {
                        mixed_encodings.push(enc);
                    }
                }
            }
        }

        // Check data fields and subfields
        for fields in record.fields.values() {
            for field in fields {
                for subfield in &field.subfields {
                    if Self::is_likely_different_encoding(&subfield.value, primary_encoding) {
                        inconsistent_field_count += 1;
                        let detected = Self::detect_encoding_from_string(&subfield.value);
                        if let Some(enc) = detected
                            && enc != primary_encoding
                            && !mixed_encodings.contains(&enc)
                        {
                            mixed_encodings.push(enc);
                        }
                    }
                }
            }
        }

        if mixed_encodings.is_empty() {
            Ok(EncodingAnalysis::Consistent(primary_encoding))
        } else {
            Ok(EncodingAnalysis::Mixed {
                primary: primary_encoding,
                secondary: mixed_encodings,
                field_count: inconsistent_field_count,
            })
        }
    }

    /// Check if a decoded value shows signs of an encoding other than the
    /// one it was decoded in.
    fn is_likely_different_encoding(data: &str, expected: MarcEncoding) -> bool {
        match expected {
            // MARC-8 escape sequences survive decoding as UTF-8 (ESC is
            // ASCII), while the MARC-8 diacritics around them become U+FFFD.
            MarcEncoding::Utf8 => contains_escape_sequences(data),
            // Decoded MARC-8 legitimately holds non-ASCII characters, and the
            // decoder consumes escape sequences, so nothing in the text can
            // reveal a different encoding.
            MarcEncoding::Marc8 => false,
        }
    }

    /// Attempt to detect the encoding used in a string
    fn detect_encoding_from_string(s: &str) -> Option<MarcEncoding> {
        let bytes = s.as_bytes();

        // Check for UTF-8 multibyte sequences
        let utf8_indicator_count = count_utf8_indicators(bytes);

        // Check for MARC-8 escape sequences
        let marc8_escape_count = bytes.windows(2).filter(|w| w[0] == 0x1B).count();

        // Check for actual valid UTF-8 encoding of high bytes
        let mut has_valid_utf8 = false;
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            if b >= 0xC0 {
                // Potential UTF-8 multibyte start
                let len = utf8_sequence_length(b);
                if len > 1 && i + len <= bytes.len() && is_valid_utf8_sequence(&bytes[i..i + len]) {
                    has_valid_utf8 = true;
                    i += len;
                    continue;
                }
            }
            i += 1;
        }

        if marc8_escape_count > 0 {
            Some(MarcEncoding::Marc8)
        } else if utf8_indicator_count > 2 || has_valid_utf8 {
            Some(MarcEncoding::Utf8)
        } else {
            None
        }
    }

    /// Validate that a record's encoding is consistent
    ///
    /// Returns `Ok(())` if encoding is consistent, or an error describing the issue.
    ///
    /// # Errors
    ///
    /// Returns an error if mixed encodings or undetermined encodings are detected.
    pub fn validate_encoding(record: &Record) -> Result<()> {
        match Self::analyze_encoding(record)? {
            EncodingAnalysis::Consistent(_) => Ok(()),
            EncodingAnalysis::Mixed {
                primary,
                secondary,
                field_count,
            } => Err(MarcError::encoding_msg(format!(
                "Mixed encodings detected: primary={primary:?}, secondary={secondary:?}, affected fields={field_count}"
            ))),
            EncodingAnalysis::Undetermined => Err(MarcError::encoding_msg(
                "Unable to determine encoding".to_string(),
            )),
        }
    }
}

/// Check if a string contains MARC-8 escape sequences
fn contains_escape_sequences(s: &str) -> bool {
    s.as_bytes().contains(&0x1B)
}

/// Get the expected length of a UTF-8 sequence from the first byte
fn utf8_sequence_length(first_byte: u8) -> usize {
    match first_byte {
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF7 => 4,
        _ => 1,
    }
}

/// Check if a byte sequence is a valid UTF-8 sequence
fn is_valid_utf8_sequence(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }

    let first = bytes[0];
    let expected_len = utf8_sequence_length(first);

    if bytes.len() != expected_len {
        return false;
    }

    // Check continuation bytes
    for (i, &byte) in bytes.iter().enumerate() {
        if i == 0 {
            continue; // First byte already checked
        }
        if (byte & 0xC0) != 0x80 {
            return false; // Invalid continuation byte
        }
    }

    true
}

/// Count indicators of UTF-8 multibyte characters
fn count_utf8_indicators(bytes: &[u8]) -> usize {
    bytes.iter().filter(|&&b| matches!(b, 0xC0..=0xF7)).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record_with(position_09: char, title: &str) -> Record {
        let mut record = Record::new(crate::Leader {
            character_coding: position_09,
            ..crate::Leader::default()
        });
        let mut field = crate::Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', title.to_string());
        record.add_field(field);
        record
    }

    #[test]
    fn test_decoded_marc8_record_with_diacritics_is_consistent() {
        // The readers decode MARC-8 into Unicode, so a MARC-8 record's values
        // legitimately hold non-ASCII characters.
        let record = record_with(' ', "Caf\u{e9} \u{41c}\u{438}\u{440}");
        assert_eq!(
            EncodingValidator::analyze_encoding(&record).unwrap(),
            EncodingAnalysis::Consistent(MarcEncoding::Marc8)
        );
    }

    #[test]
    fn test_position_09_other_than_a_is_marc8() {
        // The readers' rule, from pymarc: only `a` means UTF-8.
        let record = record_with('q', "Title");
        assert_eq!(
            EncodingValidator::analyze_encoding(&record).unwrap(),
            EncodingAnalysis::Consistent(MarcEncoding::Marc8)
        );
    }

    #[test]
    fn test_utf8_record_with_escape_sequences_is_mixed() {
        // MARC-8 bytes read as UTF-8 keep their ESC, which marks the value
        // as MARC-8.
        // A diacritic in the same record comes out as U+FFFD, which is
        // valid multibyte UTF-8 and must not hide the escape sequences.
        let record = record_with('a', "Caf\u{fffd}e H\x1Bb2\x1BsO");
        assert!(matches!(
            EncodingValidator::analyze_encoding(&record).unwrap(),
            EncodingAnalysis::Mixed {
                primary: MarcEncoding::Utf8,
                ..
            }
        ));
    }

    #[test]
    fn test_utf8_sequence_length() {
        assert_eq!(utf8_sequence_length(0x41), 1); // 'A'
        assert_eq!(utf8_sequence_length(0xC0), 2); // 2-byte
        assert_eq!(utf8_sequence_length(0xE0), 3); // 3-byte
        assert_eq!(utf8_sequence_length(0xF0), 4); // 4-byte
    }

    #[test]
    fn test_is_valid_utf8_sequence() {
        assert!(is_valid_utf8_sequence(b"A")); // 1-byte
        assert!(is_valid_utf8_sequence(&[0xC3, 0xA9])); // é in UTF-8
        assert!(is_valid_utf8_sequence(&[0xE2, 0x82, 0xAC])); // € in UTF-8
        assert!(!is_valid_utf8_sequence(&[0xC3])); // Incomplete
        assert!(!is_valid_utf8_sequence(&[0xC3, 0x28])); // Invalid continuation
    }

    #[test]
    fn test_contains_escape_sequences() {
        assert!(contains_escape_sequences("test\x1Btest"));
        assert!(!contains_escape_sequences("test"));
    }

    #[test]
    fn test_detect_encoding_utf8() {
        let utf8_str = "café"; // Contains UTF-8 encoded character
        let result = EncodingValidator::detect_encoding_from_string(utf8_str);
        assert_eq!(result, Some(MarcEncoding::Utf8));
    }

    #[test]
    fn test_detect_encoding_ascii() {
        let ascii_str = "test";
        let result = EncodingValidator::detect_encoding_from_string(ascii_str);
        // ASCII alone is ambiguous
        assert!(result.is_none() || result == Some(MarcEncoding::Utf8));
    }
}
