//! `FlatBuffers` serialization and deserialization for MARC records.
//!
//! `FlatBuffers` is a cross-language serialization library that enables fast serialization
//! and zero-copy deserialization. This module implements MARC record handling using
//! the generated `FlatBuffers` code from `proto/marc.fbs`.
//!
//! ## Features
//!
//! - **Zero-copy reads**: Data can be accessed directly from the buffer without parsing
//! - **Memory efficient**: No intermediate object allocation during reads
//! - **Round-trip fidelity**: Preserves exact field ordering, subfield code ordering,
//!   indicators, and all textual content including whitespace
//! - **UTF-8 native**: Operates on mrrc's normalized UTF-8 `MarcRecord` objects
//!
//! ## Example
//!
//! ```ignore
//! use mrrc::{Record, Field, Leader};
//! use mrrc::flatbuffers_impl::{FlatbuffersSerializer, FlatbuffersDeserializer};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a MARC record
//! let mut record = Record::new(Leader::default());
//! record.add_control_field("001".to_string(), "12345".to_string());
//! let mut field = Field::new("245".to_string(), '1', '0');
//! field.add_subfield('a', "Test Title".to_string());
//! record.add_field(field);
//!
//! // Serialize to FlatBuffers
//! let serialized = FlatbuffersSerializer::serialize(&record)?;
//!
//! // Deserialize back
//! let restored = FlatbuffersDeserializer::deserialize(&serialized)?;
//! # Ok(())
//! # }
//! ```

use crate::formats::{FormatReader, FormatWriter};
use crate::generated::{
    Field as FbField, FieldArgs, MarcRecord as FbMarcRecord, MarcRecordArgs,
    Subfield as FbSubfield, SubfieldArgs,
};
use crate::{Field as MarcField, MarcError, Record, Result};
use flatbuffers::FlatBufferBuilder;
use std::io::{Read, Write};

/// Serializes a MARC record to `FlatBuffers` binary format.
#[derive(Debug)]
pub struct FlatbuffersSerializer;

impl FlatbuffersSerializer {
    /// Serialize a MARC record to `FlatBuffers` bytes.
    ///
    /// # Arguments
    ///
    /// * `record` - The MARC record to serialize
    ///
    /// # Returns
    ///
    /// A vector of bytes containing the `FlatBuffers`-encoded record
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails (e.g., if the record structure
    /// is invalid or contains data that cannot be encoded)
    pub fn serialize(record: &Record) -> Result<Vec<u8>> {
        let mut builder = FlatBufferBuilder::with_capacity(1024);

        // Build the record
        let fb_record = build_flatbuffer_record(&mut builder, record)?;

        // Finish the buffer (without size prefix for single-record API)
        builder.finish(fb_record, None);

        Ok(builder.finished_data().to_vec())
    }

    /// Serialize a MARC record to `FlatBuffers` bytes with size prefix.
    ///
    /// This format is suitable for streaming multiple records.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn serialize_size_prefixed(record: &Record) -> Result<Vec<u8>> {
        let mut builder = FlatBufferBuilder::with_capacity(1024);

        // Build the record
        let fb_record = build_flatbuffer_record(&mut builder, record)?;

        // Finish with size prefix for streaming
        builder.finish_size_prefixed(fb_record, None);

        Ok(builder.finished_data().to_vec())
    }
}

/// Deserializes a MARC record from `FlatBuffers` binary format.
#[derive(Debug)]
pub struct FlatbuffersDeserializer;

impl FlatbuffersDeserializer {
    /// Deserialize `FlatBuffers` binary format to a MARC record.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The `FlatBuffers`-encoded data
    ///
    /// # Returns
    ///
    /// A MARC record reconstructed from the `FlatBuffers` data
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The data is not valid `FlatBuffers` format
    /// - The `FlatBuffers` message is malformed
    /// - The field/subfield data is invalid
    pub fn deserialize(bytes: &[u8]) -> Result<Record> {
        let fb_record = crate::generated::root_as_marc_record(bytes)
            .map_err(|e| MarcError::ParseError(format!("FlatBuffers verification failed: {e}")))?;

        convert_flatbuffer_to_record(&fb_record)
    }

    /// Deserialize size-prefixed `FlatBuffers` binary format to a MARC record.
    ///
    /// This format is used for streaming multiple records.
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails.
    pub fn deserialize_size_prefixed(bytes: &[u8]) -> Result<Record> {
        let fb_record = crate::generated::size_prefixed_root_as_marc_record(bytes)
            .map_err(|e| MarcError::ParseError(format!("FlatBuffers verification failed: {e}")))?;

        convert_flatbuffer_to_record(&fb_record)
    }
}

