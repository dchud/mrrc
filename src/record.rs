use crate::bibliographic_helpers::PublicationInfo;
use crate::leader::Leader;
use crate::marc_record::MarcRecord;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A MARC bibliographic record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    /// Record leader (24 bytes)
    pub leader: Leader,
    /// Control fields (000-009) - tag -> value
    pub control_fields: BTreeMap<String, String>,
    /// Data fields (010+) - tag -> fields
    pub data_fields: BTreeMap<String, Vec<Field>>,
}

/// A data field in a MARC record (fields 010 and higher)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Field {
    /// Field tag (3 digits)
    pub tag: String,
    /// First indicator
    pub indicator1: char,
    /// Second indicator
    pub indicator2: char,
    /// Subfields
    pub subfields: Vec<Subfield>,
}

/// A subfield within a field
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subfield {
    /// Subfield code (single character)
    pub code: char,
    /// Subfield value
    pub value: String,
}

impl Record {
    /// Create a new MARC record with the given leader
    #[must_use]
    pub fn new(leader: Leader) -> Self {
        Record {
            leader,
            control_fields: BTreeMap::new(),
            data_fields: BTreeMap::new(),
        }
    }

    /// Create a builder for fluently constructing MARC records
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mrrc::{Record, Leader, Field};
    ///
    /// let record = Record::builder(Leader::default())
    ///     .control_field("001".to_string(), "12345".to_string())
    ///     .field(Field::builder("245".to_string(), '1', '0')
    ///         .subfield('a', "Title".to_string())
    ///         .build())
    ///     .build();
    /// ```
    #[must_use]
    pub fn builder(leader: Leader) -> RecordBuilder {
        RecordBuilder {
            record: Record {
                leader,
                control_fields: BTreeMap::new(),
                data_fields: BTreeMap::new(),
            },
        }
    }

    /// Add a control field (000-009)
    pub fn add_control_field(&mut self, tag: String, value: String) {
        self.control_fields.insert(tag, value);
    }

    /// Add a control field using string slices
    ///
    /// Convenience method that converts &str arguments to String automatically.
    pub fn add_control_field_str(&mut self, tag: &str, value: &str) {
        self.add_control_field(tag.to_string(), value.to_string());
    }

    /// Get a control field value
    #[must_use]
    pub fn get_control_field(&self, tag: &str) -> Option<&str> {
        self.control_fields
            .get(tag)
            .map(std::string::String::as_str)
    }

    /// Add a data field
    pub fn add_field(&mut self, field: Field) {
        self.data_fields
            .entry(field.tag.clone())
            .or_default()
            .push(field);
    }

    /// Get all fields with a given tag
    #[must_use]
    pub fn get_fields(&self, tag: &str) -> Option<&[Field]> {
        self.data_fields.get(tag).map(std::vec::Vec::as_slice)
    }

    /// Get first field with a given tag
    #[must_use]
    pub fn get_field(&self, tag: &str) -> Option<&Field> {
        self.data_fields.get(tag).and_then(|v| v.first())
    }

    /// Iterate over all fields in tag order
    pub fn fields(&self) -> impl Iterator<Item = &Field> {
        self.data_fields.values().flat_map(|v| v.iter())
    }

    /// Iterate over fields matching a specific tag
    ///
    /// Returns an iterator over all fields with the given tag.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// for field in record.fields_by_tag("650") {
    ///     if let Some(subject) = field.get_subfield('a') {
    ///         println!("Subject: {}", subject);
    ///     }
    /// }
    /// ```
    pub fn fields_by_tag(&self, tag: &str) -> impl Iterator<Item = &Field> {
        self.data_fields
            .get(tag)
            .map(|v| v.iter())
            .into_iter()
            .flatten()
    }

