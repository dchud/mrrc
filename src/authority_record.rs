//! MARC Authority Record structures and utilities.
//!
//! Authority records establish standardized forms of names, subjects, and titles
//! for use as access points in bibliographic records. They differ fundamentally
//! from bibliographic records in structure and purpose.

use crate::leader::Leader;
use crate::marc_record::MarcRecord;
use crate::record::Field;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// A MARC Authority record (Type Z, Leader/06 = 'z')
///
/// Fields are stored in insertion order using `IndexMap`, preserving the order
/// in which fields were added to the record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityRecord {
    /// Record leader (24 bytes)
    pub leader: Leader,
    /// Control fields (000-009) - preserves insertion order
    pub control_fields: IndexMap<String, String>,
    /// Variable fields (010+) - unified storage, preserves insertion order
    pub fields: IndexMap<String, Vec<Field>>,
}

/// Type of heading in the authority record
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeadingType {
    /// 100 - Personal name
    PersonalName,
    /// 110 - Corporate name
    CorporateName,
    /// 111 - Meeting name
    MeetingName,
    /// 130 - Uniform title
    UniformTitle,
    /// 148 - Chronological term
    ChronologicalTerm,
    /// 150 - Topical term
    TopicalTerm,
    /// 151 - Geographic name
    GeographicName,
    /// 155 - Genre/form term
    GenreFormTerm,
}

/// Kind of authority record (008/09)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KindOfRecord {
    /// a - Established heading
    EstablishedHeading,
    /// b - Reference record (untraced reference)
    ReferenceUntracted,
    /// c - Reference record (traced reference)
    ReferenceTraced,
    /// d - Subdivision
    Subdivision,
    /// f - Established heading and subdivision
    EstablishedHeadingAndSubdivision,
    /// g - Reference and subdivision
    ReferenceAndSubdivision,
    /// e - Node label
    NodeLabel,
}

/// Level of establishment (008/33)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LevelOfEstablishment {
    /// a - Fully established
    FullyEstablished,
    /// b - Memorandum (established but not yet used)
    Memorandum,
    /// c - Provisional
    Provisional,
    /// d - Preliminary
    Preliminary,
    /// n - Not applicable
    NotApplicable,
}

impl AuthorityRecord {
    /// Create a new authority record with the given leader
    #[must_use]
    pub fn new(leader: Leader) -> Self {
        AuthorityRecord {
            leader,
            control_fields: IndexMap::new(),
            fields: IndexMap::new(),
        }
    }

    /// Create a builder for fluently constructing authority records
    #[must_use]
    pub fn builder(leader: Leader) -> AuthorityRecordBuilder {
        AuthorityRecordBuilder {
            record: AuthorityRecord::new(leader),
        }
    }

    /// Add a control field
    pub fn add_control_field(&mut self, tag: String, value: String) {
        self.control_fields.insert(tag, value);
    }

    /// Get a control field value
    #[must_use]
    pub fn get_control_field(&self, tag: &str) -> Option<&str> {
        self.control_fields.get(tag).map(String::as_str)
    }

    /// Set the heading (1XX field)
    pub fn set_heading(&mut self, field: Field) {
        self.fields
            .entry(field.tag.clone())
            .or_default()
            .push(field);
    }

    /// Get the main heading (1XX field)
    #[must_use]
    pub fn heading(&self) -> Option<&Field> {
        // Get the first 1XX field
        for tag in &["100", "110", "111", "130", "148", "150", "151", "155"] {
            if let Some(fields) = self.fields.get(*tag) {
                if let Some(field) = fields.first() {
                    return Some(field);
                }
            }
        }
        None
    }

