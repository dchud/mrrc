//! Protocol Buffers binary format support for MARC records.
//!
//! This module provides serialization and deserialization of MARC records
//! using Protocol Buffers (protobuf), a schema-based binary serialization format
//! developed by Google.
//!
//! ## Features
//!
//! - **Round-trip fidelity**: Preserves exact field ordering, subfield code ordering,
//!   indicators, and all textual content including whitespace
//! - **Compact serialization**: Binary format with reasonable compression
//! - **UTF-8 native**: Operates on mrrc's normalized UTF-8 `MarcRecord` objects
//! - **Schema evolution**: Protobuf3 provides forward/backward compatibility
//!
//! ## Example
//!
//! ```ignore
//! use mrrc::{Record, Field, Leader};
//! use mrrc::protobuf::{ProtobufSerializer, ProtobufDeserializer};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a MARC record
//! let mut record = Record::new(Leader::default());
//! record.add_control_field("001".to_string(), "12345".to_string());
//! let mut field = Field::new("245".to_string(), '1', '0');
//! field.add_subfield('a', "Test Title".to_string());
//! record.add_field(field);
//!
//! // Serialize to protobuf
//! let serialized = ProtobufSerializer::serialize(&record)?;
//!
//! // Deserialize back
//! let restored = ProtobufDeserializer::deserialize(&serialized)?;
//! # Ok(())
//! # }
//! ```

// Include generated protobuf code
include!(concat!(env!("OUT_DIR"), "/mrrc.formats.protobuf.rs"));

use crate::formats::{FormatReader, FormatWriter};
use crate::{Field as MarcField, MarcError, Record, Result};
use prost::Message;
use std::io::{Read, Write};

/// Serializes a MARC Record to Protocol Buffers binary format.
#[derive(Debug)]
pub struct ProtobufSerializer;

