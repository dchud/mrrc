//! MARC Holdings Record structures and utilities.
//!
//! Holdings records describe the specific holdings information for bibliographic items,
//! including location, call numbers, and enumeration/chronology information for serials.
//! They are linked to bibliographic records but maintain separate MARC records.

use crate::leader::Leader;
use crate::marc_record::MarcRecord;
use crate::record::Field;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// A MARC Holdings record (Type x/y/v/u, Leader/06)
///
/// Fields are stored in insertion order using `IndexMap`, preserving the order
/// in which fields were added to the record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldingsRecord {
    /// Record leader (24 bytes)
    pub leader: Leader,
    /// Control fields (000-009) - preserves insertion order
    pub control_fields: IndexMap<String, String>,
    /// Variable fields (010+) - unified storage, preserves insertion order
    pub fields: IndexMap<String, Vec<Field>>,
}

/// Type of holdings record (Leader/06)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HoldingsType {
    /// x - Single-part item holdings (monographs)
    SinglePartItem,
    /// y - Serial item holdings
    SerialItem,
    /// v - Multipart item holdings (sets and multivolume monographs)
    MultipartItem,
    /// u - Unknown
    Unknown,
}

/// Acquisition status (008/06)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcquisitionStatus {
    /// 0 - Other
    Other,
    /// 1 - Received and complete
    ReceivedAndComplete,
    /// 2 - On order
    OnOrder,
    /// 3 - Received and incomplete
    ReceivedAndIncomplete,
}

/// Method of acquisition (008/07)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MethodOfAcquisition {
    /// u - Unknown
    Unknown,
    /// f - Free
    Free,
    /// g - Gift
    Gift,
    /// l - Legal deposit
    LegalDeposit,
    /// m - Membership
    Membership,
    /// p - Purchase
    Purchase,
    /// e - Exchange
    Exchange,
}

/// Completeness of holdings (008/16)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Completeness {
    /// 1 - Complete
    Complete,
    /// 2 - Incomplete
    Incomplete,
    /// 3 - Scattered
    Scattered,
    /// 4 - Not applicable
    NotApplicable,
}

impl HoldingsRecord {
    /// Create a new holdings record with the given leader
    #[must_use]
    pub fn new(leader: Leader) -> Self {
        HoldingsRecord {
            leader,
            control_fields: IndexMap::new(),
            fields: IndexMap::new(),
        }
    }

