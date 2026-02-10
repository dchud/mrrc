//! Format-specific query traits for different MARC record types.
//!
//! This module provides specialized query traits for bibliographic, authority,
//! and holdings records, enabling domain-specific helper methods tailored to
//! each record type's purpose.

use crate::authority_record::AuthorityRecord;
use crate::holdings_record::HoldingsRecord;
use crate::record::{Field, Record};

/// Query helpers specific to bibliographic records.
///
/// This trait provides methods for navigating and querying bibliographic-specific
/// fields like titles, authors, subjects, and linked field information.
pub trait BibliographicQueries {
    /// Get all main titles and their statement of responsibility (245 fields).
    ///
    /// In MARC, there is typically only one 245 field, but this method returns
    /// any additional title fields that may be present.
    ///
    /// # Returns
    ///
    /// Vector of title fields (245) with their indicators and subfields.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let record = Record::new(leader);
    /// for title_field in record.get_titles() {
    ///     if let Some(title) = title_field.get_subfield('a') {
    ///         println!("Title: {}", title);
    ///     }
    /// }
    /// ```
    #[must_use]
    fn get_titles(&self) -> Vec<&Field>;

    /// Get all subject fields (6XX range).
    ///
    /// Returns all subject authority fields including:
    /// - 600: Personal name subject
    /// - 610: Corporate name subject
    /// - 611: Meeting name subject
    /// - 630: Uniform title subject
    /// - 648: Chronological term subject
    /// - 650: Topical term subject
    /// - 651: Geographic name subject
    /// - 655: Genre/form term
    ///
    /// # Returns
    ///
    /// Vector of all subject fields in the record.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let record = Record::new(leader);
    /// for subject in record.get_all_subjects() {
    ///     if let Some(label) = subject.get_subfield('a') {
    ///         println!("Subject: {}", label);
    ///     }
    /// }
    /// ```
    #[must_use]
    fn get_all_subjects(&self) -> Vec<&Field>;

    /// Get all topical subjects (650 fields).
    ///
    /// Topical subjects describe the main subject content of the work.
    ///
    /// # Returns
    ///
    /// Vector of all 650 fields in the record.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let record = Record::new(leader);
    /// for subject in record.get_topical_subjects() {
    ///     if let Some(term) = subject.get_subfield('a') {
    ///         println!("Topic: {}", term);
    ///     }
    /// }
    /// ```
    #[must_use]
    fn get_topical_subjects(&self) -> Vec<&Field>;

    /// Get all geographic subjects (651 fields).
    ///
    /// Geographic subjects describe places discussed or depicted in the work.
    ///
    /// # Returns
    ///
    /// Vector of all 651 fields in the record.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let record = Record::new(leader);
    /// for subject in record.get_geographic_subjects() {
    ///     if let Some(place) = subject.get_subfield('a') {
    ///         println!("Place: {}", place);
    ///     }
    /// }
    /// ```
    #[must_use]
    fn get_geographic_subjects(&self) -> Vec<&Field>;

    /// Get all name fields (1XX, 6XX name fields, 7XX).
    ///
    /// Returns all name-based fields including:
    /// - 100: Main entry—personal name
    /// - 600: Subject added entry—personal name
    /// - 610: Subject added entry—corporate name
    /// - 611: Subject added entry—meeting name
    /// - 700: Added entry—personal name
    /// - 710: Added entry—corporate name
    /// - 711: Added entry—meeting name
    ///
    /// # Returns
    ///
    /// Vector of all name fields in the record.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let record = Record::new(leader);
    /// for name in record.get_all_names() {
    ///     if let Some(label) = name.get_subfield('a') {
    ///         println!("Name: {}", label);
    ///     }
    /// }
    /// ```
    #[must_use]
    fn get_all_names(&self) -> Vec<&Field>;

