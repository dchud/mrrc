//! MARC record leader parsing and manipulation.
//!
//! The MARC leader is a 24-byte fixed-length field at the start of every MARC record.
//! It contains metadata describing the record's structure, content type, and encoding.
//!
//! # Structure
//!
//! - Positions 0-4: Record length (5 digits)
//! - Position 5: Record status
//! - Position 6: Record type (a = language material, c = music, etc.)
//! - Position 7: Bibliographic level (m = monograph, s = serial, etc.)
//! - Position 8: Control record type
//! - Position 9: Character coding (space = MARC-8, a = UTF-8)
//! - Position 10: Indicator count (usually 2)
//! - Position 11: Subfield code count (usually 2)
//! - Positions 12-16: Base address of data (5 digits)
//! - Positions 17-19: Encoding level, cataloging form, multipart level
//! - Positions 20-23: Reserved (usually "4500")

use crate::error::{MarcError, Result};
use serde::{Deserialize, Serialize};

/// MARC Leader - 24 bytes at the start of every MARC record.
///
/// Contains metadata about the record structure and content.
/// All MARC records must begin with exactly 24 bytes of leader information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Leader {
    /// Record length (5 digits) - positions 0-4
    pub record_length: u32,
    /// Record status (1 char) - position 5
    pub record_status: char,
    /// Type of record (1 char) - position 6
    pub record_type: char,
    /// Bibliographic level (1 char) - position 7
    pub bibliographic_level: char,
    /// Type of control record (1 char) - position 8
    pub control_record_type: char,
    /// Character coding scheme (1 char) - position 9
    pub character_coding: char,
    /// Indicator count (1 digit) - position 10 (usually 2)
    pub indicator_count: u8,
    /// Subfield code count (1 digit) - position 11 (usually 2)
    pub subfield_code_count: u8,
    /// Base address of data (5 digits) - positions 12-16
    pub data_base_address: u32,
    /// Encoding level (1 char) - position 17
    pub encoding_level: char,
    /// Cataloging form (1 char) - position 18
    pub cataloging_form: char,
    /// Multipart resource record level (1 char) - position 19
    pub multipart_level: char,
    /// Reserved (4 chars) - positions 20-23
    pub reserved: String,
}

impl Leader {
    /// Get valid values for a specific leader position (MARC 21 spec reference).
    ///
    /// # Arguments
    ///
    /// * `position` - The leader position (5-19)
    ///
    /// # Returns
    ///
    /// A vector of tuples containing (value, description) for valid values at that position.
    /// Returns an empty vector for unknown positions.
    ///
    /// # Example
    ///
    /// ```
    /// use mrrc::Leader;
    /// let valid_values = Leader::valid_values_at_position(5);
    /// // Returns: [("a", "increase in encoding level"), ("c", "corrected"), ...]
    /// ```
    #[must_use]
    pub fn valid_values_at_position(position: usize) -> Option<Vec<(&'static str, &'static str)>> {
        match position {
            5 => Some(vec![
                ("a", "Increase in encoding level"),
                ("c", "Corrected or revised"),
                ("d", "Deleted"),
                ("n", "New"),
                ("p", "Increase in encoding level from prepublication"),
            ]),
            6 => Some(vec![
                ("a", "Language material"),
                ("c", "Notated music"),
                ("d", "Manuscript notated music"),
                ("e", "Cartographic material"),
                ("f", "Manuscript cartographic material"),
                ("g", "Projected medium"),
                ("i", "Nonmusical sound recording"),
                ("j", "Musical sound recording"),
                ("k", "Two-dimensional nonprojectable graphic"),
                ("m", "Computer file"),
                ("o", "Kit"),
                ("p", "Mixed materials"),
                (
                    "r",
                    "Three-dimensional artifact or naturally occurring object",
                ),
                ("t", "Manuscript language material"),
            ]),
            7 => Some(vec![
                ("a", "Monographic component part"),
                ("b", "Serial component part"),
                ("c", "Collection"),
                ("d", "Subunit"),
                ("i", "Integrating resource"),
                ("m", "Monograph/Item"),
                ("s", "Serial"),
            ]),
            8 => Some(vec![("#", "No specified type"), ("a", "Archival")]),
            9 => Some(vec![(" ", "MARC-8"), ("a", "UCS/Unicode")]),
            17 => Some(vec![
                (" ", "Full level"),
                ("1", "Full level, material not examined"),
                ("2", "Less-than-full level, material not examined"),
                ("3", "Abbreviated level"),
                ("4", "Core level"),
                ("5", "Partial (preliminary) level"),
                ("7", "Minimal level"),
                ("8", "Prepublication level"),
                ("u", "Unknown"),
                ("z", "Not applicable"),
            ]),
            18 => Some(vec![
                (" ", "Non-ISBD"),
                ("a", "AACR 2"),
                ("c", "ISBD punctuation omitted"),
                ("i", "ISBD punctuation included"),
                ("n", "Non-ISBD punctuation omitted"),
                ("u", "Unknown"),
            ]),
            19 => Some(vec![
                (" ", "Not specified or not applicable"),
                ("a", "Set"),
                ("b", "Part with independent title"),
                ("c", "Part with dependent title"),
            ]),
            _ => None,
        }
    }

