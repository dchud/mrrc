//! Encoding detection and validation for MARC records.
//!
//! This module provides tools for detecting and validating character encodings
//! in MARC records, including support for mixed-encoding records and encoding
//! consistency checks.

use crate::encoding::MarcEncoding;
use crate::error::{MarcError, Result};
use crate::record::Record;

/// Result of encoding validation analysis
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
    /// Attempts to detect if the record uses mixed encodings by examining
    /// field data and comparing it to the declared encoding in the leader.
    ///
    /// # Errors
    ///
    /// Returns an error if encoding analysis fails.
    pub fn analyze_encoding(record: &Record) -> Result<EncodingAnalysis> {
        let primary_encoding = MarcEncoding::from_leader_char(record.leader.character_coding)?;

        let mut mixed_encodings = Vec::new();
        let mut inconsistent_field_count = 0usize;

        // Check control fields
        for value in record.control_fields.values() {
            if Self::is_likely_different_encoding(value, primary_encoding) {
                inconsistent_field_count += 1;
                let detected = Self::detect_encoding_from_string(value);
                if let Some(enc) = detected {
                    if enc != primary_encoding && !mixed_encodings.contains(&enc) {
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
                        if let Some(enc) = detected {
                            if enc != primary_encoding && !mixed_encodings.contains(&enc) {
                                mixed_encodings.push(enc);
                            }
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

    /// Check if a field's data is consistent with the expected encoding
    ///
    /// Returns true if the field appears to use a different encoding than expected.
    fn is_likely_different_encoding(data: &str, expected: MarcEncoding) -> bool {
        match expected {
            MarcEncoding::Utf8 => {
                // Check for patterns that suggest UTF-8 vs MARC-8
                // UTF-8 would have multi-byte sequences for non-ASCII characters
                // MARC-8 would use escape sequences
                contains_escape_sequences(data) && !contains_valid_utf8_multibyte(data)
            },
            MarcEncoding::Marc8 => {
                // Check for patterns that suggest valid UTF-8 encoding of non-ASCII
                // Count high bytes (0x80-0xFF) that form valid UTF-8 sequences
                let high_byte_count = data.as_bytes().iter().filter(|&&b| b >= 0x80).count();
                let total_bytes = data.len();

                // If we have significant high bytes but they don't form valid UTF-8
                // escape sequences, this is likely UTF-8
                if high_byte_count > total_bytes / 10 {
                    contains_valid_utf8_multibyte(data)
                } else {
                    false
                }
            },
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
            } => Err(MarcError::EncodingError(format!(
                "Mixed encodings detected: primary={primary:?}, secondary={secondary:?}, affected fields={field_count}"
            ))),
            EncodingAnalysis::Undetermined => Err(MarcError::EncodingError(
                "Unable to determine encoding".to_string(),
            )),
        }
    }
}

/// Check if a string contains MARC-8 escape sequences
fn contains_escape_sequences(s: &str) -> bool {
    s.as_bytes().contains(&0x1B)
}

/// Check if a string contains valid UTF-8 multibyte sequences
fn contains_valid_utf8_multibyte(s: &str) -> bool {
    let bytes = s.as_bytes();
    for i in 0..bytes.len() {
        let b = bytes[i];
        if b >= 0xC0 {
            let len = utf8_sequence_length(b);
            if len > 1 && i + len <= bytes.len() && is_valid_utf8_sequence(&bytes[i..i + len]) {
                return true;
            }
        }
    }
    false
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
    fn test_contains_valid_utf8_multibyte() {
        assert!(contains_valid_utf8_multibyte("café")); // Has é in UTF-8
        assert!(!contains_valid_utf8_multibyte("test")); // All ASCII
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
