//! `MessagePack` binary format support for MARC records.
//!
//! This module provides serialization and deserialization of MARC records
//! using `MessagePack`, a compact binary serialization format with broad
//! language support (50+ languages).
//!
//! ## Features
//!
//! - **Round-trip fidelity**: Preserves exact field ordering, subfield ordering,
//!   indicators, and all textual content including whitespace
//! - **Compact serialization**: ~25% smaller than equivalent JSON
//! - **Schema-less**: No schema definition required, uses serde derive
//! - **Streaming support**: Length-delimited encoding for streaming read/write
//!
//! ## Example
//!
//! ```ignore
//! use mrrc::messagepack::{MessagePackReader, MessagePackWriter};
//! use mrrc::formats::{FormatReader, FormatWriter};
//! use std::io::Cursor;
//!
//! // Write records
//! let mut buffer = Vec::new();
//! let mut writer = MessagePackWriter::new(&mut buffer);
//! writer.write_record(&record)?;
//! writer.finish()?;
//!
//! // Read records
//! let cursor = Cursor::new(buffer);
//! let mut reader = MessagePackReader::new(cursor);
//! while let Some(record) = reader.read_record()? {
//!     // process record
//! }
//! ```

use crate::formats::{FormatReader, FormatWriter};
use crate::{Field as MarcField, Leader, MarcError, Record, Result, Subfield};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

// ============================================================================
// Serialization Schema
// ============================================================================

/// `MessagePack` representation of a MARC record.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct MarcRecordMsgpack {
    /// 24-character leader string
    leader: String,
    /// Control fields (tags 001-009)
    control_fields: Vec<ControlFieldMsgpack>,
    /// Variable fields (tags 010+)
    fields: Vec<FieldMsgpack>,
}

/// `MessagePack` representation of a control field.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct ControlFieldMsgpack {
    tag: String,
    value: String,
}

/// `MessagePack` representation of a variable field.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct FieldMsgpack {
    tag: String,
    ind1: char,
    ind2: char,
    subfields: Vec<SubfieldMsgpack>,
}

/// `MessagePack` representation of a subfield.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct SubfieldMsgpack {
    code: char,
    value: String,
}

// ============================================================================
// Conversion Functions
// ============================================================================

/// Convert a MARC Record to `MessagePack` representation.
fn record_to_msgpack(record: &Record) -> Result<MarcRecordMsgpack> {
    let leader_bytes = record.leader.as_bytes()?;
    let leader = String::from_utf8(leader_bytes)
        .map_err(|_| MarcError::InvalidLeader("Leader contains invalid UTF-8".to_string()))?;

    let control_fields: Vec<ControlFieldMsgpack> = record
        .control_fields_iter()
        .map(|(tag, value)| ControlFieldMsgpack {
            tag: tag.to_string(),
            value: value.to_string(),
        })
        .collect();

    let fields: Vec<FieldMsgpack> = record
        .fields()
        .map(|field| FieldMsgpack {
            tag: field.tag.clone(),
            ind1: field.indicator1,
            ind2: field.indicator2,
            subfields: field
                .subfields
                .iter()
                .map(|sf| SubfieldMsgpack {
                    code: sf.code,
                    value: sf.value.clone(),
                })
                .collect(),
        })
        .collect();

    Ok(MarcRecordMsgpack {
        leader,
        control_fields,
        fields,
    })
}

/// Convert `MessagePack` representation to a MARC Record.
fn msgpack_to_record(msgpack: MarcRecordMsgpack) -> Result<Record> {
    let leader = Leader::from_bytes(msgpack.leader.as_bytes())?;
    let mut record = Record::new(leader);

    // Add control fields
    for cf in msgpack.control_fields {
        record.add_control_field(cf.tag, cf.value);
    }

    // Add variable fields
    for field_msg in msgpack.fields {
        let subfields: smallvec::SmallVec<[Subfield; 4]> = field_msg
            .subfields
            .iter()
            .map(|sf| Subfield {
                code: sf.code,
                value: sf.value.clone(),
            })
            .collect();

        let field = MarcField {
            tag: field_msg.tag,
            indicator1: field_msg.ind1,
            indicator2: field_msg.ind2,
            subfields,
        };

        record.add_field(field);
    }

    Ok(record)
}

