//! MARC bibliographic record structures and operations.
//!
//! This module provides the core record types for working with MARC bibliographic records:
//! - [`Record`] — Main bibliographic record structure
//! - [`Field`] — Variable data fields (010+)
//! - [`Subfield`] — Named data elements within fields
//!
//! # Examples
//!
//! Create a record with the builder API:
//!
//! ```ignore
//! use mrrc::{Record, Field, Leader};
//!
//! let leader = Leader {
//!     record_length: 0,
//!     record_status: 'n',
//!     record_type: 'a',
//!     bibliographic_level: 'm',
//!     control_record_type: ' ',
//!     character_coding: ' ',
//!     indicator_count: 2,
//!     subfield_code_count: 2,
//!     data_base_address: 0,
//!     encoding_level: ' ',
//!     cataloging_form: 'a',
//!     multipart_level: ' ',
//!     reserved: "4500".to_string(),
//! };
//!
//! let record = Record::builder(leader)
//!     .control_field_str("001", "12345")
//!     .field(
//!         Field::builder("245".to_string(), '1', '0')
//!             .subfield_str('a', "Title")
//!             .build()
//!     )
//!     .build();
//! ```
//!
//! Iterate over fields:
//!
//! ```ignore
//! for field in record.fields_by_tag("650") {
//!     for value in field.subfields_by_code('a') {
//!         println!("Subject: {}", value);
//!     }
//! }
//! ```

use crate::bibliographic_helpers::PublicationInfo;
use crate::leader::Leader;
use crate::marc_record::MarcRecord;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::BTreeMap;
use std::ops::Index;

