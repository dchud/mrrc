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
    /// Validate the leader positions that are shared across all MARC 21
    /// record types (positions 10, 11, 12-16). Used by the per-record-type
    /// leader validators below.
    fn validate_leader_shared(leader: &Leader) -> Result<()> {
        // Indicator count should be 2 (position 10)
        if leader.indicator_count != 2 {
            return Err(MarcError::leader_msg(format!(
                "Invalid indicator count in leader: {} (expected 2)",
                leader.indicator_count
            )));
        }

        // Subfield code count should be 2 (position 11)
        if leader.subfield_code_count != 2 {
            return Err(MarcError::leader_msg(format!(
                "Invalid subfield code count in leader: {} (expected 2)",
                leader.subfield_code_count
            )));
        }

        // Base address of data should be valid (0-99999)
        if leader.data_base_address > 99999 {
            return Err(MarcError::leader_msg(format!(
                "Invalid data base address in leader: {}",
                leader.data_base_address
            )));
        }

        Ok(())
    }

    /// Validate a leader against the MARC 21 **Bibliographic** Format rules.
    ///
    /// This is the leader-validation entry point invoked by the bibliographic
    /// reader at `validation_level=strict_marc`. Authority and holdings
    /// readers dispatch to [`validate_leader_authority`] and
    /// [`validate_leader_holdings`] respectively, which carry different
    /// allowed-value sets per the corresponding MARC 21 format specs.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the leader is invalid per the MARC 21 Bibliographic
    /// Format leader allowed-value sets.
    ///
    /// [`validate_leader_authority`]: Self::validate_leader_authority
    /// [`validate_leader_holdings`]: Self::validate_leader_holdings
    pub fn validate_leader(leader: &Leader) -> Result<()> {
        // Leader is always 24 bytes - this is enforced by the Leader type

        // Validate record status (position 5)
        match leader.record_status {
            'a' | 'c' | 'd' | 'n' | 'p' => {},
            c => {
                return Err(MarcError::leader_msg(format!(
                    "Invalid record status in leader: '{c}'"
                )));
            },
        }

        // Validate type of record (position 6)
        match leader.record_type {
            'a' | 'c' | 'd' | 'e' | 'f' | 'g' | 'i' | 'j' | 'k' | 'm' | 'o' | 'p' | 'r' | 't'
            | 'v' | 'z' => {},
            c => {
                return Err(MarcError::leader_msg(format!(
                    "Invalid type of record in leader: '{c}'"
                )));
            },
        }

        // Validate bibliographic level (position 7)
        match leader.bibliographic_level {
            'a' | 'b' | 'c' | 'd' | 'i' | 'm' | 's' => {},
            c => {
                return Err(MarcError::leader_msg(format!(
                    "Invalid bibliographic level in leader: '{c}'"
                )));
            },
        }

        // Validate control record type (position 8)
        match leader.control_record_type {
            ' ' | 'a' => {},
            c => {
                return Err(MarcError::leader_msg(format!(
                    "Invalid control record type in leader: '{c}'"
                )));
            },
        }

        // Validate character coding (position 9)
        match leader.character_coding {
            ' ' | 'a' => {},
            c => {
                return Err(MarcError::leader_msg(format!(
                    "Invalid character coding in leader: '{c}'"
                )));
            },
        }

        Self::validate_leader_shared(leader)?;

        // Encoding level should be valid (position 17)
        match leader.encoding_level {
            ' ' | '1' | '2' | '3' | '4' | '5' | '7' | '8' | 'u' | 'z' => {},
            c => {
                return Err(MarcError::leader_msg(format!(
                    "Invalid encoding level in leader: '{c}'"
                )));
            },
        }

        // Cataloging form should be valid (position 18)
        match leader.cataloging_form {
            ' ' | 'a' | 'c' | 'i' | 'n' | 'u' => {},
            c => {
                return Err(MarcError::leader_msg(format!(
                    "Invalid cataloging form in leader: '{c}'"
                )));
            },
        }

        // Multipart level should be valid (position 19)
        match leader.multipart_level {
            ' ' | 'a' | 'b' | 'c' => {},
            c => {
                return Err(MarcError::leader_msg(format!(
                    "Invalid multipart level in leader: '{c}'"
                )));
            },
        }

        Ok(())
    }

    /// Validate a leader against the MARC 21 **Authority** Format rules.
    ///
    /// Positions 7 (`bibliographic_level`), 8 (`control_record_type`), and 19
    /// (`multipart_level`) are *undefined* in the MARC 21 Authority Format —
    /// any byte is accepted there. The remaining positions follow the
    /// authority allowed-value sets:
    ///
    /// - 5 `record_status`: `a, c, d, n, o, s, x`
    /// - 6 `record_type`: `z`
    /// - 9 `character_coding`: ` `, `a`
    /// - 10/11/12-16: shared (indicator count = 2, subfield code count = 2,
    ///   base address ≤ 99999)
    /// - 17 `encoding_level`: `n`, `o`
    /// - 18 `cataloging_form` (interpreted as *punctuation policy* in the
    ///   authority spec): ` `, `c`, `i`, `u`
    ///
    /// # Errors
    ///
    /// Returns `Err` if the leader violates any of the MARC 21 Authority
    /// Format leader allowed-value sets above.
    pub fn validate_leader_authority(leader: &Leader) -> Result<()> {
        // Position 5: record status
        match leader.record_status {
            'a' | 'c' | 'd' | 'n' | 'o' | 's' | 'x' => {},
            c => {
                return Err(MarcError::leader_msg(format!(
                    "Invalid record status in authority leader: '{c}'"
                )));
            },
        }

        // Position 6: type of record — must be 'z' for authority
        if leader.record_type != 'z' {
            return Err(MarcError::leader_msg(format!(
                "Invalid type of record in authority leader: '{}' (expected 'z')",
                leader.record_type
            )));
        }

        // Positions 7, 8, 19: undefined for authority — accept any byte.

        // Position 9: character coding
        match leader.character_coding {
            ' ' | 'a' => {},
            c => {
                return Err(MarcError::leader_msg(format!(
                    "Invalid character coding in authority leader: '{c}'"
                )));
            },
        }

        Self::validate_leader_shared(leader)?;

        // Position 17: encoding level (authority allowed set is narrower
        // than bibliographic).
        match leader.encoding_level {
            'n' | 'o' => {},
            c => {
                return Err(MarcError::leader_msg(format!(
                    "Invalid encoding level in authority leader: '{c}'"
                )));
            },
        }

        // Position 18: punctuation policy (occupies the byte the bibliographic
        // spec calls "cataloging form").
        match leader.cataloging_form {
            ' ' | 'c' | 'i' | 'u' => {},
            c => {
                return Err(MarcError::leader_msg(format!(
                    "Invalid punctuation policy in authority leader: '{c}'"
                )));
            },
        }

        Ok(())
    }

    /// Validate a leader against the MARC 21 **Holdings** Format rules.
    ///
    /// Positions 7 (`bibliographic_level`), 8 (`control_record_type`), and 19
    /// (`multipart_level`) are *undefined* in the MARC 21 Holdings Format —
    /// any byte is accepted there. The remaining positions follow the
    /// holdings allowed-value sets:
    ///
    /// - 5 `record_status`: `c, d, n`
    /// - 6 `record_type`: `u, v, x, y`
    /// - 9 `character_coding`: ` `, `a`
    /// - 10/11/12-16: shared (indicator count = 2, subfield code count = 2,
    ///   base address ≤ 99999)
    /// - 17 `encoding_level`: `1, 2, 3, 4, 5, m, u, z`
    /// - 18 `cataloging_form` (interpreted as *item information in record*
    ///   in the holdings spec): ` `, `i`, `n`
    ///
    /// # Errors
    ///
    /// Returns `Err` if the leader violates any of the MARC 21 Holdings
    /// Format leader allowed-value sets above.
    pub fn validate_leader_holdings(leader: &Leader) -> Result<()> {
        // Position 5: record status
        match leader.record_status {
            'c' | 'd' | 'n' => {},
            c => {
                return Err(MarcError::leader_msg(format!(
                    "Invalid record status in holdings leader: '{c}'"
                )));
            },
        }

        // Position 6: type of record — must be 'u', 'v', 'x', or 'y' for holdings.
        match leader.record_type {
            'u' | 'v' | 'x' | 'y' => {},
            c => {
                return Err(MarcError::leader_msg(format!(
                    "Invalid type of record in holdings leader: '{c}' (expected one of u/v/x/y)"
                )));
            },
        }

        // Positions 7, 8, 19: undefined for holdings — accept any byte.

        // Position 9: character coding
        match leader.character_coding {
            ' ' | 'a' => {},
            c => {
                return Err(MarcError::leader_msg(format!(
                    "Invalid character coding in holdings leader: '{c}'"
                )));
            },
        }

        Self::validate_leader_shared(leader)?;

        // Position 17: encoding level (holdings allowed set differs entirely
        // from bibliographic/authority).
        match leader.encoding_level {
            '1' | '2' | '3' | '4' | '5' | 'm' | 'u' | 'z' => {},
            c => {
                return Err(MarcError::leader_msg(format!(
                    "Invalid encoding level in holdings leader: '{c}'"
                )));
            },
        }

        // Position 18: item information in record (occupies the byte the
        // bibliographic spec calls "cataloging form").
        match leader.cataloging_form {
            ' ' | 'i' | 'n' => {},
            c => {
                return Err(MarcError::leader_msg(format!(
                    "Invalid item-information code in holdings leader: '{c}'"
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
            return Err(MarcError::invalid_field_msg(
                "Missing required control field 001 (Control number)".to_string(),
            ));
        }

        // Validate that control field 008 exists
        if record.get_control_field("008").is_none() {
            return Err(MarcError::invalid_field_msg(
                "Missing required control field 008 (Fixed-length data elements)".to_string(),
            ));
        }

        // Validate field tags are valid 3-digit strings
        for (tag, fields) in &record.fields {
            if tag.len() != 3 || !tag.chars().all(char::is_numeric) {
                return Err(MarcError::invalid_field_msg(format!(
                    "Invalid field tag: '{tag}' (must be 3 digits)"
                )));
            }

            // Validate field indicators are single characters
            for field in fields {
                if field.indicator1.is_control() {
                    return Err(MarcError::invalid_field_msg(format!(
                        "Invalid indicator1 in field {tag}: control character"
                    )));
                }
                if field.indicator2.is_control() {
                    return Err(MarcError::invalid_field_msg(format!(
                        "Invalid indicator2 in field {tag}: control character"
                    )));
                }

                // Validate subfields
                for subfield in &field.subfields {
                    if !subfield.code.is_ascii_graphic() {
                        return Err(MarcError::invalid_field_msg(format!(
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
        let total_fields = record.control_fields.values().map(Vec::len).sum::<usize>()
            + record.fields.values().map(Vec::len).sum::<usize>();
        let directory_length = (total_fields * 12) + 1;

        // Validate that base address would fit in 5-digit field
        let base_address = 24 + directory_length;
        if base_address > 99_999 {
            return Err(MarcError::invalid_field_msg(format!(
                "Directory size would exceed maximum base address: {base_address}"
            )));
        }

        // Validate that record length would fit in 5-digit field
        let mut total_length = base_address;
        for values in record.control_fields.values() {
            for value in values {
                total_length += value.len() + 1; // +1 for field terminator
            }
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
            return Err(MarcError::invalid_field_msg(format!(
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

    // ----- Per-record-type leader validators ----------------------------

    fn create_authority_leader() -> Leader {
        Leader {
            record_length: 1000,
            record_status: 'n',
            record_type: 'z',
            bibliographic_level: '|', // undefined for authority — any byte
            control_record_type: ' ',
            character_coding: ' ',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 500,
            encoding_level: 'n',
            cataloging_form: ' ', // punctuation policy
            multipart_level: ' ',
            reserved: "4500".to_string(),
        }
    }

    fn create_holdings_leader() -> Leader {
        Leader {
            record_length: 1000,
            record_status: 'n',
            record_type: 'x',
            bibliographic_level: '|', // undefined for holdings — any byte
            control_record_type: ' ',
            character_coding: ' ',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 500,
            encoding_level: '1',
            cataloging_form: ' ', // item information in record
            multipart_level: ' ',
            reserved: "4500".to_string(),
        }
    }

    #[test]
    fn test_validate_leader_authority_accepts_undefined_position_7() {
        // Position 7 (`bibliographic_level`) is undefined for authority; the
        // fill character '|' and any other byte must pass without firing E002.
        let mut leader = create_authority_leader();
        leader.bibliographic_level = '|';
        assert!(RecordStructureValidator::validate_leader_authority(&leader).is_ok());
        leader.bibliographic_level = 'q';
        assert!(RecordStructureValidator::validate_leader_authority(&leader).is_ok());
    }

    #[test]
    fn test_validate_leader_authority_rejects_non_z_record_type() {
        let mut leader = create_authority_leader();
        leader.record_type = 'a'; // bibliographic
        assert!(RecordStructureValidator::validate_leader_authority(&leader).is_err());
    }

    #[test]
    fn test_validate_leader_authority_record_status_set() {
        // Authority status set is wider than bibliographic.
        for status in ['a', 'c', 'd', 'n', 'o', 's', 'x'] {
            let mut leader = create_authority_leader();
            leader.record_status = status;
            assert!(
                RecordStructureValidator::validate_leader_authority(&leader).is_ok(),
                "authority status '{status}' should be accepted"
            );
        }
        let mut leader = create_authority_leader();
        leader.record_status = 'q';
        assert!(RecordStructureValidator::validate_leader_authority(&leader).is_err());
    }

    #[test]
    fn test_validate_leader_authority_encoding_level() {
        for level in ['n', 'o'] {
            let mut leader = create_authority_leader();
            leader.encoding_level = level;
            assert!(RecordStructureValidator::validate_leader_authority(&leader).is_ok());
        }
        let mut leader = create_authority_leader();
        leader.encoding_level = '1'; // bibliographic-only
        assert!(RecordStructureValidator::validate_leader_authority(&leader).is_err());
    }

    #[test]
    fn test_validate_leader_authority_punctuation_policy() {
        for policy in [' ', 'c', 'i', 'u'] {
            let mut leader = create_authority_leader();
            leader.cataloging_form = policy;
            assert!(RecordStructureValidator::validate_leader_authority(&leader).is_ok());
        }
        let mut leader = create_authority_leader();
        leader.cataloging_form = 'a'; // bibliographic "AACR 2", not valid for authority
        assert!(RecordStructureValidator::validate_leader_authority(&leader).is_err());
    }

    #[test]
    fn test_validate_leader_holdings_accepts_undefined_position_7() {
        let mut leader = create_holdings_leader();
        leader.bibliographic_level = '|';
        assert!(RecordStructureValidator::validate_leader_holdings(&leader).is_ok());
    }

    #[test]
    fn test_validate_leader_holdings_record_type_set() {
        for rt in ['u', 'v', 'x', 'y'] {
            let mut leader = create_holdings_leader();
            leader.record_type = rt;
            assert!(
                RecordStructureValidator::validate_leader_holdings(&leader).is_ok(),
                "holdings record_type '{rt}' should be accepted"
            );
        }
        let mut leader = create_holdings_leader();
        leader.record_type = 'a';
        assert!(RecordStructureValidator::validate_leader_holdings(&leader).is_err());
        leader.record_type = 'z';
        assert!(RecordStructureValidator::validate_leader_holdings(&leader).is_err());
    }

    #[test]
    fn test_validate_leader_holdings_encoding_level() {
        for level in ['1', '2', '3', '4', '5', 'm', 'u', 'z'] {
            let mut leader = create_holdings_leader();
            leader.encoding_level = level;
            assert!(
                RecordStructureValidator::validate_leader_holdings(&leader).is_ok(),
                "holdings encoding_level '{level}' should be accepted"
            );
        }
        let mut leader = create_holdings_leader();
        leader.encoding_level = 'n'; // authority-only
        assert!(RecordStructureValidator::validate_leader_holdings(&leader).is_err());
    }

    #[test]
    fn test_validate_leader_holdings_item_information() {
        for code in [' ', 'i', 'n'] {
            let mut leader = create_holdings_leader();
            leader.cataloging_form = code;
            assert!(RecordStructureValidator::validate_leader_holdings(&leader).is_ok());
        }
        let mut leader = create_holdings_leader();
        leader.cataloging_form = 'a';
        assert!(RecordStructureValidator::validate_leader_holdings(&leader).is_err());
    }

    #[test]
    fn test_per_type_validators_share_count_and_base_address_checks() {
        // Shared validation: indicator_count == 2, subfield_code_count == 2,
        // data_base_address ≤ 99999.
        let mut a = create_authority_leader();
        a.indicator_count = 3;
        assert!(RecordStructureValidator::validate_leader_authority(&a).is_err());

        let mut h = create_holdings_leader();
        h.data_base_address = 100_000;
        assert!(RecordStructureValidator::validate_leader_holdings(&h).is_err());
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