// ============================================================================
// MessagePackWriter
// ============================================================================

/// Writer for streaming MARC records to `MessagePack` format.
///
/// `MessagePackWriter` implements the [`FormatWriter`] trait, allowing it to be used
/// interchangeably with other format writers. Records are written using length-delimited
/// encoding (4-byte big-endian length prefix) for streaming support.
#[derive(Debug)]
pub struct MessagePackWriter<W: Write> {
    writer: W,
    records_written: usize,
    finished: bool,
}

impl<W: Write> MessagePackWriter<W> {
    /// Create a new `MessagePack` writer.
    ///
    /// # Arguments
    ///
    /// * `writer` - Any destination implementing [`std::io::Write`]
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            records_written: 0,
            finished: false,
        }
    }

    /// Write a single MARC record to `MessagePack` format.
    ///
    /// Records are written with length-delimited encoding for streaming support.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or I/O fails.
    pub fn write_record(&mut self, record: &Record) -> Result<()> {
        if self.finished {
            return Err(MarcError::InvalidRecord(
                "Cannot write to a finished writer".to_string(),
            ));
        }

        let msgpack = record_to_msgpack(record)?;
        let bytes = rmp_serde::to_vec(&msgpack)
            .map_err(|e| MarcError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Write length prefix (4-byte big-endian)
        let len = u32::try_from(bytes.len()).map_err(|_| {
            MarcError::InvalidRecord("Record too large for MessagePack".to_string())
        })?;
        self.writer.write_all(&len.to_be_bytes())?;
        self.writer.write_all(&bytes)?;

        self.records_written += 1;
        Ok(())
    }

    /// Flush the writer and mark it as finished.
    ///
    /// # Errors
    ///
    /// Returns an error if flushing fails.
    pub fn finish(&mut self) -> Result<()> {
        self.writer.flush()?;
        self.finished = true;
        Ok(())
    }

    /// Returns the number of records written so far.
    #[must_use]
    pub fn records_written(&self) -> usize {
        self.records_written
    }
}

impl<W: Write + std::fmt::Debug> FormatWriter for MessagePackWriter<W> {
    fn write_record(&mut self, record: &Record) -> Result<()> {
        MessagePackWriter::write_record(self, record)
    }

    fn finish(&mut self) -> Result<()> {
        MessagePackWriter::finish(self)
    }

    fn records_written(&self) -> Option<usize> {
        Some(self.records_written)
    }
}

// ============================================================================
// MessagePackReader
// ============================================================================

/// Reader for streaming MARC records from `MessagePack` format.
///
/// `MessagePackReader` implements the [`FormatReader`] trait, allowing it to be used
/// interchangeably with other format readers. Records are read using length-delimited
/// encoding (4-byte big-endian length prefix).
#[derive(Debug)]
pub struct MessagePackReader<R: Read> {
    reader: R,
    records_read: usize,
}

impl<R: Read> MessagePackReader<R> {
    /// Create a new `MessagePack` reader.
    ///
    /// # Arguments
    ///
    /// * `reader` - Any source implementing [`std::io::Read`]
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            records_read: 0,
        }
    }

    /// Read a single MARC record from the `MessagePack` stream.
    ///
    /// Returns `Ok(Some(record))` if a record was read, `Ok(None)` at EOF,
    /// or `Err` if parsing fails.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is malformed or I/O fails.
    pub fn read_record(&mut self) -> Result<Option<Record>> {
        // Read length prefix (4-byte big-endian)
        let mut len_bytes = [0u8; 4];
        match self.reader.read_exact(&mut len_bytes) {
            Ok(()) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(MarcError::IoError(e)),
        }

        let len = u32::from_be_bytes(len_bytes) as usize;
        if len == 0 {
            return Ok(None);
        }

        // Read the message bytes
        let mut buffer = vec![0u8; len];
        match self.reader.read_exact(&mut buffer) {
            Ok(()) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Err(MarcError::TruncatedRecord(
                    "Unexpected EOF while reading MessagePack record".to_string(),
                ));
            },
            Err(e) => return Err(MarcError::IoError(e)),
        }

        // Deserialize
        let msgpack: MarcRecordMsgpack = rmp_serde::from_slice(&buffer)
            .map_err(|e| MarcError::ParseError(format!("MessagePack decoding failed: {e}")))?;

        let record = msgpack_to_record(msgpack)?;
        self.records_read += 1;
        Ok(Some(record))
    }

    /// Returns the number of records read so far.
    #[must_use]
    pub fn records_read(&self) -> usize {
        self.records_read
    }
}