/// A MARC bibliographic record
///
/// Fields are stored in insertion order using `IndexMap`, preserving the order
/// in which fields were added to the record. This ensures round-trip fidelity
/// when serializing and deserializing records.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    /// Record leader (24 bytes)
    pub leader: Leader,
    /// Control fields (000-009) - tag -> value, preserves insertion order
    pub control_fields: IndexMap<String, String>,
    /// Data fields (010+) - tag -> fields, preserves insertion order
    pub fields: IndexMap<String, Vec<Field>>,
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
    /// Subfields (stored in `SmallVec` to avoid allocation for typical fields with 4 or fewer subfields)
    pub subfields: SmallVec<[Subfield; 4]>,
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
            control_fields: IndexMap::new(),
            fields: IndexMap::new(),
        }
    }

    /// Create a builder for fluently constructing MARC records
    ///
    /// # Examples
    ///
    /// ```
    /// use mrrc::{Record, Leader, Field};
    ///
    /// let leader = Leader {
    ///     record_length: 0,
    ///     record_status: 'n',
    ///     record_type: 'a',
    ///     bibliographic_level: 'm',
    ///     control_record_type: 'a',
    ///     character_coding: ' ',
    ///     indicator_count: 2,
    ///     subfield_code_count: 2,
    ///     data_base_address: 0,
    ///     encoding_level: ' ',
    ///     cataloging_form: ' ',
    ///     multipart_level: ' ',
    ///     reserved: "4500".to_string(),
    /// };
    ///
    /// let record = Record::builder(leader)
    ///     .control_field_str("001", "12345")
    ///     .field(Field::builder("245".to_string(), '1', '0')
    ///         .subfield_str('a', "Title")
    ///         .build())
    ///     .build();
    /// ```
    #[must_use]
    pub fn builder(leader: Leader) -> RecordBuilder {
        RecordBuilder {
            record: Record {
                leader,
                control_fields: IndexMap::new(),
                fields: IndexMap::new(),
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
        self.fields
            .entry(field.tag.clone())
            .or_default()
            .push(field);
    }

    /// Get all fields with a given tag
    #[must_use]
    pub fn get_fields(&self, tag: &str) -> Option<&[Field]> {
        self.fields.get(tag).map(std::vec::Vec::as_slice)
    }

    /// Get first field with a given tag
    #[must_use]
    pub fn get_field(&self, tag: &str) -> Option<&Field> {
        self.fields.get(tag).and_then(|v| v.first())
    }

    /// Iterate over all fields in tag order
    pub fn fields(&self) -> impl Iterator<Item = &Field> {
        self.fields.values().flat_map(|v| v.iter())
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
        self.fields.get(tag).map(|v| v.iter()).into_iter().flatten()
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
    // Advanced field queries
    // ============================================================================

    /// Iterate over fields matching a specific indicator pattern.
    ///
    /// # Arguments
    ///
    /// * `tag` - The field tag to search
    /// * `indicator1` - First indicator value, or `None` to match any
    /// * `indicator2` - Second indicator value, or `None` to match any
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Find all 650 fields with indicator2='0' (LCSH)
    /// for field in record.fields_by_indicator("650", None, Some('0')) {
    ///     println!("LCSH: {:?}", field);
    /// }
    /// ```
    pub fn fields_by_indicator(
        &self,
        tag: &str,
        indicator1: Option<char>,
        indicator2: Option<char>,
    ) -> impl Iterator<Item = &Field> {
        self.fields_by_tag(tag).filter(move |field| {
            if let Some(ind1) = indicator1 {
                if field.indicator1 != ind1 {
                    return false;
                }
            }
            if let Some(ind2) = indicator2 {
                if field.indicator2 != ind2 {
                    return false;
                }
            }
            true
        })
    }

    /// Iterate over fields within a tag range (inclusive).
    ///
    /// # Arguments
    ///
    /// * `start_tag` - Start of range (inclusive)
    /// * `end_tag` - End of range (inclusive)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Find all subject-related fields (600-699)
    /// for field in record.fields_in_range("600", "699") {
    ///     println!("Subject field: {}", field.tag);
    /// }
    /// ```
    pub fn fields_in_range(&self, start_tag: &str, end_tag: &str) -> impl Iterator<Item = &Field> {
        let start = start_tag.to_string();
        let end = end_tag.to_string();
        self.fields
            .iter()
            .filter(move |(tag, _)| *tag >= &start && *tag <= &end)
            .flat_map(|(_, fields)| fields.iter())
    }

    /// Iterate over fields that have a specific subfield code.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Find all fields with subfield 'a'
    /// for field in record.fields_with_subfield("650", 'a') {
    ///     println!("Field: {}", field.tag);
    /// }
    /// ```
    pub fn fields_with_subfield(&self, tag: &str, code: char) -> impl Iterator<Item = &Field> {
        self.fields_by_tag(tag)
            .filter(move |field| field.get_subfield(code).is_some())
    }

    /// Iterate over fields that have all of the specified subfield codes.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Find all 650 fields with both 'a' and 'x' subfields
    /// for field in record.fields_with_subfields("650", &['a', 'x']) {
    ///     println!("Subject: {:?}", field);
    /// }
    /// ```
    pub fn fields_with_subfields<'a>(
        &'a self,
        tag: &'a str,
        codes: &'a [char],
    ) -> impl Iterator<Item = &'a Field> + 'a {
        self.fields_by_tag(tag)
            .filter(move |field| codes.iter().all(|&code| field.get_subfield(code).is_some()))
    }

    /// Iterate over fields matching a query.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mrrc::FieldQuery;
    ///
    /// let query = FieldQuery::new()
    ///     .tag("650")
    ///     .indicator2(Some('0'))
    ///     .has_subfield('a');
    ///
    /// for field in record.fields_matching(&query) {
    ///     println!("LCSH: {:?}", field);
    /// }
    /// ```
    pub fn fields_matching<'a>(
        &'a self,
        query: &'a crate::field_query::FieldQuery,
    ) -> impl Iterator<Item = &'a Field> + 'a {
        self.fields().filter(move |field| query.matches(field))
    }

    /// Iterate over fields matching a tag range query.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mrrc::TagRangeQuery;
    ///
    /// let query = TagRangeQuery {
    ///     start_tag: "600".to_string(),
    ///     end_tag: "699".to_string(),
    ///     indicator1: None,
    ///     indicator2: Some('0'),
    ///     required_subfields: vec!['a'],
    /// };
    ///
    /// for field in record.fields_matching_range(&query) {
    ///     println!("Subject: {:?}", field);
    /// }
    /// ```
    pub fn fields_matching_range<'a>(
        &'a self,
        query: &'a crate::field_query::TagRangeQuery,
    ) -> impl Iterator<Item = &'a Field> + 'a {
        self.fields_in_range(&query.start_tag, &query.end_tag)
            .filter(move |field| query.matches(field))
    }

    /// Find all fields where a subfield value matches a regex pattern.
    ///
    /// # Arguments
    ///
    /// * `query` - A `SubfieldPatternQuery` defining tag, subfield code, and regex pattern
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mrrc::field_query::SubfieldPatternQuery;
    ///
    /// // Find all ISBNs starting with 978
    /// let query = SubfieldPatternQuery::new("020", 'a', r"^978-.*")?;
    /// for field in record.fields_matching_pattern(&query) {
    ///     println!("ISBN: {:?}", field);
    /// }
    /// ```
    pub fn fields_matching_pattern<'a>(
        &'a self,
        query: &'a crate::field_query::SubfieldPatternQuery,
    ) -> impl Iterator<Item = &'a Field> + 'a {
        self.fields_by_tag(&query.tag)
            .filter(move |field| query.matches(field))
    }

    /// Find all fields where a subfield value matches a specific string.
    ///
    /// # Arguments
    ///
    /// * `query` - A `SubfieldValueQuery` defining tag, subfield code, and value to match
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mrrc::field_query::SubfieldValueQuery;
    ///
    /// // Find exact match
    /// let query = SubfieldValueQuery::new("650", 'a', "History");
    /// for field in record.fields_matching_value(&query) {
    ///     println!("Subject: {:?}", field);
    /// }
    ///
    /// // Find partial match
    /// let query = SubfieldValueQuery::partial("650", 'a', "History");
    /// for field in record.fields_matching_value(&query) {
    ///     println!("Subject: {:?}", field);
    /// }
    /// ```
    pub fn fields_matching_value<'a>(
        &'a self,
        query: &'a crate::field_query::SubfieldValueQuery,
    ) -> impl Iterator<Item = &'a Field> + 'a {
        self.fields_by_tag(&query.tag)
            .filter(move |field| query.matches(field))
    }

    // ============================================================================
    // Linked field navigation (880 field linkage)
    // ============================================================================

    /// Find the 880 field linked to a given original field.
    ///
    /// In MARC records, 880 fields provide alternate graphical representations
    /// (e.g., romanized text paired with original script). The linkage is
    /// established via subfield 6 which contains an occurrence number.
    ///
    /// # Arguments
    ///
    /// * `field` - The original field to find the linked 880 for
    ///
    /// # Returns
    ///
    /// The linked 880 field if found, or None if:
    /// - Field has no subfield 6
    /// - Subfield 6 is malformed
    /// - No matching 880 field exists
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let field_100 = record.get_field("100").unwrap();
    /// if let Some(field_880) = record.get_linked_field(field_100) {
    ///     println!("Original: {}", field_100.get_subfield('a').unwrap());
    ///     println!("Romanized: {}", field_880.get_subfield('a').unwrap());
    /// }
    /// ```
    #[must_use]
    pub fn get_linked_field(&self, field: &Field) -> Option<&Field> {
        // Get subfield 6 from the field
        let subfield_6 = field.get_subfield('6')?;

        // Parse the linkage information
        let linkage = crate::field_linkage::LinkageInfo::parse(subfield_6)?;

        // Find all 880 fields
        let mut found = None;
        for field_880 in self.fields_by_tag("880") {
            if let Some(sf6) = field_880.get_subfield('6') {
                if let Some(linkage_880) = crate::field_linkage::LinkageInfo::parse(sf6) {
                    // Check if this 880's linkage points back to our original field
                    if linkage_880.occurrence == linkage.occurrence {
                        found = Some(field_880);
                        break;
                    }
                }
            }
        }

        found
    }

    /// Find the original field linked from a given 880 field.
    ///
    /// If an 880 field is provided, finds its linked original field.
    /// This is the reverse of [`Self::get_linked_field`].
    ///
    /// # Arguments
    ///
    /// * `field_880` - An 880 field
    ///
    /// # Returns
    ///
    /// The linked original field if found, or None if:
    /// - Field is not an 880
    /// - 880 has no subfield 6
    /// - Subfield 6 is malformed
    /// - No matching original field exists
    #[must_use]
    pub fn get_original_field(&self, field_880: &Field) -> Option<&Field> {
        // 880 fields should link to original fields
        if field_880.tag != "880" {
            return None;
        }

        // Get subfield 6 from the 880 field
        let subfield_6 = field_880.get_subfield('6')?;

        // Parse the linkage information
        let linkage = crate::field_linkage::LinkageInfo::parse(subfield_6)?;

        // The linkage tells us which field and occurrence to find
        // Subfield 6 in 880 has format: "TAG-OCC[/r]"
        // We need to extract the TAG part
        let original_tag = if subfield_6.len() >= 3 {
            &subfield_6[0..3]
        } else {
            return None;
        };

        // Find the original field with matching tag and occurrence
        for field_orig in self.fields_by_tag(original_tag) {
            if let Some(sf6_orig) = field_orig.get_subfield('6') {
                if let Some(linkage_orig) = crate::field_linkage::LinkageInfo::parse(sf6_orig) {
                    // Check if this original field links to our 880
                    if linkage_orig.occurrence == linkage.occurrence {
                        return Some(field_orig);
                    }
                }
            }
        }

        None
    }

    /// Get all 880 fields (alternate graphical representations).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// for field_880 in record.get_all_880_fields() {
    ///     println!("880 field: {:?}", field_880);
    /// }
    /// ```
    #[must_use]
    pub fn get_all_880_fields(&self) -> Vec<&Field> {
        self.fields_by_tag("880").collect()
    }

    /// Get field pairs of original fields with their linked 880 counterparts.
    ///
    /// For a given tag, returns tuples of (`original_field`, Option<`linked_880`>).
    /// The Option will be None if the original field has no linked 880.
    ///
    /// # Arguments
    ///
    /// * `tag` - The original field tag to pair with 880s
    ///
    /// # Examples
    ///
    /// ```ignore
    /// for (orig, linked_880) in record.get_field_pairs("100") {
    ///     let name = orig.get_subfield('a').unwrap_or("unknown");
    ///     if let Some(field_880) = linked_880 {
    ///         let romanized = field_880.get_subfield('a').unwrap_or("unknown");
    ///         println!("Name: {} (romanized: {})", name, romanized);
    ///     } else {
    ///         println!("Name: {} (no alternate form)", name);
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn get_field_pairs(&self, tag: &str) -> Vec<(&Field, Option<&Field>)> {
        let mut pairs = Vec::new();

        for orig_field in self.fields_by_tag(tag) {
            let linked = self.get_linked_field(orig_field);
            pairs.push((orig_field, linked));
        }

        pairs
    }

    /// Find all fields linked by a specific occurrence number.
    ///
    /// # Arguments
    ///
    /// * `occurrence` - The occurrence number to search for (e.g., "01")
    ///
    /// # Returns
    ///
    /// Vector of all fields (original and 880) with matching occurrence in subfield 6
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let fields = record.find_linked_by_occurrence("01");
    /// // Returns both original field and its 880 counterpart, if both exist
    /// ```
    #[must_use]
    pub fn find_linked_by_occurrence(&self, occurrence: &str) -> Vec<&Field> {
        let mut results = Vec::new();

        // Search all fields
        for field in self.fields() {
            if let Some(sf6) = field.get_subfield('6') {
                if let Some(linkage) = crate::field_linkage::LinkageInfo::parse(sf6) {
                    if linkage.occurrence == occurrence {
                        results.push(field);
                    }
                }
            }
        }

        results
    }

    // ============================================================================
    // Mutable field operations
    // ============================================================================

    /// Get mutable reference to first field with a given tag
    pub fn get_field_mut(&mut self, tag: &str) -> Option<&mut Field> {
        self.fields.get_mut(tag).and_then(|v| v.first_mut())
    }

    /// Get mutable slice of fields with a given tag
    pub fn get_fields_mut(&mut self, tag: &str) -> Option<&mut [Field]> {
        self.fields.get_mut(tag).map(std::vec::Vec::as_mut_slice)
    }

    /// Iterate mutably over all fields
    pub fn fields_mut(&mut self) -> impl Iterator<Item = &mut Field> {
        self.fields.values_mut().flat_map(|v| v.iter_mut())
    }

    /// Iterate mutably over fields matching a specific tag
    ///
    /// # Examples
    ///
    /// ```ignore
    /// for field in record.fields_by_tag_mut("650") {
    ///     field.indicator2 = '0';
    /// }
    /// ```
    pub fn fields_by_tag_mut(&mut self, tag: &str) -> impl Iterator<Item = &mut Field> {
        let tag_str = tag.to_string();
        self.fields
            .get_mut(tag_str.as_str())
            .map(|v| v.iter_mut())
            .into_iter()
            .flatten()
    }

    // ============================================================================
    // Batch field operations
    // ============================================================================

    /// Remove all fields with a given tag
    ///
    /// Returns the removed fields.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let removed = record.remove_fields_by_tag("852");  // Remove holdings
    /// ```
    pub fn remove_fields_by_tag(&mut self, tag: &str) -> Vec<Field> {
        self.fields.shift_remove(tag).unwrap_or_default()
    }

    /// Remove fields matching a predicate
    ///
    /// Returns the removed fields.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let removed = record.remove_fields_where(|field| field.tag == "852");
    /// ```
    pub fn remove_fields_where<F>(&mut self, predicate: F) -> Vec<Field>
    where
        F: Fn(&Field) -> bool,
    {
        let mut removed = Vec::new();
        for fields in self.fields.values_mut() {
            fields.retain(|f| {
                if predicate(f) {
                    removed.push(f.clone());
                    false
                } else {
                    true
                }
            });
        }
        // Clean up empty tag entries
        self.fields.retain(|_, v| !v.is_empty());
        removed
    }

    /// Update fields matching a predicate
    ///
    /// Applies the given operation to each matching field.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// record.update_fields_where(
    ///     |field| field.tag == "245",
    ///     |field| field.indicator2 = '0'
    /// );
    /// ```
    pub fn update_fields_where<F, G>(&mut self, predicate: F, mut operation: G)
    where
        F: Fn(&Field) -> bool,
        G: FnMut(&mut Field),
    {
        for fields in self.fields.values_mut() {
            for field in fields.iter_mut() {
                if predicate(field) {
                    operation(field);
                }
            }
        }
    }

    /// Update all subfield values in fields with a given tag
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Update all authority codes in 650 fields
    /// record.update_subfield_values("650", 'd', "updated-value");
    /// ```
    pub fn update_subfield_values(&mut self, tag: &str, subfield_code: char, new_value: &str) {
        if let Some(fields) = self.fields.get_mut(tag) {
            for field in fields {
                for subfield in &mut field.subfields {
                    if subfield.code == subfield_code {
                        subfield.value = new_value.to_string();
                    }
                }
            }
        }
    }

    /// Update subfield values in fields matching a predicate
    ///
    /// # Examples
    ///
    /// ```ignore
    /// record.update_subfields_where(
    ///     |field| field.tag == "650",
    ///     |subfield| subfield.code == 'd',
    ///     "updated-value"
    /// );
    /// ```
    pub fn update_subfields_where<F>(&mut self, field_pred: F, subfield_code: char, new_value: &str)
    where
        F: Fn(&Field) -> bool,
    {
        for fields in self.fields.values_mut() {
            for field in fields {
                if field_pred(field) {
                    for subfield in &mut field.subfields {
                        if subfield.code == subfield_code {
                            subfield.value = new_value.to_string();
                        }
                    }
                }
            }
        }
    }

    /// Remove all fields from the record
    pub fn clear_fields(&mut self) {
        self.fields.clear();
    }

    /// Clear all control fields from the record
    pub fn clear_control_fields(&mut self) {
        self.control_fields.clear();
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

    fn get_fields(&self, tag: &str) -> Option<&[Field]> {
        self.fields.get(tag).map(std::vec::Vec::as_slice)
    }

    fn get_field(&self, tag: &str) -> Option<&Field> {
        self.fields.get(tag).and_then(|v| v.first())
    }
}

