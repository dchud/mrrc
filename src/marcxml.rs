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
use crate::iso2709::ParseContext;
use crate::leader::Leader;
use crate::record::{Field, Record};
use quick_xml::events::Event;
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
// Event-driven MARCXML reader
// ---------------------------------------------------------------------------
//
// `quick-xml`'s serde deserializer trims text content at an internal layer
// regardless of `trim_text` config, so `<controlfield tag="008">   </…>`
// raises "missing field `$value`". MARC values are often padded with
// meaningful whitespace (an all-spaces 008 is common on minimal records),
// so the reader walks the event stream directly and captures text + CDATA
// payloads verbatim.
//
// The write path still uses serde; quick-xml's serializer emits whitespace
// correctly.

/// Drive the event reader, filling a `MarcxmlRecord` from the start-tag of
/// the `<record>` element (inclusive) through its end tag (exclusive of
/// anything after).
fn read_marcxml_record<B: std::io::BufRead>(
    reader: &mut quick_xml::reader::Reader<B>,
    ctx: &ParseContext,
) -> Result<MarcxmlRecord> {
    let mut buf = Vec::new();
    let mut record = MarcxmlRecord {
        leader: String::new(),
        controlfield: Vec::new(),
        datafield: Vec::new(),
    };
    let mut current_df: Option<MarcxmlDataField> = None;

    loop {
        buf.clear();
        match reader
            .read_event_into(&mut buf)
            .map_err(|e| ctx.err_xml(e))?
        {
            Event::Start(ref e) => {
                let name_bytes = e.name().into_inner();
                let name = std::str::from_utf8(name_bytes).unwrap_or("");
                match name {
                    "leader" => {
                        record.leader = read_leaf_text(reader, name_bytes, ctx)?;
                    },
                    "controlfield" => {
                        let tag = attr_value(e, b"tag").unwrap_or_default();
                        let value = read_leaf_text(reader, name_bytes, ctx)?;
                        record.controlfield.push(MarcxmlControlField { tag, value });
                    },
                    "datafield" => {
                        current_df = Some(MarcxmlDataField {
                            tag: attr_value(e, b"tag").unwrap_or_default(),
                            ind1: attr_value(e, b"ind1").unwrap_or_default(),
                            ind2: attr_value(e, b"ind2").unwrap_or_default(),
                            subfield: Vec::new(),
                        });
                    },
                    "subfield" => {
                        let code = attr_value(e, b"code").unwrap_or_default();
                        let value = read_leaf_text(reader, name_bytes, ctx)?;
                        if let Some(df) = current_df.as_mut() {
                            df.subfield.push(MarcxmlSubfield { code, value });
                        }
                    },
                    _ => {},
                }
            },
            Event::End(ref e) => {
                let name = std::str::from_utf8(e.name().into_inner()).unwrap_or("");
                if name == "datafield" {
                    if let Some(df) = current_df.take() {
                        record.datafield.push(df);
                    }
                } else if name == "record" {
                    return Ok(record);
                }
            },
            Event::Empty(ref e) => {
                // Self-closing element: <controlfield tag="001"/> etc. Treat
                // as empty string value so callers see the field at all.
                let name = std::str::from_utf8(e.name().into_inner()).unwrap_or("");
                match name {
                    "controlfield" => {
                        record.controlfield.push(MarcxmlControlField {
                            tag: attr_value(e, b"tag").unwrap_or_default(),
                            value: String::new(),
                        });
                    },
                    "subfield" => {
                        if let Some(df) = current_df.as_mut() {
                            df.subfield.push(MarcxmlSubfield {
                                code: attr_value(e, b"code").unwrap_or_default(),
                                value: String::new(),
                            });
                        }
                    },
                    "datafield" => {
                        record.datafield.push(MarcxmlDataField {
                            tag: attr_value(e, b"tag").unwrap_or_default(),
                            ind1: attr_value(e, b"ind1").unwrap_or_default(),
                            ind2: attr_value(e, b"ind2").unwrap_or_default(),
                            subfield: Vec::new(),
                        });
                    },
                    _ => {},
                }
            },
            Event::Eof => {
                return Err(ctx.err_xml(quick_xml::DeError::Custom(
                    "unexpected EOF before </record>".to_string(),
                )));
            },
            _ => {},
        }
    }
}

