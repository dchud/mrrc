//! MODS (Metadata Object Description Schema) serialization of MARC records.
//!
//! This module provides conversion of MARC records to MODS XML format,
//! a metadata schema developed by the Library of Congress that is richer than
//! Dublin Core and suitable for detailed bibliographic description.
//!
//! MODS includes elements for:
//! - Titles (with type information)
//! - Names (personal, corporate, conference)
//! - Identifiers (ISBN, ISSN, etc.)
//! - Language
//! - Physical description
//! - Subject headings with authority information
//! - Locations and holdings information
//! - Related resources
//!
//! # Examples
//!
//! ```ignore
//! use mrrc::{Record, Field, Leader, mods};
//!
//! let mut record = Record::new(Leader::default());
//! let mut field = Field::new("245".to_string(), '1', '0');
//! field.add_subfield('a', "Title".to_string());
//! record.add_field(field);
//!
//! let mods_xml = mods::record_to_mods_xml(&record)?;
//! println!("{}", mods_xml);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::fmt::Write;

use crate::error::Result;
use crate::record::Record;

/// Convert a MARC record to MODS XML format.
///
/// Maps MARC fields to MODS elements based on standard crosswalks.
/// Common mappings:
/// - 245 (Title Statement) → mods:titleInfo/mods:title
/// - 1XX (Main Entry) → mods:name
/// - 6XX (Subject) → mods:subject
/// - 520 (Summary) → mods:abstract
/// - 260 (Publication) → mods:originInfo
/// - 300 (Physical Description) → mods:physicalDescription
/// - 020 (ISBN) / 022 (ISSN) → mods:identifier
/// - 041 (Language) → mods:language
/// - 650 (Topical Subject) → mods:subject/mods:topic
/// - 651 (Geographic Subject) → mods:subject/mods:geographic
///
/// # Examples
///
/// ```ignore
/// use mrrc::{Record, Field, Leader, mods};
///
/// let mut record = Record::new(Leader::default());
/// let mut field = Field::new("245".to_string(), '1', '0');
/// field.add_subfield('a', "My Book".to_string());
/// record.add_field(field);
///
/// let mods_xml = mods::record_to_mods_xml(&record)?;
/// assert!(mods_xml.contains("<mods:title>"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns an error if the record cannot be converted.
pub fn record_to_mods_xml(record: &Record) -> Result<String> {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<mods xmlns=\"http://www.loc.gov/mods/v3\" ");
    xml.push_str("xmlns:mods=\"http://www.loc.gov/mods/v3\" ");
    xml.push_str("xmlns:xlink=\"http://www.w3.org/1999/xlink\">\n");

    write_titles(&mut xml, record);
    write_names(&mut xml, record);
    write_type_of_resource(&mut xml, record);
    write_origin_info(&mut xml, record);
    write_physical_description(&mut xml, record);
    write_abstract(&mut xml, record);
    write_subjects(&mut xml, record);
    write_identifiers(&mut xml, record);
    write_language(&mut xml, record);

    xml.push_str("</mods>\n");
    Ok(xml)
}

fn write_titles(xml: &mut String, record: &Record) {
    if let Some(fields_245) = record.fields.get("245") {
        for field in fields_245 {
            xml.push_str("  <mods:titleInfo>\n");

            // Title (subfield a)
            if let Some(subfield) = field.subfields.iter().find(|s| s.code == 'a') {
                writeln!(
                    xml,
                    "    <mods:title>{}</mods:title>",
                    escape_xml(&subfield.value)
                )
                .ok();
            }

            // Subtitle (subfield b)
            if let Some(subfield) = field.subfields.iter().find(|s| s.code == 'b') {
                writeln!(
                    xml,
                    "    <mods:subTitle>{}</mods:subTitle>",
                    escape_xml(&subfield.value)
                )
                .ok();
            }

            xml.push_str("  </mods:titleInfo>\n");
        }
    }
}