    /// Get linked field pairs (original field with optional 880 counterpart).
    ///
    /// For a specified tag, returns tuples of the original field paired with its
    /// linked 880 (alternate graphical representation) if one exists.
    ///
    /// # Arguments
    ///
    /// * `tag` - The original field tag (e.g., "100", "245", "650")
    ///
    /// # Returns
    ///
    /// Vector of (`original_field`, Option<`880_field`>) tuples.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let record = Record::new(leader);
    /// for (original, linked_880) in record.get_linked_field_pairs("100") {
    ///     if let Some(name) = original.get_subfield('a') {
    ///         println!("Name: {}", name);
    ///         if let Some(field_880) = linked_880 {
    ///             if let Some(alt) = field_880.get_subfield('a') {
    ///                 println!("  Alternate: {}", alt);
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[must_use]
    fn get_linked_field_pairs(&self, tag: &str) -> Vec<(&Field, Option<&Field>)>;

    /// Get all 880 fields (alternate graphical representations).
    ///
    /// These fields provide alternate script forms (romanized, vernacular, etc.)
    /// linked to original fields via subfield 6 linkage.
    ///
    /// # Returns
    ///
    /// Vector of all 880 fields in the record.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let record = Record::new(leader);
    /// for field_880 in record.get_all_880_fields() {
    ///     println!("Alternate graphical: {:?}", field_880);
    /// }
    /// ```
    #[must_use]
    fn get_all_880_fields(&self) -> Vec<&Field>;
}

impl BibliographicQueries for Record {
    fn get_titles(&self) -> Vec<&Field> {
        self.get_fields("245")
            .map(|f| f.iter().collect())
            .unwrap_or_default()
    }

    fn get_all_subjects(&self) -> Vec<&Field> {
        let mut results = Vec::new();
        for tag in &["600", "610", "611", "630", "648", "650", "651", "655"] {
            if let Some(fields) = self.get_fields(tag) {
                results.extend(fields.iter());
            }
        }
        results
    }

    fn get_topical_subjects(&self) -> Vec<&Field> {
        self.get_fields("650")
            .map(|f| f.iter().collect())
            .unwrap_or_default()
    }

    fn get_geographic_subjects(&self) -> Vec<&Field> {
        self.get_fields("651")
            .map(|f| f.iter().collect())
            .unwrap_or_default()
    }

    fn get_all_names(&self) -> Vec<&Field> {
        let mut results = Vec::new();
        for tag in &["100", "600", "610", "611", "700", "710", "711"] {
            if let Some(fields) = self.get_fields(tag) {
                results.extend(fields.iter());
            }
        }
        results
    }

    fn get_linked_field_pairs(&self, tag: &str) -> Vec<(&Field, Option<&Field>)> {
        self.get_field_pairs(tag)
    }

    fn get_all_880_fields(&self) -> Vec<&Field> {
        self.get_all_880_fields()
    }
}

/// Query helpers specific to authority records.
///
/// This trait provides methods for navigating authority-specific fields
/// like see-from tracings, see-also fields, and relationship information.
pub trait AuthoritySpecificQueries {
    /// Get the preferred heading from the record.
    ///
    /// Authority records have one main heading (1XX field) that represents
    /// the established form of the term.
    ///
    /// # Returns
    ///
    /// The preferred heading field, or `None` if not found.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let record = AuthorityRecord::new(leader);
    /// if let Some(heading) = record.get_preferred_heading() {
    ///     if let Some(label) = heading.get_subfield('a') {
    ///         println!("Preferred heading: {}", label);
    ///     }
    /// }
    /// ```
    #[must_use]
    fn get_preferred_heading(&self) -> Option<&Field>;

    /// Get all variant forms (4XX fields) for this heading.
    ///
    /// These are non-preferred forms that users might search for,
    /// with instructions to "see" the preferred heading instead.
    ///
    /// # Returns
    ///
    /// Vector of all 4XX fields in the record.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let record = AuthorityRecord::new(leader);
    /// for variant in record.get_variant_headings() {
    ///     if let Some(label) = variant.get_subfield('a') {
    ///         println!("Variant: {}", label);
    ///     }
    /// }
    /// ```
    #[must_use]
    fn get_variant_headings(&self) -> Vec<&Field>;