    /// Get description for a specific value at a leader position.
    ///
    /// # Arguments
    ///
    /// * `position` - The leader position (5-19)
    /// * `value` - The character value to look up
    ///
    /// # Returns
    ///
    /// The description if found, or None if the value is invalid for the position.
    ///
    /// # Example
    ///
    /// ```
    /// use mrrc::Leader;
    /// let desc = Leader::describe_value(5, "a");
    /// // Returns: Some("increase in encoding level")
    /// ```
    #[must_use]
    pub fn describe_value(position: usize, value: &str) -> Option<&'static str> {
        Self::valid_values_at_position(position).and_then(|values| {
            values
                .into_iter()
                .find(|(v, _)| *v == value)
                .map(|(_, desc)| desc)
        })
    }

    /// Check if a value is valid for a specific leader position.
    ///
    /// If the position has no defined valid values, any value is considered valid.
    ///
    /// # Arguments
    ///
    /// * `position` - The leader position (5-19)
    /// * `value` - The character value to validate
    ///
    /// # Returns
    ///
    /// True if the value is valid for the position, false otherwise.
    #[must_use]
    pub fn is_valid_value(position: usize, value: &str) -> bool {
        match Self::valid_values_at_position(position) {
            Some(values) => values.iter().any(|(v, _)| *v == value),
            None => true, // Positions without defined values accept any value
        }
    }

    /// Parse a leader from 24 bytes
    ///
    /// # Errors
    ///
    /// Returns an error if the bytes are invalid or too short.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 24 {
            return Err(MarcError::InvalidLeader(format!(
                "Leader must be at least 24 bytes, got {}",
                bytes.len()
            )));
        }

        let record_length = parse_digits(&bytes[0..5])?;
        let record_status = bytes[5] as char;
        let record_type = bytes[6] as char;
        let bibliographic_level = bytes[7] as char;
        let control_record_type = bytes[8] as char;
        let character_coding = bytes[9] as char;

        let indicator_count = u8::try_from((bytes[10] as char).to_digit(10).ok_or_else(|| {
            MarcError::InvalidLeader(format!(
                "Invalid indicator count at position 10: {}",
                bytes[10] as char
            ))
        })?)
        .map_err(|_| MarcError::InvalidLeader("Indicator count exceeds valid range".to_string()))?;

        let subfield_code_count =
            u8::try_from((bytes[11] as char).to_digit(10).ok_or_else(|| {
                MarcError::InvalidLeader(format!(
                    "Invalid subfield code count at position 11: {}",
                    bytes[11] as char
                ))
            })?)
            .map_err(|_| {
                MarcError::InvalidLeader("Subfield code count exceeds valid range".to_string())
            })?;

        let data_base_address = parse_digits(&bytes[12..17])?;
        let encoding_level = bytes[17] as char;
        let cataloging_form = bytes[18] as char;
        let multipart_level = bytes[19] as char;
        let reserved = String::from_utf8_lossy(&bytes[20..24]).to_string();

        Ok(Leader {
            record_length,
            record_status,
            record_type,
            bibliographic_level,
            control_record_type,
            character_coding,
            indicator_count,
            subfield_code_count,
            data_base_address,
            encoding_level,
            cataloging_form,
            multipart_level,
            reserved,
        })
    }

    /// Validate that the leader is suitable for binary record reading.
    ///
    /// Checks that `record_length` and `data_base_address` are at least 24,
    /// which is required before performing arithmetic on these fields during
    /// binary ISO 2709 parsing.
    ///
    /// # Errors
    ///
    /// Returns an error if `record_length` or `data_base_address` is less than 24.
    pub fn validate_for_reading(&self) -> Result<()> {
        if self.record_length < 24 {
            return Err(MarcError::InvalidLeader(format!(
                "Record length must be at least 24, got {}",
                self.record_length
            )));
        }
        if self.data_base_address < 24 {
            return Err(MarcError::InvalidLeader(format!(
                "Base address of data must be at least 24, got {}",
                self.data_base_address
            )));
        }
        Ok(())
    }

    /// Serialize leader to 24-byte array
    ///
    /// # Errors
    ///
    /// Returns an error if the leader values are invalid for serialization.
    pub fn as_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(24);

        // Record length (5 digits, zero-padded)
        bytes.extend_from_slice(format!("{:05}", self.record_length).as_bytes());
        bytes.push(self.record_status as u8);
        bytes.push(self.record_type as u8);
        bytes.push(self.bibliographic_level as u8);
        bytes.push(self.control_record_type as u8);
        bytes.push(self.character_coding as u8);
        bytes.push(b'0' + self.indicator_count);
        bytes.push(b'0' + self.subfield_code_count);

        // Base address of data (5 digits, zero-padded)
        bytes.extend_from_slice(format!("{:05}", self.data_base_address).as_bytes());
        bytes.push(self.encoding_level as u8);
        bytes.push(self.cataloging_form as u8);
        bytes.push(self.multipart_level as u8);

        // Reserved (4 bytes)
        let reserved_bytes = self.reserved.as_bytes();
        if reserved_bytes.len() != 4 {
            return Err(MarcError::InvalidLeader(format!(
                "Reserved field must be 4 characters, got {}",
                reserved_bytes.len()
            )));
        }
        bytes.extend_from_slice(reserved_bytes);

        Ok(bytes)
    }
}

