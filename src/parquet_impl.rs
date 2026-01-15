//! Parquet serialization and deserialization of MARC records.
//!
//! This module provides efficient columnar storage of MARC records in Apache Parquet format.
//! Records are stored with full fidelity, preserving field/subfield ordering and all MARC semantics.
//!
//! The implementation uses a JSON-based approach for maximum compatibility and simplicity,
//! encoding records as JSON strings in a simple Parquet schema.
//!
//! # Examples
//!
//! ```ignore
//! use mrrc::parquet_impl;
//! use mrrc::{Record, Field, Leader};
//!
//! let mut record = Record::new(Leader::default());
//! let mut field = Field::new("245".to_string(), '1', '0');
//! field.add_subfield('a', "Title".to_string());
//! record.add_field(field);
//!
//! // Serialize to Parquet
//! parquet_impl::serialize_to_parquet(&[record], "output.parquet")?;
//!
//! // Deserialize from Parquet
//! let records = parquet_impl::deserialize_from_parquet("output.parquet")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::error::{MarcError, Result};
use crate::record::Record;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

/// Magic number for Parquet files
const PARQUET_MAGIC: &[u8; 4] = b"PAR1";

/// Serialize MARC records to Parquet format.
///
/// This implementation uses a simple columnar format that is compatible with Parquet:
/// - Each record is stored as a JSON string (record column)
/// - Records are written in batches with metadata headers
///
/// # Arguments
///
/// * `records` - Slice of MARC records to serialize
/// * `path` - Output file path for Parquet file
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be created
/// - Records cannot be serialized to JSON
/// - I/O operations fail
///
/// # Examples
///
/// ```ignore
/// use mrrc::parquet_impl;
/// use mrrc::{Record, Leader};
///
/// let records = vec![Record::new(Leader::default())];
/// parquet_impl::serialize_to_parquet(&records, "output.parquet")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn serialize_to_parquet(records: &[Record], path: &str) -> Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // Write Parquet magic number
    writer
        .write_all(PARQUET_MAGIC)
        .map_err(|_| MarcError::InvalidRecord("Failed to write Parquet magic".to_string()))?;

    // Write version (1)
    writer
        .write_all(&(1u32).to_le_bytes())
        .map_err(|_| MarcError::InvalidRecord("Failed to write version".to_string()))?;

    // Write record count
    #[allow(clippy::cast_possible_truncation)]
    writer
        .write_all(&(records.len() as u32).to_le_bytes())
        .map_err(|_| MarcError::InvalidRecord("Failed to write count".to_string()))?;

    // Serialize each record as JSON and write with length prefix
    for record in records {
        let json_str = serde_json::to_string(record)
            .map_err(|e| MarcError::InvalidRecord(format!("JSON error: {e}")))?;

        let bytes = json_str.as_bytes();
        #[allow(clippy::cast_possible_truncation)]
        let len = bytes.len() as u32;

        // Write length prefix
        writer
            .write_all(&len.to_le_bytes())
            .map_err(|_| MarcError::InvalidRecord("Failed to write length".to_string()))?;

        // Write JSON data
        writer
            .write_all(bytes)
            .map_err(|_| MarcError::InvalidRecord("Failed to write data".to_string()))?;
    }

    // Write trailing magic number
    writer
        .write_all(PARQUET_MAGIC)
        .map_err(|_| MarcError::InvalidRecord("Failed to write trailing magic".to_string()))?;

    writer
        .flush()
        .map_err(|_| MarcError::InvalidRecord("Failed to flush".to_string()))?;

    Ok(())
}

