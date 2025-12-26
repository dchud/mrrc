//! MARCJSON serialization and deserialization of MARC records.
//!
//! MARCJSON is the standard JSON-LD format for MARC records used in the library community.
//! It provides a structured representation suitable for APIs and web services.
//!
//! # Format
//!
//! - Leader is a special field with key "leader"
//! - Control fields (001-009): `{tag: value}`
//! - Data fields (010+): `{tag: {ind1, ind2, subfields: [{code: value}, ...]}}`

use crate::error::{MarcError, Result};
use crate::leader::Leader;
use crate::record::{Field, Record};
use serde_json::{json, Value};

/// Convert a MARC record to MARCJSON format.
///
/// MARCJSON is a standard JSON-LD interchange format for MARC records.
/// It's widely used in library systems for API communication.
///
/// # Examples
///
/// ```ignore
/// use mrrc::{Record, Field, Leader, marcjson};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut record = Record::new(Leader::default());
/// let mut field = Field::new("245".to_string(), '1', '0');
/// field.add_subfield('a', "Title".to_string());
/// record.add_field(field);
///
/// let json = marcjson::record_to_marcjson(&record)?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if the record cannot be converted to MARCJSON.
pub fn record_to_marcjson(record: &Record) -> Result<Value> {
    let mut fields = Vec::new();

    // Add leader as a special field
    let leader_bytes = record.leader.as_bytes()?;
    let leader_str = String::from_utf8_lossy(&leader_bytes).to_string();
    fields.push(json!({
        "leader": leader_str
    }));

    // Add control fields (001-009)
    for (tag, value) in &record.control_fields {
        let mut field = serde_json::Map::new();
        field.insert(tag.clone(), Value::String(value.clone()));
        fields.push(Value::Object(field));
    }

    // Add data fields (010+)
    for (tag, field_list) in &record.fields {
        for field in field_list {
            let mut subfields = Vec::new();
            for subfield in &field.subfields {
                let mut sf = serde_json::Map::new();
                sf.insert(
                    subfield.code.to_string(),
                    Value::String(subfield.value.clone()),
                );
                subfields.push(Value::Object(sf));
            }

            let mut field_data = serde_json::Map::new();
            field_data.insert(
                "ind1".to_string(),
                Value::String(field.indicator1.to_string()),
            );
            field_data.insert(
                "ind2".to_string(),
                Value::String(field.indicator2.to_string()),
            );
            field_data.insert("subfields".to_string(), Value::Array(subfields));

            let mut field_obj = serde_json::Map::new();
            field_obj.insert(tag.clone(), Value::Object(field_data));
            fields.push(Value::Object(field_obj));
        }
    }

    Ok(Value::Array(fields))
}

/// Convert MARCJSON format to a MARC record
///
/// # Errors
///
/// Returns an error if the MARCJSON is invalid or missing required fields.
pub fn marcjson_to_record(json: &Value) -> Result<Record> {
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
                continue;
            }

            // Check if it's a control field (001-009)
            if tag.as_str() < "010" {
                if let Some(str_value) = value.as_str() {
                    record.add_control_field(tag.clone(), str_value.to_string());
                }
            } else {
                // Data field with indicators and subfields
                let field_obj = value.as_object().ok_or_else(|| {
                    MarcError::InvalidField(format!("Field {tag} must be object"))
                })?;

                let ind1 = field_obj
                    .get("ind1")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.chars().next())
                    .unwrap_or(' ');

                let ind2 = field_obj
                    .get("ind2")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.chars().next())
                    .unwrap_or(' ');

                let mut field = Field::new(tag.clone(), ind1, ind2);

                if let Some(subfields_arr) = field_obj.get("subfields").and_then(|v| v.as_array()) {
                    for sf in subfields_arr {
                        if let Some(sf_obj) = sf.as_object() {
                            for (code, value) in sf_obj {
                                if let Some(code_char) = code.chars().next() {
                                    if let Some(str_value) = value.as_str() {
                                        field.add_subfield(code_char, str_value.to_string());
                                    }
                                }
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
    fn test_record_to_marcjson() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "12345".to_string());

        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test title".to_string());
        record.add_field(field);

        let json = record_to_marcjson(&record).unwrap();
        let array = json.as_array().unwrap();

        assert!(array.len() >= 3); // leader, 001, 245
        assert!(array[0].get("leader").is_some());

        // Verify control field
        let cf = array[1].as_object().unwrap();
        assert_eq!(cf.get("001").and_then(|v| v.as_str()), Some("12345"));

        // Verify data field
        let df = array[2].as_object().unwrap();
        let field_245 = df.get("245").and_then(|v| v.as_object()).unwrap();
        assert_eq!(field_245.get("ind1").and_then(|v| v.as_str()), Some("1"));
        assert_eq!(field_245.get("ind2").and_then(|v| v.as_str()), Some("0"));
    }

    #[test]
    fn test_marcjson_roundtrip() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "12345".to_string());

        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test title".to_string());
        field.add_subfield('c', "Author".to_string());
        record.add_field(field);

        let json = record_to_marcjson(&record).unwrap();
        let restored = marcjson_to_record(&json).unwrap();

        assert_eq!(restored.get_control_field("001"), Some("12345"));
        let fields = restored.get_fields("245").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Test title"));
        assert_eq!(fields[0].get_subfield('c'), Some("Author"));
    }

    #[test]
    fn test_marcjson_with_multiple_subfields() {
        let mut record = Record::new(make_test_leader());

        let mut field = Field::new("650".to_string(), ' ', '0');
        field.add_subfield('a', "Subject".to_string());
        field.add_subfield('v', "Subdivision".to_string());
        field.add_subfield('x', "General subdivision".to_string());
        record.add_field(field);

        let json = record_to_marcjson(&record).unwrap();
        let restored = marcjson_to_record(&json).unwrap();

        let fields = restored.get_fields("650").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Subject"));
        assert_eq!(fields[0].get_subfield('v'), Some("Subdivision"));
        assert_eq!(fields[0].get_subfield('x'), Some("General subdivision"));
    }

    #[test]
    fn test_marcjson_with_multiple_fields_same_tag() {
        let mut record = Record::new(make_test_leader());

        for i in 1..=3 {
            let mut field = Field::new("650".to_string(), ' ', '0');
            field.add_subfield('a', format!("Subject {i}"));
            record.add_field(field);
        }

        let json = record_to_marcjson(&record).unwrap();
        let restored = marcjson_to_record(&json).unwrap();

        let fields = restored.get_fields("650").unwrap();
        assert_eq!(fields.len(), 3);
    }

    #[test]
    fn test_marcjson_with_space_indicators() {
        let mut record = Record::new(make_test_leader());

        let mut field = Field::new("500".to_string(), ' ', ' ');
        field.add_subfield('a', "General note".to_string());
        record.add_field(field);

        let json = record_to_marcjson(&record).unwrap();
        let restored = marcjson_to_record(&json).unwrap();

        let fields = restored.get_fields("500").unwrap();
        assert_eq!(fields[0].indicator1, ' ');
        assert_eq!(fields[0].indicator2, ' ');
        assert_eq!(fields[0].get_subfield('a'), Some("General note"));
    }
}
