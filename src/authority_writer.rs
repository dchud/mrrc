//! Writing MARC Authority records to binary format.
//!
//! This module provides [`AuthorityMarcWriter`] for serializing [`AuthorityRecord`] instances
//! to ISO 2709 binary format. Authority records use the same binary format as bibliographic
//! records but with different content organization.

use crate::authority_record::AuthorityRecord;
use crate::error::{MarcError, Result};
use crate::iso2709::validate_directory_tag;
use std::io::Write;

const FIELD_TERMINATOR: u8 = 0x1E;
const SUBFIELD_DELIMITER: u8 = 0x1F;
const RECORD_TERMINATOR: u8 = 0x1D;

/// Writer for ISO 2709 binary MARC Authority records.
///
/// `AuthorityMarcWriter` serializes [`AuthorityRecord`] instances to ISO 2709 binary format.
/// Records are written one at a time to any destination implementing [`std::io::Write`].
#[derive(Debug)]
pub struct AuthorityMarcWriter<W: Write> {
    writer: W,
}

impl<W: Write> AuthorityMarcWriter<W> {
    /// Create a new Authority MARC writer.
    ///
    /// # Arguments
    ///
    /// * `writer` - Any destination implementing [`std::io::Write`]
    pub fn new(writer: W) -> Self {
        AuthorityMarcWriter { writer }
    }

    /// Write a single Authority MARC record.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or I/O fails.
    #[allow(clippy::too_many_lines)]
    pub fn write_record(&mut self, record: &AuthorityRecord) -> Result<()> {
        // Build directory and data section
        let mut directory = Vec::new();
        let mut data = Vec::new();

        // Helper to add a field to directory and data
        let add_field = |tag: &str,
                         indicators: Option<(char, char)>,
                         value: Option<&str>,
                         subfields: Option<&[crate::record::Subfield]>,
                         directory: &mut Vec<u8>,
                         data: &mut Vec<u8>|
         -> Result<()> {
            let start_pos = data.len();

            // Write field content
            match (indicators, value, subfields) {
                // Control field (no indicators or subfields)
                (None, Some(v), None) => {
                    data.extend_from_slice(v.as_bytes());
                },
                // Data field (indicators + subfields)
                (Some((ind1, ind2)), _, Some(subs)) => {
                    data.push(ind1 as u8);
                    data.push(ind2 as u8);
                    for subfield in subs {
                        data.push(SUBFIELD_DELIMITER);
                        data.push(subfield.code as u8);
                        data.extend_from_slice(subfield.value.as_bytes());
                    }
                },
                _ => {
                    return Err(MarcError::invalid_field_msg(
                        "Invalid field structure".to_string(),
                    ))
                },
            }

            data.push(FIELD_TERMINATOR);

            validate_directory_tag(tag, None, None)?;
            // Write directory entry (tag + length + start position)
            let length = data.len() - start_pos;
            directory.extend_from_slice(tag.as_bytes());
            directory.extend_from_slice(format!("{length:04}").as_bytes());
            directory.extend_from_slice(format!("{start_pos:05}").as_bytes());

            Ok(())
        };

        // Write control fields
        for (tag, values) in &record.control_fields {
            for value in values {
                add_field(tag, None, Some(value), None, &mut directory, &mut data)?;
            }
        }

        // Write all variable fields (010+) in tag order
        for (tag, fields) in &record.fields {
            for field in fields {
                add_field(
                    tag,
                    Some((field.indicator1, field.indicator2)),
                    None,
                    Some(&field.subfields),
                    &mut directory,
                    &mut data,
                )?;
            }
        }

        // Add directory terminator
        directory.push(FIELD_TERMINATOR);

        // Calculate record length and base address
        let base_address = 24 + directory.len();
        let record_length = 24 + directory.len() + data.len() + 1; // +1 for record terminator

        // ISO 2709 caps base address and record length at 5 ASCII digits
        // (00000-99999); larger values cannot be represented in the leader
        // so the writer refuses the record up-front.
        let record_control_number = record.get_control_field("001").map(str::to_string);
        if base_address > 99_999 {
            return Err(MarcError::WriterError {
                record_index: None,
                record_control_number: record_control_number.clone(),
                message: format!(
                    "Base address exceeds ISO 2709 limit ({base_address} bytes; max 99999)"
                ),
            });
        }
        if record_length > 99_999 {
            return Err(MarcError::WriterError {
                record_index: None,
                record_control_number,
                message: format!(
                    "Record length exceeds ISO 2709 limit ({record_length} bytes; max 99999)"
                ),
            });
        }

        // Update leader with calculated values
        let mut leader = record.leader.clone();
        #[allow(clippy::cast_possible_truncation)]
        {
            leader.record_length = record_length as u32;
            leader.data_base_address = base_address as u32;
        }

        // Write leader
        let leader_bytes = leader.as_bytes()?;
        self.writer.write_all(&leader_bytes)?;

        // Write directory
        self.writer.write_all(&directory)?;

        // Write data
        self.writer.write_all(&data)?;

        // Write record terminator
        self.writer.write_all(&[RECORD_TERMINATOR])?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::leader::Leader;

    #[test]
    fn test_authority_writer_creation() {
        let buffer = Vec::new();
        let _writer = AuthorityMarcWriter::new(buffer);
    }

    #[test]
    fn test_write_empty_authority_record() -> Result<()> {
        let leader = Leader {
            record_length: 0,
            record_status: 'n',
            record_type: 'z',
            bibliographic_level: '|',
            control_record_type: ' ',
            character_coding: ' ',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 0,
            encoding_level: 'n',
            cataloging_form: 'a',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        };

        let record = AuthorityRecord::new(leader);
        let buffer = Vec::new();
        let mut writer = AuthorityMarcWriter::new(buffer);

        writer.write_record(&record)?;

        let output = writer.writer;
        assert!(!output.is_empty());
        assert!(output.len() > 24); // At least leader + directory terminator + record terminator
        Ok(())
    }

    #[test]
    fn test_write_authority_with_control_field() -> Result<()> {
        let leader = Leader {
            record_length: 0,
            record_status: 'n',
            record_type: 'z',
            bibliographic_level: '|',
            control_record_type: ' ',
            character_coding: ' ',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 0,
            encoding_level: 'n',
            cataloging_form: 'a',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        };

        let mut record = AuthorityRecord::new(leader);
        record.add_control_field("001".to_string(), "n79021800".to_string());

        let buffer = Vec::new();
        let mut writer = AuthorityMarcWriter::new(buffer);
        writer.write_record(&record)?;

        let output = writer.writer;
        assert!(output.len() > 24);
        // Check for control field data
        assert!(output.windows(9).any(|w| w == b"n79021800"));
        Ok(())
    }
}
