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

        let indicator_count = (bytes[10] as char).to_digit(10).ok_or_else(|| {
            MarcError::InvalidLeader(format!(
                "Invalid indicator count at position 10: {}",
                bytes[10] as char
            ))
        })? as u8;

        let subfield_code_count = (bytes[11] as char).to_digit(10).ok_or_else(|| {
            MarcError::InvalidLeader(format!(
                "Invalid subfield code count at position 11: {}",
                bytes[11] as char
            ))
        })? as u8;

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
}
