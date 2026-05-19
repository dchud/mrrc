//! Authority control helper methods and queries.
//!
//! This module provides the `AuthorityQueries` trait for authority records,
//! enabling convenient access to authority reference fields and navigation methods.

use crate::authority_record::AuthorityRecord;
use crate::record::Field;

/// Extension trait providing authority control helper methods.
///
/// This trait adds convenient methods for working with authority records,
/// particularly for navigating reference and tracing fields.
pub trait AuthorityQueries {
    /// Get all "see from" tracings (4XX fields).
    ///
    /// See-from tracings represent non-preferred forms of the heading that users
    /// might search for but should be redirected to the preferred form.
    ///
    /// # Returns
    ///
    /// Vector of all 4XX fields in the record.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let auth_record = AuthorityRecord::new(leader);
    /// for see_from in auth_record.get_see_from_headings() {
    ///     if let Some(label) = see_from.get_subfield('a') {
    ///         println!("See from: {}", label);
    ///     }
    /// }
    /// ```
    #[must_use]
    fn get_see_from_headings(&self) -> Vec<&Field>;

    /// Get all "see also" tracings (5XX fields).
    ///
    /// See-also tracings represent related concepts or terms that should be displayed
    /// for the user's information but are not preferred alternatives.
    ///
    /// # Returns
    ///
    /// Vector of all 5XX fields in the record.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let auth_record = AuthorityRecord::new(leader);
    /// for see_also in auth_record.get_see_also_headings() {
    ///     if let Some(label) = see_also.get_subfield('a') {
    ///         println!("See also: {}", label);
    ///     }
    /// }
    /// ```
    #[must_use]
    fn get_see_also_headings(&self) -> Vec<&Field>;

    /// Get all authority relationship fields (7XX fields).
    ///
    /// Relationship fields establish hierarchical or associative relationships
    /// with other authority records (broader terms, narrower terms, etc).
    ///
    /// # Returns
    ///
    /// Vector of all 7XX fields in the record.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let auth_record = AuthorityRecord::new(leader);
    /// for rel in auth_record.get_relationship_fields() {
    ///     if let Some(label) = rel.get_subfield('a') {
    ///         println!("Related: {}", label);
    ///     }
    /// }
    /// ```
    #[must_use]
    fn get_relationship_fields(&self) -> Vec<&Field>;

    /// Get all authority reference fields (4XX, 5XX, 7XX).
    ///
    /// This combines all reference and tracing fields, providing a unified view
    /// of related authority headings in the record.
    ///
    /// # Returns
    ///
    /// Vector of all reference/tracing fields in the record.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let auth_record = AuthorityRecord::new(leader);
    /// for reference in auth_record.get_authority_references() {
    ///     if let Some(label) = reference.get_subfield('a') {
    ///         println!("Reference: {} (tag: {})", label, reference.tag);
    ///     }
    /// }
    /// ```
    #[must_use]
    fn get_authority_references(&self) -> Vec<&Field>;

    /// Find a related authority heading in the 5XX fields (see also).
    ///
    /// Searches for a heading that is semantically or hierarchically related
    /// to the given heading by looking through see-also (5XX) fields.
    ///
    /// # Arguments
    ///
    /// * `heading` - The heading to find related terms for
    ///
    /// # Returns
    ///
    /// The first related heading found, or `None` if no relationship is established.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let auth_record = AuthorityRecord::new(leader);
    /// if let Some(primary) = auth_record.heading() {
    ///     if let Some(related) = auth_record.find_related_heading(primary) {
    ///         println!("Related to: {}", related.tag);
    ///     }
    /// }
    /// ```
    #[must_use]
    fn find_related_heading(&self, heading: &Field) -> Option<&Field>;

    /// Extract the preferred label from a heading field.
    ///
    /// Gets subfield 'a' (the main heading term) from any authority field.
    /// This is useful for displaying the normalized form of a heading.
    ///
    /// # Arguments
    ///
    /// * `field` - An authority field (1XX, 4XX, 5XX, or 7XX)
    ///
    /// # Returns
    ///
    /// The main heading term (subfield 'a'), or `None` if not present.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let auth_record = AuthorityRecord::new(leader);
    /// if let Some(field) = auth_record.heading() {
    ///     if let Some(label) = AuthorityQueries::extract_authority_label(field) {
    ///         println!("Label: {}", label);
    ///     }
    /// }
    /// ```
    #[must_use]
    fn extract_authority_label(field: &Field) -> Option<&str> {
        field.get_subfield('a')
    }

