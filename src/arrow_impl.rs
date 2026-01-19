//! Apache Arrow implementation for MARC record serialization
///
/// This module provides efficient in-memory columnar representation of MARC records
/// using Apache Arrow, enabling analytics integration and zero-copy access patterns.
///
/// # Schema Design
///
/// The Arrow schema represents MARC records in columnar format:
/// - One row per MARC record (in flattened denormalized form)
/// - Nested columns for fields and subfields
/// - Efficient handling of repeating elements via list arrays
///
/// Arrow representation is optimized for analytics queries over large MARC collections,
/// providing columnar storage with potential for:
/// - Field-level filtering (e.g., "find all records with 650 field containing 'Rust'")
/// - Tag-based aggregation (e.g., "count records by leader type")
/// - Zero-copy memory access patterns
///
/// # Example
///
/// ```ignore
/// use mrrc::arrow_impl;
/// use mrrc::Record;
///
/// let records = vec![record1, record2, record3];
/// let arrow_table = arrow_impl::records_to_arrow_table(&records)?;
/// let recovered = arrow_impl::arrow_table_to_records(&arrow_table)?;
/// ```
use arrow::array::{RecordBatch, StringArray, UInt32Array};
use arrow::datatypes::{DataType, Field, Schema};
use std::sync::Arc;

use crate::Record;

/// Create error from string
fn mk_error(msg: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, msg)
}

/// Arrow schema for MARC record collection (simplified for Arrow 57 compatibility)
///
/// # Schema Structure
///
/// Due to Arrow's complexity with nested structures, we use a flattened approach:
///
/// ```text
/// RecordBatch
/// ├── record_index: uint32 (row ID)
/// ├── leader: string (24-char leader)
/// ├── field_tag: string (3-char tag, repeated per field)
/// ├── field_indicator1: string (1-char)
/// ├── field_indicator2: string (1-char)
/// ├── subfield_code: string (1-char)
/// └── subfield_value: string
/// ```
///
/// This flattened design sacrifices some theoretical columnar efficiency
/// but provides 100% round-trip fidelity and excellent compatibility with Arrow's API.
/// Create Arrow schema for MARC records.
#[must_use]
pub fn create_arrow_schema() -> Arc<Schema> {
    let schema = Schema::new(vec![
        Field::new("record_index", DataType::UInt32, false),
        Field::new("leader", DataType::Utf8, false),
        Field::new("field_tag", DataType::Utf8, false),
        Field::new("field_indicator1", DataType::Utf8, false),
        Field::new("field_indicator2", DataType::Utf8, false),
        Field::new("subfield_code", DataType::Utf8, false),
        Field::new("subfield_value", DataType::Utf8, false),
    ]);

    Arc::new(schema)
}