impl<R: Read + std::fmt::Debug> FormatReader for MessagePackReader<R> {
    fn read_record(&mut self) -> Result<Option<Record>> {
        MessagePackReader::read_record(self)
    }

    fn records_read(&self) -> Option<usize> {
        Some(self.records_read)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn make_test_leader() -> Leader {
        Leader::from_bytes(b"00345nam a2200133 a 4500").unwrap()
    }

    #[test]
    fn test_roundtrip_simple_record() -> Result<()> {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test001".to_string());

        let mut field = MarcField::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test Title".to_string());
        field.add_subfield('c', "Author Name".to_string());
        record.add_field(field);

        // Write
        let mut buffer = Vec::new();
        {
            let mut writer = MessagePackWriter::new(&mut buffer);
            writer.write_record(&record)?;
            writer.finish()?;
        }

        // Read
        let cursor = Cursor::new(buffer);
        let mut reader = MessagePackReader::new(cursor);
        let restored = reader.read_record()?.expect("Should have one record");

        // Verify
        assert_eq!(restored.get_control_field("001"), Some("test001"));
        let field_245 = restored.get_field("245").expect("Should have 245");
        assert_eq!(field_245.get_subfield('a'), Some("Test Title"));
        assert_eq!(field_245.get_subfield('c'), Some("Author Name"));

        Ok(())
    }

    #[test]
    fn test_streaming_multiple_records() -> Result<()> {
        let records: Vec<Record> = (0..5)
            .map(|i| {
                let mut record = Record::new(make_test_leader());
                record.add_control_field("001".to_string(), format!("rec{i:03}"));
                let mut field = MarcField::new("245".to_string(), '1', '0');
                field.add_subfield('a', format!("Title {i}"));
                record.add_field(field);
                record
            })
            .collect();

        // Write all
        let mut buffer = Vec::new();
        {
            let mut writer = MessagePackWriter::new(&mut buffer);
            for record in &records {
                writer.write_record(record)?;
            }
            writer.finish()?;
        }

        // Read all
        let cursor = Cursor::new(buffer);
        let mut reader = MessagePackReader::new(cursor);
        for i in 0..5 {
            let record = reader.read_record()?.expect("Should have record");
            assert_eq!(
                record.get_control_field("001"),
                Some(format!("rec{i:03}").as_str())
            );
        }
        assert!(reader.read_record()?.is_none());
        assert_eq!(reader.records_read(), 5);

        Ok(())
    }

    #[test]
    fn test_preserves_field_order() -> Result<()> {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test001".to_string());
        record.add_control_field("005".to_string(), "20260120".to_string());

        for i in 1..=3 {
            let mut field = MarcField::new("650".to_string(), ' ', '0');
            field.add_subfield('a', format!("Subject {i}"));
            record.add_field(field);
        }

        let mut buffer = Vec::new();
        {
            let mut writer = MessagePackWriter::new(&mut buffer);
            writer.write_record(&record)?;
            writer.finish()?;
        }

        let cursor = Cursor::new(buffer);
        let mut reader = MessagePackReader::new(cursor);
        let restored = reader.read_record()?.unwrap();

        let subjects: Vec<_> = restored.fields_by_tag("650").collect();
        assert_eq!(subjects.len(), 3);
        assert_eq!(subjects[0].get_subfield('a'), Some("Subject 1"));
        assert_eq!(subjects[1].get_subfield('a'), Some("Subject 2"));
        assert_eq!(subjects[2].get_subfield('a'), Some("Subject 3"));

        Ok(())
    }

    #[test]
    fn test_preserves_subfield_order() -> Result<()> {
        let mut record = Record::new(make_test_leader());
        let mut field = MarcField::new("245".to_string(), '1', '0');
        // Non-alphabetical order
        field.add_subfield('c', "Author".to_string());
        field.add_subfield('a', "Title".to_string());
        field.add_subfield('b', "Subtitle".to_string());
        record.add_field(field);

        let mut buffer = Vec::new();
        {
            let mut writer = MessagePackWriter::new(&mut buffer);
            writer.write_record(&record)?;
            writer.finish()?;
        }

        let cursor = Cursor::new(buffer);
        let mut reader = MessagePackReader::new(cursor);
        let restored = reader.read_record()?.unwrap();

        let field_245 = restored.get_field("245").unwrap();
        let codes: Vec<char> = field_245.subfields.iter().map(|s| s.code).collect();
        assert_eq!(codes, vec!['c', 'a', 'b']);

        Ok(())
    }

    #[test]
    fn test_preserves_utf8() -> Result<()> {
        let mut record = Record::new(make_test_leader());
        let mut field = MarcField::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test 测试 тест العربية 日本語".to_string());
        record.add_field(field);

        let mut buffer = Vec::new();
        {
            let mut writer = MessagePackWriter::new(&mut buffer);
            writer.write_record(&record)?;
            writer.finish()?;
        }

        let cursor = Cursor::new(buffer);
        let mut reader = MessagePackReader::new(cursor);
        let restored = reader.read_record()?.unwrap();

        assert_eq!(
            restored.get_field("245").unwrap().get_subfield('a'),
            Some("Test 测试 тест العربية 日本語")
        );

        Ok(())
    }

    #[test]
    fn test_preserves_whitespace() -> Result<()> {
        let mut record = Record::new(make_test_leader());
        let mut field = MarcField::new("245".to_string(), '1', '0');
        field.add_subfield('a', "  Leading and trailing  ".to_string());
        record.add_field(field);

        let mut buffer = Vec::new();
        {
            let mut writer = MessagePackWriter::new(&mut buffer);
            writer.write_record(&record)?;
            writer.finish()?;
        }

        let cursor = Cursor::new(buffer);
        let mut reader = MessagePackReader::new(cursor);
        let restored = reader.read_record()?.unwrap();

        assert_eq!(
            restored.get_field("245").unwrap().get_subfield('a'),
            Some("  Leading and trailing  ")
        );

        Ok(())
    }

    #[test]
    fn test_preserves_indicators() -> Result<()> {
        let mut record = Record::new(make_test_leader());
        let mut field = MarcField::new("245".to_string(), '1', '4');
        field.add_subfield('a', "The Title".to_string());
        record.add_field(field);

        let mut buffer = Vec::new();
        {
            let mut writer = MessagePackWriter::new(&mut buffer);
            writer.write_record(&record)?;
            writer.finish()?;
        }

        let cursor = Cursor::new(buffer);
        let mut reader = MessagePackReader::new(cursor);
        let restored = reader.read_record()?.unwrap();

        let field_245 = restored.get_field("245").unwrap();
        assert_eq!(field_245.indicator1, '1');
        assert_eq!(field_245.indicator2, '4');

        Ok(())
    }

    #[test]
    fn test_empty_stream() -> Result<()> {
        let buffer: Vec<u8> = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut reader = MessagePackReader::new(cursor);

        assert!(reader.read_record()?.is_none());
        assert_eq!(reader.records_read(), 0);

        Ok(())
    }

    #[test]
    fn test_writer_cannot_write_after_finish() -> Result<()> {
        let mut record = Record::new(make_test_leader());
        let mut field = MarcField::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test".to_string());
        record.add_field(field);

        let mut buffer = Vec::new();
        let mut writer = MessagePackWriter::new(&mut buffer);
        writer.finish()?;

        let result = writer.write_record(&record);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_format_traits() -> Result<()> {
        use crate::formats::{FormatReader, FormatWriter};

        let records: Vec<Record> = (0..3)
            .map(|i| {
                let mut record = Record::new(make_test_leader());
                let mut field = MarcField::new("245".to_string(), '1', '0');
                field.add_subfield('a', format!("Title {i}"));
                record.add_field(field);
                record
            })
            .collect();

        let mut buffer = Vec::new();
        {
            let mut writer = MessagePackWriter::new(&mut buffer);
            FormatWriter::write_batch(&mut writer, &records)?;
            assert_eq!(FormatWriter::records_written(&writer), Some(3));
            FormatWriter::finish(&mut writer)?;
        }

        let cursor = Cursor::new(buffer);
        let mut reader = MessagePackReader::new(cursor);
        let read_records = FormatReader::read_all(&mut reader)?;
        assert_eq!(read_records.len(), 3);

        Ok(())
    }
}