/// Read all Text/CData events until the end tag matching `name`. Returns
/// the concatenated, unescaped payload verbatim (whitespace preserved).
fn read_leaf_text<B: std::io::BufRead>(
    reader: &mut quick_xml::reader::Reader<B>,
    name: &[u8],
    ctx: &ParseContext,
) -> Result<String> {
    let mut out = String::new();
    let mut buf = Vec::new();
    loop {
        buf.clear();
        match reader
            .read_event_into(&mut buf)
            .map_err(|e| ctx.err_xml(e))?
        {
            Event::Text(t) => {
                // xml_content() applies XML 1.1 §2.11 end-of-line normalization
                // (CR, CRLF, NEL, LSEP → LF) per spec. quick-xml's plain decode()
                // skips that pass; quick-xml's own docs flag it as the wrong
                // default ("Usually you need xml_content() instead").
                let decoded = t.xml_content().map_err(|e| {
                    MarcError::invalid_field_msg(format!("Invalid text encoding: {e}"))
                })?;
                out.push_str(&decoded);
            },
            Event::CData(c) => {
                // CDATA section content is part of the document entity and is
                // also subject to XML 1.1 §2.11 EOL normalization. xml_content()
                // additionally honors the reader's declared encoding, so this
                // is a strict improvement over a raw UTF-8 conversion.
                let decoded = c.xml_content().map_err(|e| {
                    MarcError::invalid_field_msg(format!("Invalid CDATA encoding: {e}"))
                })?;
                out.push_str(&decoded);
            },
            Event::GeneralRef(r) => {
                // `&#NN;` / `&#xHH;` numeric character references, plus the
                // five XML built-in named entities. Any other named entity
                // would need a DTD, which MARCXML does not use.
                if let Some(ch) = r.resolve_char_ref().map_err(|e| ctx.err_xml(e))? {
                    out.push(ch);
                } else {
                    let name = r.decode().map_err(|e| {
                        MarcError::invalid_field_msg(format!("Invalid entity encoding: {e}"))
                    })?;
                    let ch = match &*name {
                        "lt" => '<',
                        "gt" => '>',
                        "amp" => '&',
                        "apos" => '\'',
                        "quot" => '"',
                        other => {
                            return Err(MarcError::invalid_field_msg(format!(
                                "Unknown entity reference: &{other};"
                            )));
                        },
                    };
                    out.push(ch);
                }
            },
            Event::End(ref e) if e.name().into_inner() == name => return Ok(out),
            Event::Eof => {
                return Err(ctx.err_xml(quick_xml::DeError::Custom(format!(
                    "unexpected EOF inside <{}>",
                    std::str::from_utf8(name).unwrap_or("?")
                ))));
            },
            // Ignore comments, PIs, nested starts (MARCXML leaf elements
            // have text-only content per schema).
            _ => {},
        }
    }
}

