//! MARCXML serialization and deserialization of MARC records.
//!
//! This module provides conversion between MARC records and standard MARCXML format,
//! as defined by the Library of Congress (<https://www.loc.gov/standards/marcxml/>).
//!
//! The output conforms to LOC's MARCXML schema: `tag`, `ind1`, `ind2`, and `code`
//! are serialized as XML **attributes**, and the root `<record>` element includes the
//! `xmlns="http://www.loc.gov/MARC21/slim"` namespace declaration.
//!
//! For deserialization, both default-namespace (`<record xmlns="...">`) and
//! prefix-namespace (`<marc:record xmlns:marc="...">`) forms are accepted.
//!
//! # Examples
//!
//! ```ignore
//! use mrrc::{Record, Field, Leader, marcxml};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut record = Record::new(Leader::default());
//! let mut field = Field::new("245".to_string(), '1', '0');
//! field.add_subfield('a', "Title".to_string());
//! record.add_field(field);
//!
//! let xml_str = marcxml::record_to_marcxml(&record)?;
//! let restored = marcxml::marcxml_to_record(&xml_str)?;
//! # Ok(())
//! # }
//! ```

use crate::error::{MarcError, Result};
use crate::leader::Leader;
use crate::record::{Field, Record};
use quick_xml::de::from_str as xml_from_str;
use quick_xml::se::to_string as xml_to_string;
use serde::{Deserialize, Serialize};

/// The MARCXML namespace URI.
const MARCXML_NS: &str = "http://www.loc.gov/MARC21/slim";

/// MARCXML record representation for serialization.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "record")]
pub struct MarcxmlRecord {
    /// MARC leader string
    pub leader: String,
    /// Control fields (tags 001-009)
    #[serde(default)]
    pub controlfield: Vec<MarcxmlControlField>,
    /// Data fields (tags 010+)
    #[serde(default)]
    pub datafield: Vec<MarcxmlDataField>,
}

/// MARCXML control field representation.
#[derive(Debug, Serialize, Deserialize)]
pub struct MarcxmlControlField {
    /// Field tag as an XML attribute (e.g., "001", "008")
    #[serde(rename = "@tag")]
    pub tag: String,
    /// Control field value (text content)
    #[serde(rename = "$value")]
    pub value: String,
}

/// MARCXML data field representation.
#[derive(Debug, Serialize, Deserialize)]
pub struct MarcxmlDataField {
    /// Field tag as an XML attribute (e.g., "245", "650")
    #[serde(rename = "@tag")]
    pub tag: String,
    /// First indicator as an XML attribute
    #[serde(rename = "@ind1")]
    pub ind1: String,
    /// Second indicator as an XML attribute
    #[serde(rename = "@ind2")]
    pub ind2: String,
    /// Subfields
    #[serde(default)]
    pub subfield: Vec<MarcxmlSubfield>,
}

/// MARCXML subfield representation.
#[derive(Debug, Serialize, Deserialize)]
pub struct MarcxmlSubfield {
    /// Subfield code as an XML attribute (e.g., "a", "b", "c")
    #[serde(rename = "@code")]
    pub code: String,
    /// Subfield value (text content)
    #[serde(rename = "$value")]
    pub value: String,
}

/// MARCXML collection wrapper for multiple records.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "collection")]
pub struct MarcxmlCollection {
    /// Records in the collection
    #[serde(default, rename = "record")]
    pub records: Vec<MarcxmlRecord>,
}

// ---------------------------------------------------------------------------
// Namespace stripping
// ---------------------------------------------------------------------------

