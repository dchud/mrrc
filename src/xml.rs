//! XML serialization and deserialization of MARC records.
//!
//! This module provides conversion between MARC records and MARCXML format,
//! which is the standard XML representation of MARC records used in libraries.
//!
//! # Examples
//!
//! ```ignore
//! use mrrc::{Record, Field, Leader, xml};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut record = Record::new(Leader::default());
//! let mut field = Field::new("245".to_string(), '1', '0');
//! field.add_subfield('a', "Title".to_string());
//! record.add_field(field);
//!
//! let xml_str = xml::record_to_xml(&record)?;
//! let restored = xml::xml_to_record(&xml_str)?;
//! # Ok(())
//! # }
//! ```

use crate::error::{MarcError, Result};
use crate::leader::Leader;
use crate::record::{Field, Record};
use quick_xml::de::from_str as xml_from_str;
use quick_xml::se::to_string as xml_to_string;
use serde::{Deserialize, Serialize};

/// MARCXML record representation for serialization.
#[derive(Debug, Serialize, Deserialize)]
pub struct MarcXmlRecord {
    /// MARC leader string
    pub leader: String,
    /// Control fields (tags 001-009)
    #[serde(default)]
    pub controlfield: Vec<MarcXmlControlField>,
    /// Data fields (tags 010+)
    #[serde(default)]
    pub datafield: Vec<MarcXmlDataField>,
}

/// MARCXML control field representation
#[derive(Debug, Serialize, Deserialize)]
pub struct MarcXmlControlField {
    /// Field tag (e.g., "001", "008")
    pub tag: String,
    /// Control field value
    #[serde(rename = "$value")]
    pub value: String,
}

/// MARCXML data field representation
#[derive(Debug, Serialize, Deserialize)]
pub struct MarcXmlDataField {
    /// Field tag (e.g., "245", "650")
    pub tag: String,
    /// First indicator
    pub ind1: String,
    /// Second indicator
    pub ind2: String,
    /// Subfields
    #[serde(default)]
    pub subfield: Vec<MarcXmlSubfield>,
}

/// MARCXML subfield representation
#[derive(Debug, Serialize, Deserialize)]
pub struct MarcXmlSubfield {
    /// Subfield code (e.g., 'a', 'b', 'c')
    pub code: String,
    /// Subfield value
    #[serde(rename = "$value")]
    pub value: String,
}

/// Convert a MARC record to MARCXML string.
///
/// Serializes a MARC record into MARCXML format, which is the standard XML
/// representation of MARC records. The leader is converted to a string, and all
/// control fields and data fields with their indicators and subfields are included.
///
/// # Arguments
///
/// * `record` - The MARC record to convert
///
/// # Returns
///
/// A string containing the MARCXML representation, or an error if serialization fails.
///
/// # Example
///
/// ```ignore
/// # use mrrc::{Record, Field, Leader, xml};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut record = Record::new(Leader::default());
/// let mut field = Field::new("245".to_string(), '1', '0');
/// field.add_subfield('a', "Test Title".to_string());
/// record.add_field(field);
/// let xml = xml::record_to_xml(&record)?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if the record cannot be serialized to XML.
pub fn record_to_xml(record: &Record) -> Result<String> {
    let leader_bytes = record.leader.as_bytes()?;
    let leader_str = String::from_utf8_lossy(&leader_bytes).to_string();

    let mut controlfields = Vec::new();
    for (tag, value) in &record.control_fields {
        controlfields.push(MarcXmlControlField {
            tag: tag.clone(),
            value: value.clone(),
        });
    }

    let mut datafields = Vec::new();
    for (tag, field_list) in &record.fields {
        for field in field_list {
            let mut subfields = Vec::new();
            for subfield in &field.subfields {
                subfields.push(MarcXmlSubfield {
                    code: subfield.code.to_string(),
                    value: subfield.value.clone(),
                });
            }

            datafields.push(MarcXmlDataField {
                tag: tag.clone(),
                ind1: field.indicator1.to_string(),
                ind2: field.indicator2.to_string(),
                subfield: subfields,
            });
        }
    }

    let xml_record = MarcXmlRecord {
        leader: leader_str,
        controlfield: controlfields,
        datafield: datafields,
    };

    xml_to_string(&xml_record)
        .map_err(|e| MarcError::ParseError(format!("Failed to serialize to XML: {e}")))
}