/// Builds a `FlatBuffers` `MarcRecord` from a mrrc Record.
fn build_flatbuffer_record<'bldr>(
    builder: &mut FlatBufferBuilder<'bldr>,
    record: &Record,
) -> Result<flatbuffers::WIPOffset<FbMarcRecord<'bldr>>> {
    // Serialize leader to string
    let leader_bytes = record.leader.as_bytes()?;
    let leader_string = String::from_utf8(leader_bytes)
        .map_err(|_| MarcError::InvalidLeader("Leader contains invalid UTF-8".to_string()))?;

    if leader_string.len() != 24 {
        return Err(MarcError::InvalidLeader(
            "Record leader must be exactly 24 characters".to_string(),
        ));
    }

    // Build fields vector
    let mut field_offsets = Vec::new();

    // First, add control fields (000-009) in order
    for (tag, value) in record.control_fields_iter() {
        let fb_field = build_control_field(builder, tag, value);
        field_offsets.push(fb_field);
    }

    // Then, add variable fields in order
    for field in record.fields() {
        let fb_field = build_variable_field(builder, field)?;
        field_offsets.push(fb_field);
    }

    // Create the fields vector
    let fields_vector = builder.create_vector(&field_offsets);

    // Create the leader string
    let leader_offset = builder.create_string(&leader_string);

    // Build the MarcRecord
    let args = MarcRecordArgs {
        leader: Some(leader_offset),
        fields: Some(fields_vector),
    };

    Ok(FbMarcRecord::create(builder, &args))
}

/// Builds a `FlatBuffers` Field for a control field.
fn build_control_field<'bldr>(
    builder: &mut FlatBufferBuilder<'bldr>,
    tag: &str,
    value: &str,
) -> flatbuffers::WIPOffset<FbField<'bldr>> {
    // Control fields have empty indicators and a single subfield with empty code
    let tag_offset = builder.create_string(tag);
    let ind1_offset = builder.create_string("");
    let ind2_offset = builder.create_string("");

    // Create subfield with empty code
    let code_offset = builder.create_string("");
    let value_offset = builder.create_string(value);

    let subfield_args = SubfieldArgs {
        code: Some(code_offset),
        value: Some(value_offset),
    };
    let subfield = FbSubfield::create(builder, &subfield_args);

    let subfields_vector = builder.create_vector(&[subfield]);

    let args = FieldArgs {
        tag: Some(tag_offset),
        indicator1: Some(ind1_offset),
        indicator2: Some(ind2_offset),
        subfields: Some(subfields_vector),
    };

    FbField::create(builder, &args)
}

/// Builds a `FlatBuffers` Field for a variable field.
fn build_variable_field<'bldr>(
    builder: &mut FlatBufferBuilder<'bldr>,
    field: &MarcField,
) -> Result<flatbuffers::WIPOffset<FbField<'bldr>>> {
    // Validate tag
    if field.tag.len() != 3 {
        return Err(MarcError::InvalidField(format!(
            "Field tag must be exactly 3 characters, got '{}'",
            field.tag
        )));
    }

    let tag_offset = builder.create_string(&field.tag);
    let ind1_offset = builder.create_string(&field.indicator1.to_string());
    let ind2_offset = builder.create_string(&field.indicator2.to_string());

    // Build subfields
    let mut subfield_offsets = Vec::new();
    for subfield in &field.subfields {
        let code_offset = builder.create_string(&subfield.code.to_string());
        let value_offset = builder.create_string(&subfield.value);

        let subfield_args = SubfieldArgs {
            code: Some(code_offset),
            value: Some(value_offset),
        };
        subfield_offsets.push(FbSubfield::create(builder, &subfield_args));
    }

    let subfields_vector = if subfield_offsets.is_empty() {
        None
    } else {
        Some(builder.create_vector(&subfield_offsets))
    };

    let args = FieldArgs {
        tag: Some(tag_offset),
        indicator1: Some(ind1_offset),
        indicator2: Some(ind2_offset),
        subfields: subfields_vector,
    };

    Ok(FbField::create(builder, &args))
}