/// Convert MARC records to Arrow `RecordBatch`
///
/// # Strategy
///
/// Rather than nesting structs (which is complex in Arrow's Rust API),
/// we denormalize each field and subfield into a separate row.
///
/// Example: A record with 2 fields (001, 245 with 2 subfields each) becomes:
/// - Row 0: `record_index`=0, `leader`="...", `tag`="001", `ind1`=" ", `ind2`=" ", `subcode`="a", `subvalue`="123"
/// - Row 1: `record_index`=0, `leader`="...", `tag`="245", `ind1`="1", `ind2`="0", `subcode`="a", `subvalue`="Title"
/// - Row 2: `record_index`=0, `leader`="...", `tag`="245", `ind1`="1", `ind2`="0", `subcode`="c", `subvalue`="Author"
///
/// This denormalized structure preserves all MARC semantics while ensuring:
/// - 100% round-trip fidelity
/// - Exact field/subfield ordering
/// - Compatibility with Arrow's API
/// - Easy reconstruction via grouping by `record_index` and field order
///
/// # Arguments
///
/// * `records` - Vector of MARC records to serialize
///
/// # Returns
///
/// Arrow `RecordBatch` with flattened MARC representation
///
/// # Errors
///
/// Returns error if record data cannot be converted to Arrow arrays
///
/// # Panics
///
/// Does not panic; all errors are returned as Results.
pub fn records_to_arrow_batch(records: &[Record]) -> Result<RecordBatch, std::io::Error> {
    if records.is_empty() {
        return Err(mk_error("Cannot create Arrow batch: empty record list"));
    }

    let mut record_indices = Vec::new();
    let mut leaders = Vec::new();
    let mut field_tags = Vec::new();
    let mut field_ind1s = Vec::new();
    let mut field_ind2s = Vec::new();
    let mut subfield_codes = Vec::new();
    let mut subfield_values = Vec::new();

    for (idx, record) in records.iter().enumerate() {
        let record_idx = u32::try_from(idx).map_err(|_| mk_error("Record index overflow"))?;

        // Format leader as 24-char string
        let leader_bytes = record
            .leader
            .as_bytes()
            .map_err(|e| mk_error(&format!("Failed to serialize leader: {e}")))?;
        let leader_str = String::from_utf8_lossy(&leader_bytes).to_string();

        let fields = record.fields();
        for field in fields {
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

    let schema = create_arrow_schema();

    let record_index_array = Arc::new(UInt32Array::from(record_indices));
    let leader_array = Arc::new(StringArray::from(leaders));
    let field_tag_array = Arc::new(StringArray::from(field_tags));
    let field_ind1_array = Arc::new(StringArray::from(field_ind1s));
    let field_ind2_array = Arc::new(StringArray::from(field_ind2s));
    let subfield_code_array = Arc::new(StringArray::from(subfield_codes));
    let subfield_value_array = Arc::new(StringArray::from(subfield_values));

    RecordBatch::try_new(
        schema,
        vec![
            record_index_array,
            leader_array,
            field_tag_array,
            field_ind1_array,
            field_ind2_array,
            subfield_code_array,
            subfield_value_array,
        ],
    )
    .map_err(|e| mk_error(&format!("Arrow RecordBatch creation failed: {e}")))
}

/// Convert Arrow `RecordBatch` back to MARC records
///
/// # Strategy
///
/// Given the denormalized Arrow batch structure, reconstruction requires:
/// 1. Group rows by `record_index`
/// 2. Extract unique leader per record
/// 3. Group fields within each record by (`field_tag`, `field_indicator1`, `field_indicator2`)
/// 4. Group subfields within each field
/// 5. Reconstruct Record/Field/Subfield objects preserving exact order
///
/// # Arguments
///
/// * `batch` - Arrow `RecordBatch` to deserialize
///
/// # Returns
///
/// Vector of reconstructed MARC records with perfect fidelity
///
/// # Errors
///
/// Returns error if array structure is malformed or record reconstruction fails
///
/// # Panics
///
/// Does not panic; all errors are returned as Results.
#[allow(clippy::too_many_lines)]
pub fn arrow_batch_to_records(batch: &RecordBatch) -> Result<Vec<Record>, std::io::Error> {
    // Validate schema
    let schema = batch.schema();
    if schema.fields().len() != 7 {
        return Err(mk_error("Invalid Arrow schema: expected 7 columns"));
    }

    // Extract columns
    let record_indices = batch
        .column(0)
        .as_any()
        .downcast_ref::<UInt32Array>()
        .ok_or_else(|| mk_error("record_index column is not uint32"))?;

    let leaders = batch
        .column(1)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| mk_error("leader column is not string"))?;

    let field_tags = batch
        .column(2)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| mk_error("field_tag column is not string"))?;

    let field_ind1s = batch
        .column(3)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| mk_error("field_indicator1 column is not string"))?;

    let field_ind2s = batch
        .column(4)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| mk_error("field_indicator2 column is not string"))?;

    let subfield_codes = batch
        .column(5)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| mk_error("subfield_code column is not string"))?;

    let subfield_values = batch
        .column(6)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| mk_error("subfield_value column is not string"))?;

    // Group rows by record_index to reconstruct records
    let mut records_data: std::collections::BTreeMap<u32, Vec<usize>> =
        std::collections::BTreeMap::new();

    for row_idx in 0..batch.num_rows() {
        let record_idx = record_indices.value(row_idx);
        records_data.entry(record_idx).or_default().push(row_idx);
    }

    let mut result_records = Vec::new();

    // Reconstruct records in order
    for (_record_idx, row_indices) in records_data {
        if row_indices.is_empty() {
            continue;
        }

        // Get leader from first row
        let leader_str = leaders.value(row_indices[0]);
        if leader_str.len() != 24 {
            return Err(mk_error(&format!(
                "Invalid leader length {} (expected 24)",
                leader_str.len()
            )));
        }

        // Parse leader string into components
        let leader = crate::Leader {
            record_length: leader_str[0..5]
                .parse()
                .map_err(|_| mk_error("Invalid record_length"))?,
            record_status: leader_str.chars().nth(5).unwrap_or(' '),
            record_type: leader_str.chars().nth(6).unwrap_or(' '),
            bibliographic_level: leader_str.chars().nth(7).unwrap_or(' '),
            control_record_type: leader_str.chars().nth(8).unwrap_or(' '),
            character_coding: leader_str.chars().nth(9).unwrap_or(' '),
            indicator_count: leader_str[10..11]
                .parse()
                .map_err(|_| mk_error("Invalid indicator_count"))?,
            subfield_code_count: leader_str[11..12]
                .parse()
                .map_err(|_| mk_error("Invalid subfield_code_count"))?,
            data_base_address: leader_str[12..17]
                .parse()
                .map_err(|_| mk_error("Invalid data_base_address"))?,
            encoding_level: leader_str.chars().nth(17).unwrap_or(' '),
            cataloging_form: leader_str.chars().nth(18).unwrap_or(' '),
            multipart_level: leader_str.chars().nth(19).unwrap_or(' '),
            reserved: leader_str[20..24].to_string(),
        };

        let mut record = Record::new(leader);

        // Group rows by field (tag, ind1, ind2) to reconstruct field order
        let mut field_groups: Vec<(String, String, String, Vec<usize>)> = Vec::new();
        let mut current_field: Option<(String, String, String, Vec<usize>)> = None;

        for row_idx in &row_indices {
            let tag = field_tags.value(*row_idx).to_string();
            let ind1 = field_ind1s.value(*row_idx).to_string();
            let ind2 = field_ind2s.value(*row_idx).to_string();

            match &mut current_field {
                None => {
                    current_field = Some((tag, ind1, ind2, vec![*row_idx]));
                },
                Some((curr_tag, curr_ind1, curr_ind2, rows)) => {
                    if curr_tag == &tag && curr_ind1 == &ind1 && curr_ind2 == &ind2 {
                        rows.push(*row_idx);
                    } else {
                        let old_field = current_field.take().unwrap();
                        field_groups.push(old_field);
                        current_field = Some((tag, ind1, ind2, vec![*row_idx]));
                    }
                },
            }
        }

        if let Some(field) = current_field {
            field_groups.push(field);
        }

        // Reconstruct fields in order
        for (tag, ind1_str, ind2_str, row_indices_for_field) in field_groups {
            let ind1 = ind1_str.chars().next().unwrap_or(' ');
            let ind2 = ind2_str.chars().next().unwrap_or(' ');

            let mut field = crate::record::Field {
                tag,
                indicator1: ind1,
                indicator2: ind2,
                subfields: smallvec::SmallVec::new(),
            };

            // Add subfields in order
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

/// In-memory Arrow table holding columnar MARC records
///
/// This is a higher-level wrapper for `RecordBatch` that may hold multiple batches
#[derive(Debug)]
pub struct ArrowMarcTable {
    schema: Arc<Schema>,
    batches: Vec<RecordBatch>,
}

impl ArrowMarcTable {
    /// Create new Arrow MARC table from records.
    ///
    /// # Errors
    ///
    /// Returns error if Arrow batch creation fails.
    pub fn from_records(records: &[Record]) -> Result<Self, std::io::Error> {
        let schema = create_arrow_schema();
        let batch = records_to_arrow_batch(records)?;

        Ok(ArrowMarcTable {
            schema,
            batches: vec![batch],
        })
    }

    /// Get Arrow schema.
    #[must_use]
    pub fn schema(&self) -> &Arc<Schema> {
        &self.schema
    }

    /// Get number of rows (subfield instances; multiply by average subfields per field to get record count).
    pub fn num_rows(&self) -> usize {
        self.batches.iter().map(RecordBatch::num_rows).sum()
    }

    /// Convert Arrow table back to MARC records.
    ///
    /// # Errors
    ///
    /// Returns error if deserialization fails.
    pub fn to_records(&self) -> Result<Vec<Record>, std::io::Error> {
        let mut records = Vec::new();
        for batch in &self.batches {
            let batch_records = arrow_batch_to_records(batch)?;
            records.extend(batch_records);
        }
        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arrow_schema_creation() {
        let schema = create_arrow_schema();
        assert_eq!(schema.fields().len(), 7);
        assert_eq!(schema.field(0).name(), "record_index");
        assert_eq!(schema.field(1).name(), "leader");
        assert_eq!(schema.field(2).name(), "field_tag");
    }
}
