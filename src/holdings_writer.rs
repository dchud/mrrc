//! Writing MARC Holdings records to binary format.
//!
//! This module provides [`HoldingsMarcWriter`] for serializing [`HoldingsRecord`] instances
//! to ISO 2709 binary format. Holdings records use the same binary format as bibliographic
//! and authority records but with different content organization.

use crate::error::{MarcError, Result};
use crate::holdings_record::HoldingsRecord;
use std::io::Write;

const FIELD_TERMINATOR: u8 = 0x1E;
const SUBFIELD_DELIMITER: u8 = 0x1F;
const RECORD_TERMINATOR: u8 = 0x1D;

/// Writer for ISO 2709 binary MARC Holdings records.
///
/// `HoldingsMarcWriter` serializes [`HoldingsRecord`] instances to ISO 2709 binary format.
/// Records are written one at a time to any destination implementing [`std::io::Write`].
#[derive(Debug)]
pub struct HoldingsMarcWriter<W: Write> {
    writer: W,
}

impl<W: Write> HoldingsMarcWriter<W> {
    /// Create a new Holdings MARC writer.
    ///
    /// # Arguments
    ///
    /// * `writer` - Any destination implementing [`std::io::Write`]
    pub fn new(writer: W) -> Self {
        HoldingsMarcWriter { writer }
    }

    /// Write a single Holdings MARC record.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or I/O fails.
    #[allow(clippy::too_many_lines)]
    pub fn write_record(&mut self, record: &HoldingsRecord) -> Result<()> {
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
                    return Err(MarcError::InvalidRecord(
                        "Invalid field structure".to_string(),
                    ))
                },
            }

            data.push(FIELD_TERMINATOR);

            // Write directory entry (tag + length + start position)
            let length = data.len() - start_pos;
            directory.extend_from_slice(tag.as_bytes());
            directory.extend_from_slice(format!("{length:04}").as_bytes());
            directory.extend_from_slice(format!("{start_pos:05}").as_bytes());

            Ok(())
        };

        // Write control fields
        for (tag, value) in &record.control_fields {
            add_field(tag, None, Some(value), None, &mut directory, &mut data)?;
        }

        // Write data fields (all organized in fields map)
        for fields in record.fields.values() {
            for field in fields {
                add_field(
                    &field.tag,
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

        // Calculate base address of data section
        let base_address = 24 + directory.len();
        if base_address > 99999 {
            return Err(MarcError::InvalidRecord(
                "Record too large (base address exceeds 5 digits)".to_string(),
            ));
        }

        // Calculate total record length
        let record_length = base_address + data.len() + 1; // +1 for record terminator
        if record_length > 99999 {
            return Err(MarcError::InvalidRecord(
                "Record too large (total length exceeds 5 digits)".to_string(),
            ));
        }

        // Write leader
        let mut leader = record.leader.clone();
        #[allow(clippy::cast_possible_truncation)]
        {
            leader.record_length = record_length as u32;
            leader.data_base_address = base_address as u32;
        }
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
    use crate::holdings_record::HoldingsRecord;
    use crate::leader::Leader;
    use crate::record::{Field, Subfield};

    fn create_test_leader() -> Leader {
        Leader {
            record_length: 1024,
            record_status: 'n',
            record_type: 'y',
            bibliographic_level: '|',
            control_record_type: ' ',
            character_coding: ' ',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 300,
            encoding_level: '1',
            cataloging_form: 'a',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        }
    }

    #[test]
    fn test_write_empty_holdings_record() {
        let leader = create_test_leader();
        let record = HoldingsRecord::new(leader);

        let mut buffer = Vec::new();
        let mut writer = HoldingsMarcWriter::new(&mut buffer);
        let result = writer.write_record(&record);
        assert!(result.is_ok());
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_write_holdings_with_control_fields() {
        let leader = create_test_leader();
        let mut record = HoldingsRecord::new(leader);

        record.add_control_field("001".to_string(), "ocm00098765".to_string());
        record.add_control_field(
            "008".to_string(),
            "0000001pzzzzzzzz1                       ".to_string(),
        );

        let mut buffer = Vec::new();
        let mut writer = HoldingsMarcWriter::new(&mut buffer);
        let result = writer.write_record(&record);
        assert!(result.is_ok());
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_write_holdings_with_location_field() {
        let leader = create_test_leader();
        let mut record = HoldingsRecord::new(leader);

        let location = Field {
            tag: "852".to_string(),
            indicator1: ' ',
            indicator2: '1',
            subfields: smallvec::smallvec![Subfield {
                code: 'b',
                value: "Main Library".to_string(),
            }],
        };

        record.add_location(location);

        let mut buffer = Vec::new();
        let mut writer = HoldingsMarcWriter::new(&mut buffer);
        let result = writer.write_record(&record);
        assert!(result.is_ok());
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_write_holdings_with_multiple_field_types() {
        let leader = create_test_leader();
        let mut record = HoldingsRecord::new(leader);

        record.add_control_field("001".to_string(), "ocm00098765".to_string());

        let location = Field {
            tag: "852".to_string(),
            indicator1: ' ',
            indicator2: '1',
            subfields: smallvec::smallvec![Subfield {
                code: 'b',
                value: "Main Library".to_string(),
            }],
        };
        record.add_location(location);

        let textual = Field {
            tag: "866".to_string(),
            indicator1: '4',
            indicator2: '1',
            subfields: smallvec::smallvec![Subfield {
                code: 'a',
                value: "v.1-5".to_string(),
            }],
        };
        record.add_textual_holdings_basic(textual);

        let mut buffer = Vec::new();
        let mut writer = HoldingsMarcWriter::new(&mut buffer);
        let result = writer.write_record(&record);
        assert!(result.is_ok());
        assert!(!buffer.is_empty());
    }
}
