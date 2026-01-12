//! Dublin Core serialization of MARC records.
//!
//! This module provides conversion of MARC records to Dublin Core metadata,
//! a simple and widely-adopted 15-element metadata schema used for resource discovery.
//!
//! The Dublin Core Metadata Element Set (DCMES) consists of:
//! - **Title** (dc:title)
//! - **Creator** (dc:creator)
//! - **Subject** (dc:subject)
//! - **Description** (dc:description)
//! - **Publisher** (dc:publisher)
//! - **Contributor** (dc:contributor)
//! - **Date** (dc:date)
//! - **Type** (dc:type)
//! - **Format** (dc:format)
//! - **Identifier** (dc:identifier)
//! - **Source** (dc:source)
//! - **Language** (dc:language)
//! - **Relation** (dc:relation)
//! - **Coverage** (dc:coverage)
//! - **Rights** (dc:rights)
//!
//! # API Patterns
//!
//! Two conversion approaches are provided:
//! - **Intermediate struct**: [`record_to_dublin_core()`] returns a `DublinCoreRecord` struct
//!   for programmatic access to the 15 elements
//! - **Direct XML**: [`record_to_dublin_core_xml()`] directly converts to XML format in one call
//!
//! # Examples
//!
//! Direct XML conversion (convenience function):
//! ```ignore
//! use mrrc::{Record, Field, Leader, dublin_core};
//!
//! let mut record = Record::new(Leader::default());
//! let mut field = Field::new("245".to_string(), '1', '0');
//! field.add_subfield('a', "Title".to_string());
//! record.add_field(field);
//!
//! let xml = dublin_core::record_to_dublin_core_xml(&record)?;
//! println!("{}", xml);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Intermediate struct (for programmatic access):
//! ```ignore
//! use mrrc::{Record, Field, Leader, dublin_core};
//!
//! let mut record = Record::new(Leader::default());
//! let mut field = Field::new("245".to_string(), '1', '0');
//! field.add_subfield('a', "Title".to_string());
//! record.add_field(field);
//!
//! let dc = dublin_core::record_to_dublin_core(&record)?;
//! println!("Title: {:?}", dc.title);
//! println!("Creator: {:?}", dc.creator);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::fmt::Write;

use crate::error::Result;
use crate::record::Record;

/// Dublin Core metadata record
#[derive(Debug, Clone, Default)]
pub struct DublinCoreRecord {
    /// dc:title - Title of the resource
    pub title: Vec<String>,
    /// dc:creator - Entity responsible for the resource
    pub creator: Vec<String>,
    /// dc:subject - Topic of the resource
    pub subject: Vec<String>,
    /// dc:description - Account of the resource
    pub description: Vec<String>,
    /// dc:publisher - Entity responsible for making the resource available
    pub publisher: Vec<String>,
    /// dc:contributor - Entity responsible for making contributions to the resource
    pub contributor: Vec<String>,
    /// dc:date - Point or period of time associated with the resource
    pub date: Vec<String>,
    /// dc:type - Nature or genre of the resource
    pub dc_type: Vec<String>,
    /// dc:format - File format, physical medium, or dimensions of the resource
    pub format: Vec<String>,
    /// dc:identifier - Unambiguous reference to the resource
    pub identifier: Vec<String>,
    /// dc:source - Related resource from which the resource is derived
    pub source: Vec<String>,
    /// dc:language - Language of the resource
    pub language: Vec<String>,
    /// dc:relation - Related resource
    pub relation: Vec<String>,
    /// dc:coverage - Spatial or temporal topic of the resource
    pub coverage: Vec<String>,
    /// dc:rights - Information about rights held in and over the resource
    pub rights: Vec<String>,
}

