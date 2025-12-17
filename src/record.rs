use crate::leader::Leader;
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
    pub fn new(leader: Leader) -> Self {
        Record {
            leader,
            control_fields: BTreeMap::new(),
            data_fields: BTreeMap::new(),
        }
    }

    /// Add a control field (000-009)
    pub fn add_control_field(&mut self, tag: String, value: String) {
        self.control_fields.insert(tag, value);
    }

    /// Get a control field value
    pub fn get_control_field(&self, tag: &str) -> Option<&str> {
        self.control_fields.get(tag).map(|s| s.as_str())
    }

    /// Add a data field
    pub fn add_field(&mut self, field: Field) {
        self.data_fields
            .entry(field.tag.clone())
            .or_default()
            .push(field);
    }

    /// Get all fields with a given tag
    pub fn get_fields(&self, tag: &str) -> Option<&[Field]> {
        self.data_fields.get(tag).map(|v| v.as_slice())
    }

    /// Get first field with a given tag
    pub fn get_field(&self, tag: &str) -> Option<&Field> {
        self.data_fields.get(tag).and_then(|v| v.first())
    }

    /// Iterate over all fields in tag order
    pub fn fields(&self) -> impl Iterator<Item = &Field> {
        self.data_fields.values().flat_map(|v| v.iter())
    }
}

impl Field {
    /// Create a new data field
    pub fn new(tag: String, indicator1: char, indicator2: char) -> Self {
        Field {
            tag,
            indicator1,
            indicator2,
            subfields: Vec::new(),
        }
    }

    /// Add a subfield
    pub fn add_subfield(&mut self, code: char, value: String) {
        self.subfields.push(Subfield { code, value });
    }

    /// Get all values for a subfield code
    pub fn get_subfield_values(&self, code: char) -> Vec<&str> {
        self.subfields
            .iter()
            .filter(|sf| sf.code == code)
            .map(|sf| sf.value.as_str())
            .collect()
    }

    /// Get first value for a subfield code
    pub fn get_subfield(&self, code: char) -> Option<&str> {
        self.subfields
            .iter()
            .find(|sf| sf.code == code)
            .map(|sf| sf.value.as_str())
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
            field.add_subfield('a', format!("Subject {}", i));
            record.add_field(field);
        }

        let fields = record.get_fields("650");
        assert_eq!(fields.unwrap().len(), 3);
    }
}
