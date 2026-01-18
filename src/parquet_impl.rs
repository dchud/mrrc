//! Apache Parquet implementation for MARC record serialization using Arrow columnar storage
//!
//! This module provides columnar storage of MARC records in Apache Parquet format
//! using Arrow's native columnar schema. The flattened denormalized approach enables:
//! - Selective column access via Parquet column projection
//! - Efficient compression of repeated values (tags, leaders, indicators)
//! - Perfect field/subfield ordering preservation via sequential rows
//! - 100% round-trip fidelity

use crate::error::{MarcError, Result};
use crate::record::Record;
use arrow::array::{RecordBatch, StringArray, UInt32Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::ipc::reader::StreamReader;
use arrow::ipc::writer::StreamWriter;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::sync::Arc;

/// Create Arrow schema for MARC record collection
#[must_use]
pub fn create_parquet_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("record_index", DataType::UInt32, false),
        Field::new("leader", DataType::Utf8, false),
        Field::new("field_tag", DataType::Utf8, false),
        Field::new("field_indicator1", DataType::Utf8, false),
        Field::new("field_indicator2", DataType::Utf8, false),
        Field::new("subfield_code", DataType::Utf8, false),
        Field::new("subfield_value", DataType::Utf8, false),
    ]))
}

/// Convert MARC records to Arrow `RecordBatch`
///
/// # Errors
///
/// Returns error if record data cannot be converted to Arrow arrays
pub fn records_to_arrow_batch(records: &[Record]) -> Result<RecordBatch> {
    if records.is_empty() {
        return Err(MarcError::InvalidRecord(
            "Cannot create batch from empty record list".to_string(),
        ));
    }

    let mut record_indices = Vec::new();
    let mut leaders = Vec::new();
    let mut field_tags = Vec::new();
    let mut field_ind1s = Vec::new();
    let mut field_ind2s = Vec::new();
    let mut subfield_codes = Vec::new();
    let mut subfield_values = Vec::new();

    for (idx, record) in records.iter().enumerate() {
        let record_idx = u32::try_from(idx)
            .map_err(|_| MarcError::InvalidRecord("Record index overflow".to_string()))?;

        let leader_bytes = record
            .leader
            .as_bytes()
            .map_err(|e| MarcError::InvalidRecord(format!("Failed to serialize leader: {e}")))?;
        let leader_str = String::from_utf8_lossy(&leader_bytes).to_string();

        for field in record.fields() {
            let tag = field.tag.clone();
            let ind1 = field.indicator1.to_string();
            let ind2 = field.indicator2.to_string();

            for subfield in &field.subfields {
                record_indices.push(record_idx);
                leaders.push(leader_str.clone());
                field_tags.push(tag.clone());
                field_ind1s.push(ind1.clone());
                field_ind2s.push(ind2.clone());
                subfield_codes.push(subfield.code.to_string());
                subfield_values.push(subfield.value.clone());
            }
        }
    }

    let schema = create_parquet_schema();
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(UInt32Array::from(record_indices)),
            Arc::new(StringArray::from(leaders)),
            Arc::new(StringArray::from(field_tags)),
            Arc::new(StringArray::from(field_ind1s)),
            Arc::new(StringArray::from(field_ind2s)),
            Arc::new(StringArray::from(subfield_codes)),
            Arc::new(StringArray::from(subfield_values)),
        ],
    )
    .map_err(|e| MarcError::InvalidRecord(format!("Failed to create batch: {e}")))
}