/// Converts a `FlatBuffers` `MarcRecord` to a mrrc Record.
fn convert_flatbuffer_to_record(fb_record: &FbMarcRecord<'_>) -> Result<Record> {
    // Parse leader
    let leader_str = fb_record.leader();
    if leader_str.len() != 24 {
        return Err(MarcError::InvalidLeader(
            "Record leader must be exactly 24 characters".to_string(),
        ));
    }

    let leader = crate::Leader::from_bytes(leader_str.as_bytes())?;
    let mut record = Record::new(leader);

    // Convert fields
    if let Some(fields) = fb_record.fields() {
        for fb_field in fields {
            let tag = fb_field.tag().to_string();

            // Parse tag to determine if control or variable field
            let tag_num: u32 = tag.parse().map_err(|_| {
                MarcError::InvalidField(format!("Field tag '{tag}' is not a valid number"))
            })?;

            if tag_num < 10 {
                // Control field - extract value from first subfield
                if let Some(subfields) = fb_field.subfields() {
                    if let Some(first_sf) = subfields.iter().next() {
                        record.add_control_field(tag, first_sf.value().to_string());
                    }
                }
            } else {
                // Variable field
                let ind1_str = fb_field.indicator1();
                let ind2_str = fb_field.indicator2();

                let ind1 = ind1_str.chars().next().unwrap_or(' ');
                let ind2 = ind2_str.chars().next().unwrap_or(' ');

                let mut field = MarcField::new(tag, ind1, ind2);

                // Add subfields
                if let Some(subfields) = fb_field.subfields() {
                    for fb_subfield in subfields {
                        let code_str = fb_subfield.code();
                        if code_str.len() != 1 {
                            return Err(MarcError::InvalidField(format!(
                                "Subfield code must be 1 character, got '{code_str}'"
                            )));
                        }

                        let code = code_str.chars().next().unwrap();
                        field.add_subfield(code, fb_subfield.value().to_string());
                    }
                }

                record.add_field(field);
            }
        }
    }

    Ok(record)
}

/// Writer for streaming MARC records to `FlatBuffers` format.
///
/// `FlatbuffersWriter` implements the [`FormatWriter`] trait, allowing it to be used
/// interchangeably with other format writers. Records are written using size-prefixed
/// encoding (each record is prefixed with its size as a 4-byte little-endian integer).
///
/// # Example
///
/// ```ignore
/// use mrrc::flatbuffers_impl::FlatbuffersWriter;
/// use mrrc::formats::FormatWriter;
///
/// let mut buffer = Vec::new();
/// let mut writer = FlatbuffersWriter::new(&mut buffer);
///
/// writer.write_record(&record)?;
/// writer.finish()?;
/// ```
#[derive(Debug)]
pub struct FlatbuffersWriter<W: Write> {
    writer: W,
    records_written: usize,
    finished: bool,
}

impl<W: Write> FlatbuffersWriter<W> {
    /// Create a new `FlatBuffers` writer.
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

    /// Write a single MARC record to `FlatBuffers` format.
    ///
    /// Records are written with size-prefixed encoding for streaming support.
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

        // Serialize with size prefix
        let bytes = FlatbuffersSerializer::serialize_size_prefixed(record)?;

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

impl<W: Write + std::fmt::Debug> FormatWriter for FlatbuffersWriter<W> {
    fn write_record(&mut self, record: &Record) -> Result<()> {
        FlatbuffersWriter::write_record(self, record)
    }

    fn finish(&mut self) -> Result<()> {
        FlatbuffersWriter::finish(self)
    }

    fn records_written(&self) -> Option<usize> {
        Some(self.records_written)
    }
}

/// Reader for streaming MARC records from `FlatBuffers` format.
///
/// `FlatbuffersReader` implements the [`FormatReader`] trait, allowing it to be used
/// interchangeably with other format readers. Records are read using size-prefixed
/// encoding (each record is prefixed with its size as a 4-byte little-endian integer).
///
/// # Example
///
/// ```ignore
/// use mrrc::flatbuffers_impl::FlatbuffersReader;
/// use mrrc::formats::FormatReader;
/// use std::fs::File;
///
/// let file = File::open("records.fb")?;
/// let mut reader = FlatbuffersReader::new(file);
///
/// while let Some(record) = reader.read_record()? {
///     println!("Record: {:?}", record);
/// }
/// ```
#[derive(Debug)]
pub struct FlatbuffersReader<R: Read> {
    reader: R,
    records_read: usize,
}

impl<R: Read> FlatbuffersReader<R> {
    /// Create a new `FlatBuffers` reader.
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