/// Convert a MARC record to Dublin Core metadata.
///
/// Maps MARC fields to Dublin Core elements based on standard crosswalks.
/// This function returns an intermediate `DublinCoreRecord` struct that can be
/// serialized to XML using [`dublin_core_to_xml()`] or used directly for programmatic
/// access to the 15 Dublin Core elements.
///
/// # Examples
///
/// ```ignore
/// use mrrc::{Record, Field, Leader, dublin_core};
///
/// let mut record = Record::new(Leader::default());
/// let mut field = Field::new("245".to_string(), '1', '0');
/// field.add_subfield('a', "My Book".to_string());
/// record.add_field(field);
///
/// let dc = dublin_core::record_to_dublin_core(&record)?;
/// assert!(!dc.title.is_empty());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # See also
///
/// - [`record_to_dublin_core_xml()`] - Convenience function combining this with XML serialization
///
/// # Errors
///
/// Returns an error if the record cannot be converted.
pub fn record_to_dublin_core(record: &Record) -> Result<DublinCoreRecord> {
    let mut dc = DublinCoreRecord::default();

    extract_titles(record, &mut dc);
    extract_creators(record, &mut dc);
    extract_subjects(record, &mut dc);
    extract_descriptions(record, &mut dc);
    extract_publishers_and_dates(record, &mut dc);
    extract_contributors(record, &mut dc);
    extract_identifiers(record, &mut dc);
    extract_language(record, &mut dc);
    extract_formats(record, &mut dc);
    extract_coverage(record, &mut dc);
    extract_rights(record, &mut dc);

    Ok(dc)
}

/// Convert a MARC record directly to Dublin Core XML format.
///
/// Convenience function that combines [`record_to_dublin_core()`] and [`dublin_core_to_xml()`]
/// in a single call for simplified API when XML output is desired.
///
/// # Examples
///
/// ```ignore
/// use mrrc::{Record, Field, Leader, dublin_core};
///
/// let mut record = Record::new(Leader::default());
/// let mut field = Field::new("245".to_string(), '1', '0');
/// field.add_subfield('a', "My Book".to_string());
/// record.add_field(field);
///
/// let xml = dublin_core::record_to_dublin_core_xml(&record)?;
/// println!("{xml}");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # See also
///
/// - [`record_to_dublin_core()`] - If you need the intermediate `DublinCoreRecord` struct
/// - [`dublin_core_to_xml()`] - For serializing an existing `DublinCoreRecord`
///
/// # Errors
///
/// Returns an error if the record cannot be converted to Dublin Core metadata.
pub fn record_to_dublin_core_xml(record: &Record) -> Result<String> {
    let dc = record_to_dublin_core(record)?;
    Ok(dublin_core_to_xml(&dc))
}

fn extract_titles(record: &Record, dc: &mut DublinCoreRecord) {
    if let Some(fields_245) = record.fields.get("245") {
        for field in fields_245 {
            let mut title_parts = Vec::new();
            for subfield in &field.subfields {
                match subfield.code {
                    'a' | 'b' | 'c' => title_parts.push(subfield.value.clone()),
                    _ => {},
                }
            }
            if !title_parts.is_empty() {
                dc.title.push(title_parts.join(" "));
            }
        }
    }
}

fn extract_creators(record: &Record, dc: &mut DublinCoreRecord) {
    // Main entry - personal name (100)
    if let Some(fields) = record.fields.get("100") {
        for field in fields {
            if let Some(subfield) = field.subfields.iter().find(|s| s.code == 'a') {
                dc.creator.push(subfield.value.clone());
            }
        }
    }

    // Main entry - corporate name (110)
    if let Some(fields) = record.fields.get("110") {
        for field in fields {
            if let Some(subfield) = field.subfields.iter().find(|s| s.code == 'a') {
                dc.creator.push(subfield.value.clone());
            }
        }
    }
}

fn extract_subjects(record: &Record, dc: &mut DublinCoreRecord) {
    // Subject - personal name (600)
    if let Some(fields) = record.fields.get("600") {
        for field in fields {
            if let Some(subfield) = field.subfields.iter().find(|s| s.code == 'a') {
                dc.subject.push(subfield.value.clone());
            }
        }
    }

    // Subject - corporate name (610)
    if let Some(fields) = record.fields.get("610") {
        for field in fields {
            if let Some(subfield) = field.subfields.iter().find(|s| s.code == 'a') {
                dc.subject.push(subfield.value.clone());
            }
        }
    }

    // Subject - topical term (650)
    if let Some(fields) = record.fields.get("650") {
        for field in fields {
            if let Some(subfield) = field.subfields.iter().find(|s| s.code == 'a') {
                dc.subject.push(subfield.value.clone());
            }
        }
    }
}

