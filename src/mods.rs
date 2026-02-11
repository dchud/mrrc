//! MODS (Metadata Object Description Schema) conversion for MARC records.
//!
//! This module provides bidirectional conversion between MARC records and MODS XML format,
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

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::{MarcError, Result};
use crate::leader::Leader;
use crate::record::{Field, Record};

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

// ---------------------------------------------------------------------------
// MODS XML → MARC Record parsing
// ---------------------------------------------------------------------------

/// Strip namespace prefix from an XML element name, returning an owned `Vec<u8>`.
///
/// Handles both `mods:title` → `title` and plain `title` → `title`.
fn strip_ns_owned(name: &[u8]) -> Vec<u8> {
    match memchr::memchr(b':', name) {
        Some(pos) => name[pos + 1..].to_vec(),
        None => name.to_vec(),
    }
}

/// Captured info from a start-element event (owned, so `buf` can be reused).
struct StartInfo {
    local_name: Vec<u8>,
    attrs: Vec<(Vec<u8>, String)>,
}

impl StartInfo {
    fn from_event(e: &quick_xml::events::BytesStart<'_>) -> Self {
        let local_name = strip_ns_owned(e.name().as_ref());
        let attrs: Vec<(Vec<u8>, String)> = e
            .attributes()
            .flatten()
            .map(|a| {
                (
                    a.key.as_ref().to_vec(),
                    String::from_utf8_lossy(&a.value).to_string(),
                )
            })
            .collect();
        Self { local_name, attrs }
    }

    fn attr(&self, key: &[u8]) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k.as_slice() == key)
            .map(|(_, v)| v.as_str())
    }
}

/// Read the text content of the current element and consume the end tag.
fn read_text(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Result<String> {
    let mut text = String::new();
    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Text(e)) => {
                text.push_str(
                    &e.unescape()
                        .map_err(|err| MarcError::ParseError(format!("XML unescape: {err}")))?
                );
            }
            Ok(Event::CData(e)) => {
                text.push_str(&String::from_utf8_lossy(&e));
            }
            Ok(Event::End(_) | Event::Eof) => break,
            Err(e) => return Err(MarcError::ParseError(format!("XML read: {e}"))),
            _ => {}
        }
        buf.clear();
    }
    Ok(text)
}

/// Skip over the current element and all its children until the matching end tag.
fn skip_element(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Result<()> {
    let mut depth: u32 = 1;
    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(_)) => depth += 1,
            Ok(Event::End(_)) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(MarcError::ParseError(format!("XML skip: {e}"))),
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

/// Create a default MARC leader suitable for records built from MODS.
fn make_default_leader() -> Leader {
    Leader {
        record_length: 0,
        record_status: 'n',
        record_type: 'a',
        bibliographic_level: 'm',
        control_record_type: ' ',
        character_coding: 'a',
        indicator_count: 2,
        subfield_code_count: 2,
        data_base_address: 0,
        encoding_level: ' ',
        cataloging_form: 'a',
        multipart_level: ' ',
        reserved: "4500".to_string(),
    }
}

/// Map a MODS `typeOfResource` string to a MARC leader record type code.
fn resource_type_to_leader_code(s: &str) -> char {
    match s.trim() {
        "cartographic" => 'e',
        "notated music" => 'c',
        "sound recording" | "sound recording-musical" | "sound recording-nonmusical" => 'i',
        "still image" => 'k',
        "moving image" => 'g',
        "software, multimedia" | "computer resource" => 'm',
        "three dimensional object" => 'r',
        "mixed material" => 'p',
        // "text" and anything unrecognized default to 'a' (language material)
        _ => 'a',
    }
}

/// Read the next start-element event, returning owned `StartInfo`. Returns `None` at EOF or end-tag.
fn next_start(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Result<Option<StartInfo>> {
    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => {
                let info = StartInfo::from_event(e);
                buf.clear();
                return Ok(Some(info));
            }
            Ok(Event::End(_) | Event::Eof) => return Ok(None),
            Err(e) => return Err(MarcError::ParseError(format!("XML read: {e}"))),
            _ => {}
        }
        buf.clear();
    }
}

/// Parse a single MODS XML document into a MARC [`Record`].
///
/// The XML should contain a single `<mods>` root element (with or without
/// namespace prefixes). The parser maps MODS elements to MARC fields following
/// the LOC MODS-to-MARC crosswalk.
///
/// # Errors
///
/// Returns an error if the XML is malformed or cannot be parsed.
pub fn mods_xml_to_record(xml: &str) -> Result<Record> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);
    let mut buf = Vec::new();

    // Advance to the <mods> start element
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = strip_ns_owned(e.name().as_ref());
                if local == b"mods" {
                    buf.clear();
                    return parse_mods_element(&mut reader, &mut buf);
                }
            }
            Ok(Event::Eof) => {
                return Err(MarcError::ParseError(
                    "No <mods> element found".to_string(),
                ));
            }
            Err(e) => return Err(MarcError::ParseError(format!("XML read: {e}"))),
            _ => {}
        }
        buf.clear();
    }
}