    /// Read a single MARC record from the `FlatBuffers` stream.
    ///
    /// Returns `Ok(Some(record))` if a record was read, `Ok(None)` at EOF,
    /// or `Err` if parsing fails.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is malformed or I/O fails.
    pub fn read_record(&mut self) -> Result<Option<Record>> {
        // Read the size prefix (4-byte little-endian u32)
        let mut size_buf = [0u8; 4];
        match self.reader.read_exact(&mut size_buf) {
            Ok(()) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(MarcError::IoError(e)),
        }

        let size = u32::from_le_bytes(size_buf) as usize;

        if size == 0 {
            return Ok(None);
        }

        // Read the FlatBuffer data (size includes the 4-byte prefix in size-prefixed format)
        // The actual data size is what we read, and the buffer starts after the size prefix
        let mut buffer = vec![0u8; size + 4]; // Include size prefix in buffer
        buffer[..4].copy_from_slice(&size_buf);

        match self.reader.read_exact(&mut buffer[4..]) {
            Ok(()) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Err(MarcError::TruncatedRecord(
                    "Unexpected EOF while reading FlatBuffers record".to_string(),
                ));
            },
            Err(e) => return Err(MarcError::IoError(e)),
        }

        // Deserialize using size-prefixed reader
        let record = FlatbuffersDeserializer::deserialize_size_prefixed(&buffer)?;
        self.records_read += 1;
        Ok(Some(record))
    }

    /// Returns the number of records read so far.
    #[must_use]
    pub fn records_read(&self) -> usize {
        self.records_read
    }
}

impl<R: Read + std::fmt::Debug> FormatReader for FlatbuffersReader<R> {
    fn read_record(&mut self) -> Result<Option<Record>> {
        FlatbuffersReader::read_record(self)
    }

    fn records_read(&self) -> Option<usize> {
        Some(self.records_read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Tests for FlatbuffersSerializer/FlatbuffersDeserializer (single-record API)
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
        let serialized = FlatbuffersSerializer::serialize(&record)?;

        // Deserialize
        let restored = FlatbuffersDeserializer::deserialize(&serialized)?;

        // Check basic structure
        let orig_leader_bytes = record.leader.as_bytes()?;
        let rest_leader_bytes = restored.leader.as_bytes()?;
        assert_eq!(orig_leader_bytes, rest_leader_bytes);
        assert_eq!(record.control_fields.len(), restored.control_fields.len());
        assert_eq!(
            record.get_control_field("001"),
            restored.get_control_field("001")
        );

        let orig_245 = record.get_field("245").unwrap();
        let rest_245 = restored.get_field("245").unwrap();
        assert_eq!(orig_245.indicator1, rest_245.indicator1);
        assert_eq!(orig_245.indicator2, rest_245.indicator2);
        assert_eq!(orig_245.subfields.len(), rest_245.subfields.len());

        Ok(())
    }

    #[test]
    fn test_roundtrip_field_ordering() -> Result<()> {
        let leader = crate::Leader::from_bytes(b"00345nam a2200133 a 4500")?;
        let mut record = Record::new(leader);

        // Add fields in specific order
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
        let serialized = FlatbuffersSerializer::serialize(&record)?;
        let restored = FlatbuffersDeserializer::deserialize(&serialized)?;

        // Verify field order is preserved
        let orig_650s: Vec<_> = record.fields_by_tag("650").collect();
        let rest_650s: Vec<_> = restored.fields_by_tag("650").collect();
        assert_eq!(orig_650s.len(), rest_650s.len());
        assert_eq!(orig_650s.len(), 2);

        assert_eq!(
            orig_650s[0].get_subfield('a'),
            rest_650s[0].get_subfield('a')
        );
        assert_eq!(
            orig_650s[1].get_subfield('a'),
            rest_650s[1].get_subfield('a')
        );

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
        let serialized = FlatbuffersSerializer::serialize(&record)?;
        let restored = FlatbuffersDeserializer::deserialize(&serialized)?;

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
        assert_eq!(original_codes, vec!['c', 'a', 'b']);

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
        let serialized = FlatbuffersSerializer::serialize(&record)?;
        let restored = FlatbuffersDeserializer::deserialize(&serialized)?;

        // Verify empty subfields are preserved
        let original_subfields = &record.get_field("245").unwrap().subfields;
        let restored_subfields = &restored.get_field("245").unwrap().subfields;

        assert_eq!(original_subfields.len(), restored_subfields.len());
        assert_eq!(original_subfields[1].value, "");
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
        let serialized = FlatbuffersSerializer::serialize(&record)?;
        let restored = FlatbuffersDeserializer::deserialize(&serialized)?;

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
        let serialized = FlatbuffersSerializer::serialize(&record)?;
        let restored = FlatbuffersDeserializer::deserialize(&serialized)?;

        // Verify whitespace is preserved exactly
        assert_eq!(
            "  Leading and trailing  ",
            &restored.get_field("245").unwrap().subfields[0].value
        );

        Ok(())
    }

    // ============================================================================
    // Tests for FlatbuffersWriter (streaming API with FormatWriter)
    // ============================================================================

    fn make_test_leader() -> crate::Leader {
        crate::Leader::from_bytes(b"00345nam a2200133 a 4500").unwrap()
    }

    #[test]
    fn test_streaming_write_single_record() -> Result<()> {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test001".to_string());

        let mut field = MarcField::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test Title".to_string());
        record.add_field(field);

        // Write using FlatbuffersWriter
        let mut buffer = Vec::new();
        {
            let mut writer = FlatbuffersWriter::new(&mut buffer);
            writer.write_record(&record)?;
            assert_eq!(writer.records_written(), 1);
            writer.finish()?;
        }

        // Verify the buffer is not empty and can be read
        assert!(!buffer.is_empty());

        // Read using size-prefixed deserializer
        let restored = FlatbuffersDeserializer::deserialize_size_prefixed(&buffer)?;

        // Verify content
        assert_eq!(restored.get_control_field("001"), Some("test001"));
        let field_245 = restored.get_field("245").expect("Should have 245");
        assert_eq!(field_245.get_subfield('a'), Some("Test Title"));

        Ok(())
    }

    #[test]
    fn test_streaming_format_writer_trait() -> Result<()> {
        use crate::formats::FormatWriter;

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
            let mut writer = FlatbuffersWriter::new(&mut buffer);
            // Use the trait's write_batch method
            FormatWriter::write_batch(&mut writer, &records)?;
            assert_eq!(FormatWriter::records_written(&writer), Some(3));
            FormatWriter::finish(&mut writer)?;
        }

        assert!(!buffer.is_empty());

        Ok(())
    }