/// Convert MARCXML string to a MARC record.
///
/// Deserializes a MARCXML string into a MARC record. The function parses the XML,
/// reconstructs the leader, and repopulates all control fields and data fields with
/// their indicators and subfields.
///
/// # Arguments
///
/// * `xml` - The MARCXML string to parse
///
/// # Returns
///
/// A MARC Record, or an error if parsing or deserialization fails.
///
/// # Example
///
/// ```ignore
/// # use mrrc::xml;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let xml_str = r#"<record><leader>...</leader></record>"#;
/// let record = xml::xml_to_record(xml_str)?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if the XML is invalid or missing required elements.
pub fn xml_to_record(xml: &str) -> Result<Record> {
    let xml_record: MarcXmlRecord = xml_from_str(xml)
        .map_err(|e| MarcError::ParseError(format!("Failed to parse XML: {e}")))?;

    let leader = Leader::from_bytes(xml_record.leader.as_bytes())?;
    let mut record = Record::new(leader);

    // Add control fields
    for cf in xml_record.controlfield {
        record.add_control_field(cf.tag, cf.value);
    }

    // Add data fields
    for df in xml_record.datafield {
        let ind1 = df.ind1.chars().next().unwrap_or(' ');
        let ind2 = df.ind2.chars().next().unwrap_or(' ');

        let mut field = Field::new(df.tag, ind1, ind2);

        for sf in df.subfield {
            let code = sf
                .code
                .chars()
                .next()
                .ok_or_else(|| MarcError::InvalidField("Missing subfield code".to_string()))?;
            field.add_subfield(code, sf.value);
        }

        record.add_field(field);
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
    fn test_record_to_xml() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "12345".to_string());

        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test title".to_string());
        record.add_field(field);

        let xml = record_to_xml(&record).unwrap();
        eprintln!("Generated XML:\n{xml}");
        assert!(xml.contains("<leader>"));
        assert!(xml.contains("001") && xml.contains("12345"));
        assert!(xml.contains("<datafield") && xml.contains("245"));
        assert!(xml.contains('1') && xml.contains('0')); // indicators
        assert!(xml.contains("Test title"));
    }

    #[test]
    fn test_xml_roundtrip() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "12345".to_string());

        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test title".to_string());
        field.add_subfield('c', "Author".to_string());
        record.add_field(field);

        let xml = record_to_xml(&record).unwrap();
        let restored = xml_to_record(&xml).unwrap();

        assert_eq!(restored.get_control_field("001"), Some("12345"));
        let fields = restored.get_fields("245").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Test title"));
        assert_eq!(fields[0].get_subfield('c'), Some("Author"));
    }

    #[test]
    fn test_xml_with_multiple_subfields() {
        let mut record = Record::new(make_test_leader());

        let mut field = Field::new("650".to_string(), ' ', '0');
        field.add_subfield('a', "Subject".to_string());
        field.add_subfield('v', "Subdivision".to_string());
        field.add_subfield('x', "General subdivision".to_string());
        record.add_field(field);

        let xml = record_to_xml(&record).unwrap();
        let restored = xml_to_record(&xml).unwrap();

        let fields = restored.get_fields("650").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Subject"));
        assert_eq!(fields[0].get_subfield('v'), Some("Subdivision"));
        assert_eq!(fields[0].get_subfield('x'), Some("General subdivision"));
    }

    #[test]
    fn test_xml_with_multiple_fields_same_tag() {
        let mut record = Record::new(make_test_leader());

        for i in 1..=3 {
            let mut field = Field::new("650".to_string(), ' ', '0');
            field.add_subfield('a', format!("Subject {i}"));
            record.add_field(field);
        }

        let xml = record_to_xml(&record).unwrap();
        let restored = xml_to_record(&xml).unwrap();

        let fields = restored.get_fields("650").unwrap();
        assert_eq!(fields.len(), 3);
    }
}