/// Parse a `<modsCollection>` containing multiple MODS records.
///
/// # Errors
///
/// Returns an error if the XML is malformed or cannot be parsed.
pub fn mods_xml_to_records(xml: &str) -> Result<Vec<Record>> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);
    let mut buf = Vec::new();
    let mut records = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = strip_ns_owned(e.name().as_ref());
                buf.clear();
                if local == b"mods" {
                    records.push(parse_mods_element(&mut reader, &mut buf)?);
                }
                // else modsCollection — continue into children
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(MarcError::ParseError(format!("XML read: {e}"))),
            _ => {}
        }
        buf.clear();
    }

    // If called on a single <mods> document, still return it
    if records.is_empty() {
        return mods_xml_to_record(xml).map(|r| vec![r]);
    }

    Ok(records)
}

/// Parse the children of a `<mods>` element into a MARC Record.
fn parse_mods_element(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Result<Record> {
    let mut record = Record::new(make_default_leader());
    // Track whether we've assigned the primary 1XX entries
    let mut has_100 = false;
    let mut has_110 = false;
    let mut has_111 = false;

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => {
                // Capture owned info so we can release the borrow on buf
                let info = StartInfo::from_event(e);
                buf.clear();
                match info.local_name.as_slice() {
                    b"titleInfo" => parse_title_info(reader, buf, &info, &mut record)?,
                    b"name" => parse_name(
                        reader,
                        buf,
                        &info,
                        &mut record,
                        &mut has_100,
                        &mut has_110,
                        &mut has_111,
                    )?,
                    b"typeOfResource" => parse_type_of_resource(reader, buf, &mut record)?,
                    b"originInfo" => parse_origin_info(reader, buf, &mut record)?,
                    b"physicalDescription" => {
                        parse_physical_description(reader, buf, &mut record)?;
                    }
                    b"abstract" => parse_abstract(reader, buf, &mut record)?,
                    b"note" => parse_note(reader, buf, &mut record)?,
                    b"subject" => parse_subject(reader, buf, &mut record)?,
                    b"identifier" => parse_identifier(reader, buf, &info, &mut record)?,
                    b"language" => parse_language(reader, buf, &mut record)?,
                    b"genre" => parse_genre(reader, buf, &mut record)?,
                    b"classification" => {
                        parse_classification(reader, buf, &info, &mut record)?;
                    }
                    b"location" => parse_location(reader, buf, &mut record)?,
                    b"relatedItem" => parse_related_item(reader, buf, &info, &mut record)?,
                    b"recordInfo" => parse_record_info(reader, buf, &mut record)?,
                    b"accessCondition" => {
                        parse_access_condition(reader, buf, &info, &mut record)?;
                    }
                    b"tableOfContents" => parse_table_of_contents(reader, buf, &mut record)?,
                    b"targetAudience" => parse_target_audience(reader, buf, &mut record)?,
                    _ => skip_element(reader, buf)?,
                }
            }
            Ok(Event::End(ref e)) => {
                let local = strip_ns_owned(e.name().as_ref());
                if local == b"mods" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(MarcError::ParseError(format!("XML read: {e}"))),
            _ => {}
        }
        buf.clear();
    }

    Ok(record)
}

/// Parse `<titleInfo>` → 245 or 246 depending on `@type`.
fn parse_title_info(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    info: &StartInfo,
    record: &mut Record,
) -> Result<()> {
    let title_type = info.attr(b"type");
    let tag = match title_type {
        Some("alternative" | "abbreviated" | "translated" | "uniform") => "246",
        _ => "245",
    };

    let (ind1, ind2) = if tag == "245" {
        ('0', '0')
    } else {
        ('1', ' ')
    };

    let mut field = Field::new(tag.to_string(), ind1, ind2);
    let mut has_content = false;

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => {
                let local = strip_ns_owned(e.name().as_ref());
                buf.clear();
                match local.as_slice() {
                    b"title" => {
                        let text = read_text(reader, buf)?;
                        if !text.is_empty() {
                            field.add_subfield('a', text);
                            has_content = true;
                        }
                    }
                    b"subTitle" => {
                        let text = read_text(reader, buf)?;
                        if !text.is_empty() {
                            field.add_subfield('b', text);
                            has_content = true;
                        }
                    }
                    b"partNumber" => {
                        let text = read_text(reader, buf)?;
                        if !text.is_empty() {
                            field.add_subfield('n', text);
                            has_content = true;
                        }
                    }
                    b"partName" => {
                        let text = read_text(reader, buf)?;
                        if !text.is_empty() {
                            field.add_subfield('p', text);
                            has_content = true;
                        }
                    }
                    b"nonSort" => {
                        let _text = read_text(reader, buf)?;
                    }
                    _ => skip_element(reader, buf)?,
                }
            }
            Ok(Event::End(ref e)) => {
                let local = strip_ns_owned(e.name().as_ref());
                if local == b"titleInfo" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(MarcError::ParseError(format!("XML read: {e}"))),
            _ => {}
        }
        buf.clear();
    }

    if has_content {
        record.add_field(field);
    }
    Ok(())
}