    /// Iterate over all control fields
    ///
    /// Returns an iterator of (tag, value) tuples.
    pub fn control_fields_iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.control_fields
            .iter()
            .map(|(tag, value)| (tag.as_str(), value.as_str()))
    }

    // ============================================================================
    // Helper methods for common bibliographic fields
    // ============================================================================

    /// Get the main title from field 245, subfield 'a'
    ///
    /// # Examples
    /// ```ignore
    /// if let Some(title) = record.title() {
    ///     println!("Title: {}", title);
    /// }
    /// ```
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.get_field("245").and_then(|f| f.get_subfield('a'))
    }

    /// Get the title and statement of responsibility from field 245
    ///
    /// Returns a tuple of (title, `statement_of_responsibility`) if available.
    /// Title comes from subfield 'a', responsibility from subfield 'c'.
    #[must_use]
    pub fn title_with_responsibility(&self) -> (Option<&str>, Option<&str>) {
        match self.get_field("245") {
            Some(field) => (field.get_subfield('a'), field.get_subfield('c')),
            None => (None, None),
        }
    }

    /// Get the primary author from field 100 (personal name), subfield 'a'
    ///
    /// Returns the first author found. Use `authors()` to get all authors.
    #[must_use]
    pub fn author(&self) -> Option<&str> {
        self.get_field("100").and_then(|f| f.get_subfield('a'))
    }

    /// Get all authors from field 700 (added entry for personal name), subfield 'a'
    ///
    /// This includes secondary authors/contributors. For the primary author, use `author()`.
    #[must_use]
    pub fn authors(&self) -> Vec<&str> {
        self.get_fields("700")
            .map(|fields| fields.iter().filter_map(|f| f.get_subfield('a')).collect())
            .unwrap_or_default()
    }

    /// Get the corporate body (publisher or organization) from field 110, subfield 'a'
    #[must_use]
    pub fn corporate_author(&self) -> Option<&str> {
        self.get_field("110").and_then(|f| f.get_subfield('a'))
    }

    /// Get the publisher from field 260, subfield 'b'
    #[must_use]
    pub fn publisher(&self) -> Option<&str> {
        self.get_field("260").and_then(|f| f.get_subfield('b'))
    }

    /// Get the publication date from field 260, subfield 'c'
    ///
    /// Falls back to the publication year extracted from field 008 (positions 7-10)
    /// if field 260$c is not available.
    #[must_use]
    pub fn publication_date(&self) -> Option<&str> {
        self.get_field("260")
            .and_then(|f| f.get_subfield('c'))
            .or_else(|| {
                self.get_control_field("008").and_then(|field_008| {
                    if field_008.len() >= 11 {
                        let year = &field_008[7..11];
                        if year != "    "
                            && year != "0000"
                            && year.chars().all(|c| c.is_ascii_digit())
                        {
                            Some(year)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            })
    }

    /// Get the ISBN from field 020, subfield 'a'
    ///
    /// Returns the first ISBN. Use `isbns()` to get all ISBNs.
    #[must_use]
    pub fn isbn(&self) -> Option<&str> {
        self.get_field("020").and_then(|f| f.get_subfield('a'))
    }

    /// Get all ISBNs from field 020, subfield 'a'
    #[must_use]
    pub fn isbns(&self) -> Vec<&str> {
        self.get_fields("020")
            .map(|fields| fields.iter().filter_map(|f| f.get_subfield('a')).collect())
            .unwrap_or_default()
    }

    /// Get the ISSN from field 022, subfield 'a'
    #[must_use]
    pub fn issn(&self) -> Option<&str> {
        self.get_field("022").and_then(|f| f.get_subfield('a'))
    }

    /// Get all subject headings from field 650, subfield 'a'
    #[must_use]
    pub fn subjects(&self) -> Vec<&str> {
        self.get_fields("650")
            .map(|fields| fields.iter().filter_map(|f| f.get_subfield('a')).collect())
            .unwrap_or_default()
    }

    /// Get the language code from field 008 (positions 35-37)
    ///
    /// Returns a 3-character language code (e.g., "eng" for English).
    #[must_use]
    pub fn language(&self) -> Option<&str> {
        self.get_control_field("008").and_then(|field_008| {
            if field_008.len() >= 38 {
                let lang = &field_008[35..38];
                if lang == "   " {
                    None
                } else {
                    Some(lang)
                }
            } else {
                None
            }
        })
    }

    /// Get the control number (system number) from field 001
    #[must_use]
    pub fn control_number(&self) -> Option<&str> {
        self.get_control_field("001")
    }

    /// Get the Library of Congress Control Number (LCCN) from field 010, subfield 'a'
    #[must_use]
    pub fn lccn(&self) -> Option<&str> {
        self.get_field("010").and_then(|f| f.get_subfield('a'))
    }

    /// Get the physical description from field 300, subfield 'a'
    ///
    /// Typically describes the extent of the resource (e.g., "256 pages").
    #[must_use]
    pub fn physical_description(&self) -> Option<&str> {
        self.get_field("300").and_then(|f| f.get_subfield('a'))
    }

    /// Get the series statement from field 490, subfield 'a'
    #[must_use]
    pub fn series(&self) -> Option<&str> {
        self.get_field("490").and_then(|f| f.get_subfield('a'))
    }

    /// Check if this is a book (leader type 'a' for language material and bib level 'm' for monograph)
    #[must_use]
    pub fn is_book(&self) -> bool {
        self.leader.record_type == 'a' && self.leader.bibliographic_level == 'm'
    }

    /// Check if this is a serial (bib level 's')
    #[must_use]
    pub fn is_serial(&self) -> bool {
        self.leader.bibliographic_level == 's'
    }

    /// Check if this is music (leader type 'c' or 'd')
    #[must_use]
    pub fn is_music(&self) -> bool {
        matches!(self.leader.record_type, 'c' | 'd')
    }

    /// Check if this is audiovisual material (leader type 'g')
    #[must_use]
    pub fn is_audiovisual(&self) -> bool {
        self.leader.record_type == 'g'
    }

    /// Extract publication information from field 260
    ///
    /// Returns a `PublicationInfo` struct containing place of publication (subfield 'a'),
    /// publisher (subfield 'b'), and date (subfield 'c').
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some(info) = record.publication_info() {
    ///     println!("Published in {} by {}", info.place.unwrap_or("unknown"), info.publisher.unwrap_or("unknown"));
    ///     if let Some(year) = info.publication_year() {
    ///         println!("Year: {}", year);
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn publication_info(&self) -> Option<PublicationInfo> {
        self.get_field("260").map(|field| {
            PublicationInfo::new(
                field.get_subfield('a').map(ToString::to_string),
                field.get_subfield('b').map(ToString::to_string),
                field.get_subfield('c').map(ToString::to_string),
            )
        })
    }

    /// Get the publication year extracted from field 260$c or field 008
    ///
    /// Attempts to extract a 4-digit year from the publication date statement.
    /// Falls back to field 008 (positions 7-10) if field 260 is not available.
    #[must_use]
    pub fn publication_year(&self) -> Option<u32> {
        // Try from field 260 first
        if let Some(info) = self.publication_info() {
            if let Some(year) = info.publication_year() {
                return Some(year);
            }
        }

        // Fall back to field 008
        self.get_control_field("008").and_then(|field_008| {
            if field_008.len() >= 11 {
                let year_str = &field_008[7..11];
                if year_str != "    "
                    && year_str != "0000"
                    && year_str.chars().all(|c| c.is_ascii_digit())
                {
                    year_str.parse().ok()
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    /// Get the place of publication from field 260, subfield 'a'
    ///
    /// Alias for accessing the 'a' subfield of field 260.
    #[must_use]
    pub fn place_of_publication(&self) -> Option<&str> {
        self.get_field("260").and_then(|f| f.get_subfield('a'))
    }
}

impl MarcRecord for Record {
    fn leader(&self) -> &Leader {
        &self.leader
    }

    fn leader_mut(&mut self) -> &mut Leader {
        &mut self.leader
    }

    fn add_control_field(&mut self, tag: impl Into<String>, value: impl Into<String>) {
        self.control_fields.insert(tag.into(), value.into());
    }

    fn get_control_field(&self, tag: &str) -> Option<&str> {
        self.control_fields.get(tag).map(String::as_str)
    }

    fn control_fields_iter(&self) -> Box<dyn Iterator<Item = (&str, &str)> + '_> {
        Box::new(
            self.control_fields
                .iter()
                .map(|(tag, value)| (tag.as_str(), value.as_str())),
        )
    }
}

/// Builder for fluently constructing MARC records
///
/// # Examples
///
/// ```ignore
/// use mrrc::{Record, Leader, Field};
///
/// let record = Record::builder(Leader::default())
///     .control_field_str("001", "12345")
///     .field(Field::builder("245".to_string(), '1', '0')
///         .subfield_str('a', "The Great Gatsby")
///         .subfield_str('c', "F. Scott Fitzgerald")
///         .build())
///     .build();
/// ```
#[derive(Debug)]
pub struct RecordBuilder {
    record: Record,
}

impl RecordBuilder {
    /// Add a control field to the record being built
    #[must_use]
    pub fn control_field(mut self, tag: String, value: String) -> Self {
        self.record.add_control_field(tag, value);
        self
    }

    /// Add a control field using string slices
    #[must_use]
    pub fn control_field_str(mut self, tag: &str, value: &str) -> Self {
        self.record.add_control_field_str(tag, value);
        self
    }

    /// Add a data field to the record being built
    #[must_use]
    pub fn field(mut self, field: Field) -> Self {
        self.record.add_field(field);
        self
    }

    /// Build the record
    #[must_use]
    pub fn build(self) -> Record {
        self.record
    }
}

impl Field {
    /// Create a new data field
    #[must_use]
    pub fn new(tag: String, indicator1: char, indicator2: char) -> Self {
        Field {
            tag,
            indicator1,
            indicator2,
            subfields: Vec::new(),
        }
    }

    /// Create a builder for constructing fields fluently
    ///
    /// # Examples
    ///
    /// ```
    /// use mrrc::Field;
    ///
    /// let field = Field::builder("245".to_string(), '1', '0')
    ///     .subfield('a', "The Great Gatsby".to_string())
    ///     .subfield('c', "F. Scott Fitzgerald".to_string())
    ///     .build();
    /// ```
    #[must_use]
    pub fn builder(tag: String, indicator1: char, indicator2: char) -> FieldBuilder {
        FieldBuilder {
            field: Field {
                tag,
                indicator1,
                indicator2,
                subfields: Vec::new(),
            },
        }
    }

    /// Add a subfield
    pub fn add_subfield(&mut self, code: char, value: String) {
        self.subfields.push(Subfield { code, value });
    }

    /// Add a subfield using a string slice
    ///
    /// Convenience method that converts &str to String automatically.
    pub fn add_subfield_str(&mut self, code: char, value: &str) {
        self.add_subfield(code, value.to_string());
    }

    /// Get all values for a subfield code
    #[must_use]
    pub fn get_subfield_values(&self, code: char) -> Vec<&str> {
        self.subfields
            .iter()
            .filter(|sf| sf.code == code)
            .map(|sf| sf.value.as_str())
            .collect()
    }

    /// Get first value for a subfield code
    #[must_use]
    pub fn get_subfield(&self, code: char) -> Option<&str> {
        self.subfields
            .iter()
            .find(|sf| sf.code == code)
            .map(|sf| sf.value.as_str())
    }

    /// Iterate over all subfields
    ///
    /// # Examples
    ///
    /// ```ignore
    /// for subfield in field.subfields() {
    ///     println!("Code: {}, Value: {}", subfield.code, subfield.value);
    /// }
    /// ```
    pub fn subfields(&self) -> impl Iterator<Item = &Subfield> {
        self.subfields.iter()
    }

    /// Iterate over subfields with a specific code
    ///
    /// # Examples
    ///
    /// ```ignore
    /// for value in field.subfields_by_code('a') {
    ///     println!("Author: {}", value);
    /// }
    /// ```
    pub fn subfields_by_code(&self, code: char) -> impl Iterator<Item = &str> {
        self.subfields
            .iter()
            .filter(move |sf| sf.code == code)
            .map(|sf| sf.value.as_str())
    }
}

/// Builder for fluently constructing MARC fields
///
/// # Examples
///
/// ```
/// use mrrc::Field;
///
/// let field = Field::builder("245".to_string(), '1', '0')
///     .subfield('a', "Title".to_string())
///     .subfield('b', "Subtitle".to_string())
///     .build();
/// ```
#[derive(Debug)]
pub struct FieldBuilder {
    field: Field,
}

impl FieldBuilder {
    /// Add a subfield to the field being built
    #[must_use]
    pub fn subfield(mut self, code: char, value: String) -> Self {
        self.field.add_subfield(code, value);
        self
    }

    /// Add a subfield using a string slice
    #[must_use]
    pub fn subfield_str(mut self, code: char, value: &str) -> Self {
        self.field.add_subfield_str(code, value);
        self
    }

    /// Build the field
    #[must_use]
    pub fn build(self) -> Field {
        self.field
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::leader::Leader;

    fn make_leader() -> Leader {
        Leader {
            record_length: 1000,
            record_status: 'a',
            record_type: 'a',
            bibliographic_level: 'm',
            control_record_type: 'a',
            character_coding: ' ',
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
    fn test_record_creation() {
        let leader = make_leader();
        let record = Record::new(leader.clone());
        assert_eq!(record.leader, leader);
        assert!(record.control_fields.is_empty());
        assert!(record.data_fields.is_empty());
    }

    #[test]
    fn test_add_control_field() {
        let leader = make_leader();
        let mut record = Record::new(leader);

        record.add_control_field("001".to_string(), "12345".to_string());
        assert_eq!(record.get_control_field("001"), Some("12345"));
    }

    #[test]
    fn test_field_subfields() {
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Title".to_string());
        field.add_subfield('c', "Author".to_string());
        field.add_subfield('a', "Title continued".to_string());

        assert_eq!(field.get_subfield('a'), Some("Title"));
        let a_values = field.get_subfield_values('a');
        assert_eq!(a_values.len(), 2);
    }

    #[test]
    fn test_add_and_retrieve_fields() {
        let leader = make_leader();
        let mut record = Record::new(leader);

        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test Title".to_string());
        record.add_field(field);

        let fields = record.get_fields("245");
        assert!(fields.is_some());
        assert_eq!(fields.unwrap().len(), 1);
    }

    #[test]
    fn test_multiple_fields_same_tag() {
        let leader = make_leader();
        let mut record = Record::new(leader);

        for i in 0..3 {
            let mut field = Field::new("650".to_string(), ' ', '0');
            field.add_subfield('a', format!("Subject {i}"));
            record.add_field(field);
        }

        let fields = record.get_fields("650");
        assert_eq!(fields.unwrap().len(), 3);
    }

    // ============================================================================
    // Tests for helper methods
    // ============================================================================

    #[test]
    fn test_helper_title() {
        let leader = make_leader();
        let mut record = Record::new(leader);

        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "The Great Gatsby".to_string());
        record.add_field(field);

        assert_eq!(record.title(), Some("The Great Gatsby"));
    }

    #[test]
    fn test_helper_title_with_responsibility() {
        let leader = make_leader();
        let mut record = Record::new(leader);

        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "The Great Gatsby /".to_string());
        field.add_subfield('c', "F. Scott Fitzgerald.".to_string());
        record.add_field(field);

        let (title, resp) = record.title_with_responsibility();
        assert_eq!(title, Some("The Great Gatsby /"));
        assert_eq!(resp, Some("F. Scott Fitzgerald."));
    }

    #[test]
    fn test_helper_author() {
        let leader = make_leader();
        let mut record = Record::new(leader);

        let mut field = Field::new("100".to_string(), '1', ' ');
        field.add_subfield('a', "Fitzgerald, F. Scott".to_string());
        record.add_field(field);

        assert_eq!(record.author(), Some("Fitzgerald, F. Scott"));
    }

    #[test]
    fn test_helper_authors() {
        let leader = make_leader();
        let mut record = Record::new(leader);

        for i in 0..2 {
            let mut field = Field::new("700".to_string(), '1', ' ');
            field.add_subfield('a', format!("Author {i}"));
            record.add_field(field);
        }

        let authors = record.authors();
        assert_eq!(authors.len(), 2);
        assert_eq!(authors[0], "Author 0");
        assert_eq!(authors[1], "Author 1");
    }

    #[test]
    fn test_helper_publisher() {
        let leader = make_leader();
        let mut record = Record::new(leader);

        let mut field = Field::new("260".to_string(), ' ', '1');
        field.add_subfield('b', "Scribner".to_string());
        field.add_subfield('c', "1925".to_string());
        record.add_field(field);

        assert_eq!(record.publisher(), Some("Scribner"));
    }

    #[test]
    fn test_helper_publication_date_from_260() {
        let leader = make_leader();
        let mut record = Record::new(leader);

        let mut field = Field::new("260".to_string(), ' ', '1');
        field.add_subfield('c', "1925".to_string());
        record.add_field(field);

        assert_eq!(record.publication_date(), Some("1925"));
    }

    #[test]
    fn test_helper_publication_date_from_008() {
        let leader = make_leader();
        let mut record = Record::new(leader);

        // Field 008 positions 7-10 contain publication year
        record.add_control_field(
            "008".to_string(),
            "200101s1925    xxu||||||||||||||||eng||".to_string(),
        );

        assert_eq!(record.publication_date(), Some("1925"));
    }

    #[test]
    fn test_helper_isbn() {
        let leader = make_leader();
        let mut record = Record::new(leader);

        let mut field = Field::new("020".to_string(), ' ', ' ');
        field.add_subfield('a', "978-0-7432-7356-5".to_string());
        record.add_field(field);

        assert_eq!(record.isbn(), Some("978-0-7432-7356-5"));
    }

    #[test]
    fn test_helper_isbns() {
        let leader = make_leader();
        let mut record = Record::new(leader);

        for i in 0..2 {
            let mut field = Field::new("020".to_string(), ' ', ' ');
            field.add_subfield('a', format!("ISBN-{i}"));
            record.add_field(field);
        }

        let isbns = record.isbns();
        assert_eq!(isbns.len(), 2);
        assert_eq!(isbns[0], "ISBN-0");
        assert_eq!(isbns[1], "ISBN-1");
    }

    #[test]
    fn test_helper_subjects() {
        let leader = make_leader();
        let mut record = Record::new(leader);

        for i in 0..3 {
            let mut field = Field::new("650".to_string(), ' ', '0');
            field.add_subfield('a', format!("Subject {i}"));
            record.add_field(field);
        }

        let subjects = record.subjects();
        assert_eq!(subjects.len(), 3);
        assert_eq!(subjects[0], "Subject 0");
    }

    #[test]
    fn test_helper_language() {
        let leader = make_leader();
        let mut record = Record::new(leader);

        // Field 008 is exactly 40 characters in MARC21
        // Index positions 35-37 (slice [35..38]) contain the language code
        // Manually build: 35 filler chars + "eng" + 2 more chars
        let mut field_008 = "12345678901234567890123456789012345".to_string(); // 35 chars
        field_008.push_str("eng"); // positions 35-37
        field_008.push_str("||"); // positions 38-39 (total 40)

        record.add_control_field("008".to_string(), field_008);
        assert_eq!(record.language(), Some("eng"));
    }

    #[test]
    fn test_helper_control_number() {
        let leader = make_leader();
        let mut record = Record::new(leader);

        record.add_control_field("001".to_string(), "12345678".to_string());

        assert_eq!(record.control_number(), Some("12345678"));
    }

    #[test]
    fn test_helper_lccn() {
        let leader = make_leader();
        let mut record = Record::new(leader);

        let mut field = Field::new("010".to_string(), ' ', ' ');
        field.add_subfield('a', "2004310216".to_string());
        record.add_field(field);

        assert_eq!(record.lccn(), Some("2004310216"));
    }

    #[test]
    fn test_helper_physical_description() {
        let leader = make_leader();
        let mut record = Record::new(leader);

        let mut field = Field::new("300".to_string(), ' ', ' ');
        field.add_subfield('a', "256 pages".to_string());
        record.add_field(field);

        assert_eq!(record.physical_description(), Some("256 pages"));
    }

    #[test]
    fn test_helper_series() {
        let leader = make_leader();
        let mut record = Record::new(leader);

        let mut field = Field::new("490".to_string(), '1', ' ');
        field.add_subfield('a', "Classic literature".to_string());
        record.add_field(field);

        assert_eq!(record.series(), Some("Classic literature"));
    }

    #[test]
    fn test_helper_corporate_author() {
        let leader = make_leader();
        let mut record = Record::new(leader);

        let mut field = Field::new("110".to_string(), '2', ' ');
        field.add_subfield('a', "United States. Congress.".to_string());
        record.add_field(field);

        assert_eq!(record.corporate_author(), Some("United States. Congress."));
    }

    #[test]
    fn test_helper_issn() {
        let leader = make_leader();
        let mut record = Record::new(leader);

        let mut field = Field::new("022".to_string(), ' ', ' ');
        field.add_subfield('a', "0028-0836".to_string());
        record.add_field(field);

        assert_eq!(record.issn(), Some("0028-0836"));
    }

    #[test]
    fn test_helper_is_book() {
        let mut leader = make_leader();
        leader.record_type = 'a';
        leader.bibliographic_level = 'm';
        let record = Record::new(leader);

        assert!(record.is_book());
    }

    #[test]
    fn test_helper_is_serial() {
        let mut leader = make_leader();
        leader.bibliographic_level = 's';
        let record = Record::new(leader);

        assert!(record.is_serial());
    }

    #[test]
    fn test_helper_is_music() {
        let mut leader = make_leader();
        leader.record_type = 'c';
        let record = Record::new(leader);

        assert!(record.is_music());

        let mut leader2 = make_leader();
        leader2.record_type = 'd';
        let record2 = Record::new(leader2);

        assert!(record2.is_music());
    }

    #[test]
    fn test_helper_is_audiovisual() {
        let mut leader = make_leader();
        leader.record_type = 'g';
        let record = Record::new(leader);

        assert!(record.is_audiovisual());
    }

    #[test]
    fn test_helper_no_title_returns_none() {
        let leader = make_leader();
        let record = Record::new(leader);

        assert_eq!(record.title(), None);
    }

    #[test]
    fn test_helper_empty_authors_returns_empty_vec() {
        let leader = make_leader();
        let record = Record::new(leader);

        assert_eq!(record.authors(), Vec::<&str>::new());
    }

    // ============================================================================
    // Tests for builder API
    // ============================================================================

    #[test]
    fn test_field_builder() {
        let field = Field::builder("245".to_string(), '1', '0')
            .subfield('a', "The Great Gatsby".to_string())
            .subfield('c', "F. Scott Fitzgerald".to_string())
            .build();

        assert_eq!(field.tag, "245");
        assert_eq!(field.indicator1, '1');
        assert_eq!(field.indicator2, '0');
        assert_eq!(field.get_subfield('a'), Some("The Great Gatsby"));
        assert_eq!(field.get_subfield('c'), Some("F. Scott Fitzgerald"));
    }

    #[test]
    fn test_field_builder_with_str() {
        let field = Field::builder("650".to_string(), ' ', '0')
            .subfield_str('a', "Computer science")
            .subfield_str('x', "History")
            .build();

        assert_eq!(field.get_subfield('a'), Some("Computer science"));
        assert_eq!(field.get_subfield('x'), Some("History"));
    }

    #[test]
    fn test_record_builder() {
        let record = Record::builder(make_leader())
            .control_field_str("001", "12345")
            .field(
                Field::builder("245".to_string(), '1', '0')
                    .subfield_str('a', "Test Title")
                    .build(),
            )
            .build();

        assert_eq!(record.get_control_field("001"), Some("12345"));
        assert_eq!(record.title(), Some("Test Title"));
    }

    #[test]
    fn test_field_subfields_iterator() {
        let mut field = Field::new("650".to_string(), ' ', '0');
        field.add_subfield_str('a', "Subject 1");
        field.add_subfield_str('x', "Subdivision");

        let mut count = 0;
        for _ in field.subfields() {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_field_subfields_by_code_iterator() {
        let mut field = Field::new("650".to_string(), ' ', '0');
        field.add_subfield_str('a', "Primary Subject");
        field.add_subfield_str('x', "Subdivision 1");
        field.add_subfield_str('x', "Subdivision 2");

        let x_values: Vec<&str> = field.subfields_by_code('x').collect();
        assert_eq!(x_values.len(), 2);
        assert!(x_values.contains(&"Subdivision 1"));
        assert!(x_values.contains(&"Subdivision 2"));
    }

    #[test]
    fn test_record_fields_by_tag_iterator() {
        let mut record = Record::new(make_leader());

        for i in 0..3 {
            let mut field = Field::new("650".to_string(), ' ', '0');
            field.add_subfield('a', format!("Subject {i}"));
            record.add_field(field);
        }

        let subjects: Vec<&str> = record
            .fields_by_tag("650")
            .filter_map(|f| f.get_subfield('a'))
            .collect();

        assert_eq!(subjects.len(), 3);
    }

    #[test]
    fn test_record_control_fields_iterator() {
        let mut record = Record::new(make_leader());
        record.add_control_field_str("001", "id1");
        record.add_control_field_str("003", "source");

        let mut found = 0;
        for (tag, _value) in record.control_fields_iter() {
            if tag == "001" || tag == "003" {
                found += 1;
            }
        }

        assert_eq!(found, 2);
    }
}