fn write_names(xml: &mut String, record: &Record) {
    // Personal names (100, 700)
    for tag in &["100", "700"] {
        if let Some(fields) = record.fields.get(*tag) {
            for field in fields {
                if let Some(name_subfield) = field.subfields.iter().find(|s| s.code == 'a') {
                    xml.push_str("  <mods:name type=\"personal\">\n");
                    writeln!(
                        xml,
                        "    <mods:namePart>{}</mods:namePart>",
                        escape_xml(&name_subfield.value)
                    )
                    .ok();

                    // Dates (subfield d)
                    if let Some(date_subfield) = field.subfields.iter().find(|s| s.code == 'd') {
                        writeln!(
                            xml,
                            "    <mods:namePart type=\"date\">{}</mods:namePart>",
                            escape_xml(&date_subfield.value)
                        )
                        .ok();
                    }

                    // Role (subfield e)
                    if let Some(role_subfield) = field.subfields.iter().find(|s| s.code == 'e') {
                        writeln!(
                            xml,
                            "    <mods:role><mods:roleTerm>{}</mods:roleTerm></mods:role>",
                            escape_xml(&role_subfield.value)
                        )
                        .ok();
                    } else if *tag == "100" {
                        xml.push_str(
                            "    <mods:role><mods:roleTerm>creator</mods:roleTerm></mods:role>\n",
                        );
                    }

                    xml.push_str("  </mods:name>\n");
                }
            }
        }
    }

    // Corporate names (110, 710)
    for tag in &["110", "710"] {
        if let Some(fields) = record.fields.get(*tag) {
            for field in fields {
                if let Some(name_subfield) = field.subfields.iter().find(|s| s.code == 'a') {
                    xml.push_str("  <mods:name type=\"corporate\">\n");
                    writeln!(
                        xml,
                        "    <mods:namePart>{}</mods:namePart>",
                        escape_xml(&name_subfield.value)
                    )
                    .ok();

                    // Role (subfield e)
                    if let Some(role_subfield) = field.subfields.iter().find(|s| s.code == 'e') {
                        writeln!(
                            xml,
                            "    <mods:role><mods:roleTerm>{}</mods:roleTerm></mods:role>",
                            escape_xml(&role_subfield.value)
                        )
                        .ok();
                    }

                    xml.push_str("  </mods:name>\n");
                }
            }
        }
    }
}

fn write_type_of_resource(xml: &mut String, record: &Record) {
    let resource_type = match record.leader.record_type {
        'a' | 'c' | 'd' | 't' => "text",
        'e' | 'f' => "cartographic",
        'g' | 'k' => "still image",
        'i' | 'j' => "sound recording",
        'm' => "computer resource",
        'p' => "mixed material",
        'r' => "three dimensional object",
        _ => "unknown",
    };

    writeln!(
        xml,
        "  <mods:typeOfResource>{resource_type}</mods:typeOfResource>"
    )
    .ok();
}

fn write_origin_info(xml: &mut String, record: &Record) {
    if let Some(fields) = record.fields.get("260") {
        for field in fields {
            xml.push_str("  <mods:originInfo>\n");

            // Place of publication (subfield a)
            for subfield in field.subfields.iter().filter(|s| s.code == 'a') {
                writeln!(
                    xml,
                    "    <mods:place><mods:placeTerm>{}</mods:placeTerm></mods:place>",
                    escape_xml(&subfield.value)
                )
                .ok();
            }

            // Publisher (subfield b)
            for subfield in field.subfields.iter().filter(|s| s.code == 'b') {
                writeln!(
                    xml,
                    "    <mods:publisher>{}</mods:publisher>",
                    escape_xml(&subfield.value)
                )
                .ok();
            }

            // Date (subfield c)
            for subfield in field.subfields.iter().filter(|s| s.code == 'c') {
                writeln!(
                    xml,
                    "    <mods:dateIssued>{}</mods:dateIssued>",
                    escape_xml(&subfield.value)
                )
                .ok();
            }

            xml.push_str("  </mods:originInfo>\n");
        }
    }
}

fn write_physical_description(xml: &mut String, record: &Record) {
    if let Some(fields) = record.fields.get("300") {
        for field in fields {
            xml.push_str("  <mods:physicalDescription>\n");

            // Extent (subfield a)
            for subfield in field.subfields.iter().filter(|s| s.code == 'a') {
                writeln!(
                    xml,
                    "    <mods:extent>{}</mods:extent>",
                    escape_xml(&subfield.value)
                )
                .ok();
            }

            // Physical form (subfield b)
            for subfield in field.subfields.iter().filter(|s| s.code == 'b') {
                writeln!(
                    xml,
                    "    <mods:form>{}</mods:form>",
                    escape_xml(&subfield.value)
                )
                .ok();
            }

            // Dimensions (subfield c)
            for subfield in field.subfields.iter().filter(|s| s.code == 'c') {
                writeln!(
                    xml,
                    "    <mods:dimensions>{}</mods:dimensions>",
                    escape_xml(&subfield.value)
                )
                .ok();
            }

            xml.push_str("  </mods:physicalDescription>\n");
        }
    }
}

