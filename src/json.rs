//! JSON serialization and deserialization of MARC records.
//!
//! This module provides conversion between MARC records and a generic JSON representation.
//! The JSON format is a simple, flat structure with fields as keys and values/subfields
//! as objects.
//!
//! # Examples
//!
//! ```ignore
//! use mrrc::{Record, Field, Leader, json};
//!
//! let mut record = Record::new(Leader::default());
//! let mut field = Field::new("245".to_string(), '1', '0');
//! field.add_subfield('a', "Title".to_string());
//! record.add_field(field);
//!
//! let json = json::record_to_json(&record)?;
//! println!("{}", json);
//!
//! let restored = json::json_to_record(&json)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::error::Result;
use crate::record::{Field, Record};
use serde_json::{json, Value};

/// Convert a MARC record to JSON.
///
/// Produces a JSON array where:
/// - First element is the leader
/// - Subsequent elements are control and data fields
/// - Each field is an object with the tag as key
///
/// # Examples
///
/// ```ignore
/// use mrrc::{Record, Field, Leader, json};
///
/// let mut record = Record::new(Leader::default());
/// let mut field = Field::new("245".to_string(), '1', '0');
/// field.add_subfield('a', "Test".to_string());
/// record.add_field(field);
///
/// let json_value = json::record_to_json(&record)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns an error if the record cannot be converted to JSON.
pub fn record_to_json(record: &Record) -> Result<Value> {
    let mut fields = Vec::new();

    // Add leader as first item
    fields.push(json!({
        "leader": String::from_utf8_lossy(&record.leader.as_bytes()?).to_string()
    }));

    // Add control fields
    for (tag, value) in &record.control_fields {
        fields.push(json!({
            tag: value
        }));
    }

    // Add data fields
    for (tag, field_list) in &record.fields {
        for field in field_list {
            let mut subfield_map = serde_json::Map::new();
            for subfield in &field.subfields {
                subfield_map.insert(
                    subfield.code.to_string(),
                    Value::String(subfield.value.clone()),
                );
            }

            let field_obj = json!({
                "ind1": field.indicator1.to_string(),
                "ind2": field.indicator2.to_string(),
                "subfields": subfield_map
            });

            let mut tag_obj = serde_json::Map::new();
            tag_obj.insert(tag.clone(), field_obj);
            fields.push(Value::Object(tag_obj));
        }
    }

    Ok(Value::Array(fields))
}

/// Convert JSON back to a MARC record.
///
/// Reverses the transformation performed by [`record_to_json`].
///
/// # Examples
///
/// ```ignore
/// use mrrc::{Record, Field, Leader, json};
///
/// let mut record = Record::new(Leader::default());
/// let mut field = Field::new("245".to_string(), '1', '0');
/// field.add_subfield('a', "Test".to_string());
/// record.add_field(field);
///
/// let json_value = json::record_to_json(&record)?;
/// let restored = json::json_to_record(&json_value)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns an error if the JSON is invalid or missing required fields.
pub fn json_to_record(json: &Value) -> Result<Record> {
    use crate::error::MarcError;
    use crate::leader::Leader;

    let array = json
        .as_array()
        .ok_or_else(|| MarcError::InvalidRecord("Expected JSON array".to_string()))?;

    if array.is_empty() {
        return Err(MarcError::InvalidRecord("Empty JSON array".to_string()));
    }

    // First item should be leader
    let leader_obj = array[0]
        .as_object()
        .ok_or_else(|| MarcError::InvalidRecord("First item must be object".to_string()))?;

    let leader_str = leader_obj
        .get("leader")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MarcError::InvalidRecord("Missing leader field".to_string()))?;

    let leader = Leader::from_bytes(leader_str.as_bytes())?;
    let mut record = Record::new(leader);

    // Process remaining fields
    for item in &array[1..] {
        let obj = item
            .as_object()
            .ok_or_else(|| MarcError::InvalidRecord("Field must be object".to_string()))?;

        for (tag, value) in obj {
            if tag.len() != 3 {
                continue; // Skip invalid tags
            }

            // Check if it's a control field (001-009)
            if tag.as_str() < "010" {
                if let Some(str_value) = value.as_str() {
                    record.add_control_field(tag.clone(), str_value.to_string());
                }
            } else {
                // Data field
                let field_obj = value.as_object().ok_or_else(|| {
                    MarcError::InvalidField(format!("Field {tag} must be object"))
                })?;

                let ind1 = field_obj
                    .get("ind1")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.chars().next())
                    .ok_or_else(|| MarcError::InvalidField("Missing ind1".to_string()))?;

                let ind2 = field_obj
                    .get("ind2")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.chars().next())
                    .ok_or_else(|| MarcError::InvalidField("Missing ind2".to_string()))?;

                let mut field = Field::new(tag.clone(), ind1, ind2);

                if let Some(subfields_obj) = field_obj.get("subfields").and_then(|v| v.as_object())
                {
                    for (code, value) in subfields_obj {
                        if let Some(code_char) = code.chars().next() {
                            if let Some(str_value) = value.as_str() {
                                field.add_subfield(code_char, str_value.to_string());
                            }
                        }
                    }
                }

                record.add_field(field);
            }
        }
    }

    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::leader::Leader;

    fn make_test_leader() -> Leader {
        Leader {
            record_length: 1000,
            record_status: 'n',
            record_type: 'a',
            bibliographic_level: 'm',
            control_record_type: ' ',
            character_coding: 'a',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 100,
            encoding_level: ' ',
            cataloging_form: ' ',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        }
    }

    #[test]
    fn test_record_to_json() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "12345".to_string());

        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test title".to_string());
        record.add_field(field);

        let json = record_to_json(&record).unwrap();
        let array = json.as_array().unwrap();

        assert!(array.len() >= 3); // leader, 001, 245
        assert!(array[0].get("leader").is_some());
    }

    #[test]
    fn test_json_roundtrip() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "12345".to_string());

        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test title".to_string());
        field.add_subfield('c', "Author".to_string());
        record.add_field(field);

        let json = record_to_json(&record).unwrap();
        let restored = json_to_record(&json).unwrap();

        assert_eq!(restored.get_control_field("001"), Some("12345"));
        let fields = restored.get_fields("245").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Test title"));
        assert_eq!(fields[0].get_subfield('c'), Some("Author"));
    }

    #[test]
    fn test_json_with_multiple_subfields() {
        let mut record = Record::new(make_test_leader());

        let mut field = Field::new("650".to_string(), ' ', '0');
        field.add_subfield('a', "Subject".to_string());
        field.add_subfield('v', "Subdivision".to_string());
        field.add_subfield('x', "General subdivision".to_string());
        record.add_field(field);

        let json = record_to_json(&record).unwrap();
        let restored = json_to_record(&json).unwrap();

        let fields = restored.get_fields("650").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Subject"));
        assert_eq!(fields[0].get_subfield('v'), Some("Subdivision"));
        assert_eq!(fields[0].get_subfield('x'), Some("General subdivision"));
    }

    #[test]
    fn test_json_with_multiple_fields_same_tag() {
        let mut record = Record::new(make_test_leader());

        for i in 1..=3 {
            let mut field = Field::new("650".to_string(), ' ', '0');
            field.add_subfield('a', format!("Subject {i}"));
            record.add_field(field);
        }

        let json = record_to_json(&record).unwrap();
        let restored = json_to_record(&json).unwrap();

        let fields = restored.get_fields("650").unwrap();
        assert_eq!(fields.len(), 3);
    }
}