    #[test]
    fn test_streaming_writer_cannot_write_after_finish() -> Result<()> {
        let mut record = Record::new(make_test_leader());
        let mut field = MarcField::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test".to_string());
        record.add_field(field);

        let mut buffer = Vec::new();
        let mut writer = FlatbuffersWriter::new(&mut buffer);
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
            let mut writer = FlatbuffersWriter::new(&mut buffer);
            writer.write_record(&record)?;
            writer.finish()?;
        }

        let restored = FlatbuffersDeserializer::deserialize_size_prefixed(&buffer)?;

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
            let mut writer = FlatbuffersWriter::new(&mut buffer);
            writer.write_record(&record)?;
            writer.finish()?;
        }

        let restored = FlatbuffersDeserializer::deserialize_size_prefixed(&buffer)?;

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
            let mut writer = FlatbuffersWriter::new(&mut buffer);
            writer.write_record(&record)?;
            writer.finish()?;
        }

        let restored = FlatbuffersDeserializer::deserialize_size_prefixed(&buffer)?;

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
            let mut writer = FlatbuffersWriter::new(&mut buffer);
            writer.write_record(&record)?;
            writer.finish()?;
        }

        let restored = FlatbuffersDeserializer::deserialize_size_prefixed(&buffer)?;

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
            let mut writer = FlatbuffersWriter::new(&mut buffer);
            writer.write_record(&record)?;
            writer.finish()?;
        }

        let restored = FlatbuffersDeserializer::deserialize_size_prefixed(&buffer)?;

        let field_245 = restored.get_field("245").unwrap();
        assert_eq!(field_245.indicator1, '1');
        assert_eq!(field_245.indicator2, '4');

        let field_650 = restored.get_field("650").unwrap();
        assert_eq!(field_650.indicator1, ' ');
        assert_eq!(field_650.indicator2, '0');

        Ok(())
    }

    // ============================================================================
    // Tests for FlatbuffersReader (streaming API with FormatReader)
    // ============================================================================

    #[test]
    fn test_reader_read_single_record() -> Result<()> {
        use std::io::Cursor;

        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test001".to_string());

        let mut field = MarcField::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test Title".to_string());
        record.add_field(field);

        // Write using FlatbuffersWriter
        let mut buffer = Vec::new();
        {
            let mut writer = FlatbuffersWriter::new(&mut buffer);
            writer.write_record(&record)?;
            writer.finish()?;
        }

        // Read using FlatbuffersReader
        let cursor = Cursor::new(buffer);
        let mut reader = FlatbuffersReader::new(cursor);

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
    fn test_reader_read_multiple_records() -> Result<()> {
        use std::io::Cursor;

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
            let mut writer = FlatbuffersWriter::new(&mut buffer);
            for record in &records {
                writer.write_record(record)?;
            }
            assert_eq!(writer.records_written(), 5);
            writer.finish()?;
        }

        // Read all records back
        let cursor = Cursor::new(buffer);
        let mut reader = FlatbuffersReader::new(cursor);

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
    fn test_reader_format_reader_trait() -> Result<()> {
        use std::io::Cursor;

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
            let mut writer = FlatbuffersWriter::new(&mut buffer);
            for record in &records {
                writer.write_record(record)?;
            }
            writer.finish()?;
        }

        // Read back using FormatReader trait
        let cursor = Cursor::new(buffer);
        let mut reader = FlatbuffersReader::new(cursor);

        let read_records = FormatReader::read_all(&mut reader)?;
        assert_eq!(read_records.len(), 3);
        assert_eq!(FormatReader::records_read(&reader), Some(3));

        Ok(())
    }

    #[test]
    fn test_reader_format_reader_iterator() -> Result<()> {
        use crate::formats::FormatReaderExt;
        use std::io::Cursor;

        let mut buffer = Vec::new();
        {
            let mut writer = FlatbuffersWriter::new(&mut buffer);
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
        let mut reader = FlatbuffersReader::new(cursor);

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
    fn test_reader_empty_file() -> Result<()> {
        use std::io::Cursor;

        let buffer: Vec<u8> = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut reader = FlatbuffersReader::new(cursor);

        assert!(reader.read_record()?.is_none());
        assert_eq!(reader.records_read(), 0);

        Ok(())
    }

    #[test]
    fn test_reader_preserves_field_order() -> Result<()> {
        use std::io::Cursor;

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
            let mut writer = FlatbuffersWriter::new(&mut buffer);
            writer.write_record(&record)?;
            writer.finish()?;
        }

        let cursor = Cursor::new(buffer);
        let mut reader = FlatbuffersReader::new(cursor);
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
    fn test_reader_preserves_subfield_order() -> Result<()> {
        use std::io::Cursor;

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
            let mut writer = FlatbuffersWriter::new(&mut buffer);
            writer.write_record(&record)?;
            writer.finish()?;
        }

        let cursor = Cursor::new(buffer);
        let mut reader = FlatbuffersReader::new(cursor);
        let restored = reader.read_record()?.unwrap();

        // Verify subfield order preserved (c, a, b)
        let field_245 = restored.get_field("245").unwrap();
        let codes: Vec<char> = field_245.subfields.iter().map(|s| s.code).collect();
        assert_eq!(codes, vec!['c', 'a', 'b']);

        Ok(())
    }

    #[test]
    fn test_reader_preserves_utf8() -> Result<()> {
        use std::io::Cursor;

        let mut record = Record::new(make_test_leader());

        let mut field = MarcField::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test 测试 тест العربية 日本語".to_string());
        record.add_field(field);

        let mut buffer = Vec::new();
        {
            let mut writer = FlatbuffersWriter::new(&mut buffer);
            writer.write_record(&record)?;
            writer.finish()?;
        }

        let cursor = Cursor::new(buffer);
        let mut reader = FlatbuffersReader::new(cursor);
        let restored = reader.read_record()?.unwrap();

        assert_eq!(
            restored.get_field("245").unwrap().get_subfield('a'),
            Some("Test 测试 тест العربية 日本語")
        );

        Ok(())
    }

    #[test]
    fn test_reader_preserves_indicators() -> Result<()> {
        use std::io::Cursor;

        let mut record = Record::new(make_test_leader());

        let mut field = MarcField::new("245".to_string(), '1', '4');
        field.add_subfield('a', "The Title".to_string());
        record.add_field(field);

        let mut field2 = MarcField::new("650".to_string(), ' ', '0');
        field2.add_subfield('a', "Subject".to_string());
        record.add_field(field2);

        let mut buffer = Vec::new();
        {
            let mut writer = FlatbuffersWriter::new(&mut buffer);
            writer.write_record(&record)?;
            writer.finish()?;
        }

        let cursor = Cursor::new(buffer);
        let mut reader = FlatbuffersReader::new(cursor);
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