fn extract_descriptions(record: &Record, dc: &mut DublinCoreRecord) {
    // Summary (520)
    if let Some(fields) = record.fields.get("520") {
        for field in fields {
            if let Some(subfield) = field.subfields.iter().find(|s| s.code == 'a') {
                dc.description.push(subfield.value.clone());
            }
        }
    }

    // General note (500)
    if let Some(fields) = record.fields.get("500") {
        for field in fields {
            if let Some(subfield) = field.subfields.iter().find(|s| s.code == 'a') {
                dc.description.push(subfield.value.clone());
            }
        }
    }
}

fn extract_publishers_and_dates(record: &Record, dc: &mut DublinCoreRecord) {
    if let Some(fields) = record.fields.get("260") {
        for field in fields {
            for subfield in &field.subfields {
                match subfield.code {
                    'a' => dc.publisher.push(subfield.value.clone()),
                    'c' => dc.date.push(subfield.value.clone()),
                    _ => {},
                }
            }
        }
    }
}

fn extract_contributors(record: &Record, dc: &mut DublinCoreRecord) {
    // Added entry - personal name (700)
    if let Some(fields) = record.fields.get("700") {
        for field in fields {
            if let Some(subfield) = field.subfields.iter().find(|s| s.code == 'a') {
                dc.contributor.push(subfield.value.clone());
            }
        }
    }

    // Added entry - corporate name (710)
    if let Some(fields) = record.fields.get("710") {
        for field in fields {
            if let Some(subfield) = field.subfields.iter().find(|s| s.code == 'a') {
                dc.contributor.push(subfield.value.clone());
            }
        }
    }
}

fn extract_identifiers(record: &Record, dc: &mut DublinCoreRecord) {
    // ISBN (020)
    if let Some(fields) = record.fields.get("020") {
        for field in fields {
            if let Some(subfield) = field.subfields.iter().find(|s| s.code == 'a') {
                dc.identifier.push(format!("ISBN: {}", subfield.value));
            }
        }
    }

    // Control number (001)
    if let Some(control_001) = record.control_fields.get("001") {
        dc.identifier.push(format!("Control#: {control_001}"));
    }
}

fn extract_language(record: &Record, dc: &mut DublinCoreRecord) {
    if let Some(fields) = record.fields.get("041") {
        for field in fields {
            for subfield in &field.subfields {
                if subfield.code == 'a' {
                    let langs: Vec<&str> = subfield.value.split_whitespace().collect();
                    for lang in langs {
                        if !lang.is_empty() {
                            dc.language.push(lang.to_string());
                        }
                    }
                }
            }
        }
    }
}

fn extract_formats(record: &Record, dc: &mut DublinCoreRecord) {
    if let Some(fields) = record.fields.get("300") {
        for field in fields {
            if let Some(subfield) = field.subfields.iter().find(|s| s.code == 'a') {
                dc.format.push(subfield.value.clone());
            }
        }
    }
}

fn extract_coverage(record: &Record, dc: &mut DublinCoreRecord) {
    if let Some(fields) = record.fields.get("651") {
        for field in fields {
            if let Some(subfield) = field.subfields.iter().find(|s| s.code == 'a') {
                dc.coverage.push(subfield.value.clone());
            }
        }
    }
}

fn extract_rights(record: &Record, dc: &mut DublinCoreRecord) {
    if let Some(fields) = record.fields.get("540") {
        for field in fields {
            if let Some(subfield) = field.subfields.iter().find(|s| s.code == 'a') {
                dc.rights.push(subfield.value.clone());
            }
        }
    }
}

/// Serialize Dublin Core record to XML format.
///
/// Produces RDF/XML serialization compatible with the Dublin Core vocabulary.
#[must_use]
pub fn dublin_core_to_xml(dc: &DublinCoreRecord) -> String {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\" ");
    xml.push_str("xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n");
    xml.push_str("  <rdf:Description>\n");

    write_elements(&mut xml, "dc:title", &dc.title);
    write_elements(&mut xml, "dc:creator", &dc.creator);
    write_elements(&mut xml, "dc:subject", &dc.subject);
    write_elements(&mut xml, "dc:description", &dc.description);
    write_elements(&mut xml, "dc:publisher", &dc.publisher);
    write_elements(&mut xml, "dc:contributor", &dc.contributor);
    write_elements(&mut xml, "dc:date", &dc.date);
    write_elements(&mut xml, "dc:type", &dc.dc_type);
    write_elements(&mut xml, "dc:format", &dc.format);
    write_elements(&mut xml, "dc:identifier", &dc.identifier);
    write_elements(&mut xml, "dc:source", &dc.source);
    write_elements(&mut xml, "dc:language", &dc.language);
    write_elements(&mut xml, "dc:relation", &dc.relation);
    write_elements(&mut xml, "dc:coverage", &dc.coverage);
    write_elements(&mut xml, "dc:rights", &dc.rights);

    xml.push_str("  </rdf:Description>\n");
    xml.push_str("</rdf:RDF>\n");

    xml
}