    /// Get all related headings (5XX fields) for this term.
    ///
    /// These are established headings that are related but not synonymous,
    /// with instructions to "see also" these related terms.
    ///
    /// # Returns
    ///
    /// Vector of all 5XX fields in the record.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let record = AuthorityRecord::new(leader);
    /// for related in record.get_broader_related_headings() {
    ///     if let Some(label) = related.get_subfield('a') {
    ///         println!("Related: {}", label);
    ///     }
    /// }
    /// ```
    #[must_use]
    fn get_broader_related_headings(&self) -> Vec<&Field>;

    /// Get the scope note from the record.
    ///
    /// Scope notes (field 680) explain the meaning and usage of the heading.
    ///
    /// # Returns
    ///
    /// The scope note text, or `None` if not found.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let record = AuthorityRecord::new(leader);
    /// if let Some(note) = record.get_scope_note() {
    ///     println!("Scope: {}", note);
    /// }
    /// ```
    #[must_use]
    fn get_scope_note(&self) -> Option<&str>;
}

impl AuthoritySpecificQueries for AuthorityRecord {
    fn get_preferred_heading(&self) -> Option<&Field> {
        self.heading()
    }

    fn get_variant_headings(&self) -> Vec<&Field> {
        self.see_from_tracings()
    }

    fn get_broader_related_headings(&self) -> Vec<&Field> {
        self.see_also_tracings()
    }

    fn get_scope_note(&self) -> Option<&str> {
        self.get_fields("680")
            .and_then(|fields| fields.first())
            .and_then(|f| f.get_subfield('a'))
    }
}

/// Query helpers specific to holdings records.
///
/// This trait provides methods for navigating holdings-specific information
/// like item locations, call numbers, and holdings notes.
pub trait HoldingsSpecificQueries {
    /// Get the call number from field 050 or 090.
    ///
    /// Call numbers are used to organize and locate items in a library.
    /// Tries field 090 (local call number) first, then 050 (LC call number).
    ///
    /// # Returns
    ///
    /// The call number, or `None` if not found.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let record = HoldingsRecord::new(leader);
    /// if let Some(call_num) = record.get_call_number() {
    ///     println!("Call number: {}", call_num);
    /// }
    /// ```
    #[must_use]
    fn get_call_number(&self) -> Option<&str>;

    /// Get the holding location from field 852 (Location).
    ///
    /// Specifies the physical location or repository of the item.
    ///
    /// # Returns
    ///
    /// The location text, or `None` if not found.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let record = HoldingsRecord::new(leader);
    /// if let Some(location) = record.get_holding_location() {
    ///     println!("Location: {}", location);
    /// }
    /// ```
    #[must_use]
    fn get_holding_location(&self) -> Option<&str>;

    /// Get all notes on holdings (5XX fields).
    ///
    /// General notes that apply to the holdings information.
    ///
    /// # Returns
    ///
    /// Vector of all note text values from 5XX fields.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let record = HoldingsRecord::new(leader);
    /// for note in record.get_holding_notes() {
    ///     println!("Note: {}", note);
    /// }
    /// ```
    #[must_use]
    fn get_holding_notes(&self) -> Vec<&str>;
}

impl HoldingsSpecificQueries for HoldingsRecord {
    fn get_call_number(&self) -> Option<&str> {
        // Try local call number first (090), then LC call number (050)
        self.get_fields("090")
            .and_then(|f| f.first())
            .and_then(|f| f.get_subfield('a'))
            .or_else(|| {
                self.get_fields("050")
                    .and_then(|f| f.first())
                    .and_then(|f| f.get_subfield('a'))
            })
    }

    fn get_holding_location(&self) -> Option<&str> {
        self.get_fields("852")
            .and_then(|f| f.first())
            .and_then(|f| f.get_subfield('b'))
    }