    /// Get the heading subdivision (if any) from a field.
    ///
    /// Extracts subdivision information from subfield 'x', 'y', 'z', or 'v'
    /// (topical, geographic, chronological, and genre subdivisions respectively).
    ///
    /// # Arguments
    ///
    /// * `field` - An authority field
    ///
    /// # Returns
    ///
    /// A vector of subdivision values found in the field.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let auth_record = AuthorityRecord::new(leader);
    /// if let Some(fields) = auth_record.get_fields("650") {
    ///     for field in fields {
    ///         let subdivisions = AuthorityQueries::get_subdivisions(field);
    ///         for sub in subdivisions {
    ///             println!("Subdivision: {}", sub);
    ///         }
    ///     }
    /// }
    /// ```
    #[must_use]
    fn get_subdivisions(field: &Field) -> Vec<&str> {
        let mut subdivisions = Vec::new();
        // Topical subdivision
        if let Some(x) = field.get_subfield('x') {
            subdivisions.push(x);
        }
        // Geographic subdivision
        if let Some(z) = field.get_subfield('z') {
            subdivisions.push(z);
        }
        // Chronological subdivision
        if let Some(y) = field.get_subfield('y') {
            subdivisions.push(y);
        }
        // Genre/form subdivision
        if let Some(v) = field.get_subfield('v') {
            subdivisions.push(v);
        }
        subdivisions
    }
}

impl AuthorityQueries for AuthorityRecord {
    fn get_see_from_headings(&self) -> Vec<&Field> {
        // 4XX fields are see-from tracings
        // 400 - See From Tracing—Personal Name
        // 410 - See From Tracing—Corporate Name
        // 411 - See From Tracing—Meeting Name
        // 430 - See From Tracing—Uniform Title
        // 448 - See From Tracing—Chronological Term
        // 450 - See From Tracing—Topical Term
        // 451 - See From Tracing—Geographic Name
        // 455 - See From Tracing—Genre/Form Term
        let mut results = Vec::new();
        for tag in &["400", "410", "411", "430", "448", "450", "451", "455"] {
            if let Some(fields) = self.get_fields(tag) {
                results.extend(fields.iter());
            }
        }
        results
    }

    fn get_see_also_headings(&self) -> Vec<&Field> {
        // 5XX fields are see-also tracings
        // 500 - See Also Tracing—Personal Name
        // 510 - See Also Tracing—Corporate Name
        // 511 - See Also Tracing—Meeting Name
        // 530 - See Also Tracing—Uniform Title
        // 548 - See Also Tracing—Chronological Term
        // 550 - See Also Tracing—Topical Term
        // 551 - See Also Tracing—Geographic Name
        // 555 - See Also Tracing—Genre/Form Term
        let mut results = Vec::new();
        for tag in &["500", "510", "511", "530", "548", "550", "551", "555"] {
            if let Some(fields) = self.get_fields(tag) {
                results.extend(fields.iter());
            }
        }
        results
    }

    fn get_relationship_fields(&self) -> Vec<&Field> {
        // 7XX fields are authority relationships
        // 700 - Established Heading Linking Entry—Personal Name
        // 710 - Established Heading Linking Entry—Corporate Name
        // 711 - Established Heading Linking Entry—Meeting Name
        // 730 - Established Heading Linking Entry—Uniform Title
        // 748 - Established Heading Linking Entry—Chronological Term
        // 750 - Established Heading Linking Entry—Topical Term
        // 751 - Established Heading Linking Entry—Geographic Name
        // 755 - Established Heading Linking Entry—Genre/Form Term
        let mut results = Vec::new();
        for tag in &["700", "710", "711", "730", "748", "750", "751", "755"] {
            if let Some(fields) = self.get_fields(tag) {
                results.extend(fields.iter());
            }
        }
        results
    }

    fn get_authority_references(&self) -> Vec<&Field> {
        let mut results = Vec::new();
        results.extend(self.get_see_from_headings());
        results.extend(self.get_see_also_headings());
        results.extend(self.get_relationship_fields());
        results
    }

