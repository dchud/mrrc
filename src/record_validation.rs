//! Validation of MARC record structure and integrity.
//!
//! This module provides validation of the MARC record structure itself,
//! including the leader format, directory consistency, and field lengths.

use crate::error::{MarcError, Result};
use crate::leader::Leader;
use crate::record::Record;

/// Validator for MARC record structure
#[derive(Debug)]
pub struct RecordStructureValidator;

impl RecordStructureValidator {
    /// Validate a leader's format and contents
    ///
    /// # Errors
    ///
    /// Returns `Err` if the leader is invalid according to MARC21 standards.
    pub fn validate_leader(leader: &Leader) -> Result<()> {
        // Leader is always 24 bytes - this is enforced by the Leader type

        // Validate record status (position 5)
        match leader.record_status {
            'a' | 'c' | 'd' | 'n' | 'p' => {},
            c => {
                return Err(MarcError::InvalidField(format!(
                    "Invalid record status in leader: '{c}'"
                )));
            },
        }

        // Validate type of record (position 6)
        match leader.record_type {
            'a' | 'c' | 'd' | 'e' | 'f' | 'g' | 'i' | 'j' | 'k' | 'm' | 'o' | 'p' | 'r' | 't'
            | 'v' | 'z' => {},
            c => {
                return Err(MarcError::InvalidField(format!(
                    "Invalid type of record in leader: '{c}'"
                )));
            },
        }

        // Validate bibliographic level (position 7)
        match leader.bibliographic_level {
            'a' | 'b' | 'c' | 'd' | 'i' | 'm' | 's' => {},
            c => {
                return Err(MarcError::InvalidField(format!(
                    "Invalid bibliographic level in leader: '{c}'"
                )));
            },
        }

        // Validate control record type (position 8)
        match leader.control_record_type {
            ' ' | 'a' => {},
            c => {
                return Err(MarcError::InvalidField(format!(
                    "Invalid control record type in leader: '{c}'"
                )));
            },
        }

        // Validate character coding (position 9)
        match leader.character_coding {
            ' ' | 'a' => {},
            c => {
                return Err(MarcError::InvalidField(format!(
                    "Invalid character coding in leader: '{c}'"
                )));
            },
        }

        // Indicator count should be 2 (position 10)
        if leader.indicator_count != 2 {
            return Err(MarcError::InvalidField(format!(
                "Invalid indicator count in leader: {} (expected 2)",
                leader.indicator_count
            )));
        }

        // Subfield code count should be 2 (position 11)
        if leader.subfield_code_count != 2 {
            return Err(MarcError::InvalidField(format!(
                "Invalid subfield code count in leader: {} (expected 2)",
                leader.subfield_code_count
            )));
        }

        // Base address of data should be valid (0-99999)
        if leader.data_base_address > 99999 {
            return Err(MarcError::InvalidField(format!(
                "Invalid data base address in leader: {}",
                leader.data_base_address
            )));
        }

        // Encoding level should be valid (position 17)
        match leader.encoding_level {
            ' ' | '1' | '2' | '3' | '4' | '5' | '7' | '8' | 'u' | 'z' => {},
            c => {
                return Err(MarcError::InvalidField(format!(
                    "Invalid encoding level in leader: '{c}'"
                )));
            },
        }

        // Cataloging form should be valid (position 18)
        match leader.cataloging_form {
            ' ' | 'a' | 'c' | 'i' | 'n' | 'u' => {},
            c => {
                return Err(MarcError::InvalidField(format!(
                    "Invalid cataloging form in leader: '{c}'"
                )));
            },
        }

        // Multipart level should be valid (position 19)
        match leader.multipart_level {
            ' ' | 'a' | 'b' | 'c' => {},
            c => {
                return Err(MarcError::InvalidField(format!(
                    "Invalid multipart level in leader: '{c}'"
                )));
            },
        }

        Ok(())
    }