    /// Create a builder for fluently constructing holdings records
    #[must_use]
    pub fn builder(leader: Leader) -> HoldingsRecordBuilder {
        HoldingsRecordBuilder {
            record: HoldingsRecord::new(leader),
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

    /// Add a location field (852)
    pub fn add_location(&mut self, field: Field) {
        self.fields
            .entry("852".to_string())
            .or_default()
            .push(field);
    }

    /// Get location fields (852)
    #[must_use]
    pub fn locations(&self) -> &[Field] {
        self.fields.get("852").map_or(&[], Vec::as_slice)
    }

    /// Add a captions and pattern field for basic units (853)
    pub fn add_captions_basic(&mut self, field: Field) {
        self.fields
            .entry("853".to_string())
            .or_default()
            .push(field);
    }

    /// Get captions and pattern fields for basic units (853)
    #[must_use]
    pub fn captions_basic(&self) -> &[Field] {
        self.fields.get("853").map_or(&[], Vec::as_slice)
    }

    /// Add a captions and pattern field for supplements (854)
    pub fn add_captions_supplements(&mut self, field: Field) {
        self.fields
            .entry("854".to_string())
            .or_default()
            .push(field);
    }

    /// Get captions and pattern fields for supplements (854)
    #[must_use]
    pub fn captions_supplements(&self) -> &[Field] {
        self.fields.get("854").map_or(&[], Vec::as_slice)
    }

    /// Add a captions and pattern field for indexes (855)
    pub fn add_captions_indexes(&mut self, field: Field) {
        self.fields
            .entry("855".to_string())
            .or_default()
            .push(field);
    }

    /// Get captions and pattern fields for indexes (855)
    #[must_use]
    pub fn captions_indexes(&self) -> &[Field] {
        self.fields.get("855").map_or(&[], Vec::as_slice)
    }

    /// Add an enumeration and chronology field for basic units (863)
    pub fn add_enumeration_basic(&mut self, field: Field) {
        self.fields
            .entry("863".to_string())
            .or_default()
            .push(field);
    }

    /// Get enumeration and chronology fields for basic units (863)
    #[must_use]
    pub fn enumeration_basic(&self) -> &[Field] {
        self.fields.get("863").map_or(&[], Vec::as_slice)
    }

    /// Add an enumeration and chronology field for supplements (864)
    pub fn add_enumeration_supplements(&mut self, field: Field) {
        self.fields
            .entry("864".to_string())
            .or_default()
            .push(field);
    }

    /// Get enumeration and chronology fields for supplements (864)
    #[must_use]
    pub fn enumeration_supplements(&self) -> &[Field] {
        self.fields.get("864").map_or(&[], Vec::as_slice)
    }

    /// Add an enumeration and chronology field for indexes (865)
    pub fn add_enumeration_indexes(&mut self, field: Field) {
        self.fields
            .entry("865".to_string())
            .or_default()
            .push(field);
    }

    /// Get enumeration and chronology fields for indexes (865)
    #[must_use]
    pub fn enumeration_indexes(&self) -> &[Field] {
        self.fields.get("865").map_or(&[], Vec::as_slice)
    }

    /// Add a textual holdings field for basic units (866)
    pub fn add_textual_holdings_basic(&mut self, field: Field) {
        self.fields
            .entry("866".to_string())
            .or_default()
            .push(field);
    }

    /// Get textual holdings fields for basic units (866)
    #[must_use]
    pub fn textual_holdings_basic(&self) -> &[Field] {
        self.fields.get("866").map_or(&[], Vec::as_slice)
    }

    /// Add a textual holdings field for supplements (867)
    pub fn add_textual_holdings_supplements(&mut self, field: Field) {
        self.fields
            .entry("867".to_string())
            .or_default()
            .push(field);
    }

    /// Get textual holdings fields for supplements (867)
    #[must_use]
    pub fn textual_holdings_supplements(&self) -> &[Field] {
        self.fields.get("867").map_or(&[], Vec::as_slice)
    }

    /// Add a textual holdings field for indexes (868)
    pub fn add_textual_holdings_indexes(&mut self, field: Field) {
        self.fields
            .entry("868".to_string())
            .or_default()
            .push(field);
    }

    /// Get textual holdings fields for indexes (868)
    #[must_use]
    pub fn textual_holdings_indexes(&self) -> &[Field] {
        self.fields.get("868").map_or(&[], Vec::as_slice)
    }

    /// Add an item information field (876-878)
    pub fn add_item_information(&mut self, field: Field) {
        self.fields
            .entry(field.tag.clone())
            .or_default()
            .push(field);
    }

    /// Get item information fields by tag
    #[must_use]
    pub fn get_item_information(&self, tag: &str) -> Option<&[Field]> {
        self.fields.get(tag).map(Vec::as_slice)
    }

    /// Add a field
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

    /// Get holdings type from leader
    #[must_use]
    pub fn holdings_type(&self) -> HoldingsType {
        match self.leader.record_type {
            'x' => HoldingsType::SinglePartItem,
            'y' => HoldingsType::SerialItem,
            'v' => HoldingsType::MultipartItem,
            _ => HoldingsType::Unknown,
        }
    }

    /// Get acquisition status from 008/06
    #[must_use]
    pub fn acquisition_status(&self) -> Option<AcquisitionStatus> {
        self.get_control_field("008").and_then(|field| {
            if field.len() > 6 {
                match field.chars().nth(6) {
                    Some('0') => Some(AcquisitionStatus::Other),
                    Some('1') => Some(AcquisitionStatus::ReceivedAndComplete),
                    Some('2') => Some(AcquisitionStatus::OnOrder),
                    Some('3') => Some(AcquisitionStatus::ReceivedAndIncomplete),
                    _ => None,
                }
            } else {
                None
            }
        })
    }

    /// Get method of acquisition from 008/07
    #[must_use]
    pub fn method_of_acquisition(&self) -> Option<MethodOfAcquisition> {
        self.get_control_field("008").and_then(|field| {
            if field.len() > 7 {
                match field.chars().nth(7) {
                    Some('u') => Some(MethodOfAcquisition::Unknown),
                    Some('f') => Some(MethodOfAcquisition::Free),
                    Some('g') => Some(MethodOfAcquisition::Gift),
                    Some('l') => Some(MethodOfAcquisition::LegalDeposit),
                    Some('m') => Some(MethodOfAcquisition::Membership),
                    Some('p') => Some(MethodOfAcquisition::Purchase),
                    Some('e') => Some(MethodOfAcquisition::Exchange),
                    _ => None,
                }
            } else {
                None
            }
        })
    }

    /// Get completeness of holdings from 008/16
    #[must_use]
    pub fn completeness(&self) -> Option<Completeness> {
        self.get_control_field("008").and_then(|field| {
            if field.len() > 16 {
                match field.chars().nth(16) {
                    Some('1') => Some(Completeness::Complete),
                    Some('2') => Some(Completeness::Incomplete),
                    Some('3') => Some(Completeness::Scattered),
                    Some('4') => Some(Completeness::NotApplicable),
                    _ => None,
                }
            } else {
                None
            }
        })
    }

    /// Check if this is a serial holdings record
    #[must_use]
    pub fn is_serial(&self) -> bool {
        self.holdings_type() == HoldingsType::SerialItem
    }

    /// Check if this is a multipart item holdings record
    #[must_use]
    pub fn is_multipart(&self) -> bool {
        self.holdings_type() == HoldingsType::MultipartItem
    }
}

impl MarcRecord for HoldingsRecord {
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

/// Builder for fluently constructing holdings records
#[derive(Debug)]
pub struct HoldingsRecordBuilder {
    record: HoldingsRecord,
}

impl HoldingsRecordBuilder {
    /// Add a control field
    #[must_use]
    pub fn control_field(mut self, tag: String, value: String) -> Self {
        self.record.add_control_field(tag, value);
        self
    }

    /// Add a location field (852)
    #[must_use]
    pub fn location(mut self, field: Field) -> Self {
        self.record.add_location(field);
        self
    }

    /// Add a captions and pattern field for basic units (853)
    #[must_use]
    pub fn captions_basic(mut self, field: Field) -> Self {
        self.record.add_captions_basic(field);
        self
    }

    /// Add a captions and pattern field for supplements (854)
    #[must_use]
    pub fn captions_supplements(mut self, field: Field) -> Self {
        self.record.add_captions_supplements(field);
        self
    }

    /// Add a captions and pattern field for indexes (855)
    #[must_use]
    pub fn captions_indexes(mut self, field: Field) -> Self {
        self.record.add_captions_indexes(field);
        self
    }

    /// Add an enumeration and chronology field for basic units (863)
    #[must_use]
    pub fn enumeration_basic(mut self, field: Field) -> Self {
        self.record.add_enumeration_basic(field);
        self
    }

    /// Add an enumeration and chronology field for supplements (864)
    #[must_use]
    pub fn enumeration_supplements(mut self, field: Field) -> Self {
        self.record.add_enumeration_supplements(field);
        self
    }

    /// Add an enumeration and chronology field for indexes (865)
    #[must_use]
    pub fn enumeration_indexes(mut self, field: Field) -> Self {
        self.record.add_enumeration_indexes(field);
        self
    }

    /// Add a textual holdings field for basic units (866)
    #[must_use]
    pub fn textual_holdings_basic(mut self, field: Field) -> Self {
        self.record.add_textual_holdings_basic(field);
        self
    }

    /// Add a textual holdings field for supplements (867)
    #[must_use]
    pub fn textual_holdings_supplements(mut self, field: Field) -> Self {
        self.record.add_textual_holdings_supplements(field);
        self
    }

    /// Add a textual holdings field for indexes (868)
    #[must_use]
    pub fn textual_holdings_indexes(mut self, field: Field) -> Self {
        self.record.add_textual_holdings_indexes(field);
        self
    }

    /// Add an item information field
    #[must_use]
    pub fn item_information(mut self, field: Field) -> Self {
        self.record.add_item_information(field);
        self
    }

    /// Add a field to `other_fields`
    #[must_use]
    pub fn add_field(mut self, field: Field) -> Self {
        self.record.add_field(field);
        self
    }

    /// Build the holdings record
    #[must_use]
    pub fn build(self) -> HoldingsRecord {
        self.record
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::Subfield;

    fn create_test_leader() -> Leader {
        Leader {
            record_length: 1024,
            record_status: 'n',
            record_type: 'y',
            bibliographic_level: '|',
            control_record_type: ' ',
            character_coding: ' ',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 300,
            encoding_level: '1',
            cataloging_form: 'a',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        }
    }

    #[test]
    fn test_create_holdings_record() {
        let leader = create_test_leader();
        let record = HoldingsRecord::new(leader);
        assert!(record.locations().is_empty());
        assert!(record.textual_holdings_basic().is_empty());
    }

    #[test]
    fn test_holdings_record_builder() {
        let leader = create_test_leader();
        let record = HoldingsRecord::builder(leader)
            .control_field(
                "008".to_string(),
                "000101n| a azannaabn          |a aaa      ".to_string(),
            )
            .build();

        assert_eq!(
            record.get_control_field("008"),
            Some("000101n| a azannaabn          |a aaa      ")
        );
    }

    #[test]
    fn test_holdings_type_detection() {
        // Serial holdings
        let mut leader = create_test_leader();
        leader.record_type = 'y';
        let record = HoldingsRecord::new(leader.clone());
        assert_eq!(record.holdings_type(), HoldingsType::SerialItem);
        assert!(record.is_serial());

        // Multipart item holdings
        leader.record_type = 'v';
        let record = HoldingsRecord::new(leader.clone());
        assert_eq!(record.holdings_type(), HoldingsType::MultipartItem);
        assert!(record.is_multipart());

        // Single-part item holdings
        leader.record_type = 'x';
        let record = HoldingsRecord::new(leader);
        assert_eq!(record.holdings_type(), HoldingsType::SinglePartItem);
    }

    #[test]
    fn test_acquisition_status_parsing() {
        let leader = create_test_leader();

        // Received and complete (position 6 = '1')
        let record = HoldingsRecord::builder(leader.clone())
            .control_field(
                "008".to_string(),
                "0001011 a azannaabn          |a aaa      ".to_string(),
            )
            .build();
        assert_eq!(
            record.acquisition_status(),
            Some(AcquisitionStatus::ReceivedAndComplete)
        );

        // On order (position 6 = '2')
        let record = HoldingsRecord::builder(leader.clone())
            .control_field(
                "008".to_string(),
                "0001012 a azannaabn          |a aaa      ".to_string(),
            )
            .build();
        assert_eq!(
            record.acquisition_status(),
            Some(AcquisitionStatus::OnOrder)
        );

        // Received and incomplete (position 6 = '3')
        let record = HoldingsRecord::builder(leader)
            .control_field(
                "008".to_string(),
                "0001013 a azannaabn          |a aaa      ".to_string(),
            )
            .build();
        assert_eq!(
            record.acquisition_status(),
            Some(AcquisitionStatus::ReceivedAndIncomplete)
        );
    }

    #[test]
    fn test_method_of_acquisition_parsing() {
        let leader = create_test_leader();

        // Purchase (position 7 = 'p')
        let record = HoldingsRecord::builder(leader.clone())
            .control_field(
                "008".to_string(),
                "00010n1pzannaabn          |a aaa      ".to_string(),
            )
            .build();
        assert_eq!(
            record.method_of_acquisition(),
            Some(MethodOfAcquisition::Purchase)
        );

        // Gift (position 7 = 'g')
        let record = HoldingsRecord::builder(leader)
            .control_field(
                "008".to_string(),
                "00010n1gzannaabn          |a aaa      ".to_string(),
            )
            .build();
        assert_eq!(
            record.method_of_acquisition(),
            Some(MethodOfAcquisition::Gift)
        );
    }

    #[test]
    fn test_completeness_parsing() {
        let leader = create_test_leader();

        // Create a 40-character 008 field with position 16 = '1' (Complete)
        let field_008_str = "0000001pzzzzzzzz1                       ".to_string();
        let record = HoldingsRecord::builder(leader)
            .control_field("008".to_string(), field_008_str)
            .build();
        assert_eq!(record.completeness(), Some(Completeness::Complete));
    }

    #[test]
    fn test_add_location() {
        let leader = create_test_leader();
        let location = Field {
            tag: "852".to_string(),
            indicator1: ' ',
            indicator2: '1',
            subfields: smallvec::smallvec![Subfield {
                code: 'b',
                value: "Main Library".to_string(),
            }],
        };

        let record = HoldingsRecord::builder(leader).location(location).build();

        assert_eq!(record.locations().len(), 1);
        assert_eq!(record.locations()[0].tag, "852");
    }

    #[test]
    fn test_add_textual_holdings() {
        let leader = create_test_leader();
        let holdings = Field {
            tag: "866".to_string(),
            indicator1: '4',
            indicator2: '1',
            subfields: smallvec::smallvec![Subfield {
                code: 'a',
                value: "v.1 (1990)-v.10 (2000)".to_string(),
            }],
        };

        let record = HoldingsRecord::builder(leader)
            .textual_holdings_basic(holdings)
            .build();

        assert_eq!(record.textual_holdings_basic().len(), 1);
        assert_eq!(
            record.textual_holdings_basic()[0].get_subfield('a'),
            Some("v.1 (1990)-v.10 (2000)")
        );
    }

    #[test]
    fn test_control_field_operations() {
        let leader = create_test_leader();
        let mut record = HoldingsRecord::new(leader);

        record.add_control_field("001".to_string(), "ocm00123456".to_string());
        record.add_control_field("005".to_string(), "20250101".to_string());

        assert_eq!(record.get_control_field("001"), Some("ocm00123456"));
        assert_eq!(record.get_control_field("005"), Some("20250101"));
    }

    #[test]
    fn test_acquisition_status_none() {
        let leader = create_test_leader();

        // Missing 008 field
        let record = HoldingsRecord::new(leader.clone());
        assert_eq!(record.acquisition_status(), None);

        // 008 field too short
        let record = HoldingsRecord::builder(leader)
            .control_field("008".to_string(), "000".to_string())
            .build();
        assert_eq!(record.acquisition_status(), None);
    }

    #[test]
    fn test_method_of_acquisition_all_values() {
        let leader = create_test_leader();

        // Unknown
        let record = HoldingsRecord::builder(leader.clone())
            .control_field(
                "008".to_string(),
                "00010n1uzannaabn          |a aaa      ".to_string(),
            )
            .build();
        assert_eq!(
            record.method_of_acquisition(),
            Some(MethodOfAcquisition::Unknown)
        );

        // Free
        let record = HoldingsRecord::builder(leader.clone())
            .control_field(
                "008".to_string(),
                "00010n1fzannaabn          |a aaa      ".to_string(),
            )
            .build();
        assert_eq!(
            record.method_of_acquisition(),
            Some(MethodOfAcquisition::Free)
        );

        // Legal deposit
        let record = HoldingsRecord::builder(leader.clone())
            .control_field(
                "008".to_string(),
                "00010n1lzannaabn          |a aaa      ".to_string(),
            )
            .build();
        assert_eq!(
            record.method_of_acquisition(),
            Some(MethodOfAcquisition::LegalDeposit)
        );

        // Membership
        let record = HoldingsRecord::builder(leader.clone())
            .control_field(
                "008".to_string(),
                "00010n1mzannaabn          |a aaa      ".to_string(),
            )
            .build();
        assert_eq!(
            record.method_of_acquisition(),
            Some(MethodOfAcquisition::Membership)
        );

        // Exchange
        let record = HoldingsRecord::builder(leader)
            .control_field(
                "008".to_string(),
                "00010n1ezannaabn          |a aaa      ".to_string(),
            )
            .build();
        assert_eq!(
            record.method_of_acquisition(),
            Some(MethodOfAcquisition::Exchange)
        );
    }

    #[test]
    fn test_completeness_all_values() {
        let leader = create_test_leader();

        // Incomplete
        let record = HoldingsRecord::builder(leader.clone())
            .control_field(
                "008".to_string(),
                "0000001pzzzzzzzz2                       ".to_string(),
            )
            .build();
        assert_eq!(record.completeness(), Some(Completeness::Incomplete));

        // Scattered
        let record = HoldingsRecord::builder(leader.clone())
            .control_field(
                "008".to_string(),
                "0000001pzzzzzzzz3                       ".to_string(),
            )
            .build();
        assert_eq!(record.completeness(), Some(Completeness::Scattered));

        // Not applicable
        let record = HoldingsRecord::builder(leader)
            .control_field(
                "008".to_string(),
                "0000001pzzzzzzzz4                       ".to_string(),
            )
            .build();
        assert_eq!(record.completeness(), Some(Completeness::NotApplicable));
    }

    #[test]
    fn test_captions_and_enumeration_fields() {
        let leader = create_test_leader();

        let caption_field = Field {
            tag: "853".to_string(),
            indicator1: ' ',
            indicator2: '1',
            subfields: smallvec::smallvec![Subfield {
                code: 'a',
                value: "v.".to_string(),
            }],
        };

        let enum_field = Field {
            tag: "863".to_string(),
            indicator1: ' ',
            indicator2: '1',
            subfields: smallvec::smallvec![Subfield {
                code: 'a',
                value: "v.1".to_string(),
            }],
        };

        let record = HoldingsRecord::builder(leader)
            .captions_basic(caption_field)
            .enumeration_basic(enum_field)
            .build();

        assert_eq!(record.captions_basic().len(), 1);
        assert_eq!(record.enumeration_basic().len(), 1);
        assert_eq!(record.captions_basic()[0].tag, "853");
        assert_eq!(record.enumeration_basic()[0].tag, "863");
    }

    #[test]
    fn test_item_information_fields() {
        let leader = create_test_leader();

        let item_876 = Field {
            tag: "876".to_string(),
            indicator1: ' ',
            indicator2: ' ',
            subfields: smallvec::smallvec![Subfield {
                code: 'p',
                value: "12345".to_string(),
            }],
        };

        let item_877 = Field {
            tag: "877".to_string(),
            indicator1: ' ',
            indicator2: ' ',
            subfields: smallvec::smallvec![Subfield {
                code: 'p',
                value: "12346".to_string(),
            }],
        };

        let record = HoldingsRecord::builder(leader)
            .item_information(item_876)
            .item_information(item_877)
            .build();

        assert!(record.get_item_information("876").is_some());
        assert!(record.get_item_information("877").is_some());
        assert_eq!(record.get_item_information("876").unwrap().len(), 1);
    }

    #[test]
    fn test_other_fields() {
        let leader = create_test_leader();

        let field_500 = Field {
            tag: "500".to_string(),
            indicator1: ' ',
            indicator2: ' ',
            subfields: smallvec::smallvec![Subfield {
                code: 'a',
                value: "General note".to_string(),
            }],
        };

        let record = HoldingsRecord::builder(leader).add_field(field_500).build();

        assert!(record.get_fields("500").is_some());
        assert_eq!(record.get_fields("500").unwrap()[0].tag, "500");
    }

    #[test]
    fn test_single_part_item_type() {
        let mut leader = create_test_leader();
        leader.record_type = 'x';
        let record = HoldingsRecord::new(leader);
        assert_eq!(record.holdings_type(), HoldingsType::SinglePartItem);
        assert!(!record.is_serial());
        assert!(!record.is_multipart());
    }

    #[test]
    fn test_unknown_holdings_type() {
        let mut leader = create_test_leader();
        leader.record_type = 'z';
        let record = HoldingsRecord::new(leader);
        assert_eq!(record.holdings_type(), HoldingsType::Unknown);
    }
}