    /// Get the heading type from the 1XX field tag
    #[must_use]
    pub fn heading_type(&self) -> Option<HeadingType> {
        self.heading().and_then(|f| match f.tag.as_str() {
            "100" => Some(HeadingType::PersonalName),
            "110" => Some(HeadingType::CorporateName),
            "111" => Some(HeadingType::MeetingName),
            "130" => Some(HeadingType::UniformTitle),
            "148" => Some(HeadingType::ChronologicalTerm),
            "150" => Some(HeadingType::TopicalTerm),
            "151" => Some(HeadingType::GeographicName),
            "155" => Some(HeadingType::GenreFormTerm),
            _ => None,
        })
    }

    /// Add a See From Tracing field (4XX)
    pub fn add_see_from_tracing(&mut self, field: Field) {
        self.fields
            .entry(field.tag.clone())
            .or_default()
            .push(field);
    }

    /// Get all See From Tracing fields (4XX)
    #[must_use]
    pub fn see_from_tracings(&self) -> Vec<&Field> {
        self.fields
            .iter()
            .filter(|(tag, _)| tag.starts_with('4'))
            .flat_map(|(_, fields)| fields.iter())
            .collect()
    }

    /// Add a See Also From Tracing field (5XX)
    pub fn add_see_also_tracing(&mut self, field: Field) {
        self.fields
            .entry(field.tag.clone())
            .or_default()
            .push(field);
    }

    /// Get all See Also From Tracing fields (5XX)
    #[must_use]
    pub fn see_also_tracings(&self) -> Vec<&Field> {
        self.fields
            .iter()
            .filter(|(tag, _)| tag.starts_with('5'))
            .flat_map(|(_, fields)| fields.iter())
            .collect()
    }

    /// Add a note field
    pub fn add_note(&mut self, field: Field) {
        self.fields
            .entry(field.tag.clone())
            .or_default()
            .push(field);
    }

    /// Get all note fields
    #[must_use]
    pub fn notes(&self) -> Vec<&Field> {
        // Notes are typically in 6XX, 67X, 68X fields
        self.fields
            .iter()
            .filter(|(tag, _)| {
                tag.starts_with('6') && (tag != &"650" && tag != &"651" && tag != &"655")
            })
            .flat_map(|(_, fields)| fields.iter())
            .collect()
    }

    /// Get source data found notes (670)
    #[must_use]
    pub fn source_data_found(&self) -> Vec<&Field> {
        self.fields
            .get("670")
            .map(|fields| fields.iter().collect())
            .unwrap_or_default()
    }

    /// Get source data not found notes (671)
    #[must_use]
    pub fn source_data_not_found(&self) -> Vec<&Field> {
        self.fields
            .get("671")
            .map(|fields| fields.iter().collect())
            .unwrap_or_default()
    }

    /// Add a heading linking entry field (7XX)
    pub fn add_linking_entry(&mut self, field: Field) {
        self.fields
            .entry(field.tag.clone())
            .or_default()
            .push(field);
    }

    /// Get all heading linking entry fields (7XX)
    #[must_use]
    pub fn linking_entries(&self) -> Vec<&Field> {
        self.fields
            .iter()
            .filter(|(tag, _)| tag.starts_with('7'))
            .flat_map(|(_, fields)| fields.iter())
            .collect()
    }

    /// Add a field to `fields`
    pub fn add_field(&mut self, field: Field) {
        self.fields
            .entry(field.tag.clone())
            .or_default()
            .push(field);
    }

    /// Get fields by tag
    #[must_use]
    pub fn get_fields(&self, tag: &str) -> Option<&[Field]> {
        self.fields.get(tag).map(Vec::as_slice)
    }

    /// Get kind of record from 008/09
    #[must_use]
    pub fn kind_of_record(&self) -> Option<KindOfRecord> {
        self.get_control_field("008").and_then(|field| {
            if field.len() > 9 {
                match field.chars().nth(9) {
                    Some('a') => Some(KindOfRecord::EstablishedHeading),
                    Some('b') => Some(KindOfRecord::ReferenceUntracted),
                    Some('c') => Some(KindOfRecord::ReferenceTraced),
                    Some('d') => Some(KindOfRecord::Subdivision),
                    Some('e') => Some(KindOfRecord::NodeLabel),
                    Some('f') => Some(KindOfRecord::EstablishedHeadingAndSubdivision),
                    Some('g') => Some(KindOfRecord::ReferenceAndSubdivision),
                    _ => None,
                }
            } else {
                None
            }
        })
    }