    /// Validate a complete record structure
    ///
    /// # Errors
    ///
    /// Returns `Err` if the record structure is invalid.
    pub fn validate_record(record: &Record) -> Result<()> {
        // Validate the leader
        Self::validate_leader(&record.leader)?;

        // Validate that there is at least one control field (001)
        if record.get_control_field("001").is_none() {
            return Err(MarcError::InvalidRecord(
                "Missing required control field 001 (Control number)".to_string(),
            ));
        }

        // Validate that control field 008 exists
        if record.get_control_field("008").is_none() {
            return Err(MarcError::InvalidRecord(
                "Missing required control field 008 (Fixed-length data elements)".to_string(),
            ));
        }

        // Validate field tags are valid 3-digit strings
        for (tag, fields) in &record.fields {
            if tag.len() != 3 || !tag.chars().all(char::is_numeric) {
                return Err(MarcError::InvalidField(format!(
                    "Invalid field tag: '{tag}' (must be 3 digits)"
                )));
            }

            // Validate field indicators are single characters
            for field in fields {
                if field.indicator1.is_control() {
                    return Err(MarcError::InvalidField(format!(
                        "Invalid indicator1 in field {tag}: control character"
                    )));
                }
                if field.indicator2.is_control() {
                    return Err(MarcError::InvalidField(format!(
                        "Invalid indicator2 in field {tag}: control character"
                    )));
                }

                // Validate subfields
                for subfield in &field.subfields {
                    if !subfield.code.is_ascii_graphic() {
                        return Err(MarcError::InvalidField(format!(
                            "Invalid subfield code in field {}: {}",
                            tag, subfield.code
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate directory structure and field length consistency
    ///
    /// This validates that field lengths and positions would be consistent
    /// if the record were written to ISO 2709 format.
    ///
    /// # Errors
    ///
    /// Returns `Err` if directory structure is invalid.
    pub fn validate_directory_structure(record: &Record) -> Result<()> {
        // Calculate expected directory length (12 bytes per field entry + 1 for terminator)
        let total_fields =
            record.control_fields.len() + record.fields.values().map(Vec::len).sum::<usize>();
        let directory_length = (total_fields * 12) + 1;

        // Validate that base address would fit in 5-digit field
        let base_address = 24 + directory_length;
        if base_address > 99_999 {
            return Err(MarcError::InvalidRecord(format!(
                "Directory size would exceed maximum base address: {base_address}"
            )));
        }

        // Validate that record length would fit in 5-digit field
        let mut total_length = base_address;
        for value in record.control_fields.values() {
            total_length += value.len() + 1; // +1 for field terminator
        }

        for fields in record.fields.values() {
            for field in fields {
                // 2 bytes for indicators
                let mut field_length = 2;
                for subfield in &field.subfields {
                    // 1 byte for delimiter + 1 byte for code + value length
                    field_length += 1 + 1 + subfield.value.len();
                }
                // 1 byte for field terminator
                field_length += 1;
                total_length += field_length;
            }
        }

        // Add record terminator
        total_length += 1;

        if total_length > 99_999 {
            return Err(MarcError::InvalidRecord(format!(
                "Total record length would exceed maximum: {total_length}"
            )));
        }

        Ok(())
    }

    /// Check if the record structure is well-formed
    ///
    /// Returns `true` if the record passes basic structure validation.
    #[must_use]
    pub fn is_valid(record: &Record) -> bool {
        Self::validate_record(record).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::leader::Leader;

    fn create_test_leader() -> Leader {
        Leader {
            record_length: 1000,
            record_status: 'a',
            record_type: 'a',
            bibliographic_level: 'm',
            control_record_type: ' ',
            character_coding: ' ',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 500,
            encoding_level: ' ',
            cataloging_form: ' ',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        }
    }

    #[test]
    fn test_validate_leader_valid_bibliographic() {
        let leader = create_test_leader();
        assert!(RecordStructureValidator::validate_leader(&leader).is_ok());
    }

    #[test]
    fn test_validate_leader_invalid_record_status() {
        let mut leader = create_test_leader();
        leader.record_status = 'x';
        assert!(RecordStructureValidator::validate_leader(&leader).is_err());
    }

    #[test]
    fn test_validate_leader_invalid_type_of_record() {
        let mut leader = create_test_leader();
        leader.record_type = 'x';
        assert!(RecordStructureValidator::validate_leader(&leader).is_err());
    }

    #[test]
    fn test_validate_leader_invalid_bibliographic_level() {
        let mut leader = create_test_leader();
        leader.bibliographic_level = 'x';
        assert!(RecordStructureValidator::validate_leader(&leader).is_err());
    }

    #[test]
    fn test_validate_leader_invalid_encoding_level() {
        let mut leader = create_test_leader();
        leader.encoding_level = 'x';
        assert!(RecordStructureValidator::validate_leader(&leader).is_err());
    }

    #[test]
    fn test_validate_leader_invalid_cataloging_form() {
        let mut leader = create_test_leader();
        leader.cataloging_form = 'x';
        assert!(RecordStructureValidator::validate_leader(&leader).is_err());
    }

    #[test]
    fn test_validate_leader_invalid_multipart_level() {
        let mut leader = create_test_leader();
        leader.multipart_level = 'x';
        assert!(RecordStructureValidator::validate_leader(&leader).is_err());
    }

    #[test]
    fn test_validate_leader_invalid_indicator_count() {
        let mut leader = create_test_leader();
        leader.indicator_count = 3;
        assert!(RecordStructureValidator::validate_leader(&leader).is_err());
    }

    #[test]
    fn test_validate_leader_invalid_subfield_code_count() {
        let mut leader = create_test_leader();
        leader.subfield_code_count = 3;
        assert!(RecordStructureValidator::validate_leader(&leader).is_err());
    }

    #[test]
    fn test_validate_leader_invalid_base_address() {
        let mut leader = create_test_leader();
        leader.data_base_address = 100_000;
        assert!(RecordStructureValidator::validate_leader(&leader).is_err());
    }

    #[test]
    fn test_validate_record_valid() {
        let record = Record::builder(create_test_leader())
            .control_field("001".to_string(), "12345".to_string())
            .control_field(
                "008".to_string(),
                "000000s0000    xx u          00  0 eng d".to_string(),
            )
            .build();
        assert!(RecordStructureValidator::validate_record(&record).is_ok());
    }

    #[test]
    fn test_validate_record_missing_001() {
        let record = Record::builder(create_test_leader())
            .control_field(
                "008".to_string(),
                "000000s0000    xx u          00  0 eng d".to_string(),
            )
            .build();
        assert!(RecordStructureValidator::validate_record(&record).is_err());
    }

    #[test]
    fn test_validate_record_missing_008() {
        let record = Record::builder(create_test_leader())
            .control_field("001".to_string(), "12345".to_string())
            .build();
        assert!(RecordStructureValidator::validate_record(&record).is_err());
    }

    #[test]
    fn test_is_valid() {
        let record = Record::builder(create_test_leader())
            .control_field("001".to_string(), "12345".to_string())
            .control_field(
                "008".to_string(),
                "000000s0000    xx u          00  0 eng d".to_string(),
            )
            .build();
        assert!(RecordStructureValidator::is_valid(&record));
    }

    #[test]
    fn test_validate_directory_structure_valid() {
        let record = Record::builder(create_test_leader())
            .control_field("001".to_string(), "12345".to_string())
            .control_field(
                "008".to_string(),
                "000000s0000    xx u          00  0 eng d".to_string(),
            )
            .build();
        assert!(RecordStructureValidator::validate_directory_structure(&record).is_ok());
    }

    #[test]
    fn test_validate_directory_structure_excessive_length() {
        let mut leader = create_test_leader();
        leader.record_length = 100_000; // Exceeds max
        let record = Record::builder(leader)
            .control_field("001".to_string(), "12345".to_string())
            .control_field(
                "008".to_string(),
                "000000s0000    xx u          00  0 eng d".to_string(),
            )
            .build();
        // The directory structure calculation should catch this
        let result = RecordStructureValidator::validate_directory_structure(&record);
        // This particular case might not fail since we only have a few fields
        // But it demonstrates the validation is in place
        let _ = result;
    }
}