/// Parse `<name>` → 100/110/111/700/710/711 depending on `@type` and role.
#[allow(clippy::too_many_arguments)]
fn parse_name(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    info: &StartInfo,
    record: &mut Record,
    has_100: &mut bool,
    has_110: &mut bool,
    has_111: &mut bool,
) -> Result<()> {
    let name_type = info.attr(b"type").map(String::from);
    let mut name_parts: Vec<String> = Vec::new();
    let mut date_part: Option<String> = None;
    let mut role_term: Option<String> = None;

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => {
                let child = StartInfo::from_event(e);
                buf.clear();
                match child.local_name.as_slice() {
                    b"namePart" => {
                        let part_type = child.attr(b"type").map(String::from);
                        let text = read_text(reader, buf)?;
                        if !text.is_empty() {
                            if part_type.as_deref() == Some("date") {
                                date_part = Some(text);
                            } else {
                                name_parts.push(text);
                            }
                        }
                    }
                    b"role" => {
                        // Read inside <role> to find <roleTerm>
                        while let Some(role_child) = next_start(reader, buf)? {
                            if role_child.local_name == b"roleTerm" {
                                let text = read_text(reader, buf)?;
                                if !text.is_empty() {
                                    role_term = Some(text);
                                }
                            } else {
                                skip_element(reader, buf)?;
                            }
                        }
                    }
                    _ => skip_element(reader, buf)?,
                }
            }
            Ok(Event::End(ref e)) => {
                let local = strip_ns_owned(e.name().as_ref());
                if local == b"name" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(MarcError::ParseError(format!("XML read: {e}"))),
            _ => {}
        }
        buf.clear();
    }

    if name_parts.is_empty() {
        return Ok(());
    }

    let is_creator = role_term.as_deref() == Some("creator");

    // Determine MARC tag based on name type and whether primary entry is taken
    let (tag, ind1) = match name_type.as_deref() {
        Some("corporate") => {
            if is_creator && !*has_110 {
                *has_110 = true;
                ("110", '2')
            } else {
                ("710", '2')
            }
        }
        Some("conference") => {
            if is_creator && !*has_111 {
                *has_111 = true;
                ("111", '2')
            } else {
                ("711", '2')
            }
        }
        // "personal" and unspecified types default to personal name
        _ => {
            if is_creator && !*has_100 {
                *has_100 = true;
                ("100", '1')
            } else {
                ("700", '1')
            }
        }
    };

    let mut field = Field::new(tag.to_string(), ind1, ' ');
    field.add_subfield('a', name_parts.join(" "));
    if let Some(d) = date_part {
        field.add_subfield('d', d);
    }
    if let Some(ref role) = role_term {
        field.add_subfield('e', role.clone());
    }
    record.add_field(field);
    Ok(())
}

/// Parse `<typeOfResource>` → Leader record type.
fn parse_type_of_resource(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    record: &mut Record,
) -> Result<()> {
    let text = read_text(reader, buf)?;
    if !text.is_empty() {
        record.leader.record_type = resource_type_to_leader_code(&text);
    }
    Ok(())
}

/// Parse `<originInfo>` → 260 (place/publisher/date) and 250 (edition).
fn parse_origin_info(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    record: &mut Record,
) -> Result<()> {
    let mut place: Option<String> = None;
    let mut publisher: Option<String> = None;
    let mut date_issued: Option<String> = None;
    let mut edition: Option<String> = None;

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => {
                let local = strip_ns_owned(e.name().as_ref());
                buf.clear();
                match local.as_slice() {
                    b"place" => {
                        // Read inside <place> to find <placeTerm>
                        while let Some(child) = next_start(reader, buf)? {
                            if child.local_name == b"placeTerm" {
                                let text = read_text(reader, buf)?;
                                if !text.is_empty() {
                                    place = Some(text);
                                }
                            } else {
                                skip_element(reader, buf)?;
                            }
                        }
                    }
                    b"publisher" => {
                        let text = read_text(reader, buf)?;
                        if !text.is_empty() {
                            publisher = Some(text);
                        }
                    }
                    b"dateIssued" => {
                        let text = read_text(reader, buf)?;
                        if !text.is_empty() {
                            date_issued = Some(text);
                        }
                    }
                    b"dateCreated" => {
                        let text = read_text(reader, buf)?;
                        if date_issued.is_none() && !text.is_empty() {
                            date_issued = Some(text);
                        }
                    }
                    b"edition" => {
                        let text = read_text(reader, buf)?;
                        if !text.is_empty() {
                            edition = Some(text);
                        }
                    }
                    _ => skip_element(reader, buf)?,
                }
            }
            Ok(Event::End(ref e)) => {
                let local = strip_ns_owned(e.name().as_ref());
                if local == b"originInfo" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(MarcError::ParseError(format!("XML read: {e}"))),
            _ => {}
        }
        buf.clear();
    }

    if place.is_some() || publisher.is_some() || date_issued.is_some() {
        let mut field = Field::new("260".to_string(), ' ', ' ');
        if let Some(p) = place {
            field.add_subfield('a', p);
        }
        if let Some(p) = publisher {
            field.add_subfield('b', p);
        }
        if let Some(d) = date_issued {
            field.add_subfield('c', d);
        }
        record.add_field(field);
    }

    if let Some(ed) = edition {
        let mut field = Field::new("250".to_string(), ' ', ' ');
        field.add_subfield('a', ed);
        record.add_field(field);
    }

    Ok(())
}