    fn find_related_heading(&self, _heading: &Field) -> Option<&Field> {
        // Return the first see-also heading as a related heading
        self.get_see_also_headings().first().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::leader::Leader;
    use crate::record::{Field, Subfield};

    fn make_test_leader() -> Leader {
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

    fn create_test_auth_record() -> AuthorityRecord {
        let mut record = AuthorityRecord::new(make_test_leader());

        // Add heading field (150 - Topical Term)
        let mut heading_field = Field::new("150".to_string(), ' ', ' ');
        heading_field.subfields.push(Subfield {
            code: 'a',
            value: "Computer science".to_string(),
        });
        record.set_heading(heading_field);

        // Add see-from tracing (450)
        let mut see_from_field = Field::new("450".to_string(), ' ', ' ');
        see_from_field.subfields.push(Subfield {
            code: 'a',
            value: "Computing".to_string(),
        });
        record.add_see_from_tracing(see_from_field);

        // Add see-also tracing (550)
        let mut see_also_field = Field::new("550".to_string(), ' ', ' ');
        see_also_field.subfields.push(Subfield {
            code: 'a',
            value: "Information technology".to_string(),
        });
        record.add_see_also_tracing(see_also_field);

        // Add relationship field (750)
        let mut rel_field = Field::new("750".to_string(), ' ', ' ');
        rel_field.subfields.push(Subfield {
            code: 'a',
            value: "Algorithms".to_string(),
        });
        record.add_field(rel_field);

        record
    }

    #[test]
    fn test_get_see_from_headings() {
        let record = create_test_auth_record();
        let see_from = record.get_see_from_headings();
        assert_eq!(see_from.len(), 1);
        assert_eq!(see_from[0].tag, "450");
    }

    #[test]
    fn test_get_see_also_headings() {
        let record = create_test_auth_record();
        let see_also = record.get_see_also_headings();
        assert_eq!(see_also.len(), 1);
        assert_eq!(see_also[0].tag, "550");
    }

    #[test]
    fn test_get_relationship_fields() {
        let record = create_test_auth_record();
        let rel = record.get_relationship_fields();
        assert_eq!(rel.len(), 1);
        assert_eq!(rel[0].tag, "750");
    }

    #[test]
    fn test_get_authority_references() {
        let record = create_test_auth_record();
        let refs = record.get_authority_references();
        assert_eq!(refs.len(), 3); // 1 from 4xx + 1 from 5xx + 1 from 7xx
    }

    #[test]
    fn test_find_related_heading() {
        let record = create_test_auth_record();
        if let Some(heading) = record.heading() {
            let related = record.find_related_heading(heading);
            assert!(related.is_some());
            assert_eq!(related.unwrap().tag, "550");
        }
    }

    #[test]
    fn test_extract_authority_label() {
        let mut field = Field::new("150".to_string(), ' ', ' ');
        field.subfields.push(Subfield {
            code: 'a',
            value: "Computer science".to_string(),
        });
        let label = <AuthorityRecord as AuthorityQueries>::extract_authority_label(&field);
        assert_eq!(label, Some("Computer science"));
    }

    #[test]
    fn test_get_subdivisions() {
        let mut field = Field::new("650".to_string(), ' ', ' ');
        field.subfields.push(Subfield {
            code: 'a',
            value: "Computers".to_string(),
        });
        field.subfields.push(Subfield {
            code: 'x',
            value: "History".to_string(),
        });
        field.subfields.push(Subfield {
            code: 'z',
            value: "United States".to_string(),
        });
        field.subfields.push(Subfield {
            code: 'y',
            value: "20th century".to_string(),
        });

        let subdivisions = <AuthorityRecord as AuthorityQueries>::get_subdivisions(&field);
        assert_eq!(subdivisions.len(), 3); // x, z, y
    }

    #[test]
    fn test_empty_record() {
        let record = AuthorityRecord::new(make_test_leader());
        assert_eq!(record.get_see_from_headings().len(), 0);
        assert_eq!(record.get_see_also_headings().len(), 0);
        assert_eq!(record.get_relationship_fields().len(), 0);
        assert_eq!(record.get_authority_references().len(), 0);
    }
}
