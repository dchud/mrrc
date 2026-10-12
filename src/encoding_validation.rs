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

        let control_values = record.control_fields.values().flatten();
        let subfield_values = record
            .fields
            .values()
            .flatten()
            .flat_map(|field| field.subfields.iter().map(|subfield| &subfield.value));
        for value in control_values.chain(subfield_values) {
            if let Some(enc) = Self::other_encoding_in(value, primary_encoding) {
                inconsistent_field_count += 1;
                if !mixed_encodings.contains(&enc) {
                    mixed_encodings.push(enc);
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

    /// The encoding a decoded value shows signs of, if it differs from the
    /// one the value was decoded in.
    fn other_encoding_in(data: &str, decoded_as: MarcEncoding) -> Option<MarcEncoding> {
        match decoded_as {
            // MARC-8 escape sequences survive decoding as UTF-8 (ESC is
            // ASCII). The MARC-8 diacritics around them are invalid UTF-8,
            // which the reader rejects, replaces with U+FFFD, or drops,
            // depending on its `Utf8Handling`.
            MarcEncoding::Utf8 => contains_escape_sequences(data).then_some(MarcEncoding::Marc8),
            // Decoded MARC-8 legitimately holds non-ASCII characters, and the
            // decoder consumes escape sequences, so nothing in the text can
            // reveal a different encoding.
            MarcEncoding::Marc8 => None,
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
        // Under `Utf8Handling::Replace`, a diacritic in the same record
        // comes out as U+FFFD, which is valid multibyte UTF-8 and must not
        // hide the escape sequences.
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
    fn test_utf8_record_with_trailing_escape_is_mixed() {
        // An escape sequence cut off by the end of the value leaves ESC as
        // its last character.
        let record = record_with('a', "Title \x1B");
        assert_eq!(
            EncodingValidator::analyze_encoding(&record).unwrap(),
            EncodingAnalysis::Mixed {
                primary: MarcEncoding::Utf8,
                secondary: vec![MarcEncoding::Marc8],
                field_count: 1,
            }
        );
    }

    #[test]
    fn test_contains_escape_sequences() {
        assert!(contains_escape_sequences("test\x1Btest"));
        assert!(!contains_escape_sequences("test"));
    }
}