    fn get_holding_notes(&self) -> Vec<&str> {
        let mut notes = Vec::new();
        for tag in &[
            "500", "501", "502", "503", "504", "505", "506", "507", "508", "509",
        ] {
            if let Some(fields) = self.get_fields(tag) {
                for field in fields {
                    if let Some(note) = field.get_subfield('a') {
                        notes.push(note);
                    }
                }
            }
        }
        notes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::leader::Leader;
    use crate::record::Subfield;

    fn make_bib_leader() -> Leader {
        Leader {
            record_length: 1000,
            record_status: 'a',
            record_type: 'a', // Bibliographic record
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

    fn make_auth_leader() -> Leader {
        Leader {
            record_length: 1000,
            record_status: 'a',
            record_type: 'z', // Authority record
            bibliographic_level: ' ',
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

    fn make_hold_leader() -> Leader {
        Leader {
            record_length: 1000,
            record_status: 'a',
            record_type: 'x', // Holdings record
            bibliographic_level: ' ',
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
    fn test_bibliographic_get_titles() {
        let mut record = Record::new(make_bib_leader());
        let mut title_field = Field::new("245".to_string(), '1', '0');
        title_field.subfields.push(Subfield {
            code: 'a',
            value: "Test title".to_string(),
        });
        record.add_field(title_field);

        let titles = record.get_titles();
        assert_eq!(titles.len(), 1);
        assert_eq!(titles[0].tag, "245");
    }

    #[test]
    fn test_bibliographic_get_all_subjects() {
        let mut record = Record::new(make_bib_leader());
        let mut subject1 = Field::new("650".to_string(), ' ', '0');
        subject1.subfields.push(Subfield {
            code: 'a',
            value: "Topic".to_string(),
        });
        record.add_field(subject1);

        let mut subject2 = Field::new("651".to_string(), ' ', '0');
        subject2.subfields.push(Subfield {
            code: 'a',
            value: "Place".to_string(),
        });
        record.add_field(subject2);

        let subjects = record.get_all_subjects();
        assert_eq!(subjects.len(), 2);
    }

    #[test]
    fn test_bibliographic_get_all_names() {
        let mut record = Record::new(make_bib_leader());
        let mut name1 = Field::new("100".to_string(), '1', ' ');
        name1.subfields.push(Subfield {
            code: 'a',
            value: "Author".to_string(),
        });
        record.add_field(name1);

        let mut name2 = Field::new("700".to_string(), '1', ' ');
        name2.subfields.push(Subfield {
            code: 'a',
            value: "Contributor".to_string(),
        });
        record.add_field(name2);

        let names = record.get_all_names();
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn test_authority_get_preferred_heading() {
        let mut record = AuthorityRecord::new(make_auth_leader());
        let mut heading = Field::new("150".to_string(), ' ', ' ');
        heading.subfields.push(Subfield {
            code: 'a',
            value: "Computer science".to_string(),
        });
        record.set_heading(heading);

        assert!(record.get_preferred_heading().is_some());
    }

    #[test]
    fn test_authority_get_variant_headings() {
        let mut record = AuthorityRecord::new(make_auth_leader());
        let mut variant = Field::new("450".to_string(), ' ', ' ');
        variant.subfields.push(Subfield {
            code: 'a',
            value: "Computing".to_string(),
        });
        record.add_see_from_tracing(variant);

        let variants = record.get_variant_headings();
        assert_eq!(variants.len(), 1);
    }

    #[test]
    fn test_holdings_get_call_number() {
        let mut record = HoldingsRecord::new(make_hold_leader());
        let mut call_field = Field::new("090".to_string(), ' ', ' ');
        call_field.subfields.push(Subfield {
            code: 'a',
            value: "QA76.9.D3".to_string(),
        });
        record.add_field(call_field);

        assert_eq!(record.get_call_number(), Some("QA76.9.D3"));
    }

    #[test]
    fn test_holdings_get_holding_location() {
        let mut record = HoldingsRecord::new(make_hold_leader());
        let mut loc_field = Field::new("852".to_string(), ' ', ' ');
        loc_field.subfields.push(Subfield {
            code: 'b',
            value: "Main Library".to_string(),
        });
        record.add_field(loc_field);

        assert_eq!(record.get_holding_location(), Some("Main Library"));
    }
}