/// Parse `<physicalDescription>` → 300.
fn parse_physical_description(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    record: &mut Record,
) -> Result<()> {
    let mut extent: Option<String> = None;
    let mut form: Option<String> = None;
    let mut dimensions: Option<String> = None;

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => {
                let local = strip_ns_owned(e.name().as_ref());
                buf.clear();
                match local.as_slice() {
                    b"extent" => {
                        let text = read_text(reader, buf)?;
                        if !text.is_empty() {
                            extent = Some(text);
                        }
                    }
                    b"form" => {
                        let text = read_text(reader, buf)?;
                        if !text.is_empty() {
                            form = Some(text);
                        }
                    }
                    b"dimensions" => {
                        let text = read_text(reader, buf)?;
                        if !text.is_empty() {
                            dimensions = Some(text);
                        }
                    }
                    _ => skip_element(reader, buf)?,
                }
            }
            Ok(Event::End(ref e)) => {
                let local = strip_ns_owned(e.name().as_ref());
                if local == b"physicalDescription" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(MarcError::ParseError(format!("XML read: {e}"))),
            _ => {}
        }
        buf.clear();
    }

    if extent.is_some() || form.is_some() || dimensions.is_some() {
        let mut field = Field::new("300".to_string(), ' ', ' ');
        if let Some(ext) = extent {
            field.add_subfield('a', ext);
        }
        if let Some(f) = form {
            field.add_subfield('b', f);
        }
        if let Some(d) = dimensions {
            field.add_subfield('c', d);
        }
        record.add_field(field);
    }

    Ok(())
}

/// Parse `<abstract>` → 520 $a.
fn parse_abstract(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    record: &mut Record,
) -> Result<()> {
    let text = read_text(reader, buf)?;
    if !text.is_empty() {
        let mut field = Field::new("520".to_string(), ' ', ' ');
        field.add_subfield('a', text);
        record.add_field(field);
    }
    Ok(())
}

/// Parse `<note>` → 500 $a.
fn parse_note(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    record: &mut Record,
) -> Result<()> {
    let text = read_text(reader, buf)?;
    if !text.is_empty() {
        let mut field = Field::new("500".to_string(), ' ', ' ');
        field.add_subfield('a', text);
        record.add_field(field);
    }
    Ok(())
}