fn write_abstract(xml: &mut String, record: &Record) {
    // Summary/Abstract (520)
    if let Some(fields) = record.fields.get("520") {
        for field in fields {
            if let Some(subfield) = field.subfields.iter().find(|s| s.code == 'a') {
                writeln!(
                    xml,
                    "  <mods:abstract>{}</mods:abstract>",
                    escape_xml(&subfield.value)
                )
                .ok();
            }
        }
    }

    // General notes (500)
    if let Some(fields) = record.fields.get("500") {
        for field in fields {
            if let Some(subfield) = field.subfields.iter().find(|s| s.code == 'a') {
                writeln!(
                    xml,
                    "  <mods:note>{}</mods:note>",
                    escape_xml(&subfield.value)
                )
                .ok();
            }
        }
    }
}

fn write_subjects(xml: &mut String, record: &Record) {
    // Topical subjects (650)
    if let Some(fields) = record.fields.get("650") {
        for field in fields {
            if let Some(subfield) = field.subfields.iter().find(|s| s.code == 'a') {
                xml.push_str("  <mods:subject>\n");
                writeln!(
                    xml,
                    "    <mods:topic>{}</mods:topic>",
                    escape_xml(&subfield.value)
                )
                .ok();
                xml.push_str("  </mods:subject>\n");
            }
        }
    }

    // Geographic subjects (651)
    if let Some(fields) = record.fields.get("651") {
        for field in fields {
            if let Some(subfield) = field.subfields.iter().find(|s| s.code == 'a') {
                xml.push_str("  <mods:subject>\n");
                writeln!(
                    xml,
                    "    <mods:geographic>{}</mods:geographic>",
                    escape_xml(&subfield.value)
                )
                .ok();
                xml.push_str("  </mods:subject>\n");
            }
        }
    }
}

fn write_identifiers(xml: &mut String, record: &Record) {
    // ISBN (020)
    if let Some(fields) = record.fields.get("020") {
        for field in fields {
            if let Some(subfield) = field.subfields.iter().find(|s| s.code == 'a') {
                writeln!(
                    xml,
                    "  <mods:identifier type=\"isbn\">{}</mods:identifier>",
                    escape_xml(&subfield.value)
                )
                .ok();
            }
        }
    }

    // ISSN (022)
    if let Some(fields) = record.fields.get("022") {
        for field in fields {
            if let Some(subfield) = field.subfields.iter().find(|s| s.code == 'a') {
                writeln!(
                    xml,
                    "  <mods:identifier type=\"issn\">{}</mods:identifier>",
                    escape_xml(&subfield.value)
                )
                .ok();
            }
        }
    }

    // Control number (001)
    if let Some(control_001) = record.control_fields.get("001") {
        writeln!(
            xml,
            "  <mods:identifier type=\"local\">{}</mods:identifier>",
            escape_xml(control_001)
        )
        .ok();
    }
}

