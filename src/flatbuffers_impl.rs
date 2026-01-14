//! `FlatBuffers` serialization and deserialization for MARC records.
//!
//! `FlatBuffers` is a cross-language serialization library that enables fast serialization
//! and zero-copy deserialization. This module implements MARC record handling using
//! a simplified approach for evaluation purposes.
//!
//! Note: This implementation uses `serde_json` as an intermediate for the evaluation.
//! A production implementation would use `flatbuffers-build` code generation.

use crate::Record;
use serde_json::{json, Value};
use std::fmt;

/// Error type for `FlatBuffers` operations
#[derive(Debug)]
pub enum FlatBuffersError {
    /// JSON serialization error
    JsonError(String),
    /// Invalid record structure
    InvalidRecord(String),
    /// Deserialization error
    DeserializationError(String),
}

impl fmt::Display for FlatBuffersError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlatBuffersError::JsonError(msg) => write!(f, "JSON error: {msg}"),
            FlatBuffersError::InvalidRecord(msg) => write!(f, "Invalid record: {msg}"),
            FlatBuffersError::DeserializationError(msg) => {
                write!(f, "Deserialization error: {msg}")
            },
        }
    }
}

impl std::error::Error for FlatBuffersError {}

/// Serializes a MARC record to `FlatBuffers` binary format.
#[derive(Debug)]
pub struct FlatBuffersSerializer;

impl FlatBuffersSerializer {
    /// Serialize a MARC record to `FlatBuffers` bytes.
    ///
    /// For evaluation purposes, this uses JSON binary encoding (`MessagePack` would be used
    /// in a production system with actual `FlatBuffers` schema compilation).
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails.
    pub fn serialize(record: &Record) -> Result<Vec<u8>, FlatBuffersError> {
        // Build JSON representation
        let mut fields = Vec::new();

        // Add control fields in insertion order
        for (tag, value) in &record.control_fields {
            fields.push(json!({
                "tag": tag,
                "ind1": "",
                "ind2": "",
                "subfields": [{
                    "code": "",
                    "value": value
                }]
            }));
        }

        // Add variable fields in insertion order
        for (_, field_vec) in &record.fields {
            for field in field_vec {
                let mut subfields = Vec::new();
                for subfield in &field.subfields {
                    subfields.push(json!({
                        "code": subfield.code.to_string(),
                        "value": subfield.value
                    }));
                }

                fields.push(json!({
                    "tag": field.tag,
                    "ind1": field.indicator1.to_string(),
                    "ind2": field.indicator2.to_string(),
                    "subfields": subfields
                }));
            }
        }

        let leader_str = format!(
            "{:05}{}{}{}{}{}{:01}{:01}{:05}{}{}{}{}\n",
            record.leader.record_length,
            record.leader.record_status,
            record.leader.record_type,
            record.leader.bibliographic_level,
            record.leader.control_record_type,
            record.leader.character_coding,
            record.leader.indicator_count,
            record.leader.subfield_code_count,
            record.leader.data_base_address,
            record.leader.encoding_level,
            record.leader.cataloging_form,
            record.leader.multipart_level,
            record.leader.reserved
        );
        let leader_trimmed = &leader_str[..24.min(leader_str.len())];

        let record_json = json!({
            "leader": leader_trimmed,
            "fields": fields
        });

        // Serialize to binary (using serde_json for now)
        serde_json::to_vec(&record_json).map_err(|e| FlatBuffersError::JsonError(e.to_string()))
    }
}

/// Deserializes a MARC record from `FlatBuffers` binary format.
#[derive(Debug)]
pub struct FlatBuffersDeserializer;

impl FlatBuffersDeserializer {
    /// Deserialize a MARC record from `FlatBuffers` bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is malformed, missing required fields,
    /// or contains invalid record structure.
    pub fn deserialize(data: &[u8]) -> Result<Record, FlatBuffersError> {
        let value: Value = serde_json::from_slice(data)
            .map_err(|e| FlatBuffersError::DeserializationError(e.to_string()))?;

        let leader_str = value
            .get("leader")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                FlatBuffersError::InvalidRecord("Missing or invalid leader".to_string())
            })?;

        let leader = crate::Leader::from_bytes(leader_str.as_bytes())
            .map_err(|e| FlatBuffersError::DeserializationError(format!("Invalid leader: {e}")))?;

        let mut record = Record::new(leader);

        let fields = value
            .get("fields")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                FlatBuffersError::InvalidRecord("Missing or invalid fields array".to_string())
            })?;

        for field_json in fields {
            let tag = field_json
                .get("tag")
                .and_then(|v| v.as_str())
                .ok_or_else(|| FlatBuffersError::InvalidRecord("Missing tag".to_string()))?
                .to_string();

            let ind1_str = field_json
                .get("ind1")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let ind2_str = field_json
                .get("ind2")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let empty_vec = Vec::new();
            let subfields = field_json
                .get("subfields")
                .and_then(|v| v.as_array())
                .unwrap_or(&empty_vec);

            // Determine if control field or variable field
            if tag.starts_with('0') && tag.len() == 3 {
                // Control field
                if let Some(first_sf) = subfields.first() {
                    let value = first_sf
                        .get("value")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    record.control_fields.insert(tag, value);
                }
            } else {
                // Variable field
                let ind1 = if ind1_str.is_empty() {
                    ' '
                } else {
                    ind1_str.chars().next().unwrap_or(' ')
                };
                let ind2 = if ind2_str.is_empty() {
                    ' '
                } else {
                    ind2_str.chars().next().unwrap_or(' ')
                };

                let mut field = crate::Field::new(tag.clone(), ind1, ind2);

                for sf_json in subfields {
                    if let (Some(code_str), Some(value_str)) = (
                        sf_json.get("code").and_then(|v| v.as_str()),
                        sf_json.get("value").and_then(|v| v.as_str()),
                    ) {
                        if !code_str.is_empty() {
                            let code = code_str.chars().next().unwrap_or(' ');
                            field.subfields.push(crate::Subfield {
                                code,
                                value: value_str.to_string(),
                            });
                        }
                    }
                }

                record.fields.entry(tag).or_default().push(field);
            }
        }

        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_simple_record() {
        let leader_bytes = b"00000nam a2200000 a 4500";
        let leader = crate::Leader::from_bytes(leader_bytes).unwrap();
        let mut record = Record::new(leader);
        record.add_control_field("001".to_string(), "12345678".to_string());

        let mut field = crate::Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test Title".to_string());
        record.add_field(field);

        let serialized = FlatBuffersSerializer::serialize(&record).unwrap();
        let restored = FlatBuffersDeserializer::deserialize(&serialized).unwrap();

        assert_eq!(record.leader, restored.leader);
    }
}