impl crate::field_query_helpers::FieldQueryHelpers for Record {
    fn fields_matching_pattern(
        &self,
        query: &crate::field_query::SubfieldPatternQuery,
    ) -> Vec<&Field> {
        self.fields_by_tag(&query.tag)
            .filter(|field| query.matches(field))
            .collect()
    }

    fn fields_matching_value(&self, query: &crate::field_query::SubfieldValueQuery) -> Vec<&Field> {
        self.fields_by_tag(&query.tag)
            .filter(|field| query.matches(field))
            .collect()
    }

    fn names_in_range(&self, start_tag: &str, end_tag: &str) -> Vec<&Field> {
        self.fields_in_range(start_tag, end_tag).collect()
    }

    fn authors_with_dates(&self) -> Vec<(&str, &str)> {
        let mut results = Vec::new();

        // Check primary author (100)
        if let Some(field) = self.get_field("100") {
            if let Some(name) = field.get_subfield('a') {
                if let Some(dates) = field.get_subfield('d') {
                    results.push((name, dates));
                }
            }
        }

        // Check added entry authors (700)
        if let Some(fields) = self.get_fields("700") {
            for field in fields {
                if let Some(name) = field.get_subfield('a') {
                    if let Some(dates) = field.get_subfield('d') {
                        results.push((name, dates));
                    }
                }
            }
        }

        results
    }
}