/// Extract an attribute value as an owned `String`, decoding XML escapes.
fn attr_value(start: &quick_xml::events::BytesStart, name: &[u8]) -> Option<String> {
    start
        .attributes()
        .with_checks(false)
        .filter_map(std::result::Result::ok)
        .find(|a| a.key.into_inner() == name)
        .and_then(|a| a.unescape_value().ok().map(std::borrow::Cow::into_owned))
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
    for (tag, values) in &record.control_fields {
        for value in values {
            controlfields.push(MarcxmlControlField {
                tag: tag.clone(),
                value: value.clone(),
            });
        }
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

    let body = xml_to_string(&xml_record).map_err(|e| {
        MarcError::invalid_field_msg(format!("Failed to serialize to MARCXML: {e}"))
    })?;

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
    let ctx = ParseContext::new();
    let cleaned = strip_marcxml_ns(xml);
    let mut reader = quick_xml::reader::Reader::from_str(&cleaned);

    // Walk to the first <record> Start event.
    let mut buf = Vec::new();
    loop {
        buf.clear();
        match reader
            .read_event_into(&mut buf)
            .map_err(|e| ctx.err_xml(e))?
        {
            Event::Start(ref e) if e.name().into_inner() == b"record" => break,
            Event::Eof => {
                return Err(ctx.err_xml(quick_xml::DeError::Custom(
                    "no <record> element found".to_string(),
                )));
            },
            _ => {},
        }
    }

    let xml_record = read_marcxml_record(&mut reader, &ctx)?;
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
    let ctx = ParseContext::new();
    let cleaned = strip_marcxml_ns(xml);
    let mut reader = quick_xml::reader::Reader::from_str(&cleaned);

    let mut records = Vec::new();
    let mut buf = Vec::new();
    loop {
        buf.clear();
        match reader
            .read_event_into(&mut buf)
            .map_err(|e| ctx.err_xml(e))?
        {
            Event::Start(ref e) if e.name().into_inner() == b"record" => {
                let xml_record = read_marcxml_record(&mut reader, &ctx)?;
                records.push(marcxml_record_to_record(xml_record)?);
            },
            Event::Eof => return Ok(records),
            _ => {},
        }
    }
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
            let code =
                sf.code.chars().next().ok_or_else(|| {
                    MarcError::invalid_field_msg("Missing subfield code".to_string())
                })?;
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

    /// MARCXML must round-trip whitespace-only control-field values without
    /// stripping them. `008` (fixed-length 40-char control field) in
    /// particular is often all spaces on minimal records; losing that
    /// content silently changes the record's semantics.
    #[test]
    fn test_marcxml_roundtrip_whitespace_only_control_field() {
        let mut record = Record::new(make_test_leader());
        // 3 spaces — a whitespace-only control value
        record.add_control_field("008".to_string(), "   ".to_string());

        let xml = record_to_marcxml(&record).unwrap();
        let restored = marcxml_to_record(&xml).unwrap();

        assert_eq!(restored.get_control_field("008"), Some("   "));
    }

    /// MARCXML must round-trip a subfield whose text content is a single
    /// space.
    #[test]
    fn test_marcxml_roundtrip_whitespace_only_subfield() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', " ".to_string());
        record.add_field(field);

        let xml = record_to_marcxml(&record).unwrap();
        let restored = marcxml_to_record(&xml).unwrap();

        let fields = restored.get_fields("245").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some(" "));
    }

    /// XML 1.1 §2.11 requires the parser to normalize CR, CRLF, NEL, and
    /// LSEP to LF in text content. Raw CRLF in subfield text must surface
    /// as LF after parse.
    #[test]
    fn test_marcxml_normalizes_crlf_in_subfield_text() {
        let xml = "<record>\
            <leader>01234nam a2200289 a 4500</leader>\
            <controlfield tag=\"001\">x</controlfield>\
            <datafield tag=\"500\" ind1=\" \" ind2=\" \">\
                <subfield code=\"a\">line1\r\nline2\rline3</subfield>\
            </datafield>\
        </record>";

        let record = marcxml_to_record(xml).unwrap();
        let fields = record.get_fields("500").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("line1\nline2\nline3"));
    }

    /// CDATA section content is also part of the document entity per
    /// XML 1.1 §2.11; quick-xml's plain `decode()` skips normalization, so
    /// guard the `xml_content()` switch with a CDATA-specific case.
    #[test]
    fn test_marcxml_normalizes_crlf_in_cdata() {
        let xml = "<record>\
            <leader>01234nam a2200289 a 4500</leader>\
            <controlfield tag=\"001\">x</controlfield>\
            <datafield tag=\"500\" ind1=\" \" ind2=\" \">\
                <subfield code=\"a\"><![CDATA[a\r\nb\rc]]></subfield>\
            </datafield>\
        </record>";

        let record = marcxml_to_record(xml).unwrap();
        let fields = record.get_fields("500").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("a\nb\nc"));
    }

    /// Round-trip a subfield carrying a literal LF: the writer preserves
    /// the byte and the reader's normalization leaves LF unchanged.
    #[test]
    fn test_marcxml_roundtrip_subfield_with_newline() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("500".to_string(), ' ', ' ');
        field.add_subfield('a', "first\nsecond".to_string());
        record.add_field(field);

        let xml = record_to_marcxml(&record).unwrap();
        let restored = marcxml_to_record(&xml).unwrap();

        let fields = restored.get_fields("500").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("first\nsecond"));
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