impl ProtobufSerializer {
    /// Serialize a MARC record to protobuf binary format.
    ///
    /// # Arguments
    ///
    /// * `record` - The MARC record to serialize
    ///
    /// # Returns
    ///
    /// A vector of bytes containing the protobuf-encoded record
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails (e.g., if the record structure
    /// is invalid or contains data that cannot be encoded)
    pub fn serialize(record: &Record) -> Result<Vec<u8>> {
        let pb_record = convert_record_to_protobuf(record)?;
        Ok(pb_record.encode_to_vec())
    }
}

/// Deserializes Protocol Buffers binary format to a MARC Record.
#[derive(Debug)]
pub struct ProtobufDeserializer;

impl ProtobufDeserializer {
    /// Deserialize protobuf binary format to a MARC record.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The protobuf-encoded data
    ///
    /// # Returns
    ///
    /// A MARC record reconstructed from the protobuf data
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The data is not valid protobuf format
    /// - The protobuf message is malformed
    /// - The field/subfield data is invalid
    pub fn deserialize(bytes: &[u8]) -> Result<Record> {
        let pb_record = MarcRecord::decode(bytes)
            .map_err(|e| MarcError::ParseError(format!("Protobuf decoding failed: {e}")))?;

        convert_protobuf_to_record(&pb_record)
    }
}

/// Converts a mrrc Record to a protobuf `MarcRecord` message.
fn convert_record_to_protobuf(record: &Record) -> Result<MarcRecord> {
    // Serialize leader to bytes then to string for protobuf representation
    let leader_bytes = record.leader.as_bytes()?;
    let leader_string = String::from_utf8(leader_bytes)
        .map_err(|_| MarcError::InvalidLeader("Leader contains invalid UTF-8".to_string()))?;

    if leader_string.len() != 24 {
        return Err(MarcError::InvalidLeader(
            "Record leader must be exactly 24 characters".to_string(),
        ));
    }

    let mut pb_fields = Vec::new();

    // First, add control fields (000-009) in order
    for (tag, value) in record.control_fields_iter() {
        let pb_field = Field {
            tag: tag.to_string(),
            indicator1: String::new(),
            indicator2: String::new(),
            subfields: vec![Subfield {
                code: String::new(),
                value: value.to_string(),
            }],
        };
        pb_fields.push(pb_field);
    }

    // Then, add variable fields in order
    for field in record.fields() {
        let pb_field = convert_field_to_protobuf(field)?;
        pb_fields.push(pb_field);
    }

    Ok(MarcRecord {
        leader: leader_string,
        fields: pb_fields,
    })
}

/// Converts a mrrc Field to a protobuf Field message.
fn convert_field_to_protobuf(field: &MarcField) -> Result<Field> {
    // Validate tag (3-character string)
    if field.tag.len() != 3 {
        return Err(MarcError::InvalidField(format!(
            "Field tag must be exactly 3 characters, got '{}'",
            field.tag
        )));
    }

    // Parse tag to determine if control or variable field
    let tag_num: u32 = field.tag.parse().map_err(|_| {
        MarcError::InvalidField(format!("Field tag '{}' is not a valid number", field.tag))
    })?;

    let (indicator1, indicator2, pb_subfields) = if tag_num < 10 {
        // Control field: has no indicators, subfields stored directly
        let mut subfields = Vec::new();

        for subfield in &field.subfields {
            subfields.push(Subfield {
                code: subfield.code.to_string(),
                value: subfield.value.clone(),
            });
        }

        (String::new(), String::new(), subfields)
    } else {
        // Variable field: has indicators and subfields
        let mut pb_subfields = Vec::new();
        for subfield in &field.subfields {
            pb_subfields.push(Subfield {
                code: subfield.code.to_string(),
                value: subfield.value.clone(),
            });
        }

        (
            field.indicator1.to_string(),
            field.indicator2.to_string(),
            pb_subfields,
        )
    };

    Ok(Field {
        tag: field.tag.clone(),
        indicator1,
        indicator2,
        subfields: pb_subfields,
    })
}

/// Writer for streaming MARC records to protobuf format.
///
/// `ProtobufWriter` implements the [`FormatWriter`] trait, allowing it to be used
/// interchangeably with other format writers. Records are written using length-delimited
/// encoding (each record is prefixed with its size as a varint).
///
/// # Example
///
/// ```ignore
/// use mrrc::protobuf::ProtobufWriter;
/// use mrrc::formats::FormatWriter;
///
/// let mut buffer = Vec::new();
/// let mut writer = ProtobufWriter::new(&mut buffer);
///
/// writer.write_record(&record)?;
/// writer.finish()?;
/// ```
#[derive(Debug)]
pub struct ProtobufWriter<W: Write> {
    writer: W,
    records_written: usize,
    finished: bool,
}

impl<W: Write> ProtobufWriter<W> {
    /// Create a new protobuf writer.
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

    /// Write a single MARC record to protobuf format.
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

        let pb_record = convert_record_to_protobuf(record)?;