fn write_language(xml: &mut String, record: &Record) {
    if let Some(fields) = record.fields.get("041") {
        for field in fields {
            for subfield in field.subfields.iter().filter(|s| s.code == 'a') {
                xml.push_str("  <mods:language>\n");
                let langs: Vec<&str> = subfield.value.split_whitespace().collect();
                for lang in langs {
                    if !lang.is_empty() {
                        writeln!(
                            xml,
                            "    <mods:languageTerm type=\"code\" authority=\"iso639-2b\">{lang}</mods:languageTerm>"
                        )
                        .ok();
                    }
                }
                xml.push_str("  </mods:language>\n");
            }
        }
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
    fn test_basic_mods_structure() {
        let record = Record::new(make_test_leader());
        let mods = record_to_mods_xml(&record).expect("Failed to generate MODS");

        assert!(mods.contains("<?xml"));
        assert!(mods.contains("<mods"));
        assert!(mods.contains("</mods>"));
    }

    #[test]
    fn test_title_extraction() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test Title".to_string());
        record.add_field(field);

        let mods = record_to_mods_xml(&record).expect("Failed to generate MODS");
        assert!(mods.contains("<mods:title>Test Title</mods:title>"));
    }

    #[test]
    fn test_title_with_subtitle() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Main Title".to_string());
        field.add_subfield('b', "Subtitle".to_string());
        record.add_field(field);

        let mods = record_to_mods_xml(&record).expect("Failed to generate MODS");
        assert!(mods.contains("<mods:title>Main Title</mods:title>"));
        assert!(mods.contains("<mods:subTitle>Subtitle</mods:subTitle>"));
    }

    #[test]
    fn test_personal_name() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("100".to_string(), '1', ' ');
        field.add_subfield('a', "Smith, John".to_string());
        field.add_subfield('d', "1920-2000".to_string());
        record.add_field(field);

        let mods = record_to_mods_xml(&record).expect("Failed to generate MODS");
        assert!(mods.contains("<mods:name type=\"personal\">"));
        assert!(mods.contains("Smith, John"));
        assert!(mods.contains("1920-2000"));
    }

    #[test]
    fn test_corporate_name() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("110".to_string(), '2', ' ');
        field.add_subfield('a', "Library of Congress".to_string());
        record.add_field(field);

        let mods = record_to_mods_xml(&record).expect("Failed to generate MODS");
        assert!(mods.contains("<mods:name type=\"corporate\">"));
        assert!(mods.contains("Library of Congress"));
    }

    #[test]
    fn test_origin_info() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("260".to_string(), ' ', ' ');
        field.add_subfield('a', "New York :".to_string());
        field.add_subfield('b', "Penguin Books,".to_string());
        field.add_subfield('c', "2020.".to_string());
        record.add_field(field);

        let mods = record_to_mods_xml(&record).expect("Failed to generate MODS");
        assert!(mods.contains("<mods:originInfo>"));
        assert!(mods.contains("New York"));
        assert!(mods.contains("Penguin Books"));
        assert!(mods.contains("2020"));
    }

    #[test]
    fn test_physical_description() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("300".to_string(), ' ', ' ');
        field.add_subfield('a', "300 pages :".to_string());
        field.add_subfield('b', "illustrations ;".to_string());
        field.add_subfield('c', "24 cm".to_string());
        record.add_field(field);

        let mods = record_to_mods_xml(&record).expect("Failed to generate MODS");
        assert!(mods.contains("<mods:physicalDescription>"));
        assert!(mods.contains("<mods:extent>300 pages :</mods:extent>"));
        assert!(mods.contains("<mods:dimensions>24 cm</mods:dimensions>"));
    }

    #[test]
    fn test_subject_topical() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("650".to_string(), ' ', '0');
        field.add_subfield('a', "Science Fiction".to_string());
        record.add_field(field);

        let mods = record_to_mods_xml(&record).expect("Failed to generate MODS");
        assert!(mods.contains("<mods:subject>"));
        assert!(mods.contains("<mods:topic>Science Fiction</mods:topic>"));
    }

    #[test]
    fn test_subject_geographic() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("651".to_string(), ' ', '0');
        field.add_subfield('a', "United States".to_string());
        record.add_field(field);

        let mods = record_to_mods_xml(&record).expect("Failed to generate MODS");
        assert!(mods.contains("<mods:subject>"));
        assert!(mods.contains("<mods:geographic>United States</mods:geographic>"));
    }

    #[test]
    fn test_isbn() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("020".to_string(), ' ', ' ');
        field.add_subfield('a', "9780142424346".to_string());
        record.add_field(field);

        let mods = record_to_mods_xml(&record).expect("Failed to generate MODS");
        assert!(mods.contains("<mods:identifier type=\"isbn\">9780142424346</mods:identifier>"));
    }

    #[test]
    fn test_xml_escaping() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Title with <brackets> & ampersand".to_string());
        record.add_field(field);

        let mods = record_to_mods_xml(&record).expect("Failed to generate MODS");
        assert!(mods.contains("&lt;"));
        assert!(mods.contains("&gt;"));
        assert!(mods.contains("&amp;"));
    }

    #[test]
    fn test_type_of_resource() {
        let record = Record::new(make_test_leader());
        let mods = record_to_mods_xml(&record).expect("Failed to generate MODS");
        assert!(mods.contains("<mods:typeOfResource>text</mods:typeOfResource>"));
    }
}