fn write_elements(xml: &mut String, tag: &str, values: &[String]) {
    for value in values {
        writeln!(xml, "    <{tag}>{}</{tag}>", escape_xml(value)).ok();
    }
}

/// Escape XML special characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{Field, Record};
    use crate::Leader;

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
            cataloging_form: 'a',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        }
    }

    #[test]
    fn test_title_extraction() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test Title".to_string());
        record.add_field(field);

        let dc = record_to_dublin_core(&record).expect("Failed to convert");
        assert_eq!(dc.title.len(), 1);
        assert_eq!(dc.title[0], "Test Title");
    }

    #[test]
    fn test_title_with_subtitle() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Main Title".to_string());
        field.add_subfield('b', "Subtitle".to_string());
        record.add_field(field);

        let dc = record_to_dublin_core(&record).expect("Failed to convert");
        assert_eq!(dc.title.len(), 1);
        assert!(dc.title[0].contains("Main Title"));
        assert!(dc.title[0].contains("Subtitle"));
    }

    #[test]
    fn test_creator_extraction() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("100".to_string(), '1', ' ');
        field.add_subfield('a', "Smith, John".to_string());
        record.add_field(field);

        let dc = record_to_dublin_core(&record).expect("Failed to convert");
        assert_eq!(dc.creator.len(), 1);
        assert_eq!(dc.creator[0], "Smith, John");
    }

    #[test]
    fn test_subject_extraction() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("650".to_string(), ' ', '0');
        field.add_subfield('a', "Science Fiction".to_string());
        record.add_field(field);

        let dc = record_to_dublin_core(&record).expect("Failed to convert");
        assert_eq!(dc.subject.len(), 1);
        assert_eq!(dc.subject[0], "Science Fiction");
    }

    #[test]
    fn test_isbn_identifier() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("020".to_string(), ' ', ' ');
        field.add_subfield('a', "1234567890".to_string());
        record.add_field(field);

        let dc = record_to_dublin_core(&record).expect("Failed to convert");
        assert!(dc.identifier.iter().any(|id| id.contains("ISBN")));
    }

    #[test]
    fn test_control_number_identifier() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "12345".to_string());

        let dc = record_to_dublin_core(&record).expect("Failed to convert");
        assert!(dc.identifier.iter().any(|id| id.contains("Control#")));
    }

    #[test]
    fn test_dublin_core_to_xml() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test".to_string());
        record.add_field(field);

        let dc = record_to_dublin_core(&record).expect("Failed to convert");
        let xml = dublin_core_to_xml(&dc);

        assert!(xml.contains("<?xml"));
        assert!(xml.contains("rdf:RDF"));
        assert!(xml.contains("dc:title"));
        assert!(xml.contains("Test"));
    }

    #[test]
    fn test_xml_escaping() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Title with <brackets> & ampersand".to_string());
        record.add_field(field);

        let dc = record_to_dublin_core(&record).expect("Failed to convert");
        let xml = dublin_core_to_xml(&dc);

        assert!(xml.contains("&lt;"));
        assert!(xml.contains("&gt;"));
        assert!(xml.contains("&amp;"));
    }

    #[test]
    fn test_language_extraction() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("041".to_string(), ' ', ' ');
        field.add_subfield('a', "eng fre".to_string());
        record.add_field(field);

        let dc = record_to_dublin_core(&record).expect("Failed to convert");
        assert!(!dc.language.is_empty());
    }

    #[test]
    fn test_description_from_summary() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("520".to_string(), ' ', ' ');
        field.add_subfield('a', "This is a summary".to_string());
        record.add_field(field);

        let dc = record_to_dublin_core(&record).expect("Failed to convert");
        assert!(dc.description.iter().any(|d| d.contains("summary")));
    }
}