/// Strip XML namespace prefixes and declarations from MARCXML input.
///
/// Handles both `marc:record` → `record` (prefixed namespace) and
/// `xmlns="..."` / `xmlns:marc="..."` (namespace declarations).
fn strip_marcxml_ns(xml: &str) -> String {
    use regex::Regex;

    // Strip xmlns declarations (both default and prefixed)
    let re_xmlns = Regex::new(r#"\s+xmlns(?::\w+)?="[^"]*""#).unwrap();
    let stripped = re_xmlns.replace_all(xml, "");

    // Strip namespace prefixes on element names: <marc:record> → <record>,
    // </marc:record> → </record>
    let re_prefix = Regex::new(r"<(/?)(\w+):").unwrap();
    re_prefix.replace_all(&stripped, "<$1").to_string()
}

// ---------------------------------------------------------------------------
// Serialization: Record → MARCXML
// ---------------------------------------------------------------------------

/// Convert a MARC record to a standard MARCXML string.
///
/// The output includes an XML declaration and the
/// `xmlns="http://www.loc.gov/MARC21/slim"` namespace on the root `<record>` element.
/// All `tag`, `ind1`, `ind2`, and `code` values are serialized as XML attributes,
/// conforming to the LOC MARCXML schema.
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
/// # use mrrc::{Record, Field, Leader, marcxml};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut record = Record::new(Leader::default());
/// let mut field = Field::new("245".to_string(), '1', '0');
/// field.add_subfield('a', "Test Title".to_string());
/// record.add_field(field);
/// let xml = marcxml::record_to_marcxml(&record)?;
/// assert!(xml.contains(r#"xmlns="http://www.loc.gov/MARC21/slim""#));
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if the record cannot be serialized to XML.
pub fn record_to_marcxml(record: &Record) -> Result<String> {
    let leader_bytes = record.leader.as_bytes()?;
    let leader_str = String::from_utf8_lossy(&leader_bytes).to_string();

    let mut controlfields = Vec::new();
    for (tag, value) in &record.control_fields {
        controlfields.push(MarcxmlControlField {
            tag: tag.clone(),
            value: value.clone(),
        });
    }

    let mut datafields = Vec::new();
    for (tag, field_list) in &record.fields {
        for field in field_list {
            let mut subfields = Vec::new();
            for subfield in &field.subfields {
                subfields.push(MarcxmlSubfield {
                    code: subfield.code.to_string(),
                    value: subfield.value.clone(),
                });
            }

            datafields.push(MarcxmlDataField {
                tag: tag.clone(),
                ind1: field.indicator1.to_string(),
                ind2: field.indicator2.to_string(),
                subfield: subfields,
            });
        }
    }

    let xml_record = MarcxmlRecord {
        leader: leader_str,
        controlfield: controlfields,
        datafield: datafields,
    };

    let body = xml_to_string(&xml_record)
        .map_err(|e| MarcError::ParseError(format!("Failed to serialize to MARCXML: {e}")))?;

    // Insert xmlns attribute into the root <record> element
    let body = body.replacen("<record>", &format!("<record xmlns=\"{MARCXML_NS}\">"), 1);

    Ok(format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>{body}"))
}

// ---------------------------------------------------------------------------
// Deserialization: MARCXML → Record
// ---------------------------------------------------------------------------

/// Convert a MARCXML string to a MARC record.
///
/// Accepts standard MARCXML in any of these forms:
/// - `<record xmlns="http://www.loc.gov/MARC21/slim">` (default namespace)
/// - `<marc:record xmlns:marc="...">` (prefixed namespace)
/// - `<record>` (no namespace)
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
/// # use mrrc::marcxml;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let xml_str = r#"<record><leader>01234nam a2200289 a 4500</leader></record>"#;
/// let record = marcxml::marcxml_to_record(xml_str)?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if the XML is invalid or missing required elements.
pub fn marcxml_to_record(xml: &str) -> Result<Record> {
    let cleaned = strip_marcxml_ns(xml);
    let xml_record: MarcxmlRecord = xml_from_str(&cleaned)
        .map_err(|e| MarcError::ParseError(format!("Failed to parse MARCXML: {e}")))?;

    marcxml_record_to_record(xml_record)
}

/// Convert a MARCXML `<collection>` string to multiple MARC records.
///
/// Accepts a `<collection>` wrapper (with or without namespace prefixes)
/// containing one or more `<record>` elements.
///
/// # Arguments
///
/// * `xml` - The MARCXML collection string to parse
///
/// # Returns
///
/// A vector of MARC Records, or an error if parsing fails.
///
/// # Errors
///
/// Returns an error if the XML is invalid or cannot be parsed.
pub fn marcxml_to_records(xml: &str) -> Result<Vec<Record>> {
    let cleaned = strip_marcxml_ns(xml);
    let collection: MarcxmlCollection = xml_from_str(&cleaned)
        .map_err(|e| MarcError::ParseError(format!("Failed to parse MARCXML collection: {e}")))?;

    collection
        .records
        .into_iter()
        .map(marcxml_record_to_record)
        .collect()
}

/// Internal helper: convert a deserialized `MarcxmlRecord` into a `Record`.
fn marcxml_record_to_record(xml_record: MarcxmlRecord) -> Result<Record> {
    let leader = Leader::from_bytes(xml_record.leader.as_bytes())?;
    let mut record = Record::new(leader);

    for cf in xml_record.controlfield {
        record.add_control_field(cf.tag, cf.value);
    }

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
    fn test_record_to_marcxml_output_format() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "12345".to_string());

        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test title".to_string());
        record.add_field(field);

        let xml = record_to_marcxml(&record).unwrap();

        // Verify XML declaration
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        // Verify xmlns attribute
        assert!(xml.contains(&format!("xmlns=\"{MARCXML_NS}\"")));
        // Verify attributes (not child elements)
        assert!(xml.contains("<controlfield tag=\"001\">12345</controlfield>"));
        assert!(xml.contains("<datafield tag=\"245\" ind1=\"1\" ind2=\"0\">"));
        assert!(xml.contains("<subfield code=\"a\">Test title</subfield>"));
        assert!(xml.contains("<leader>"));
    }

    #[test]
    fn test_marcxml_roundtrip() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "12345".to_string());

        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test title".to_string());
        field.add_subfield('c', "Author".to_string());
        record.add_field(field);

        let xml = record_to_marcxml(&record).unwrap();
        let restored = marcxml_to_record(&xml).unwrap();

        assert_eq!(restored.get_control_field("001"), Some("12345"));
        let fields = restored.get_fields("245").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Test title"));
        assert_eq!(fields[0].get_subfield('c'), Some("Author"));
    }

    #[test]
    fn test_parse_standard_marcxml_no_namespace() {
        let xml = r#"<record>
            <leader>01234nam a2200289 a 4500</leader>
            <controlfield tag="001">12345</controlfield>
            <datafield tag="245" ind1="1" ind2="0">
                <subfield code="a">Test title</subfield>
            </datafield>
        </record>"#;

        let record = marcxml_to_record(xml).unwrap();
        assert_eq!(record.get_control_field("001"), Some("12345"));
        let fields = record.get_fields("245").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Test title"));
    }

    #[test]
    fn test_parse_marcxml_with_default_namespace() {
        let xml = r#"<record xmlns="http://www.loc.gov/MARC21/slim">
            <leader>01234nam a2200289 a 4500</leader>
            <controlfield tag="001">99999</controlfield>
            <datafield tag="245" ind1="0" ind2="0">
                <subfield code="a">Namespaced title</subfield>
            </datafield>
        </record>"#;

        let record = marcxml_to_record(xml).unwrap();
        assert_eq!(record.get_control_field("001"), Some("99999"));
        let fields = record.get_fields("245").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Namespaced title"));
    }

    #[test]
    fn test_parse_marcxml_with_prefix_namespace() {
        let xml = r#"<marc:record xmlns:marc="http://www.loc.gov/MARC21/slim">
            <marc:leader>01234nam a2200289 a 4500</marc:leader>
            <marc:controlfield tag="001">88888</marc:controlfield>
            <marc:datafield tag="245" ind1="1" ind2="0">
                <marc:subfield code="a">Prefixed title</marc:subfield>
            </marc:datafield>
        </marc:record>"#;

        let record = marcxml_to_record(xml).unwrap();
        assert_eq!(record.get_control_field("001"), Some("88888"));
        let fields = record.get_fields("245").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Prefixed title"));
    }

    #[test]
    fn test_parse_marcxml_collection() {
        let xml = r#"<collection xmlns="http://www.loc.gov/MARC21/slim">
            <record>
                <leader>01234nam a2200289 a 4500</leader>
                <controlfield tag="001">rec1</controlfield>
            </record>
            <record>
                <leader>01234nam a2200289 a 4500</leader>
                <controlfield tag="001">rec2</controlfield>
            </record>
        </collection>"#;

        let records = marcxml_to_records(xml).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].get_control_field("001"), Some("rec1"));
        assert_eq!(records[1].get_control_field("001"), Some("rec2"));
    }

    #[test]
    fn test_parse_marcxml_collection_with_prefix() {
        let xml = r#"<marc:collection xmlns:marc="http://www.loc.gov/MARC21/slim">
            <marc:record>
                <marc:leader>01234nam a2200289 a 4500</marc:leader>
                <marc:controlfield tag="001">pfx1</marc:controlfield>
            </marc:record>
            <marc:record>
                <marc:leader>01234nam a2200289 a 4500</marc:leader>
                <marc:controlfield tag="001">pfx2</marc:controlfield>
            </marc:record>
        </marc:collection>"#;

        let records = marcxml_to_records(xml).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].get_control_field("001"), Some("pfx1"));
        assert_eq!(records[1].get_control_field("001"), Some("pfx2"));
    }

    #[test]
    fn test_marcxml_with_multiple_subfields() {
        let mut record = Record::new(make_test_leader());

        let mut field = Field::new("650".to_string(), ' ', '0');
        field.add_subfield('a', "Subject".to_string());
        field.add_subfield('v', "Subdivision".to_string());
        field.add_subfield('x', "General subdivision".to_string());
        record.add_field(field);

        let xml = record_to_marcxml(&record).unwrap();
        let restored = marcxml_to_record(&xml).unwrap();

        let fields = restored.get_fields("650").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Subject"));
        assert_eq!(fields[0].get_subfield('v'), Some("Subdivision"));
        assert_eq!(fields[0].get_subfield('x'), Some("General subdivision"));
    }

    #[test]
    fn test_marcxml_with_multiple_fields_same_tag() {
        let mut record = Record::new(make_test_leader());

        for i in 1..=3 {
            let mut field = Field::new("650".to_string(), ' ', '0');
            field.add_subfield('a', format!("Subject {i}"));
            record.add_field(field);
        }

        let xml = record_to_marcxml(&record).unwrap();
        let restored = marcxml_to_record(&xml).unwrap();

        let fields = restored.get_fields("650").unwrap();
        assert_eq!(fields.len(), 3);
    }

    #[test]
    fn test_parse_loc_style_record() {
        // Simulates the LOC MARCXML format from issue #15
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <record xmlns="http://www.loc.gov/MARC21/slim">
            <leader>01142cam  2200301 a 4500</leader>
            <controlfield tag="001">92005291</controlfield>
            <controlfield tag="008">920219s1990    mau           001 0 eng  </controlfield>
            <datafield tag="020" ind1=" " ind2=" ">
                <subfield code="a">0262031418</subfield>
            </datafield>
            <datafield tag="245" ind1="1" ind2="0">
                <subfield code="a">Introduction to algorithms /</subfield>
                <subfield code="c">Thomas H. Cormen ... [et al.].</subfield>
            </datafield>
            <datafield tag="650" ind1=" " ind2="0">
                <subfield code="a">Computer programming.</subfield>
            </datafield>
            <datafield tag="650" ind1=" " ind2="0">
                <subfield code="a">Computer algorithms.</subfield>
            </datafield>
        </record>"#;

        let record = marcxml_to_record(xml).unwrap();
        assert_eq!(record.get_control_field("001"), Some("92005291"));

        let title_fields = record.get_fields("245").unwrap();
        assert_eq!(
            title_fields[0].get_subfield('a'),
            Some("Introduction to algorithms /")
        );
        assert_eq!(
            title_fields[0].get_subfield('c'),
            Some("Thomas H. Cormen ... [et al.].")
        );

        let subjects = record.get_fields("650").unwrap();
        assert_eq!(subjects.len(), 2);
        assert_eq!(subjects[0].get_subfield('a'), Some("Computer programming."));
        assert_eq!(subjects[1].get_subfield('a'), Some("Computer algorithms."));
    }
}
