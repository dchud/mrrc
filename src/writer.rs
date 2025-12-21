#![allow(clippy::cast_possible_truncation)]

//! Writing MARC records to binary format.
//!
//! This module provides [`MarcWriter`] for serializing [`Record`] instances
//! to ISO 2709 binary format that can be written to any destination implementing
//! [`std::io::Write`].
//!
//! # Examples
//!
//! Writing records to a file:
//!
//! ```ignore
//! use mrrc::{MarcWriter, Record, Field, Leader};
//! use std::fs::File;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut file = File::create("output.mrc")?;
//! let mut writer = MarcWriter::new(&mut file);
//!
//! let mut record = Record::new(Leader::default());
//! let mut field = Field::new("245".to_string(), '1', '0');
//! field.add_subfield('a', "Title".to_string());
//! record.add_field(field);
//!
//! writer.write_record(&record)?;
//! # Ok(())
//! # }
//! ```
//!
//! Writing to a buffer:
//!
//! ```ignore
//! use mrrc::{MarcWriter, Record, Leader};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut buffer = Vec::new();
//! {
//!     let mut writer = MarcWriter::new(&mut buffer);
//!     let record = Record::new(Leader::default());
//!     writer.write_record(&record)?;
//! }
//! # Ok(())
//! # }
//! ```

use crate::error::Result;
use crate::record::Record;
use std::io::Write;

const FIELD_TERMINATOR: u8 = 0x1E;
const SUBFIELD_DELIMITER: u8 = 0x1F;
const RECORD_TERMINATOR: u8 = 0x1D;

/// Writer for ISO 2709 binary MARC format.
///
/// `MarcWriter` serializes [`Record`] instances to ISO 2709 binary format.
/// Records are written one at a time to any destination implementing [`std::io::Write`].
///
/// # Examples
///
/// ```ignore
/// use mrrc::{MarcWriter, Record, Leader};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut buffer = Vec::new();
/// {
///     let mut writer = MarcWriter::new(&mut buffer);
///     let record = Record::new(Leader::default());
///     writer.write_record(&record)?;
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct MarcWriter<W: Write> {
    writer: W,
}

impl<W: Write> MarcWriter<W> {
    /// Create a new MARC writer.
    ///
    /// # Arguments
    ///
    /// * `writer` - Any destination implementing [`std::io::Write`]
    ///
    /// # Examples
    ///
    /// ```
    /// use mrrc::MarcWriter;
    /// let buffer = Vec::new();
    /// let writer = MarcWriter::new(buffer);
    /// ```
    pub fn new(writer: W) -> Self {
        MarcWriter { writer }
    }