/// Parse `<subject>` → 650/651 (topic/geographic).
fn parse_subject(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    record: &mut Record,
) -> Result<()> {
    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => {
                let local = strip_ns_owned(e.name().as_ref());
                buf.clear();
                match local.as_slice() {
                    b"topic" => {
                        let text = read_text(reader, buf)?;
                        if !text.is_empty() {
                            let mut field = Field::new("650".to_string(), ' ', '0');
                            field.add_subfield('a', text);
                            record.add_field(field);
                        }
                    }
                    b"geographic" => {
                        let text = read_text(reader, buf)?;
                        if !text.is_empty() {
                            let mut field = Field::new("651".to_string(), ' ', '0');
                            field.add_subfield('a', text);
                            record.add_field(field);
                        }
                    }
                    b"temporal" => {
                        let text = read_text(reader, buf)?;
                        if !text.is_empty() {
                            let mut field = Field::new("650".to_string(), ' ', '0');
                            field.add_subfield('y', text);
                            record.add_field(field);
                        }
                    }
                    _ => skip_element(reader, buf)?,
                }
            }
            Ok(Event::End(ref e)) => {
                let local = strip_ns_owned(e.name().as_ref());
                if local == b"subject" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(MarcError::ParseError(format!("XML read: {e}"))),
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

/// Parse `<identifier>` → 020/022/010/024/001 depending on `@type`.
fn parse_identifier(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    info: &StartInfo,
    record: &mut Record,
) -> Result<()> {
    let id_type = info.attr(b"type").map(String::from);
    let text = read_text(reader, buf)?;

    if text.is_empty() {
        return Ok(());
    }

    match id_type.as_deref() {
        Some("isbn") => {
            let mut field = Field::new("020".to_string(), ' ', ' ');
            field.add_subfield('a', text);
            record.add_field(field);
        }
        Some("issn") => {
            let mut field = Field::new("022".to_string(), ' ', ' ');
            field.add_subfield('a', text);
            record.add_field(field);
        }
        Some("lccn") => {
            let mut field = Field::new("010".to_string(), ' ', ' ');
            field.add_subfield('a', text);
            record.add_field(field);
        }
        Some("doi" | "hdl" | "uri") => {
            let mut field = Field::new("024".to_string(), '7', ' ');
            field.add_subfield('a', text);
            if let Some(ref t) = id_type {
                field.add_subfield('2', t.clone());
            }
            record.add_field(field);
        }
        Some("local") => {
            record.add_control_field("001".to_string(), text);
        }
        _ => {
            let mut field = Field::new("024".to_string(), '8', ' ');
            field.add_subfield('a', text);
            record.add_field(field);
        }
    }

    Ok(())
}

/// Parse `<language>` → 041 $a.
fn parse_language(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    record: &mut Record,
) -> Result<()> {
    let mut codes = Vec::new();

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => {
                let local = strip_ns_owned(e.name().as_ref());
                buf.clear();
                if local == b"languageTerm" {
                    let text = read_text(reader, buf)?;
                    if !text.is_empty() {
                        codes.push(text);
                    }
                } else {
                    skip_element(reader, buf)?;
                }
            }
            Ok(Event::End(ref e)) => {
                let local = strip_ns_owned(e.name().as_ref());
                if local == b"language" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(MarcError::ParseError(format!("XML read: {e}"))),
            _ => {}
        }
        buf.clear();
    }

    for code in codes {
        let mut field = Field::new("041".to_string(), ' ', ' ');
        field.add_subfield('a', code);
        record.add_field(field);
    }

    Ok(())
}

/// Parse `<genre>` → 655 $a.
fn parse_genre(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    record: &mut Record,
) -> Result<()> {
    let text = read_text(reader, buf)?;
    if !text.is_empty() {
        let mut field = Field::new("655".to_string(), ' ', '7');
        field.add_subfield('a', text);
        record.add_field(field);
    }
    Ok(())
}

/// Parse `<classification>` → 050/082 depending on `@authority`.
fn parse_classification(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    info: &StartInfo,
    record: &mut Record,
) -> Result<()> {
    let authority = info.attr(b"authority").map(String::from);
    let text = read_text(reader, buf)?;

    if text.is_empty() {
        return Ok(());
    }

    match authority.as_deref() {
        Some("lcc") => {
            let mut field = Field::new("050".to_string(), ' ', '4');
            field.add_subfield('a', text);
            record.add_field(field);
        }
        Some("ddc") => {
            let mut field = Field::new("082".to_string(), '0', '4');
            field.add_subfield('a', text);
            record.add_field(field);
        }
        _ => {
            let mut field = Field::new("084".to_string(), ' ', ' ');
            field.add_subfield('a', text);
            if let Some(auth) = authority {
                field.add_subfield('2', auth);
            }
            record.add_field(field);
        }
    }

    Ok(())
}

/// Parse `<location>` → 856 $u or 852 $a.
fn parse_location(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    record: &mut Record,
) -> Result<()> {
    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => {
                let local = strip_ns_owned(e.name().as_ref());
                buf.clear();
                match local.as_slice() {
                    b"url" => {
                        let text = read_text(reader, buf)?;
                        if !text.is_empty() {
                            let mut field = Field::new("856".to_string(), '4', '0');
                            field.add_subfield('u', text);
                            record.add_field(field);
                        }
                    }
                    b"physicalLocation" => {
                        let text = read_text(reader, buf)?;
                        if !text.is_empty() {
                            let mut field = Field::new("852".to_string(), ' ', ' ');
                            field.add_subfield('a', text);
                            record.add_field(field);
                        }
                    }
                    b"shelfLocator" => {
                        let text = read_text(reader, buf)?;
                        if !text.is_empty() {
                            let mut field = Field::new("852".to_string(), ' ', ' ');
                            field.add_subfield('h', text);
                            record.add_field(field);
                        }
                    }
                    _ => skip_element(reader, buf)?,
                }
            }
            Ok(Event::End(ref e)) => {
                let local = strip_ns_owned(e.name().as_ref());
                if local == b"location" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(MarcError::ParseError(format!("XML read: {e}"))),
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

/// Parse `<relatedItem>` → 773/780/785/830 depending on `@type`.
fn parse_related_item(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    info: &StartInfo,
    record: &mut Record,
) -> Result<()> {
    let rel_type = info.attr(b"type");
    let tag = match rel_type {
        Some("host") => "773",
        Some("preceding") => "780",
        Some("succeeding") => "785",
        Some("series") => "830",
        _ => "787",
    };

    let mut title: Option<String> = None;

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => {
                let local = strip_ns_owned(e.name().as_ref());
                buf.clear();
                if local == b"titleInfo" {
                    // Read children to find <title>
                    while let Some(child) = next_start(reader, buf)? {
                        if child.local_name == b"title" {
                            let text = read_text(reader, buf)?;
                            if !text.is_empty() {
                                title = Some(text);
                            }
                        } else {
                            skip_element(reader, buf)?;
                        }
                    }
                } else {
                    skip_element(reader, buf)?;
                }
            }
            Ok(Event::End(ref e)) => {
                let local = strip_ns_owned(e.name().as_ref());
                if local == b"relatedItem" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(MarcError::ParseError(format!("XML read: {e}"))),
            _ => {}
        }
        buf.clear();
    }

    if let Some(t) = title {
        let ind1 = if tag == "830" { ' ' } else { '0' };
        let mut field = Field::new(tag.to_string(), ind1, ' ');
        let sub_code = if tag == "830" { 'a' } else { 't' };
        field.add_subfield(sub_code, t);
        record.add_field(field);
    }

    Ok(())
}

/// Parse `<recordInfo>` → 001, 003, 040.
fn parse_record_info(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    record: &mut Record,
) -> Result<()> {
    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => {
                let child = StartInfo::from_event(e);
                buf.clear();
                match child.local_name.as_slice() {
                    b"recordIdentifier" => {
                        let source = child.attr(b"source").map(String::from);
                        let text = read_text(reader, buf)?;
                        if !text.is_empty() {
                            record.add_control_field("001".to_string(), text);
                        }
                        if let Some(src) = source {
                            if !src.is_empty() {
                                record.add_control_field("003".to_string(), src);
                            }
                        }
                    }
                    b"recordContentSource" => {
                        let text = read_text(reader, buf)?;
                        if !text.is_empty() {
                            let mut field = Field::new("040".to_string(), ' ', ' ');
                            field.add_subfield('a', text);
                            record.add_field(field);
                        }
                    }
                    b"languageOfCataloging" => {
                        while let Some(inner) = next_start(reader, buf)? {
                            if inner.local_name == b"languageTerm" {
                                let text = read_text(reader, buf)?;
                                if !text.is_empty() {
                                    let mut field = Field::new("040".to_string(), ' ', ' ');
                                    field.add_subfield('b', text);
                                    record.add_field(field);
                                }
                            } else {
                                skip_element(reader, buf)?;
                            }
                        }
                    }
                    _ => skip_element(reader, buf)?,
                }
            }
            Ok(Event::End(ref e)) => {
                let local = strip_ns_owned(e.name().as_ref());
                if local == b"recordInfo" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(MarcError::ParseError(format!("XML read: {e}"))),
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

/// Parse `<accessCondition>` → 506 or 540.
fn parse_access_condition(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    info: &StartInfo,
    record: &mut Record,
) -> Result<()> {
    let cond_type = info.attr(b"type");
    let text = read_text(reader, buf)?;

    if text.is_empty() {
        return Ok(());
    }

    let tag = match cond_type {
        Some("restriction on access" | "restrictionOnAccess") => "506",
        // "use and reproduction", "useAndReproduction", and unspecified types default to 540
        _ => "540",
    };

    let mut field = Field::new(tag.to_string(), ' ', ' ');
    field.add_subfield('a', text);
    record.add_field(field);
    Ok(())
}

/// Parse `<tableOfContents>` → 505 $a.
fn parse_table_of_contents(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    record: &mut Record,
) -> Result<()> {
    let text = read_text(reader, buf)?;
    if !text.is_empty() {
        let mut field = Field::new("505".to_string(), '0', ' ');
        field.add_subfield('a', text);
        record.add_field(field);
    }
    Ok(())
}

/// Parse `<targetAudience>` → 521 $a.
fn parse_target_audience(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    record: &mut Record,
) -> Result<()> {
    let text = read_text(reader, buf)?;
    if !text.is_empty() {
        let mut field = Field::new("521".to_string(), ' ', ' ');
        field.add_subfield('a', text);
        record.add_field(field);
    }
    Ok(())
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

    // -----------------------------------------------------------------------
    // MODS parsing tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_mods_parse_unprefixed() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <titleInfo><title>Test Title</title></titleInfo>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        let fields = record.get_fields("245").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Test Title"));
    }

    #[test]
    fn test_mods_parse_prefixed() {
        let xml = r#"<mods:mods xmlns:mods="http://www.loc.gov/mods/v3">
            <mods:titleInfo><mods:title>Prefixed Title</mods:title></mods:titleInfo>
        </mods:mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        let fields = record.get_fields("245").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Prefixed Title"));
    }

    #[test]
    fn test_mods_parse_title_info() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <titleInfo><title>Main Title</title></titleInfo>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        let fields = record.get_fields("245").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Main Title"));
    }

    #[test]
    fn test_mods_parse_title_with_subtitle() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <titleInfo>
                <title>Main Title</title>
                <subTitle>A Subtitle</subTitle>
            </titleInfo>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        let fields = record.get_fields("245").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Main Title"));
        assert_eq!(fields[0].get_subfield('b'), Some("A Subtitle"));
    }

    #[test]
    fn test_mods_parse_alternative_title() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <titleInfo type="alternative">
                <title>Alt Title</title>
            </titleInfo>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        let fields = record.get_fields("246").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Alt Title"));
    }

    #[test]
    fn test_mods_parse_personal_name() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <name type="personal">
                <namePart>Smith, John</namePart>
                <namePart type="date">1920-2000</namePart>
                <role><roleTerm>creator</roleTerm></role>
            </name>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        let fields = record.get_fields("100").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Smith, John"));
        assert_eq!(fields[0].get_subfield('d'), Some("1920-2000"));
        assert_eq!(fields[0].get_subfield('e'), Some("creator"));
    }

    #[test]
    fn test_mods_parse_corporate_name() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <name type="corporate">
                <namePart>Library of Congress</namePart>
                <role><roleTerm>creator</roleTerm></role>
            </name>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        let fields = record.get_fields("110").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Library of Congress"));
    }

    #[test]
    fn test_mods_parse_multiple_names() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <name type="personal">
                <namePart>Smith, Jane</namePart>
                <role><roleTerm>creator</roleTerm></role>
            </name>
            <name type="personal">
                <namePart>Jones, Bob</namePart>
                <role><roleTerm>author</roleTerm></role>
            </name>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        // First creator goes to 100
        let fields_100 = record.get_fields("100").unwrap();
        assert_eq!(fields_100[0].get_subfield('a'), Some("Smith, Jane"));
        // Second name goes to 700 (100 already taken)
        let fields_700 = record.get_fields("700").unwrap();
        assert_eq!(fields_700[0].get_subfield('a'), Some("Jones, Bob"));
    }

    #[test]
    fn test_mods_parse_type_of_resource() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <typeOfResource>text</typeOfResource>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        assert_eq!(record.leader.record_type, 'a');

        let xml2 = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <typeOfResource>cartographic</typeOfResource>
        </mods>"#;
        let record2 = mods_xml_to_record(xml2).unwrap();
        assert_eq!(record2.leader.record_type, 'e');

        let xml3 = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <typeOfResource>sound recording</typeOfResource>
        </mods>"#;
        let record3 = mods_xml_to_record(xml3).unwrap();
        assert_eq!(record3.leader.record_type, 'i');
    }

    #[test]
    fn test_mods_parse_origin_info() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <originInfo>
                <place><placeTerm>New York</placeTerm></place>
                <publisher>Penguin Books</publisher>
                <dateIssued>2020</dateIssued>
            </originInfo>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        let fields = record.get_fields("260").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("New York"));
        assert_eq!(fields[0].get_subfield('b'), Some("Penguin Books"));
        assert_eq!(fields[0].get_subfield('c'), Some("2020"));
    }

    #[test]
    fn test_mods_parse_physical_description() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <physicalDescription>
                <extent>300 pages</extent>
                <form>print</form>
                <dimensions>24 cm</dimensions>
            </physicalDescription>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        let fields = record.get_fields("300").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("300 pages"));
        assert_eq!(fields[0].get_subfield('b'), Some("print"));
        assert_eq!(fields[0].get_subfield('c'), Some("24 cm"));
    }

    #[test]
    fn test_mods_parse_abstract() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <abstract>This is a test abstract.</abstract>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        let fields = record.get_fields("520").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("This is a test abstract."));
    }

    #[test]
    fn test_mods_parse_note() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <note>A general note.</note>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        let fields = record.get_fields("500").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("A general note."));
    }

    #[test]
    fn test_mods_parse_subject_topic() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <subject><topic>Rust programming</topic></subject>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        let fields = record.get_fields("650").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Rust programming"));
    }

    #[test]
    fn test_mods_parse_subject_geographic() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <subject><geographic>United States</geographic></subject>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        let fields = record.get_fields("651").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("United States"));
    }

    #[test]
    fn test_mods_parse_identifiers() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <identifier type="isbn">9780142424346</identifier>
            <identifier type="issn">0028-0836</identifier>
            <identifier type="lccn">2020012345</identifier>
            <identifier type="local">ocm12345</identifier>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();

        let isbn = record.get_fields("020").unwrap();
        assert_eq!(isbn[0].get_subfield('a'), Some("9780142424346"));

        let issn = record.get_fields("022").unwrap();
        assert_eq!(issn[0].get_subfield('a'), Some("0028-0836"));

        let lccn = record.get_fields("010").unwrap();
        assert_eq!(lccn[0].get_subfield('a'), Some("2020012345"));

        assert_eq!(record.get_control_field("001"), Some("ocm12345"));
    }

    #[test]
    fn test_mods_parse_language() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <language>
                <languageTerm type="code" authority="iso639-2b">eng</languageTerm>
            </language>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        let fields = record.get_fields("041").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("eng"));
    }

    #[test]
    fn test_mods_parse_genre() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <genre>Handbooks and manuals</genre>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        let fields = record.get_fields("655").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Handbooks and manuals"));
    }

    #[test]
    fn test_mods_parse_classification() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <classification authority="lcc">QA76.73</classification>
            <classification authority="ddc">005.133</classification>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();

        let lcc = record.get_fields("050").unwrap();
        assert_eq!(lcc[0].get_subfield('a'), Some("QA76.73"));

        let ddc = record.get_fields("082").unwrap();
        assert_eq!(ddc[0].get_subfield('a'), Some("005.133"));
    }

    #[test]
    fn test_mods_parse_location_url() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <location>
                <url>https://example.com/resource</url>
            </location>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        let fields = record.get_fields("856").unwrap();
        assert_eq!(fields[0].get_subfield('u'), Some("https://example.com/resource"));
    }

    #[test]
    fn test_mods_parse_related_item() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <relatedItem type="host">
                <titleInfo><title>Host Journal</title></titleInfo>
            </relatedItem>
            <relatedItem type="series">
                <titleInfo><title>Book Series</title></titleInfo>
            </relatedItem>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();

        let host = record.get_fields("773").unwrap();
        assert_eq!(host[0].get_subfield('t'), Some("Host Journal"));

        let series = record.get_fields("830").unwrap();
        assert_eq!(series[0].get_subfield('a'), Some("Book Series"));
    }

    #[test]
    fn test_mods_parse_record_info() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <recordInfo>
                <recordIdentifier source="OCoLC">12345678</recordIdentifier>
                <recordContentSource>DLC</recordContentSource>
            </recordInfo>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        assert_eq!(record.get_control_field("001"), Some("12345678"));
        assert_eq!(record.get_control_field("003"), Some("OCoLC"));
        let fields_040 = record.get_fields("040").unwrap();
        assert_eq!(fields_040[0].get_subfield('a'), Some("DLC"));
    }

    #[test]
    fn test_mods_parse_access_condition() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <accessCondition type="restriction on access">Restricted</accessCondition>
            <accessCondition type="use and reproduction">Public domain</accessCondition>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();

        let f506 = record.get_fields("506").unwrap();
        assert_eq!(f506[0].get_subfield('a'), Some("Restricted"));

        let f540 = record.get_fields("540").unwrap();
        assert_eq!(f540[0].get_subfield('a'), Some("Public domain"));
    }

    #[test]
    fn test_mods_parse_table_of_contents() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <tableOfContents>Chapter 1 -- Chapter 2</tableOfContents>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        let fields = record.get_fields("505").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Chapter 1 -- Chapter 2"));
    }

    #[test]
    fn test_mods_parse_target_audience() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <targetAudience>General</targetAudience>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        let fields = record.get_fields("521").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("General"));
    }

    #[test]
    fn test_mods_parse_empty_elements() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <titleInfo><title></title></titleInfo>
            <abstract></abstract>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        // Empty elements should not produce fields
        assert!(record.get_fields("245").is_none());
        assert!(record.get_fields("520").is_none());
    }

    #[test]
    fn test_mods_parse_xml_entities() {
        let xml = r#"<mods xmlns="http://www.loc.gov/mods/v3">
            <titleInfo><title>Title &amp; More &lt;stuff&gt;</title></titleInfo>
        </mods>"#;
        let record = mods_xml_to_record(xml).unwrap();
        let fields = record.get_fields("245").unwrap();
        assert_eq!(fields[0].get_subfield('a'), Some("Title & More <stuff>"));
    }

    #[test]
    fn test_mods_collection_parse() {
        let xml = r#"<modsCollection xmlns="http://www.loc.gov/mods/v3">
            <mods>
                <titleInfo><title>First Record</title></titleInfo>
            </mods>
            <mods>
                <titleInfo><title>Second Record</title></titleInfo>
            </mods>
        </modsCollection>"#;
        let records = mods_xml_to_records(xml).unwrap();
        assert_eq!(records.len(), 2);

        let f1 = records[0].get_fields("245").unwrap();
        assert_eq!(f1[0].get_subfield('a'), Some("First Record"));

        let f2 = records[1].get_fields("245").unwrap();
        assert_eq!(f2[0].get_subfield('a'), Some("Second Record"));
    }

    #[test]
    fn test_mods_roundtrip() {
        // Build a MARC record, write to MODS, read back
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test123".to_string());

        let mut f245 = Field::new("245".to_string(), '1', '0');
        f245.add_subfield('a', "Test Title".to_string());
        f245.add_subfield('b', "A Subtitle".to_string());
        record.add_field(f245);

        let mut f100 = Field::new("100".to_string(), '1', ' ');
        f100.add_subfield('a', "Smith, Jane".to_string());
        record.add_field(f100);

        let mut f260 = Field::new("260".to_string(), ' ', ' ');
        f260.add_subfield('a', "New York :".to_string());
        f260.add_subfield('b', "Penguin,".to_string());
        f260.add_subfield('c', "2020.".to_string());
        record.add_field(f260);

        let mut f020 = Field::new("020".to_string(), ' ', ' ');
        f020.add_subfield('a', "9780142424346".to_string());
        record.add_field(f020);

        let mut f650 = Field::new("650".to_string(), ' ', '0');
        f650.add_subfield('a', "Rust programming".to_string());
        record.add_field(f650);

        // Write to MODS
        let mods_xml = record_to_mods_xml(&record).unwrap();

        // Parse back
        let restored = mods_xml_to_record(&mods_xml).unwrap();

        // Verify key fields roundtripped
        let title = restored.get_fields("245").unwrap();
        assert_eq!(title[0].get_subfield('a'), Some("Test Title"));
        assert_eq!(title[0].get_subfield('b'), Some("A Subtitle"));

        let name = restored.get_fields("100").unwrap();
        assert_eq!(name[0].get_subfield('a'), Some("Smith, Jane"));

        let pub_info = restored.get_fields("260").unwrap();
        assert_eq!(pub_info[0].get_subfield('a'), Some("New York :"));
        assert_eq!(pub_info[0].get_subfield('b'), Some("Penguin,"));
        assert_eq!(pub_info[0].get_subfield('c'), Some("2020."));

        let isbn = restored.get_fields("020").unwrap();
        assert_eq!(isbn[0].get_subfield('a'), Some("9780142424346"));

        let subj = restored.get_fields("650").unwrap();
        assert_eq!(subj[0].get_subfield('a'), Some("Rust programming"));

        // Control number roundtrips via identifier type="local"
        assert_eq!(restored.get_control_field("001"), Some("test123"));
    }
}