/// Enable dictionary-like access to Record fields using `record["245"]`.
///
/// Returns the first field with the given tag, or panics if not found.
/// For fallible access, use `Record::get_field()` instead.
///
/// # Examples
///
/// ```ignore
/// let field = &record["245"];
/// ```
impl Index<&str> for Record {
    type Output = Field;

    fn index(&self, tag: &str) -> &Self::Output {
        self.get_field(tag).expect("field not found")
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
            subfields: SmallVec::new(),
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
                subfields: SmallVec::new(),
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

    /// Get all subfield values matching any of the given codes
    ///
    /// Returns a list of subfield values in the order they appear in the field.
    /// Accepts one or more subfield codes.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let values = field.get_subfields('a', 'b', 'c');
    /// ```
    #[must_use]
    pub fn get_subfields(&self, codes: &[char]) -> Vec<&str> {
        self.subfields
            .iter()
            .filter(|sf| codes.contains(&sf.code))
            .map(|sf| sf.value.as_str())
            .collect()
    }

    /// Get all subfields as a dictionary
    ///
    /// Returns a `BTreeMap` where keys are subfield codes (as `String`)
    /// and values are vectors of subfield values. Since subfield codes
    /// can repeat, the values are always vectors.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let subfields = field.subfields_as_dict();
    /// for (code, values) in subfields {
    ///     println!("Code {}: {:?}", code, values);
    /// }
    /// ```
    pub fn subfields_as_dict(&self) -> BTreeMap<String, Vec<String>> {
        let mut result = BTreeMap::new();
        for subfield in &self.subfields {
            result
                .entry(subfield.code.to_string())
                .or_insert_with(Vec::new)
                .push(subfield.value.clone());
        }
        result
    }

    /// Get the field's content as a formatted string
    ///
    /// For control fields (000-009), returns the field data.
    /// For data fields, concatenates all subfield values with spaces.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let value_str = field.value();
    /// ```
    #[must_use]
    pub fn value(&self) -> String {
        self.subfields
            .iter()
            .map(|sf| sf.value.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Get the field's content as a pretty-formatted string
    ///
    /// For control fields, returns the field data.
    /// For data fields, concatenates subfield values with spaces.
    /// For subject fields (6xx), uses " -- " before subdivision subfields (v, x, y, z).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let formatted = field.format_field();
    /// // For 650 (subject): "Subject -- Subdivision -- Geographic"
    /// ```
    #[must_use]
    pub fn format_field(&self) -> String {
        if self.subfields.is_empty() {
            return String::new();
        }

        let is_subject = self.tag.starts_with('6');
        let mut result = Vec::new();

        for subfield in &self.subfields {
            // Skip field linking subfield
            if subfield.code == '6' {
                continue;
            }

            if is_subject && matches!(subfield.code, 'v' | 'x' | 'y' | 'z') {
                result.push(format!(" -- {}", subfield.value));
            } else {
                result.push(format!(" {}", subfield.value));
            }
        }

        result.join("").trim().to_string()
    }

    // ============================================================================
    // Mutable subfield operations
    // ============================================================================

    /// Get mutable reference to first subfield with a given code
    pub fn get_subfield_mut(&mut self, code: char) -> Option<&mut Subfield> {
        self.subfields.iter_mut().find(|sf| sf.code == code)
    }

    /// Iterate mutably over all subfields
    pub fn subfields_mut(&mut self) -> impl Iterator<Item = &mut Subfield> {
        self.subfields.iter_mut()
    }

    /// Iterate mutably over subfields with a specific code
    pub fn subfields_by_code_mut(&mut self, code: char) -> impl Iterator<Item = &mut Subfield> {
        self.subfields.iter_mut().filter(move |sf| sf.code == code)
    }

    /// Remove all subfields with a given code
    ///
    /// Returns the removed subfields.
    pub fn remove_subfields(&mut self, code: char) -> Vec<Subfield> {
        let mut removed = Vec::new();
        self.subfields.retain(|sf| {
            if sf.code == code {
                removed.push(sf.clone());
                false
            } else {
                true
            }
        });
        removed
    }

    /// Remove subfields matching a predicate
    ///
    /// Returns the removed subfields.
    pub fn remove_subfields_where<F>(&mut self, predicate: F) -> Vec<Subfield>
    where
        F: Fn(&Subfield) -> bool,
    {
        let mut removed = Vec::new();
        self.subfields.retain(|sf| {
            if predicate(sf) {
                removed.push(sf.clone());
                false
            } else {
                true
            }
        });
        removed
    }

    /// Update all subfield values with a given code
    ///
    /// # Examples
    ///
    /// ```ignore
    /// field.update_subfield_values('a', "new value");
    /// ```
    pub fn update_subfield_values(&mut self, code: char, new_value: &str) {
        for subfield in &mut self.subfields {
            if subfield.code == code {
                subfield.value = new_value.to_string();
            }
        }
    }

    /// Update subfield values matching a predicate
    pub fn update_subfields_where<F>(&mut self, predicate: F, new_value: &str)
    where
        F: Fn(&Subfield) -> bool,
    {
        for subfield in &mut self.subfields {
            if predicate(subfield) {
                subfield.value = new_value.to_string();
            }
        }
    }

    /// Clear all subfields from the field
    pub fn clear_subfields(&mut self) {
        self.subfields.clear();
    }
}

/// Enable dictionary-like access to Field subfields using `field['a']`.
///
/// Returns the first subfield with the given code, or panics if not found.
/// For fallible access, use `Field::get_subfield()` instead.
///
/// # Examples
///
/// ```ignore
/// let title = &field['a'];
/// ```
impl Index<char> for Field {
    type Output = str;

    fn index(&self, code: char) -> &Self::Output {
        self.get_subfield(code).expect("subfield not found")
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
    use crate::record_helpers::RecordHelpers;

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
        assert!(record.fields.is_empty());
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
    fn test_helper_subjects_all_6xx() {
        let leader = make_leader();
        let mut record = Record::new(leader);

        // 600 — Personal Name Subject
        let mut f600 = Field::new("600".to_string(), '1', '0');
        f600.add_subfield('a', "Maimonides, Moses,".to_string());
        record.add_field(f600);

        // 610 — Corporate Name Subject
        let mut f610 = Field::new("610".to_string(), '2', '0');
        f610.add_subfield('a', "United Nations".to_string());
        record.add_field(f610);

        // 630 — Uniform Title Subject
        let mut f630 = Field::new("630".to_string(), '0', '4');
        f630.add_subfield('a', "Talmud Bavli.".to_string());
        record.add_field(f630);

        // 650 — Topical Term
        let mut f650 = Field::new("650".to_string(), ' ', '0');
        f650.add_subfield('a', "Jewish law.".to_string());
        record.add_field(f650);

        // 651 — Geographic Name
        let mut f651 = Field::new("651".to_string(), ' ', '0');
        f651.add_subfield('a', "Jerusalem".to_string());
        record.add_field(f651);

        // 655 — Genre/Form
        let mut f655 = Field::new("655".to_string(), ' ', '7');
        f655.add_subfield('a', "Commentaries.".to_string());
        record.add_field(f655);

        let subjects = record.subjects();
        assert_eq!(subjects.len(), 6);
        assert!(subjects.contains(&"Maimonides, Moses,"));
        assert!(subjects.contains(&"United Nations"));
        assert!(subjects.contains(&"Talmud Bavli."));
        assert!(subjects.contains(&"Jewish law."));
        assert!(subjects.contains(&"Jerusalem"));
        assert!(subjects.contains(&"Commentaries."));
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

    // =======================================================================
    // Integration tests for Phase 2 field query helpers
    // =======================================================================

    #[test]
    fn test_fields_matching_pattern_isbn() {
        use crate::field_query::SubfieldPatternQuery;

        let mut record = Record::new(make_leader());

        // Add ISBNs with different patterns
        let mut isbn1 = Field::new("020".to_string(), ' ', ' ');
        isbn1.add_subfield_str('a', "978-0-12345-678-9");
        record.add_field(isbn1);

        let mut isbn2 = Field::new("020".to_string(), ' ', ' ');
        isbn2.add_subfield_str('a', "979-10-000000-00-0");
        record.add_field(isbn2);

        let mut isbn3 = Field::new("020".to_string(), ' ', ' ');
        isbn3.add_subfield_str('a', "978-1-111111-11-1");
        record.add_field(isbn3);

        // Find all ISBNs starting with 978
        let query = SubfieldPatternQuery::new("020", 'a', r"^978-.*").unwrap();
        let matches: Vec<_> = record.fields_matching_pattern(&query).collect();
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_fields_matching_value_exact() {
        use crate::field_query::SubfieldValueQuery;

        let mut record = Record::new(make_leader());

        let mut subject1 = Field::new("650".to_string(), ' ', '0');
        subject1.add_subfield_str('a', "History");
        record.add_field(subject1);

        let mut subject2 = Field::new("650".to_string(), ' ', '0');
        subject2.add_subfield_str('a', "Science");
        record.add_field(subject2);

        let mut subject3 = Field::new("650".to_string(), ' ', '0');
        subject3.add_subfield_str('a', "History");
        record.add_field(subject3);

        let query = SubfieldValueQuery::new("650", 'a', "History");
        let matches: Vec<_> = record.fields_matching_value(&query).collect();
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_fields_matching_value_partial() {
        use crate::field_query::SubfieldValueQuery;

        let mut record = Record::new(make_leader());

        let mut subject1 = Field::new("650".to_string(), ' ', '0');
        subject1.add_subfield_str('a', "World History");
        record.add_field(subject1);

        let mut subject2 = Field::new("650".to_string(), ' ', '0');
        subject2.add_subfield_str('a', "Medieval History");
        record.add_field(subject2);

        let mut subject3 = Field::new("650".to_string(), ' ', '0');
        subject3.add_subfield_str('a', "Science");
        record.add_field(subject3);

        let query = SubfieldValueQuery::partial("650", 'a', "History");
        let matches: Vec<_> = record.fields_matching_value(&query).collect();
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_subjects_with_subdivision() {
        use crate::FieldQueryHelpers;

        let mut record = Record::new(make_leader());

        let mut subject1 = Field::new("650".to_string(), ' ', '0');
        subject1.add_subfield_str('a', "World");
        subject1.add_subfield_str('x', "History");
        record.add_field(subject1);

        let mut subject2 = Field::new("650".to_string(), ' ', '0');
        subject2.add_subfield_str('a', "Philosophy");
        subject2.add_subfield_str('x', "History");
        record.add_field(subject2);

        let mut subject3 = Field::new("650".to_string(), ' ', '0');
        subject3.add_subfield_str('a', "Science");
        subject3.add_subfield_str('y', "Geography");
        record.add_field(subject3);

        let results = record.subjects_with_subdivision('x', "History");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_isbns_matching() {
        use crate::FieldQueryHelpers;

        let mut record = Record::new(make_leader());

        // Add multiple ISBNs
        let mut isbn1 = Field::new("020".to_string(), ' ', ' ');
        isbn1.add_subfield_str('a', "978-0-201-61622-4");
        record.add_field(isbn1);

        let mut isbn2 = Field::new("020".to_string(), ' ', ' ');
        isbn2.add_subfield_str('a', "979-10-90636-07-1");
        record.add_field(isbn2);

        let mut isbn3 = Field::new("020".to_string(), ' ', ' ');
        isbn3.add_subfield_str('a', "978-1-449-35582-1");
        record.add_field(isbn3);

        let results = record.isbns_matching(r"^978-.*").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_names_in_range() {
        use crate::FieldQueryHelpers;

        let mut record = Record::new(make_leader());

        // Add primary author
        let mut field100 = Field::new("100".to_string(), ' ', ' ');
        field100.add_subfield_str('a', "Smith, John");
        record.add_field(field100);

        // Add added entries
        let mut field700 = Field::new("700".to_string(), ' ', ' ');
        field700.add_subfield_str('a', "Doe, Jane");
        record.add_field(field700);

        let mut field710 = Field::new("710".to_string(), ' ', ' ');
        field710.add_subfield_str('a', "Publisher Inc.");
        record.add_field(field710);

        let names = record.names_in_range("700", "711");
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn test_authors_with_dates() {
        use crate::FieldQueryHelpers;

        let mut record = Record::new(make_leader());

        // Add primary author with dates
        let mut field100 = Field::new("100".to_string(), ' ', ' ');
        field100.add_subfield_str('a', "Smith, John");
        field100.add_subfield_str('d', "1873-1944");
        record.add_field(field100);

        // Add added entry with dates
        let mut field700a = Field::new("700".to_string(), ' ', ' ');
        field700a.add_subfield_str('a', "Doe, Jane");
        field700a.add_subfield_str('d', "1902-1989");
        record.add_field(field700a);

        // Add added entry without dates
        let mut field700b = Field::new("700".to_string(), ' ', ' ');
        field700b.add_subfield_str('a', "Johnson, Robert");
        record.add_field(field700b);

        let authors = record.authors_with_dates();
        assert_eq!(authors.len(), 2);
        assert_eq!(authors[0], ("Smith, John", "1873-1944"));
        assert_eq!(authors[1], ("Doe, Jane", "1902-1989"));
    }

    #[test]
    fn test_subjects_with_note() {
        use crate::FieldQueryHelpers;

        let mut record = Record::new(make_leader());

        let mut subject1 = Field::new("650".to_string(), ' ', '0');
        subject1.add_subfield_str('a', "World");
        subject1.add_subfield_str('x', "Medieval History");
        record.add_field(subject1);

        let mut subject2 = Field::new("650".to_string(), ' ', '0');
        subject2.add_subfield_str('a', "Philosophy");
        subject2.add_subfield_str('x', "Ancient History");
        record.add_field(subject2);

        let mut subject3 = Field::new("650".to_string(), ' ', '0');
        subject3.add_subfield_str('a', "Science");
        subject3.add_subfield_str('y', "Geography");
        record.add_field(subject3);

        let results = record.subjects_with_note("History");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_record_index_field_by_tag() {
        let mut record = Record::new(make_leader());
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield_str('a', "Test Title");
        record.add_field(field);

        let indexed_field = &record["245"];
        assert_eq!(indexed_field.tag, "245");
        assert_eq!(indexed_field.indicator1, '1');
        assert_eq!(indexed_field.indicator2, '0');
    }

    #[test]
    #[should_panic(expected = "field not found")]
    fn test_record_index_missing_field() {
        let record = Record::new(make_leader());
        let _ = &record["999"];
    }

    #[test]
    fn test_field_index_subfield_by_code() {
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield_str('a', "Test Title");
        field.add_subfield_str('b', "Subtitle");

        assert_eq!(&field['a'], "Test Title");
        assert_eq!(&field['b'], "Subtitle");
    }

    #[test]
    #[should_panic(expected = "subfield not found")]
    fn test_field_index_missing_subfield() {
        let field = Field::new("245".to_string(), '1', '0');
        let _ = &field['a'];
    }

    #[test]
    fn test_field_index_multiple_subfields_same_code() {
        let mut field = Field::new("650".to_string(), ' ', '0');
        field.add_subfield_str('a', "First Subject");
        field.add_subfield_str('x', "Subdivision 1");
        field.add_subfield_str('x', "Subdivision 2");

        // Index access should return the first matching subfield
        assert_eq!(&field['a'], "First Subject");
        assert_eq!(&field['x'], "Subdivision 1");
    }

    #[test]
    fn test_chained_index_access() {
        let mut record = Record::new(make_leader());
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield_str('a', "Title");
        field.add_subfield_str('c', "Author");
        record.add_field(field);

        // Test chained index access: record["245"]['a']
        assert_eq!(&record["245"]['a'], "Title");
        assert_eq!(&record["245"]['c'], "Author");
    }

    #[test]
    fn test_field_get_subfields_single_code() {
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield_str('a', "Title");
        field.add_subfield_str('b', "Subtitle");
        field.add_subfield_str('c', "Author");

        let values = field.get_subfields(&['a']);
        assert_eq!(values, vec!["Title"]);
    }

    #[test]
    fn test_field_get_subfields_multiple_codes() {
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield_str('a', "Title");
        field.add_subfield_str('b', "Subtitle");
        field.add_subfield_str('c', "Author");

        let values = field.get_subfields(&['a', 'c']);
        assert_eq!(values, vec!["Title", "Author"]);
    }

    #[test]
    fn test_field_get_subfields_preserves_order() {
        let mut field = Field::new("650".to_string(), ' ', '0');
        field.add_subfield_str('a', "Subject");
        field.add_subfield_str('x', "Subdivision 1");
        field.add_subfield_str('y', "Subdivision 2");
        field.add_subfield_str('z', "Geographic");

        let values = field.get_subfields(&['z', 'a', 'y']);
        assert_eq!(values.len(), 3);
        // Should preserve the order they appear in the field, not the order of codes requested
        assert_eq!(values, vec!["Subject", "Subdivision 2", "Geographic"]);
    }

    #[test]
    fn test_field_get_subfields_with_repeating_codes() {
        let mut field = Field::new("650".to_string(), ' ', '0');
        field.add_subfield_str('a', "Subject 1");
        field.add_subfield_str('x', "Subdivision 1");
        field.add_subfield_str('x', "Subdivision 2");
        field.add_subfield_str('z', "Geographic");

        let values = field.get_subfields(&['x']);
        assert_eq!(values, vec!["Subdivision 1", "Subdivision 2"]);
    }

    #[test]
    fn test_field_get_subfields_empty_result() {
        let field = Field::new("245".to_string(), '1', '0');
        let values = field.get_subfields(&['z']);
        assert!(values.is_empty());
    }

    #[test]
    fn test_field_subfields_as_dict_basic() {
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield_str('a', "Title");
        field.add_subfield_str('b', "Subtitle");
        field.add_subfield_str('c', "Author");

        let dict = field.subfields_as_dict();
        assert_eq!(dict.get("a"), Some(&vec!["Title".to_string()]));
        assert_eq!(dict.get("b"), Some(&vec!["Subtitle".to_string()]));
        assert_eq!(dict.get("c"), Some(&vec!["Author".to_string()]));
    }

    #[test]
    fn test_field_subfields_as_dict_repeating_codes() {
        let mut field = Field::new("650".to_string(), ' ', '0');
        field.add_subfield_str('a', "Subject 1");
        field.add_subfield_str('x', "Subdivision 1");
        field.add_subfield_str('x', "Subdivision 2");
        field.add_subfield_str('y', "Geographic 1");
        field.add_subfield_str('y', "Geographic 2");

        let dict = field.subfields_as_dict();
        assert_eq!(dict.get("a"), Some(&vec!["Subject 1".to_string()]));
        assert_eq!(
            dict.get("x"),
            Some(&vec![
                "Subdivision 1".to_string(),
                "Subdivision 2".to_string()
            ])
        );
        assert_eq!(
            dict.get("y"),
            Some(&vec![
                "Geographic 1".to_string(),
                "Geographic 2".to_string()
            ])
        );
    }

    #[test]
    fn test_field_subfields_as_dict_empty_field() {
        let field = Field::new("245".to_string(), '1', '0');
        let dict = field.subfields_as_dict();
        assert!(dict.is_empty());
    }

    #[test]
    fn test_field_subfields_as_dict_keys_sorted() {
        let mut field = Field::new("650".to_string(), ' ', '0');
        field.add_subfield_str('z', "Geographic");
        field.add_subfield_str('a', "Subject");
        field.add_subfield_str('x', "Subdivision");

        let dict = field.subfields_as_dict();
        let keys: Vec<_> = dict.keys().cloned().collect();
        // BTreeMap should keep keys sorted
        assert_eq!(
            keys,
            vec!["a".to_string(), "x".to_string(), "z".to_string()]
        );
    }

    #[test]
    fn test_field_value_simple() {
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield_str('a', "Title");
        field.add_subfield_str('b', "Subtitle");
        field.add_subfield_str('c', "Author");

        assert_eq!(field.value(), "Title Subtitle Author");
    }

    #[test]
    fn test_field_value_empty_field() {
        let field = Field::new("245".to_string(), '1', '0');
        assert_eq!(field.value(), "");
    }

    #[test]
    fn test_field_value_single_subfield() {
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield_str('a', "Title");

        assert_eq!(field.value(), "Title");
    }

    #[test]
    fn test_field_value_preserves_spaces_in_values() {
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield_str('a', "The Title");
        field.add_subfield_str('b', "With Spaces");

        assert_eq!(field.value(), "The Title With Spaces");
    }

    #[test]
    fn test_field_format_field_non_subject() {
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield_str('a', "Title");
        field.add_subfield_str('b', "Subtitle");
        field.add_subfield_str('c', "Author");

        // Non-subject fields should just have spaces between values
        assert_eq!(field.format_field(), "Title Subtitle Author");
    }

    #[test]
    fn test_field_format_field_subject_with_subdivisions() {
        let mut field = Field::new("650".to_string(), ' ', '0');
        field.add_subfield_str('a', "Subject");
        field.add_subfield_str('x', "Subdivision 1");
        field.add_subfield_str('y', "Geographic");
        field.add_subfield_str('z', "Time");

        // Subject field with subdivisions should use " -- " before v, x, y, z
        assert_eq!(
            field.format_field(),
            "Subject -- Subdivision 1 -- Geographic -- Time"
        );
    }

    #[test]
    fn test_field_format_field_subject_skips_linking() {
        let mut field = Field::new("650".to_string(), ' ', '0');
        field.add_subfield_str('a', "Subject");
        field.add_subfield_str('6', "880-01");
        field.add_subfield_str('x', "Subdivision");

        // Field linking subfield (6) should be skipped
        assert_eq!(field.format_field(), "Subject -- Subdivision");
    }

    #[test]
    fn test_field_format_field_empty() {
        let field = Field::new("245".to_string(), '1', '0');
        assert_eq!(field.format_field(), "");
    }

    #[test]
    fn test_field_format_field_only_linking() {
        let mut field = Field::new("650".to_string(), ' ', '0');
        field.add_subfield_str('6', "880-01");

        // Only linking subfield should result in empty string
        assert_eq!(field.format_field(), "");
    }

    #[test]
    fn test_field_format_field_non_subject_ignores_subdivision_markers() {
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield_str('a', "Title");
        field.add_subfield_str('x', "Something");
        field.add_subfield_str('y', "Something Else");

        // Non-subject fields don't treat x, y, z as subdivisions
        assert_eq!(field.format_field(), "Title Something Something Else");
    }

    #[test]
    fn test_field_format_field_other_subject_fields() {
        // Test that any 6xx field uses subdivision formatting
        let mut field = Field::new("651".to_string(), ' ', '0');
        field.add_subfield_str('a', "Geographic Name");
        field.add_subfield_str('x', "Subdivision");

        assert_eq!(field.format_field(), "Geographic Name -- Subdivision");
    }

    #[test]
    fn test_field_insertion_order_preserved() {
        let leader = crate::leader::Leader {
            record_length: 0,
            record_status: 'n',
            record_type: 'a',
            bibliographic_level: 'm',
            control_record_type: 'a',
            character_coding: ' ',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 0,
            encoding_level: ' ',
            cataloging_form: ' ',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        };

        let mut record = Record::new(leader);

        record.add_field(Field::new("650".to_string(), ' ', '0'));
        record.add_field(Field::new("245".to_string(), '1', '0'));
        record.add_field(Field::new("001".to_string(), ' ', ' '));
        record.add_field(Field::new("650".to_string(), ' ', '1'));

        let tags: Vec<&str> = record.fields().map(|f| f.tag.as_str()).collect();

        assert_eq!(tags, vec!["650", "650", "245", "001"]);
    }

    #[test]
    fn test_control_field_insertion_order_preserved() {
        let leader = crate::leader::Leader {
            record_length: 0,
            record_status: 'n',
            record_type: 'a',
            bibliographic_level: 'm',
            control_record_type: 'a',
            character_coding: ' ',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 0,
            encoding_level: ' ',
            cataloging_form: ' ',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        };

        let mut record = Record::new(leader);

        record.add_control_field_str("008", "Fixed length data");
        record.add_control_field_str("001", "Control number");
        record.add_control_field_str("005", "Date time");

        let tags: Vec<&str> = record.control_fields_iter().map(|(tag, _)| tag).collect();

        assert_eq!(tags, vec!["008", "001", "005"]);
    }

    #[test]
    fn test_mixed_field_insertion_order_preserved() {
        let leader = crate::leader::Leader {
            record_length: 0,
            record_status: 'n',
            record_type: 'a',
            bibliographic_level: 'm',
            control_record_type: 'a',
            character_coding: ' ',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 0,
            encoding_level: ' ',
            cataloging_form: ' ',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        };

        let mut record = Record::new(leader);

        record.add_control_field_str("001", "id1");
        record.add_field(Field::new("650".to_string(), ' ', '0'));
        record.add_field(Field::new("245".to_string(), '1', '0'));
        record.add_control_field_str("008", "fixed");
        record.add_field(Field::new("100".to_string(), '1', ' '));

        let control_tags: Vec<&str> = record.control_fields_iter().map(|(tag, _)| tag).collect();
        assert_eq!(control_tags, vec!["001", "008"]);

        let field_tags: Vec<&str> = record.fields().map(|f| f.tag.as_str()).collect();
        assert_eq!(field_tags, vec!["650", "245", "100"]);
    }
}