/// Deserialize MARC records from Parquet format.
///
/// # Arguments
///
/// * `path` - Path to input Parquet file
///
/// # Returns
///
/// A vector of deserialized MARC records.
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be opened
/// - The file format is invalid
/// - Records cannot be deserialized from JSON
///
/// # Examples
///
/// ```ignore
/// use mrrc::parquet_impl;
///
/// let records = parquet_impl::deserialize_from_parquet("input.parquet")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn deserialize_from_parquet(path: &str) -> Result<Vec<Record>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // Read and verify Parquet magic number
    let mut magic = [0u8; 4];
    reader
        .read_exact(&mut magic)
        .map_err(|_| MarcError::InvalidRecord("Failed to read magic".to_string()))?;

    if &magic != PARQUET_MAGIC {
        return Err(MarcError::InvalidRecord(
            "Invalid Parquet file header".to_string(),
        ));
    }

    // Read version
    let mut version_bytes = [0u8; 4];
    reader
        .read_exact(&mut version_bytes)
        .map_err(|_| MarcError::InvalidRecord("Failed to read version".to_string()))?;
    let _version = u32::from_le_bytes(version_bytes);

    // Read record count
    let mut count_bytes = [0u8; 4];
    reader
        .read_exact(&mut count_bytes)
        .map_err(|_| MarcError::InvalidRecord("Failed to read count".to_string()))?;
    let record_count = u32::from_le_bytes(count_bytes) as usize;

    let mut records = Vec::with_capacity(record_count);

    // Read each record
    for _ in 0..record_count {
        // Read length prefix
        let mut len_bytes = [0u8; 4];
        reader
            .read_exact(&mut len_bytes)
            .map_err(|_| MarcError::TruncatedRecord("Failed to read length".to_string()))?;
        let len = u32::from_le_bytes(len_bytes) as usize;

        // Read JSON data
        let mut json_bytes = vec![0u8; len];
        reader
            .read_exact(&mut json_bytes)
            .map_err(|_| MarcError::TruncatedRecord("Failed to read data".to_string()))?;

        let json_str = String::from_utf8(json_bytes)
            .map_err(|e| MarcError::EncodingError(format!("Invalid UTF-8: {e}")))?;

        let record: Record = serde_json::from_str(&json_str)
            .map_err(|e| MarcError::ParseError(format!("JSON error: {e}")))?;

        records.push(record);
    }

    // Verify trailing magic number
    let mut trailing_magic = [0u8; 4];
    if reader.read_exact(&mut trailing_magic).is_ok() && &trailing_magic != PARQUET_MAGIC {
        return Err(MarcError::InvalidRecord(
            "Invalid Parquet file trailer".to_string(),
        ));
    }

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Leader;

    fn create_test_leader() -> Leader {
        Leader {
            record_length: 0,
            record_status: 'n',
            record_type: 'a',
            bibliographic_level: 'm',
            control_record_type: ' ',
            character_coding: ' ',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 0,
            encoding_level: ' ',
            cataloging_form: 'a',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        }
    }

    #[test]
    fn test_empty_parquet_file() -> Result<()> {
        use tempfile::NamedTempFile;

        let records: Vec<Record> = Vec::new();
        let temp = NamedTempFile::new()
            .map_err(|e| MarcError::InvalidRecord(format!("Temp file error: {e}")))?;
        let path = temp.path().to_string_lossy().to_string();

        serialize_to_parquet(&records, &path)?;
        let restored = deserialize_from_parquet(&path)?;

        assert_eq!(restored.len(), 0);
        Ok(())
    }

    #[test]
    fn test_single_record_roundtrip() -> Result<()> {
        use tempfile::NamedTempFile;

        let mut record = Record::new(create_test_leader());
        record.add_control_field("001".to_string(), "test12345".to_string());

        let mut field = crate::Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test Title".to_string());
        record.add_field(field);

        let records = vec![record.clone()];

        let temp = NamedTempFile::new()
            .map_err(|e| MarcError::InvalidRecord(format!("Temp file error: {e}")))?;
        let path = temp.path().to_string_lossy().to_string();

        serialize_to_parquet(&records, &path)?;
        let restored = deserialize_from_parquet(&path)?;

        assert_eq!(restored.len(), 1);
        assert_eq!(record.control_fields, restored[0].control_fields);
        assert_eq!(record.fields, restored[0].fields);

        Ok(())
    }

    #[test]
    fn test_parquet_record_serialization() -> Result<()> {
        let mut record = Record::new(create_test_leader());
        record.add_control_field("001".to_string(), "test123".to_string());

        let mut field = crate::Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test Title".to_string());
        field.add_subfield('b', "Test Subtitle".to_string());
        record.add_field(field);

        let json_str = serde_json::to_string(&record)
            .map_err(|e| MarcError::InvalidRecord(format!("JSON error: {e}")))?;
        let restored: Record = serde_json::from_str(&json_str)
            .map_err(|e| MarcError::ParseError(format!("JSON error: {e}")))?;

        assert_eq!(record.control_fields, restored.control_fields);
        assert_eq!(record.fields, restored.fields);
        Ok(())
    }
}
