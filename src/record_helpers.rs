//! Helper methods for accessing common bibliographic fields.
//!
//! This module provides the `RecordHelpers` trait, which adds convenient methods
//! for accessing frequently-used MARC fields. The trait is automatically implemented
//! for all types that implement `MarcRecord`, making these methods available on
//! bibliographic records, authority records, and holdings records.
//!
//! # Examples
//!
//! ```ignore
//! use mrrc::{Record, RecordHelpers};
//!
//! let record = Record::new(leader);
//! if let Some(title) = record.title() {
//!     println!("Title: {}", title);
//! }
//! ```

use crate::bibliographic_helpers::PublicationInfo;
use crate::marc_record::MarcRecord;

/// MARC 6XX subject tags matching pymarc's `subjects()` coverage.
///
/// Includes standard subject fields (600-662) and local subject fields (690-699)
/// that appear frequently in OCLC and RLIN records.
pub const SUBJECT_TAGS: &[&str] = &[
    "600", "610", "611", "630", "648", "650", "651", "653", "654", "655", "656", "657", "658",
    "662", "690", "691", "696", "697", "698", "699",
];

/// Extension trait providing convenient helper methods for MARC records.
///
/// This trait is automatically implemented for all types that implement `MarcRecord`,
/// providing access to common bibliographic fields and metadata without needing
/// to manually navigate the field structure.
pub trait RecordHelpers: MarcRecord {
    /// Get the main title from field 245, subfield 'a'
    ///
    /// # Examples
    /// ```ignore
    /// if let Some(title) = record.title() {
    ///     println!("Title: {}", title);
    /// }
    /// ```
    #[must_use]
    fn title(&self) -> Option<&str> {
        self.get_field("245").and_then(|f| f.get_subfield('a'))
    }

    /// Get the title and statement of responsibility from field 245
    ///
    /// Returns a tuple of (title, `statement_of_responsibility`) if available.
    /// Title comes from subfield 'a', responsibility from subfield 'c'.
    #[must_use]
    fn title_with_responsibility(&self) -> (Option<&str>, Option<&str>) {
        match self.get_field("245") {
            Some(field) => (field.get_subfield('a'), field.get_subfield('c')),
            None => (None, None),
        }
    }

    /// Get the primary author from field 100 (personal name), subfield 'a'
    ///
    /// Returns the first author found. Use `authors()` to get all authors.
    #[must_use]
    fn author(&self) -> Option<&str> {
        self.get_field("100").and_then(|f| f.get_subfield('a'))
    }

    /// Get all authors from field 700 (added entry for personal name), subfield 'a'
    ///
    /// This includes secondary authors/contributors. For the primary author, use `author()`.
    #[must_use]
    fn authors(&self) -> Vec<&str> {
        self.get_fields("700")
            .map(|fields| fields.iter().filter_map(|f| f.get_subfield('a')).collect())
            .unwrap_or_default()
    }

    /// Get the corporate body (publisher or organization) from field 110, subfield 'a'
    #[must_use]
    fn corporate_author(&self) -> Option<&str> {
        self.get_field("110").and_then(|f| f.get_subfield('a'))
    }

    /// Get the publisher from field 260 or 264 (RDA), subfield 'b'
    ///
    /// Checks field 260$b first. If absent, falls back to field 264 with
    /// indicator2='1' (publication) to support RDA-cataloged records.
    #[must_use]
    fn publisher(&self) -> Option<&str> {
        self.get_field("260")
            .and_then(|f| f.get_subfield('b'))
            .or_else(|| {
                self.get_fields("264").and_then(|fields| {
                    fields
                        .iter()
                        .find(|f| f.indicator2 == '1')
                        .and_then(|f| f.get_subfield('b'))
                })
            })
    }