    /// Write a single MARC record.
    ///
    /// Serializes the record to ISO 2709 binary format and writes it to the
    /// underlying writer.
    ///
    /// # Arguments
    ///
    /// * `record` - The record to write
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mrrc::{MarcWriter, Record, Field, Leader};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut buffer = Vec::new();
    /// {
    ///     let mut writer = MarcWriter::new(&mut buffer);
    ///     let mut record = Record::new(Leader::default());
    ///     let mut field = Field::new("245".to_string(), '1', '0');
    ///     field.add_subfield('a', "Title".to_string());
    ///     record.add_field(field);
    ///     writer.write_record(&record)?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The record structure is invalid
    /// - An I/O error occurs during writing
    pub fn write_record(&mut self, record: &Record) -> Result<()> {
        // Build the data area first
        let mut data_area = Vec::new();
        let mut directory = Vec::new();
        let mut current_position = 0;

        // Write control fields first (001-009)
        for (tag, value) in &record.control_fields {
            if tag.as_str() < "010" {
                let field_data = value.as_bytes();
                let field_length = field_data.len() + 1; // +1 for terminator

                // Add directory entry
                directory.extend_from_slice(tag.as_bytes());
                directory.extend_from_slice(format!("{field_length:04}").as_bytes());
                directory.extend_from_slice(format!("{current_position:05}").as_bytes());

                // Add data
                data_area.extend_from_slice(field_data);
                data_area.push(FIELD_TERMINATOR);
                current_position += field_length;
            }
        }

        // Write data fields (010+)
        for (tag, fields) in &record.data_fields {
            for field in fields {
                let mut field_data = Vec::new();
                field_data.push(field.indicator1 as u8);
                field_data.push(field.indicator2 as u8);

                for subfield in &field.subfields {
                    field_data.push(SUBFIELD_DELIMITER);
                    field_data.push(subfield.code as u8);
                    field_data.extend_from_slice(subfield.value.as_bytes());
                }

                field_data.push(FIELD_TERMINATOR);
                let field_length = field_data.len();

                // Add directory entry
                directory.extend_from_slice(tag.as_bytes());
                directory.extend_from_slice(format!("{field_length:04}").as_bytes());
                directory.extend_from_slice(format!("{current_position:05}").as_bytes());

                // Add data
                data_area.extend_from_slice(&field_data);
                current_position += field_length;
            }
        }

        // Finalize directory
        directory.push(FIELD_TERMINATOR);

        // Calculate addresses and lengths
        let base_address = 24 + directory.len();
        let record_length = base_address + data_area.len() + 1; // +1 for record terminator

        // Update leader with correct values
        let mut leader = record.leader.clone();
        leader.record_length = record_length as u32;
        leader.data_base_address = base_address as u32;

        // Write leader
        let leader_bytes = leader.as_bytes()?;
        self.writer.write_all(&leader_bytes)?;

        // Write directory
        self.writer.write_all(&directory)?;

        // Write data area
        self.writer.write_all(&data_area)?;

        // Write record terminator
        self.writer.write_all(&[RECORD_TERMINATOR])?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::leader::Leader;
    use crate::record::Field;
    use std::io::Cursor;

    fn make_test_leader() -> Leader {
        Leader {
            record_length: 0, // Will be updated by writer
            record_status: 'n',
            record_type: 'a',
            bibliographic_level: 'm',
            control_record_type: ' ',
            character_coding: 'a',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 0, // Will be updated by writer
            encoding_level: ' ',
            cataloging_form: ' ',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        }
    }

    #[test]
    fn test_write_simple_record() {
        let mut record = Record::new(make_test_leader());

        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test title".to_string());
        record.add_field(field);

        let mut buffer = Vec::new();
        let mut writer = MarcWriter::new(&mut buffer);
        writer.write_record(&record).unwrap();

        // Verify basic structure
        assert!(buffer.len() > 24); // At least leader + data
                                    // Record length: 24 (leader) + 13 (directory: 245 + 0015 + 00000 + terminator) + 15 (field data) + 1 (record term) = 53
        assert_eq!(&buffer[0..5], b"00053"); // Record length
        assert_eq!(buffer[24], b'2'); // Start of directory (tag '245')
    }

    #[test]
    fn test_write_and_read_roundtrip() {
        use crate::reader::MarcReader;

        let mut record = Record::new(make_test_leader());

        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test title".to_string());
        field.add_subfield('c', "Author".to_string());
        record.add_field(field);

        record.add_control_field("001".to_string(), "12345".to_string());

        // Write to buffer
        let mut buffer = Vec::new();
        {
            let mut writer = MarcWriter::new(&mut buffer);
            writer.write_record(&record).unwrap();
        }

        // Read back from buffer
        let cursor = Cursor::new(buffer);
        let mut reader = MarcReader::new(cursor);
        let read_record = reader.read_record().unwrap().unwrap();

        // Verify fields
        assert_eq!(read_record.get_control_field("001"), Some("12345"));

        let fields = read_record.get_fields("245").unwrap();
        assert_eq!(fields[0].indicator1, '1');
        assert_eq!(fields[0].indicator2, '0');
        assert_eq!(fields[0].get_subfield('a'), Some("Test title"));
        assert_eq!(fields[0].get_subfield('c'), Some("Author"));
    }

    #[test]
    fn test_write_multiple_subfields() {
        use crate::reader::MarcReader;

        let mut record = Record::new(make_test_leader());

        let mut field = Field::new("650".to_string(), ' ', '0');
        field.add_subfield('a', "Subject 1".to_string());
        field.add_subfield('v', "subdivision".to_string());
        record.add_field(field);

        let mut buffer = Vec::new();
        {
            let mut writer = MarcWriter::new(&mut buffer);
            writer.write_record(&record).unwrap();
        }

        let cursor = Cursor::new(buffer);
        let mut reader = MarcReader::new(cursor);
        let read_record = reader.read_record().unwrap().unwrap();

        let fields = read_record.get_fields("650").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Subject 1"));
        assert_eq!(fields[0].get_subfield('v'), Some("subdivision"));
    }

    #[test]
    fn test_write_multiple_fields_same_tag() {
        use crate::reader::MarcReader;

        let mut record = Record::new(make_test_leader());

        for i in 1..=3 {
            let mut field = Field::new("650".to_string(), ' ', '0');
            field.add_subfield('a', format!("Subject {i}"));
            record.add_field(field);
        }

        let mut buffer = Vec::new();
        {
            let mut writer = MarcWriter::new(&mut buffer);
            writer.write_record(&record).unwrap();
        }

        let cursor = Cursor::new(buffer);
        let mut reader = MarcReader::new(cursor);
        let read_record = reader.read_record().unwrap().unwrap();

        let fields = read_record.get_fields("650").unwrap();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].get_subfield('a'), Some("Subject 1"));
        assert_eq!(fields[1].get_subfield('a'), Some("Subject 2"));
        assert_eq!(fields[2].get_subfield('a'), Some("Subject 3"));
    }
}