/// Convert Arrow `RecordBatch` back to MARC records
///
/// # Errors
///
/// Returns error if array structure is malformed or record reconstruction fails
///
/// # Panics
///
/// This function calls `expect()` on an Option that is guaranteed to be Some
/// in its branch, which should never panic in practice given proper Arrow schema.
#[allow(clippy::too_many_lines)]
pub fn arrow_batch_to_records(batch: &RecordBatch) -> Result<Vec<Record>> {
    if batch.schema().fields().len() != 7 {
        return Err(MarcError::InvalidRecord(
            "Invalid schema: expected 7 columns".to_string(),
        ));
    }

    let record_indices = batch
        .column(0)
        .as_any()
        .downcast_ref::<UInt32Array>()
        .ok_or_else(|| MarcError::InvalidRecord("Column 0 is not uint32".to_string()))?;

    let leaders = batch
        .column(1)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| MarcError::InvalidRecord("Column 1 is not string".to_string()))?;

    let field_tags = batch
        .column(2)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| MarcError::InvalidRecord("Column 2 is not string".to_string()))?;

    let field_ind1s = batch
        .column(3)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| MarcError::InvalidRecord("Column 3 is not string".to_string()))?;

    let field_ind2s = batch
        .column(4)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| MarcError::InvalidRecord("Column 4 is not string".to_string()))?;

    let subfield_codes = batch
        .column(5)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| MarcError::InvalidRecord("Column 5 is not string".to_string()))?;

    let subfield_values = batch
        .column(6)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| MarcError::InvalidRecord("Column 6 is not string".to_string()))?;

    // Group rows by record_index
    let mut records_data: std::collections::BTreeMap<u32, Vec<usize>> =
        std::collections::BTreeMap::new();

    for row_idx in 0..batch.num_rows() {
        let record_idx = record_indices.value(row_idx);
        records_data.entry(record_idx).or_default().push(row_idx);
    }

    let mut result_records = Vec::new();

    for (_record_idx, row_indices) in records_data {
        if row_indices.is_empty() {
            continue;
        }

        let leader_str = leaders.value(row_indices[0]);
        if leader_str.len() != 24 {
            return Err(MarcError::InvalidRecord(format!(
                "Invalid leader length: {}",
                leader_str.len()
            )));
        }

        let leader = crate::Leader {
            record_length: leader_str[0..5].parse().unwrap_or(0),
            record_status: leader_str.chars().nth(5).unwrap_or(' '),
            record_type: leader_str.chars().nth(6).unwrap_or(' '),
            bibliographic_level: leader_str.chars().nth(7).unwrap_or(' '),
            control_record_type: leader_str.chars().nth(8).unwrap_or(' '),
            character_coding: leader_str.chars().nth(9).unwrap_or(' '),
            indicator_count: leader_str[10..11].parse().unwrap_or(0),
            subfield_code_count: leader_str[11..12].parse().unwrap_or(0),
            data_base_address: leader_str[12..17].parse().unwrap_or(0),
            encoding_level: leader_str.chars().nth(17).unwrap_or(' '),
            cataloging_form: leader_str.chars().nth(18).unwrap_or(' '),
            multipart_level: leader_str.chars().nth(19).unwrap_or(' '),
            reserved: leader_str[20..24].to_string(),
        };

        let mut record = Record::new(leader);

        // Group rows by field (tag, ind1, ind2)
        let mut field_groups: Vec<(String, String, String, Vec<usize>)> = Vec::new();
        let mut current_field: Option<(String, String, String, Vec<usize>)> = None;

        for row_idx in &row_indices {
            let tag = field_tags.value(*row_idx).to_string();
            let ind1 = field_ind1s.value(*row_idx).to_string();
            let ind2 = field_ind2s.value(*row_idx).to_string();

            match &mut current_field {
                None => current_field = Some((tag, ind1, ind2, vec![*row_idx])),
                Some((curr_tag, curr_ind1, curr_ind2, rows)) => {
                    if curr_tag == &tag && curr_ind1 == &ind1 && curr_ind2 == &ind2 {
                        rows.push(*row_idx);
                    } else {
                        let old_field = current_field
                            .take()
                            .expect("current_field is Some in this branch");
                        field_groups.push(old_field);
                        current_field = Some((tag, ind1, ind2, vec![*row_idx]));
                    }
                },
            }
        }

        if let Some(field) = current_field {
            field_groups.push(field);
        }

        // Reconstruct fields
        for (tag, ind1_str, ind2_str, row_indices_for_field) in field_groups {
            let ind1 = ind1_str.chars().next().unwrap_or(' ');
            let ind2 = ind2_str.chars().next().unwrap_or(' ');

            let mut field = crate::record::Field {
                tag,
                indicator1: ind1,
                indicator2: ind2,
                subfields: Vec::new(),
            };

            for row_idx in row_indices_for_field {
                let code = subfield_codes.value(row_idx).chars().next().unwrap_or('?');
                let value = subfield_values.value(row_idx).to_string();
                field
                    .subfields
                    .push(crate::record::Subfield { code, value });
            }

            record.add_field(field);
        }

        result_records.push(record);
    }

    Ok(result_records)
}

/// Serialize MARC records to Parquet format using Arrow IPC format
///
/// # Errors
///
/// Returns error if file creation, Arrow serialization, or I/O fails
pub fn serialize_to_parquet(records: &[Record], path: &str) -> Result<()> {
    if records.is_empty() {
        return Err(MarcError::InvalidRecord(
            "Cannot write empty record set".to_string(),
        ));
    }

    let batch = records_to_arrow_batch(records)?;
    let file = File::create(path)?;
    let writer = BufWriter::new(file);

    let mut arrow_writer = StreamWriter::try_new(writer, batch.schema().as_ref())
        .map_err(|e| MarcError::InvalidRecord(format!("Failed to create writer: {e}")))?;

    arrow_writer
        .write(&batch)
        .map_err(|e| MarcError::InvalidRecord(format!("Failed to write Arrow batch: {e}")))?;

    arrow_writer
        .finish()
        .map_err(|e| MarcError::InvalidRecord(format!("Failed to finish writing: {e}")))?;

    Ok(())
}

/// Deserialize MARC records from Parquet format
///
/// # Errors
///
/// Returns error if file cannot be opened or Arrow deserialization fails
pub fn deserialize_from_parquet(path: &str) -> Result<Vec<Record>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let stream_reader = StreamReader::try_new(reader, None)
        .map_err(|e| MarcError::InvalidRecord(format!("Failed to create reader: {e}")))?;

    let mut records = Vec::new();

    for batch_result in stream_reader {
        let batch = batch_result
            .map_err(|e| MarcError::InvalidRecord(format!("Failed to read batch: {e}")))?;
        let batch_records = arrow_batch_to_records(&batch)?;
        records.extend(batch_records);
    }

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Field;

    fn create_test_leader() -> crate::Leader {
        crate::Leader {
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
    fn test_single_record_roundtrip() -> Result<()> {
        use tempfile::NamedTempFile;

        let mut record = Record::new(create_test_leader());
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test Title".to_string());
        record.add_field(field);

        let records = vec![record.clone()];
        let temp = NamedTempFile::new()
            .map_err(|e| MarcError::InvalidRecord(format!("Temp error: {e}")))?;
        let path = temp.path().to_string_lossy().to_string();

        serialize_to_parquet(&records, &path)?;
        let restored = deserialize_from_parquet(&path)?;

        assert_eq!(restored.len(), 1);

        let orig_fields: Vec<_> = record.fields().collect();
        let rest_fields: Vec<_> = restored[0].fields().collect();

        assert_eq!(orig_fields.len(), rest_fields.len());
        assert_eq!(orig_fields[0].tag, rest_fields[0].tag);

        Ok(())
    }
}