    /// Get the publication date from field 260 or 264 (RDA), subfield 'c'
    ///
    /// Checks field 260$c first, then 264 (ind2='1') $c, then falls back to
    /// the publication year extracted from field 008 (positions 7-10).
    #[must_use]
    fn publication_date(&self) -> Option<&str> {
        self.get_field("260")
            .and_then(|f| f.get_subfield('c'))
            .or_else(|| {
                self.get_fields("264").and_then(|fields| {
                    fields
                        .iter()
                        .find(|f| f.indicator2 == '1')
                        .and_then(|f| f.get_subfield('c'))
                })
            })
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
    fn isbn(&self) -> Option<&str> {
        self.get_field("020").and_then(|f| f.get_subfield('a'))
    }

    /// Get all ISBNs from field 020, subfield 'a'
    #[must_use]
    fn isbns(&self) -> Vec<&str> {
        self.get_fields("020")
            .map(|fields| fields.iter().filter_map(|f| f.get_subfield('a')).collect())
            .unwrap_or_default()
    }

    /// Get the ISSN from field 022, subfield 'a'
    #[must_use]
    fn issn(&self) -> Option<&str> {
        self.get_field("022").and_then(|f| f.get_subfield('a'))
    }

    /// Get all subject headings from 6XX fields, subfield 'a'
    ///
    /// Covers the same tags as pymarc's `subjects()`: 600, 610, 611, 630, 648,
    /// 650, 651, 653, 654, 655, 656, 657, 658, 662, 690, 691, 696, 697, 698, 699.
    #[must_use]
    fn subjects(&self) -> Vec<&str> {
        let mut result = Vec::new();
        for tag in SUBJECT_TAGS {
            if let Some(fields) = self.get_fields(tag) {
                for field in fields {
                    if let Some(val) = field.get_subfield('a') {
                        result.push(val);
                    }
                }
            }
        }
        result
    }

    /// Get the language code from field 008 (positions 35-37)
    ///
    /// Returns a 3-character language code (e.g., "eng" for English).
    #[must_use]
    fn language(&self) -> Option<&str> {
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
    fn control_number(&self) -> Option<&str> {
        self.get_control_field("001")
    }

    /// Get the Library of Congress Control Number (LCCN) from field 010, subfield 'a'
    #[must_use]
    fn lccn(&self) -> Option<&str> {
        self.get_field("010").and_then(|f| f.get_subfield('a'))
    }

    /// Get the physical description from field 300, subfield 'a'
    ///
    /// Typically describes the extent of the resource (e.g., "256 pages").
    #[must_use]
    fn physical_description(&self) -> Option<&str> {
        self.get_field("300").and_then(|f| f.get_subfield('a'))
    }

    /// Get the series statement from field 490, subfield 'a'
    #[must_use]
    fn series(&self) -> Option<&str> {
        self.get_field("490").and_then(|f| f.get_subfield('a'))
    }

    /// Check if this is a book (leader type 'a' for language material and bib level 'm' for monograph)
    #[must_use]
    fn is_book(&self) -> bool {
        self.leader().record_type == 'a' && self.leader().bibliographic_level == 'm'
    }

    /// Check if this is a serial (bib level 's')
    #[must_use]
    fn is_serial(&self) -> bool {
        self.leader().bibliographic_level == 's'
    }

    /// Check if this is music (leader type 'c' or 'd')
    #[must_use]
    fn is_music(&self) -> bool {
        matches!(self.leader().record_type, 'c' | 'd')
    }

    /// Check if this is audiovisual material (leader type 'g')
    #[must_use]
    fn is_audiovisual(&self) -> bool {
        self.leader().record_type == 'g'
    }

    /// Extract publication information from field 260 or 264 (RDA)
    ///
    /// Checks field 260 first. If absent, falls back to field 264 with
    /// indicator2='1' (publication) to support RDA-cataloged records.
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
    fn publication_info(&self) -> Option<PublicationInfo> {
        let field = self.get_field("260").or_else(|| {
            self.get_fields("264")
                .and_then(|fields| fields.iter().find(|f| f.indicator2 == '1'))
        });
        field.map(|f| {
            PublicationInfo::new(
                f.get_subfield('a').map(ToString::to_string),
                f.get_subfield('b').map(ToString::to_string),
                f.get_subfield('c').map(ToString::to_string),
            )
        })
    }

    /// Get the publication year extracted from field 260$c or field 008
    ///
    /// Attempts to extract a 4-digit year from the publication date statement.
    /// Falls back to field 008 (positions 7-10) if field 260 is not available.
    #[must_use]
    fn publication_year(&self) -> Option<u32> {
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

    /// Get the place of publication from field 260 or 264 (RDA), subfield 'a'
    ///
    /// Checks field 260$a first. If absent, falls back to field 264 with
    /// indicator2='1' (publication) to support RDA-cataloged records.
    #[must_use]
    fn place_of_publication(&self) -> Option<&str> {
        self.get_field("260")
            .and_then(|f| f.get_subfield('a'))
            .or_else(|| {
                self.get_fields("264").and_then(|fields| {
                    fields
                        .iter()
                        .find(|f| f.indicator2 == '1')
                        .and_then(|f| f.get_subfield('a'))
                })
            })
    }

    /// Get all location fields (field 852)
    ///
    /// Returns a list of location fields, which contain institution-specific
    /// shelving locations and call numbers.
    #[must_use]
    fn location(&self) -> Vec<&str> {
        self.get_fields("852")
            .map(|fields| fields.iter().filter_map(|f| f.get_subfield('a')).collect())
            .unwrap_or_default()
    }

    /// Get all note fields (all 5xx fields)
    ///
    /// Returns a vector of all general note, bibliography, etc. fields.
    /// This includes fields 500-599.
    #[must_use]
    fn notes(&self) -> Vec<&str> {
        let mut result = Vec::new();
        for tag_num in 500..=599 {
            let tag = format!("{tag_num:03}");
            if let Some(fields) = self.get_fields(&tag) {
                for field in fields {
                    if let Some(note) = field.get_subfield('a') {
                        result.push(note);
                    }
                }
            }
        }
        result
    }

    /// Get the uniform title from field 130, subfield 'a'
    ///
    /// The uniform title is a standardized form of the title used for cataloging.
    #[must_use]
    fn uniform_title(&self) -> Option<&str> {
        self.get_field("130").and_then(|f| f.get_subfield('a'))
    }

    /// Get the government document classification from field 086, subfield 'a'
    ///
    /// Also known as `SuDoc` (Superintendent of Documents) number.
    #[must_use]
    fn sudoc(&self) -> Option<&str> {
        self.get_field("086").and_then(|f| f.get_subfield('a'))
    }

    /// Get the key title (ISSN title) from field 222
    ///
    /// Returns the key title from subfield 'a', optionally with the remainder
    /// from subfield 'b' if present.
    #[must_use]
    fn issn_title(&self) -> Option<&str> {
        self.get_field("222").and_then(|f| f.get_subfield('a'))
    }

    /// Get the ISSN-L (ISSN Linking number) from field 024, subfield 'a'
    ///
    /// The ISSN-L is a standardized identifier that links all versions of a serial.
    #[must_use]
    fn issnl(&self) -> Option<&str> {
        self.get_field("024").and_then(|f| f.get_subfield('a'))
    }

    /// Alias for `publication_year()` for pymarc compatibility
    ///
    /// Returns the publication year as extracted from field 260$c or field 008.
    #[must_use]
    fn pubyear(&self) -> Option<u32> {
        self.publication_year()
    }
}

// Implement RecordHelpers for all types that implement MarcRecord
impl<T: MarcRecord + ?Sized> RecordHelpers for T {}

#[cfg(test)]
mod tests {
    use crate::leader::Leader;
    use crate::record::{Field, Record, Subfield};
    #[allow(unused_imports)]
    use crate::record_helpers::RecordHelpers;

    fn create_test_record() -> Record {
        let mut record = Record::new(Leader {
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
        });

        record.add_control_field("001".to_string(), "12345".to_string());

        let mut title_field = Field::new("245".to_string(), '1', '0');
        title_field.subfields.push(Subfield {
            code: 'a',
            value: "Test Title".to_string(),
        });
        record.add_field(title_field);

        record
    }

    #[test]
    fn test_trait_title() {
        let record = create_test_record();
        assert_eq!(record.title(), Some("Test Title"));
    }

    #[test]
    fn test_trait_control_number() {
        let record = create_test_record();
        assert_eq!(record.control_number(), Some("12345"));
    }

    #[test]
    fn test_trait_is_book() {
        let record = create_test_record();
        assert!(record.is_book());
    }

    #[test]
    fn test_trait_multiple_methods() {
        let record = create_test_record();
        assert_eq!(record.title(), Some("Test Title"));
        assert_eq!(record.control_number(), Some("12345"));
        assert!(record.is_book());
        assert!(!record.is_serial());
    }

    #[test]
    fn test_trait_location() {
        let mut record = create_test_record();
        let mut location_field = Field::new("852".to_string(), ' ', ' ');
        location_field.subfields.push(Subfield {
            code: 'a',
            value: "Main Library".to_string(),
        });
        record.add_field(location_field);

        let locations = record.location();
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0], "Main Library");
    }

    #[test]
    fn test_trait_notes() {
        let mut record = create_test_record();
        let mut notes_field = Field::new("500".to_string(), ' ', ' ');
        notes_field.subfields.push(Subfield {
            code: 'a',
            value: "General note".to_string(),
        });
        record.add_field(notes_field);

        let notes = record.notes();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0], "General note");
    }

    #[test]
    fn test_trait_pubyear_alias() {
        let record = create_test_record();
        // Since our test record doesn't have publication info, this should return None
        assert_eq!(record.pubyear(), None);
    }

    #[test]
    fn test_trait_uniform_title() {
        let mut record = create_test_record();
        let mut uniform_title_field = Field::new("130".to_string(), ' ', ' ');
        uniform_title_field.subfields.push(Subfield {
            code: 'a',
            value: "Standardized Title".to_string(),
        });
        record.add_field(uniform_title_field);

        assert_eq!(record.uniform_title(), Some("Standardized Title"));
    }

    #[test]
    fn test_trait_sudoc() {
        let mut record = create_test_record();
        let mut sudoc_field = Field::new("086".to_string(), ' ', ' ');
        sudoc_field.subfields.push(Subfield {
            code: 'a',
            value: "I 19.2:En 3".to_string(),
        });
        record.add_field(sudoc_field);

        assert_eq!(record.sudoc(), Some("I 19.2:En 3"));
    }

    #[test]
    fn test_trait_issn_title() {
        let mut record = create_test_record();
        let mut issn_title_field = Field::new("222".to_string(), ' ', ' ');
        issn_title_field.subfields.push(Subfield {
            code: 'a',
            value: "Key Title".to_string(),
        });
        record.add_field(issn_title_field);

        assert_eq!(record.issn_title(), Some("Key Title"));
    }

    #[test]
    fn test_trait_issnl() {
        let mut record = create_test_record();
        let mut issnl_field = Field::new("024".to_string(), ' ', ' ');
        issnl_field.subfields.push(Subfield {
            code: 'a',
            value: "1234-5678".to_string(),
        });
        record.add_field(issnl_field);

        assert_eq!(record.issnl(), Some("1234-5678"));
    }

    #[test]
    fn test_publisher_from_260() {
        let mut record = create_test_record();
        let mut f260 = Field::new("260".to_string(), ' ', ' ');
        f260.subfields.push(Subfield {
            code: 'b',
            value: "Addison-Wesley,".to_string(),
        });
        record.add_field(f260);

        assert_eq!(record.publisher(), Some("Addison-Wesley,"));
    }

    #[test]
    fn test_publisher_from_264_rda() {
        let mut record = create_test_record();
        // 264 with ind2='1' (publication)
        let mut f264 = Field::new("264".to_string(), ' ', '1');
        f264.subfields.push(Subfield {
            code: 'b',
            value: "The MIT Press,".to_string(),
        });
        record.add_field(f264);

        assert_eq!(record.publisher(), Some("The MIT Press,"));
    }

    #[test]
    fn test_publisher_prefers_260_over_264() {
        let mut record = create_test_record();
        let mut f260 = Field::new("260".to_string(), ' ', ' ');
        f260.subfields.push(Subfield {
            code: 'b',
            value: "Old Publisher,".to_string(),
        });
        record.add_field(f260);

        let mut f264 = Field::new("264".to_string(), ' ', '1');
        f264.subfields.push(Subfield {
            code: 'b',
            value: "New Publisher,".to_string(),
        });
        record.add_field(f264);

        assert_eq!(record.publisher(), Some("Old Publisher,"));
    }

    #[test]
    fn test_publisher_ignores_264_non_publication() {
        let mut record = create_test_record();
        // 264 with ind2='3' (manufacture), should be ignored
        let mut f264_mfg = Field::new("264".to_string(), ' ', '3');
        f264_mfg.subfields.push(Subfield {
            code: 'b',
            value: "Some Printer,".to_string(),
        });
        record.add_field(f264_mfg);

        assert_eq!(record.publisher(), None);
    }

    #[test]
    fn test_place_of_publication_from_264_rda() {
        let mut record = create_test_record();
        let mut f264 = Field::new("264".to_string(), ' ', '1');
        f264.subfields.push(Subfield {
            code: 'a',
            value: "Cambridge, Massachusetts :".to_string(),
        });
        record.add_field(f264);

        assert_eq!(
            record.place_of_publication(),
            Some("Cambridge, Massachusetts :")
        );
    }

    #[test]
    fn test_publication_date_from_264_rda() {
        let mut record = create_test_record();
        let mut f264 = Field::new("264".to_string(), ' ', '1');
        f264.subfields.push(Subfield {
            code: 'c',
            value: "[2022]".to_string(),
        });
        record.add_field(f264);

        assert_eq!(record.publication_date(), Some("[2022]"));
    }

    #[test]
    fn test_publication_info_from_264_rda() {
        let mut record = create_test_record();
        let mut f264 = Field::new("264".to_string(), ' ', '1');
        f264.subfields.push(Subfield {
            code: 'a',
            value: "Cambridge, Massachusetts :".to_string(),
        });
        f264.subfields.push(Subfield {
            code: 'b',
            value: "The MIT Press,".to_string(),
        });
        f264.subfields.push(Subfield {
            code: 'c',
            value: "[2022]".to_string(),
        });
        record.add_field(f264);

        let info = record.publication_info().unwrap();
        assert_eq!(info.place, Some("Cambridge, Massachusetts :".to_string()));
        assert_eq!(info.publisher, Some("The MIT Press,".to_string()));
        assert_eq!(info.date, Some("[2022]".to_string()));
        assert_eq!(info.publication_year(), Some(2022));
    }

    #[test]
    fn test_publication_year_from_264_rda() {
        let mut record = create_test_record();
        let mut f264 = Field::new("264".to_string(), ' ', '1');
        f264.subfields.push(Subfield {
            code: 'c',
            value: "[2022]".to_string(),
        });
        record.add_field(f264);

        assert_eq!(record.publication_year(), Some(2022));
    }

    #[test]
    fn test_trait_subjects_all_6xx() {
        let mut record = create_test_record();

        // 600 — Personal Name Subject
        let mut f600 = Field::new("600".to_string(), '1', '0');
        f600.subfields.push(Subfield {
            code: 'a',
            value: "Maimonides, Moses,".to_string(),
        });
        record.add_field(f600);

        // 650 — Topical Term
        let mut f650 = Field::new("650".to_string(), ' ', '0');
        f650.subfields.push(Subfield {
            code: 'a',
            value: "Jewish law.".to_string(),
        });
        record.add_field(f650);

        // 655 — Genre/Form
        let mut f655 = Field::new("655".to_string(), ' ', '7');
        f655.subfields.push(Subfield {
            code: 'a',
            value: "Commentaries.".to_string(),
        });
        record.add_field(f655);

        // 690 — Local Subject
        let mut f690 = Field::new("690".to_string(), ' ', ' ');
        f690.subfields.push(Subfield {
            code: 'a',
            value: "Local topic".to_string(),
        });
        record.add_field(f690);

        let subjects = record.subjects();
        assert_eq!(subjects.len(), 4);
        assert!(subjects.contains(&"Maimonides, Moses,"));
        assert!(subjects.contains(&"Jewish law."));
        assert!(subjects.contains(&"Commentaries."));
        assert!(subjects.contains(&"Local topic"));
    }
}