    /// Get level of establishment from 008/33
    #[must_use]
    pub fn level_of_establishment(&self) -> Option<LevelOfEstablishment> {
        self.get_control_field("008").and_then(|field| {
            if field.len() > 33 {
                match field.chars().nth(33) {
                    Some('a') => Some(LevelOfEstablishment::FullyEstablished),
                    Some('b') => Some(LevelOfEstablishment::Memorandum),
                    Some('c') => Some(LevelOfEstablishment::Provisional),
                    Some('d') => Some(LevelOfEstablishment::Preliminary),
                    Some('n') => Some(LevelOfEstablishment::NotApplicable),
                    _ => None,
                }
            } else {
                None
            }
        })
    }

    /// Check if this is an established heading
    #[must_use]
    pub fn is_established(&self) -> bool {
        matches!(
            self.kind_of_record(),
            Some(KindOfRecord::EstablishedHeading | KindOfRecord::EstablishedHeadingAndSubdivision)
        )
    }

    /// Check if this is a reference record
    #[must_use]
    pub fn is_reference(&self) -> bool {
        matches!(
            self.kind_of_record(),
            Some(
                KindOfRecord::ReferenceUntracted
                    | KindOfRecord::ReferenceTraced
                    | KindOfRecord::ReferenceAndSubdivision
            )
        )
    }
}

impl MarcRecord for AuthorityRecord {
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

    fn get_fields(&self, tag: &str) -> Option<&[Field]> {
        self.fields.get(tag).map(std::vec::Vec::as_slice)
    }

    fn get_field(&self, tag: &str) -> Option<&Field> {
        self.fields.get(tag).and_then(|v| v.first())
    }
}

/// Builder for fluently constructing authority records
#[derive(Debug)]
pub struct AuthorityRecordBuilder {
    record: AuthorityRecord,
}

impl AuthorityRecordBuilder {
    /// Add a control field
    #[must_use]
    pub fn control_field(mut self, tag: String, value: String) -> Self {
        self.record.add_control_field(tag, value);
        self
    }

    /// Set the main heading (1XX)
    #[must_use]
    pub fn heading(mut self, field: Field) -> Self {
        self.record.set_heading(field);
        self
    }

    /// Add a See From Tracing field (4XX)
    #[must_use]
    pub fn add_see_from(mut self, field: Field) -> Self {
        self.record.add_see_from_tracing(field);
        self
    }

    /// Add a See Also From Tracing field (5XX)
    #[must_use]
    pub fn add_see_also(mut self, field: Field) -> Self {
        self.record.add_see_also_tracing(field);
        self
    }

    /// Add a note field
    #[must_use]
    pub fn add_note(mut self, field: Field) -> Self {
        self.record.add_note(field);
        self
    }

    /// Add a heading linking entry field (7XX)
    #[must_use]
    pub fn add_linking_entry(mut self, field: Field) -> Self {
        self.record.add_linking_entry(field);
        self
    }

    /// Add a field to `other_fields`
    #[must_use]
    pub fn add_field(mut self, field: Field) -> Self {
        self.record.add_field(field);
        self
    }