        // Encode to a buffer first, then write
        let mut buffer = Vec::new();
        pb_record
            .encode_length_delimited(&mut buffer)
            .map_err(|e| MarcError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        self.writer.write_all(&buffer)?;
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

impl<W: Write + std::fmt::Debug> FormatWriter for ProtobufWriter<W> {
    fn write_record(&mut self, record: &Record) -> Result<()> {
        ProtobufWriter::write_record(self, record)
    }

    fn finish(&mut self) -> Result<()> {
        ProtobufWriter::finish(self)
    }

    fn records_written(&self) -> Option<usize> {
        Some(self.records_written)
    }
}

/// Reader for streaming MARC records from protobuf format.
///
/// `ProtobufReader` implements the [`FormatReader`] trait, allowing it to be used
/// interchangeably with other format readers. Records are read using length-delimited
/// encoding (each record is prefixed with its size as a varint).
///
/// # Example
///
/// ```ignore
/// use mrrc::protobuf::ProtobufReader;
/// use mrrc::formats::FormatReader;
/// use std::fs::File;
///
/// let file = File::open("records.pb")?;
/// let mut reader = ProtobufReader::new(file);
///
/// while let Some(record) = reader.read_record()? {
///     println!("Record: {:?}", record);
/// }
/// ```
#[derive(Debug)]
pub struct ProtobufReader<R: Read> {
    reader: R,
    records_read: usize,
}

impl<R: Read> ProtobufReader<R> {
    /// Create a new protobuf reader.
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

    /// Read a single MARC record from the protobuf stream.
    ///
    /// Returns `Ok(Some(record))` if a record was read, `Ok(None)` at EOF,
    /// or `Err` if parsing fails.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is malformed or I/O fails.
    pub fn read_record(&mut self) -> Result<Option<Record>> {
        // Read length-delimited protobuf message
        // First, read the varint length prefix
        let len = match read_varint(&mut self.reader) {
            Ok(len) => len,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(MarcError::IoError(e)),
        };

        if len == 0 {
            return Ok(None);
        }

        // Read the message bytes
        let len_usize: usize = len.try_into().map_err(|_| {
            MarcError::ParseError("Protobuf message length exceeds platform limit".to_string())
        })?;
        let mut buffer = vec![0u8; len_usize];
        match self.reader.read_exact(&mut buffer) {
            Ok(()) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Err(MarcError::TruncatedRecord(
                    "Unexpected EOF while reading protobuf record".to_string(),
                ));
            },
            Err(e) => return Err(MarcError::IoError(e)),
        }

        // Decode the protobuf message
        let pb_record = MarcRecord::decode(&buffer[..])
            .map_err(|e| MarcError::ParseError(format!("Protobuf decoding failed: {e}")))?;

        let record = convert_protobuf_to_record(&pb_record)?;
        self.records_read += 1;
        Ok(Some(record))
    }

    /// Returns the number of records read so far.
    #[must_use]
    pub fn records_read(&self) -> usize {
        self.records_read
    }
}

impl<R: Read + std::fmt::Debug> FormatReader for ProtobufReader<R> {
    fn read_record(&mut self) -> Result<Option<Record>> {
        ProtobufReader::read_record(self)
    }

    fn records_read(&self) -> Option<usize> {
        Some(self.records_read)
    }
}

/// Read a varint from a reader.
///
/// Varints are used by protobuf for length-delimited encoding.
fn read_varint<R: Read>(reader: &mut R) -> std::io::Result<u64> {
    let mut result: u64 = 0;
    let mut shift = 0;

    loop {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;

        let b = byte[0];
        result |= u64::from(b & 0x7F) << shift;

        if b & 0x80 == 0 {
            return Ok(result);
        }

        shift += 7;
        if shift >= 64 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Varint too large",
            ));
        }
    }
}