/// Parse 5-digit ASCII number from bytes
fn parse_digits(bytes: &[u8]) -> Result<u32> {
    if bytes.len() != 5 {
        return Err(MarcError::InvalidLeader(format!(
            "Expected 5-digit field, got {} bytes",
            bytes.len()
        )));
    }

    let s = String::from_utf8_lossy(bytes);
    s.parse::<u32>()
        .map_err(|_| MarcError::InvalidLeader(format!("Invalid numeric field: '{s}'")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leader_from_bytes() {
        let bytes = b"01234567890120123456DUMMY";
        let leader = Leader::from_bytes(bytes).unwrap();

        assert_eq!(leader.record_length, 1234);
        assert_eq!(leader.record_status, '5');
        assert_eq!(leader.record_type, '6');
        assert_eq!(leader.bibliographic_level, '7');
        assert_eq!(leader.control_record_type, '8');
        assert_eq!(leader.character_coding, '9');
        assert_eq!(leader.indicator_count, 0);
        assert_eq!(leader.subfield_code_count, 1);
        assert_eq!(leader.data_base_address, 20123);
        assert_eq!(leader.encoding_level, '4');
        assert_eq!(leader.cataloging_form, '5');
        assert_eq!(leader.multipart_level, '6');
        assert_eq!(leader.reserved, "DUMM");
    }

    #[test]
    fn test_leader_roundtrip() {
        let original = Leader {
            record_length: 2048,
            record_status: 'a',
            record_type: 'a',
            bibliographic_level: 'm',
            control_record_type: 'a',
            character_coding: ' ',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 256,
            encoding_level: ' ',
            cataloging_form: ' ',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        };

        let bytes = original.as_bytes().unwrap();
        let parsed = Leader::from_bytes(&bytes).unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_leader_too_short() {
        let bytes = b"0123456789012";
        let result = Leader::from_bytes(bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_leader_invalid_indicator_count() {
        let bytes = b"01234567890X20123456DUMMY";
        let result = Leader::from_bytes(bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_values_position_5() {
        let values = Leader::valid_values_at_position(5);
        assert!(values.is_some());

        let values = values.unwrap();
        let value_codes: Vec<&str> = values.iter().map(|(code, _)| *code).collect();
        assert!(value_codes.contains(&"a"));
        assert!(value_codes.contains(&"n"));
    }

    #[test]
    fn test_valid_values_position_6() {
        let values = Leader::valid_values_at_position(6);
        assert!(values.is_some());

        let values = values.unwrap();
        let value_codes: Vec<&str> = values.iter().map(|(code, _)| *code).collect();
        assert!(value_codes.contains(&"a"));
        assert!(value_codes.contains(&"t"));
        assert!(value_codes.contains(&"m"));
    }

    #[test]
    fn test_valid_values_position_7() {
        let values = Leader::valid_values_at_position(7);
        assert!(values.is_some());

        let values = values.unwrap();
        let value_codes: Vec<&str> = values.iter().map(|(code, _)| *code).collect();
        assert!(value_codes.contains(&"m"));
        assert!(value_codes.contains(&"s"));
    }

    #[test]
    fn test_valid_values_invalid_position() {
        let values = Leader::valid_values_at_position(0);
        assert!(values.is_none());

        let values = Leader::valid_values_at_position(99);
        assert!(values.is_none());
    }

    #[test]
    fn test_describe_value_position_5() {
        let desc = Leader::describe_value(5, "a");
        assert_eq!(desc, Some("Increase in encoding level"));

        let desc = Leader::describe_value(5, "n");
        assert_eq!(desc, Some("New"));
    }

    #[test]
    fn test_describe_value_position_6() {
        let desc = Leader::describe_value(6, "a");
        assert_eq!(desc, Some("Language material"));

        let desc = Leader::describe_value(6, "t");
        assert_eq!(desc, Some("Manuscript language material"));
    }

    #[test]
    fn test_describe_value_invalid_value() {
        let desc = Leader::describe_value(5, "z");
        assert_eq!(desc, None);

        let desc = Leader::describe_value(99, "a");
        assert_eq!(desc, None);
    }

    #[test]
    fn test_describe_value_all_valid_position_5() {
        let values = Leader::valid_values_at_position(5).unwrap();
        for (code, expected_desc) in values {
            let desc = Leader::describe_value(5, code);
            assert_eq!(desc, Some(expected_desc));
        }
    }

    #[test]
    fn test_describe_value_all_valid_position_6() {
        let values = Leader::valid_values_at_position(6).unwrap();
        for (code, expected_desc) in values {
            let desc = Leader::describe_value(6, code);
            assert_eq!(desc, Some(expected_desc));
        }
    }

    #[test]
    fn test_describe_value_all_valid_position_7() {
        let values = Leader::valid_values_at_position(7).unwrap();
        for (code, expected_desc) in values {
            let desc = Leader::describe_value(7, code);
            assert_eq!(desc, Some(expected_desc));
        }
    }

    #[test]
    fn test_validate_for_reading_rejects_small_record_length() {
        // Leader with record_length=00010 (< 24)
        let bytes = b"00010nam a2200025 i 4500";
        let leader = Leader::from_bytes(bytes).unwrap();
        let result = leader.validate_for_reading();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Record length must be at least 24"),
            "got: {err}"
        );
    }

    #[test]
    fn test_validate_for_reading_rejects_small_base_address() {
        // Leader with valid record_length=00050 but base_address=00010 (< 24)
        let bytes = b"00050nam a2200010 i 4500";
        let leader = Leader::from_bytes(bytes).unwrap();
        let result = leader.validate_for_reading();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Base address of data must be at least 24"),
            "got: {err}"
        );
    }
}