    /// Build the authority record
    #[must_use]
    pub fn build(self) -> AuthorityRecord {
        self.record
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::Subfield;

    fn create_test_leader() -> Leader {
        // Create a minimal authority record leader
        // This is a valid 24-byte leader for authority records (type 'z')
        Leader {
            record_length: 1024,
            record_status: 'n',
            record_type: 'z',
            bibliographic_level: '|',
            control_record_type: ' ',
            character_coding: ' ',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 300,
            encoding_level: 'n',
            cataloging_form: 'a',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        }
    }

    #[test]
    fn test_create_authority_record() {
        let leader = create_test_leader();
        let record = AuthorityRecord::new(leader);
        assert!(record.heading().is_none());
        assert!(record.see_from_tracings().is_empty());
        assert!(record.see_also_tracings().is_empty());
    }

    #[test]
    fn test_authority_record_builder() {
        let leader = create_test_leader();
        let record = AuthorityRecord::builder(leader)
            .control_field(
                "008".to_string(),
                "850101n| a azannaabn          |a aaa      ".to_string(),
            )
            .build();

        assert_eq!(
            record.get_control_field("008"),
            Some("850101n| a azannaabn          |a aaa      ")
        );
    }

    #[test]
    fn test_heading_type_detection() {
        let leader = create_test_leader();

        // Test personal name heading
        let field = Field {
            tag: "100".to_string(),
            indicator1: '1',
            indicator2: ' ',
            subfields: smallvec::smallvec![],
        };
        let record = AuthorityRecord::builder(leader).heading(field).build();
        assert_eq!(record.heading_type(), Some(HeadingType::PersonalName));

        // Test topical term heading
        let field = Field {
            tag: "150".to_string(),
            indicator1: ' ',
            indicator2: '0',
            subfields: smallvec::smallvec![],
        };
        let record = AuthorityRecord::builder(create_test_leader())
            .heading(field)
            .build();
        assert_eq!(record.heading_type(), Some(HeadingType::TopicalTerm));
    }

    #[test]
    fn test_kind_of_record_parsing() {
        let leader = create_test_leader();

        // Test established heading (position 9 = 'a')
        // 008 field is 40 chars: yymmdd (0-5) + 06 (6-7) + 07 (8-8) + 08 (9-9) + ...
        let record = AuthorityRecord::builder(leader)
            .control_field(
                "008".to_string(),
                "850101n| a azannaabn          |a aaa      ".to_string(),
            )
            .build();
        assert_eq!(
            record.kind_of_record(),
            Some(KindOfRecord::EstablishedHeading)
        );

        // Test reference record (position 9 = 'b')
        let record = AuthorityRecord::builder(create_test_leader())
            .control_field(
                "008".to_string(),
                "850101n| b azannaabn          |a aaa      ".to_string(),
            )
            .build();
        assert_eq!(
            record.kind_of_record(),
            Some(KindOfRecord::ReferenceUntracted)
        );

        // Test subdivision (position 9 = 'd')
        let record = AuthorityRecord::builder(create_test_leader())
            .control_field(
                "008".to_string(),
                "850101n| d azannaabn          |a aaa      ".to_string(),
            )
            .build();
        assert_eq!(record.kind_of_record(), Some(KindOfRecord::Subdivision));
    }

    #[test]
    fn test_level_of_establishment_parsing() {
        let leader = create_test_leader();

        // Position 33 for level of establishment
        // Authority 008 is 40 chars: positions 0-33 are variable data
        // Create string with exact 40 chars: position 33 = 'a' (fully established)
        let mut field_008 = vec!['8', '5', '0', '1', '0', '1', 'n', '|', ' ', 'a'];
        // positions 10-32 (23 chars)
        field_008.extend(vec![
            'z', 'a', 'n', 'n', 'a', 'a', 'b', 'n', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ' ', ' ', ' ', ' ', ' ', ' ',
        ]);
        // position 33 (level of establishment)
        field_008.push('a');
        // positions 34-39 (6 chars)
        field_008.extend(vec![' ', ' ', ' ', ' ', ' ', ' ']);
        let field_008_str: String = field_008.iter().collect();

        let record = AuthorityRecord::builder(leader)
            .control_field("008".to_string(), field_008_str)
            .build();
        assert_eq!(
            record.level_of_establishment(),
            Some(LevelOfEstablishment::FullyEstablished)
        );

        // Same but with 'd' at position 33 (preliminary)
        let mut field_008 = vec!['8', '5', '0', '1', '0', '1', 'n', '|', ' ', 'a'];
        field_008.extend(vec![
            'z', 'a', 'n', 'n', 'a', 'a', 'b', 'n', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ' ', ' ', ' ', ' ', ' ', ' ',
        ]);
        field_008.push('d');
        field_008.extend(vec![' ', ' ', ' ', ' ', ' ', ' ']);
        let field_008_str: String = field_008.iter().collect();

        let record = AuthorityRecord::builder(create_test_leader())
            .control_field("008".to_string(), field_008_str)
            .build();
        assert_eq!(
            record.level_of_establishment(),
            Some(LevelOfEstablishment::Preliminary)
        );
    }

    #[test]
    fn test_is_established() {
        let leader = create_test_leader();

        // Established heading (position 9 = 'a')
        let record = AuthorityRecord::builder(leader)
            .control_field(
                "008".to_string(),
                "850101n| a azannaabn          |a aaa      ".to_string(),
            )
            .build();
        assert!(record.is_established());

        // Reference record (position 9 = 'b')
        let record = AuthorityRecord::builder(create_test_leader())
            .control_field(
                "008".to_string(),
                "850101n| b azannaabn          |a aaa      ".to_string(),
            )
            .build();
        assert!(!record.is_established());
    }

    #[test]
    fn test_is_reference() {
        let leader = create_test_leader();

        // Reference record (position 9 = 'b')
        let record = AuthorityRecord::builder(leader)
            .control_field(
                "008".to_string(),
                "850101n| b azannaabn          |a aaa      ".to_string(),
            )
            .build();
        assert!(record.is_reference());

        // Established heading (position 9 = 'a')
        let record = AuthorityRecord::builder(create_test_leader())
            .control_field(
                "008".to_string(),
                "850101n| a azannaabn          |a aaa      ".to_string(),
            )
            .build();
        assert!(!record.is_reference());
    }

    #[test]
    fn test_add_tracings() {
        let leader = create_test_leader();
        let see_from = Field {
            tag: "400".to_string(),
            indicator1: '1',
            indicator2: ' ',
            subfields: smallvec::smallvec![Subfield {
                code: 'a',
                value: "Smith, John".to_string(),
            }],
        };

        let see_also = Field {
            tag: "500".to_string(),
            indicator1: '1',
            indicator2: ' ',
            subfields: smallvec::smallvec![Subfield {
                code: 'a',
                value: "Smith, J. (John)".to_string(),
            }],
        };

        let record = AuthorityRecord::builder(leader)
            .add_see_from(see_from)
            .add_see_also(see_also)
            .build();

        assert_eq!(record.see_from_tracings().len(), 1);
        assert_eq!(record.see_also_tracings().len(), 1);
    }

    #[test]
    fn test_add_notes() {
        let leader = create_test_leader();
        let source_note = Field {
            tag: "670".to_string(),
            indicator1: ' ',
            indicator2: ' ',
            subfields: smallvec::smallvec![Subfield {
                code: 'a',
                value: "DNB, 1985".to_string(),
            }],
        };

        let record = AuthorityRecord::builder(leader)
            .add_note(source_note)
            .build();

        assert_eq!(record.notes().len(), 1);
        assert_eq!(record.source_data_found().len(), 1);
    }

    #[test]
    fn test_control_field_operations() {
        let leader = create_test_leader();
        let mut record = AuthorityRecord::new(leader);

        record.add_control_field("001".to_string(), "n79021800".to_string());
        record.add_control_field("005".to_string(), "19850104".to_string());

        assert_eq!(record.get_control_field("001"), Some("n79021800"));
        assert_eq!(record.get_control_field("005"), Some("19850104"));
    }
}