/// Converts a protobuf `MarcRecord` message to a mrrc `Record`.
fn convert_protobuf_to_record(pb_record: &MarcRecord) -> Result<Record> {
    // Validate leader
    if pb_record.leader.len() != 24 {
        return Err(MarcError::InvalidLeader(
            "Record leader must be exactly 24 characters".to_string(),
        ));
    }

    // Parse leader from string bytes
    let leader = crate::Leader::from_bytes(pb_record.leader.as_bytes())?;
    let mut record = Record::new(leader);

    // Convert each field
    for pb_field in &pb_record.fields {
        let tag_num: u32 = pb_field.tag.parse().map_err(|_| {
            MarcError::InvalidField(format!(
                "Field tag '{}' is not a valid number",
                pb_field.tag
            ))
        })?;

        if tag_num < 10 {
            // Control field
            if let Some(subfield) = pb_field.subfields.first() {
                record.add_control_field(pb_field.tag.clone(), subfield.value.clone());
            }
        } else {
            // Variable field
            let ind1 = pb_field.indicator1.chars().next().unwrap_or(' ');
            let ind2 = pb_field.indicator2.chars().next().unwrap_or(' ');

            let mut field = MarcField::new(pb_field.tag.clone(), ind1, ind2);

            // Add subfields in exact order
            for subfield in &pb_field.subfields {
                if subfield.code.len() != 1 {
                    return Err(MarcError::InvalidField(format!(
                        "Subfield code must be 1 character, got '{}'",
                        subfield.code
                    )));
                }

                let code = subfield.code.chars().next().unwrap();
                field.add_subfield(code, subfield.value.clone());
            }

            record.add_field(field);
        }
    }

    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // ============================================================================
    // Tests for ProtobufSerializer/ProtobufDeserializer (single-record API)
    // ============================================================================

    #[test]
    fn test_roundtrip_simple_record() -> Result<()> {
        let leader = crate::Leader::from_bytes(b"00345nam a2200133 a 4500")?;
        let mut record = Record::new(leader);
        record.add_control_field("001".to_string(), "test001".to_string());

        let mut field = MarcField::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test Title".to_string());
        field.add_subfield('c', "Author Name".to_string());
        record.add_field(field);

        // Serialize
        let serialized = ProtobufSerializer::serialize(&record)?;

        // Deserialize
        let restored = ProtobufDeserializer::deserialize(&serialized)?;

        // Check basic structure
        let orig_leader_bytes = record.leader.as_bytes()?;
        let rest_leader_bytes = restored.leader.as_bytes()?;
        assert_eq!(orig_leader_bytes, rest_leader_bytes);
        assert_eq!(record.control_fields.len(), restored.control_fields.len());

        Ok(())
    }

    #[test]
    fn test_roundtrip_field_ordering() -> Result<()> {
        let leader = crate::Leader::from_bytes(b"00345nam a2200133 a 4500")?;
        let mut record = Record::new(leader);

        // Add fields in specific order to test ordering preservation
        record.add_control_field("001".to_string(), "test001".to_string());
        record.add_control_field("005".to_string(), "20260113120000.0".to_string());

        let mut field_245 = MarcField::new("245".to_string(), '1', '0');
        field_245.add_subfield('a', "Title".to_string());
        record.add_field(field_245);

        let mut field_650_1 = MarcField::new("650".to_string(), ' ', '0');
        field_650_1.add_subfield('a', "Subject 1".to_string());
        record.add_field(field_650_1);

        let mut field_650_2 = MarcField::new("650".to_string(), ' ', '0');
        field_650_2.add_subfield('a', "Subject 2".to_string());
        record.add_field(field_650_2);

        // Serialize and deserialize
        let serialized = ProtobufSerializer::serialize(&record)?;
        let restored = ProtobufDeserializer::deserialize(&serialized)?;

        // Verify field order is preserved (Record uses IndexMap to preserve insertion order)
        let orig_650s: Vec<_> = record.fields_by_tag("650").collect();
        let rest_650s: Vec<_> = restored.fields_by_tag("650").collect();
        assert_eq!(orig_650s.len(), rest_650s.len());

        Ok(())
    }

    #[test]
    fn test_roundtrip_subfield_ordering() -> Result<()> {
        let leader = crate::Leader::from_bytes(b"00345nam a2200133 a 4500")?;
        let mut record = Record::new(leader);

        let mut field = MarcField::new("245".to_string(), '1', '0');
        // Add subfields in non-alphabetical order
        field.add_subfield('c', "Author".to_string());
        field.add_subfield('a', "Title".to_string());
        field.add_subfield('b', "Subtitle".to_string());
        record.add_field(field);

        // Serialize and deserialize
        let serialized = ProtobufSerializer::serialize(&record)?;
        let restored = ProtobufDeserializer::deserialize(&serialized)?;

        // Verify subfield order is preserved
        let original_codes: Vec<_> = record
            .get_field("245")
            .unwrap()
            .subfields
            .iter()
            .map(|s| s.code)
            .collect();
        let restored_codes: Vec<_> = restored
            .get_field("245")
            .unwrap()
            .subfields
            .iter()
            .map(|s| s.code)
            .collect();

        assert_eq!(original_codes, restored_codes);

        Ok(())
    }

    #[test]
    fn test_empty_subfield_values() -> Result<()> {
        let leader = crate::Leader::from_bytes(b"00345nam a2200133 a 4500")?;
        let mut record = Record::new(leader);

        let mut field = MarcField::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Title".to_string());
        field.add_subfield('b', String::new()); // Empty subfield
        field.add_subfield('c', "Author".to_string());
        record.add_field(field);

        // Serialize and deserialize
        let serialized = ProtobufSerializer::serialize(&record)?;
        let restored = ProtobufDeserializer::deserialize(&serialized)?;

        // Verify empty subfields are preserved
        let original_subfields = &record.get_field("245").unwrap().subfields;
        let restored_subfields = &restored.get_field("245").unwrap().subfields;

        assert_eq!(original_subfields.len(), restored_subfields.len());
        assert_eq!(original_subfields[1].value, ""); // Empty value preserved
        assert_eq!(restored_subfields[1].value, "");

        Ok(())
    }

    #[test]
    fn test_utf8_content() -> Result<()> {
        let leader = crate::Leader::from_bytes(b"00345nam a2200133 a 4500")?;
        let mut record = Record::new(leader);

        let mut field = MarcField::new("245".to_string(), '1', '0');
        // Test with multilingual content
        field.add_subfield('a', "Test 测试 тест العربية".to_string());
        record.add_field(field);

        // Serialize and deserialize
        let serialized = ProtobufSerializer::serialize(&record)?;
        let restored = ProtobufDeserializer::deserialize(&serialized)?;

        // Verify UTF-8 content is preserved
        assert_eq!(
            record.get_field("245").unwrap().subfields[0].value,
            restored.get_field("245").unwrap().subfields[0].value
        );

        Ok(())
    }

    #[test]
    fn test_whitespace_preservation() -> Result<()> {
        let leader = crate::Leader::from_bytes(b"00345nam a2200133 a 4500")?;
        let mut record = Record::new(leader);

        let mut field = MarcField::new("245".to_string(), '1', '0');
        field.add_subfield('a', "  Leading and trailing  ".to_string());
        record.add_field(field);

        // Serialize and deserialize
        let serialized = ProtobufSerializer::serialize(&record)?;
        let restored = ProtobufDeserializer::deserialize(&serialized)?;

        // Verify whitespace is preserved exactly
        assert_eq!(
            "  Leading and trailing  ",
            &restored.get_field("245").unwrap().subfields[0].value
        );

        Ok(())
    }

    // ============================================================================
    // Tests for ProtobufWriter/ProtobufReader (streaming API with FormatWriter/FormatReader)
    // ============================================================================

    fn make_test_leader() -> crate::Leader {
        crate::Leader::from_bytes(b"00345nam a2200133 a 4500").unwrap()
    }

    #[test]
    fn test_streaming_write_read_single_record() -> Result<()> {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test001".to_string());

        let mut field = MarcField::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test Title".to_string());
        record.add_field(field);

        // Write using ProtobufWriter
        let mut buffer = Vec::new();
        {
            let mut writer = ProtobufWriter::new(&mut buffer);
            writer.write_record(&record)?;
            assert_eq!(writer.records_written(), 1);
            writer.finish()?;
        }

        // Read using ProtobufReader
        let cursor = Cursor::new(buffer);
        let mut reader = ProtobufReader::new(cursor);

        let restored = reader.read_record()?.expect("Should read one record");
        assert_eq!(reader.records_read(), 1);

        // Verify content
        assert_eq!(restored.get_control_field("001"), Some("test001"));
        let field_245 = restored.get_field("245").expect("Should have 245");
        assert_eq!(field_245.get_subfield('a'), Some("Test Title"));

        // Should return None on second read
        assert!(reader.read_record()?.is_none());

        Ok(())
    }

    #[test]
    fn test_streaming_write_read_multiple_records() -> Result<()> {
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

        // Write all records
        let mut buffer = Vec::new();
        {
            let mut writer = ProtobufWriter::new(&mut buffer);
            for record in &records {
                writer.write_record(record)?;
            }
            assert_eq!(writer.records_written(), 5);
            writer.finish()?;
        }

        // Read all records back
        let cursor = Cursor::new(buffer);
        let mut reader = ProtobufReader::new(cursor);

        for i in 0..5 {
            let record = reader.read_record()?.expect("Should have record");
            assert_eq!(
                record.get_control_field("001"),
                Some(format!("rec{i:03}").as_str())
            );
            let field_245 = record.get_field("245").expect("Should have 245");
            assert_eq!(
                field_245.get_subfield('a'),
                Some(format!("Title {i}").as_str())
            );
        }

        assert_eq!(reader.records_read(), 5);
        assert!(reader.read_record()?.is_none());

        Ok(())
    }

    #[test]
    fn test_streaming_format_writer_trait() -> Result<()> {
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
            let mut writer = ProtobufWriter::new(&mut buffer);
            // Use the trait's write_batch method
            FormatWriter::write_batch(&mut writer, &records)?;
            assert_eq!(FormatWriter::records_written(&writer), Some(3));
            FormatWriter::finish(&mut writer)?;
        }

        // Read back using FormatReader trait
        let cursor = Cursor::new(buffer);
        let mut reader = ProtobufReader::new(cursor);

        let read_records = FormatReader::read_all(&mut reader)?;
        assert_eq!(read_records.len(), 3);
        assert_eq!(FormatReader::records_read(&reader), Some(3));

        Ok(())
    }

    #[test]
    fn test_streaming_format_reader_iterator() -> Result<()> {
        use crate::formats::FormatReaderExt;

        let mut buffer = Vec::new();
        {
            let mut writer = ProtobufWriter::new(&mut buffer);
            for i in 0..4 {
                let mut record = Record::new(make_test_leader());
                let mut field = MarcField::new("245".to_string(), '1', '0');
                field.add_subfield('a', format!("Title {i}"));
                record.add_field(field);
                writer.write_record(&record)?;
            }
            writer.finish()?;
        }

        let cursor = Cursor::new(buffer);
        let mut reader = ProtobufReader::new(cursor);

        let mut count = 0;
        for result in reader.records() {
            result?;
            count += 1;
        }
        assert_eq!(count, 4);
        assert_eq!(reader.records_read(), 4);

        Ok(())
    }

    #[test]
    fn test_streaming_empty_file() -> Result<()> {
        let buffer: Vec<u8> = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut reader = ProtobufReader::new(cursor);

        assert!(reader.read_record()?.is_none());
        assert_eq!(reader.records_read(), 0);

        Ok(())
    }

    #[test]
    fn test_streaming_writer_cannot_write_after_finish() -> Result<()> {
        let mut record = Record::new(make_test_leader());
        let mut field = MarcField::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test".to_string());
        record.add_field(field);

        let mut buffer = Vec::new();
        let mut writer = ProtobufWriter::new(&mut buffer);
        writer.finish()?;

        let result = writer.write_record(&record);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_streaming_preserves_field_order() -> Result<()> {
        let mut record = Record::new(make_test_leader());

        // Add fields in specific order
        record.add_control_field("001".to_string(), "test001".to_string());
        record.add_control_field("005".to_string(), "20260120".to_string());

        let mut field_100 = MarcField::new("100".to_string(), '1', ' ');
        field_100.add_subfield('a', "Author".to_string());
        record.add_field(field_100);

        let mut field_245 = MarcField::new("245".to_string(), '1', '0');
        field_245.add_subfield('a', "Title".to_string());
        record.add_field(field_245);

        // Add multiple 650 fields
        for i in 1..=3 {
            let mut field_650 = MarcField::new("650".to_string(), ' ', '0');
            field_650.add_subfield('a', format!("Subject {i}"));
            record.add_field(field_650);
        }

        // Write and read back
        let mut buffer = Vec::new();
        {
            let mut writer = ProtobufWriter::new(&mut buffer);
            writer.write_record(&record)?;
            writer.finish()?;
        }

        let cursor = Cursor::new(buffer);
        let mut reader = ProtobufReader::new(cursor);
        let restored = reader.read_record()?.unwrap();

        // Verify field ordering preserved
        let subjects: Vec<_> = restored.fields_by_tag("650").collect();
        assert_eq!(subjects.len(), 3);
        assert_eq!(subjects[0].get_subfield('a'), Some("Subject 1"));
        assert_eq!(subjects[1].get_subfield('a'), Some("Subject 2"));
        assert_eq!(subjects[2].get_subfield('a'), Some("Subject 3"));

        Ok(())
    }

    #[test]
    fn test_streaming_preserves_subfield_order() -> Result<()> {
        let mut record = Record::new(make_test_leader());

        let mut field = MarcField::new("245".to_string(), '1', '0');
        // Add subfields in non-alphabetical order
        field.add_subfield('c', "Author".to_string());
        field.add_subfield('a', "Title".to_string());
        field.add_subfield('b', "Subtitle".to_string());
        record.add_field(field);

        // Write and read back
        let mut buffer = Vec::new();
        {
            let mut writer = ProtobufWriter::new(&mut buffer);
            writer.write_record(&record)?;
            writer.finish()?;
        }

        let cursor = Cursor::new(buffer);
        let mut reader = ProtobufReader::new(cursor);
        let restored = reader.read_record()?.unwrap();

        // Verify subfield order preserved (c, a, b)
        let field_245 = restored.get_field("245").unwrap();
        let codes: Vec<char> = field_245.subfields.iter().map(|s| s.code).collect();
        assert_eq!(codes, vec!['c', 'a', 'b']);

        Ok(())
    }

    #[test]
    fn test_streaming_preserves_utf8() -> Result<()> {
        let mut record = Record::new(make_test_leader());

        let mut field = MarcField::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test 测试 тест العربية 日本語".to_string());
        record.add_field(field);

        let mut buffer = Vec::new();
        {
            let mut writer = ProtobufWriter::new(&mut buffer);
            writer.write_record(&record)?;
            writer.finish()?;
        }

        let cursor = Cursor::new(buffer);
        let mut reader = ProtobufReader::new(cursor);
        let restored = reader.read_record()?.unwrap();

        assert_eq!(
            restored.get_field("245").unwrap().get_subfield('a'),
            Some("Test 测试 тест العربية 日本語")
        );

        Ok(())
    }

    #[test]
    fn test_streaming_preserves_whitespace() -> Result<()> {
        let mut record = Record::new(make_test_leader());

        let mut field = MarcField::new("245".to_string(), '1', '0');
        field.add_subfield('a', "  Leading and trailing  ".to_string());
        field.add_subfield('b', "\ttab\there\t".to_string());
        record.add_field(field);

        let mut buffer = Vec::new();
        {
            let mut writer = ProtobufWriter::new(&mut buffer);
            writer.write_record(&record)?;
            writer.finish()?;
        }

        let cursor = Cursor::new(buffer);
        let mut reader = ProtobufReader::new(cursor);
        let restored = reader.read_record()?.unwrap();

        let field_245 = restored.get_field("245").unwrap();
        assert_eq!(
            field_245.get_subfield('a'),
            Some("  Leading and trailing  ")
        );
        assert_eq!(field_245.get_subfield('b'), Some("\ttab\there\t"));

        Ok(())
    }

    #[test]
    fn test_streaming_preserves_indicators() -> Result<()> {
        let mut record = Record::new(make_test_leader());

        let mut field = MarcField::new("245".to_string(), '1', '4');
        field.add_subfield('a', "The Title".to_string());
        record.add_field(field);

        let mut field2 = MarcField::new("650".to_string(), ' ', '0');
        field2.add_subfield('a', "Subject".to_string());
        record.add_field(field2);

        let mut buffer = Vec::new();
        {
            let mut writer = ProtobufWriter::new(&mut buffer);
            writer.write_record(&record)?;
            writer.finish()?;
        }

        let cursor = Cursor::new(buffer);
        let mut reader = ProtobufReader::new(cursor);
        let restored = reader.read_record()?.unwrap();

        let field_245 = restored.get_field("245").unwrap();
        assert_eq!(field_245.indicator1, '1');
        assert_eq!(field_245.indicator2, '4');

        let field_650 = restored.get_field("650").unwrap();
        assert_eq!(field_650.indicator1, ' ');
        assert_eq!(field_650.indicator2, '0');

        Ok(())
    }
}
