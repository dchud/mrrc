//! Convenience helper methods for advanced field queries.
//!
//! This module provides the `FieldQueryHelpers` trait which adds convenient methods
//! for common field query patterns. The trait is automatically implemented for all
//! Record types that implement the `MarcRecord` trait.
//!
//! # Examples
//!
//! ```ignore
//! use mrrc::field_query_helpers::FieldQueryHelpers;
//!
//! // Find subjects with a specific subdivision
//! for subject in record.subjects_with_subdivision('x', "History") {
//!     println!("Subject: {}", subject);
//! }
//!
//! // Find all ISBNs matching a pattern
//! for isbn in record.isbns_matching(r"^978-.*") {
//!     println!("ISBN: {}", isbn);
//! }
//! ```

use crate::field_query::{SubfieldPatternQuery, SubfieldValueQuery};
use crate::record::Field;

/// Extension trait providing convenient helper methods for advanced field queries.
///
/// This trait is automatically implemented for all record types, providing
/// domain-specific query patterns built on top of the core field query API.
pub trait FieldQueryHelpers {
    /// Get all fields matching a subfield pattern query.
    fn fields_matching_pattern(&self, query: &SubfieldPatternQuery) -> Vec<&Field>;

    /// Get all fields matching a subfield value query.
    fn fields_matching_value(&self, query: &SubfieldValueQuery) -> Vec<&Field>;

    /// Find all subject headings with a specific subdivision subfield and value.
    ///
    /// # Arguments
    ///
    /// * `code` - The subfield code for the subdivision (e.g., 'x', 'y', 'z')
    /// * `value` - The subdivision value to match (exact match)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Find all subjects with "History" subdivision
    /// for subject in record.subjects_with_subdivision('x', "History") {
    ///     println!("Subject: {}", subject);
    /// }
    /// ```
    fn subjects_with_subdivision(&self, code: char, value: &str) -> Vec<&Field> {
        let mut results = Vec::new();
        for tag in crate::record_helpers::SUBJECT_TAGS {
            let query = SubfieldValueQuery::new(*tag, code, value);
            results.extend(self.fields_matching_value(&query));
        }
        results
    }

    /// Find all ISBNs matching a regex pattern.
    ///
    /// # Arguments
    ///
    /// * `pattern` - A regex pattern to match ISBN values
    ///
    /// # Returns
    ///
    /// An iterator of fields (020) with ISBNs matching the pattern, or an error if
    /// the pattern is invalid.
    ///
    /// # Errors
    ///
    /// Returns a `regex::Error` if the pattern is not a valid regular expression.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Find all ISBNs starting with 978
    /// if let Ok(isbns) = record.isbns_matching(r"^978-.*") {
    ///     for isbn in isbns {
    ///         println!("ISBN: {:?}", isbn);
    ///     }
    /// }
    /// ```
    fn isbns_matching(&self, pattern: &str) -> Result<Vec<&Field>, regex::Error> {
        let query = SubfieldPatternQuery::new("020", 'a', pattern)?;
        Ok(self.fields_matching_pattern(&query))
    }

    /// Find all name fields within a tag range.
    ///
    /// Convenience method for finding names in a specific tag range (e.g., 700-711 for added entries).
    ///
    /// # Arguments
    ///
    /// * `start_tag` - Start of tag range (inclusive)
    /// * `end_tag` - End of tag range (inclusive)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Find all added entry names (700, 710, 711)
    /// for name in record.names_in_range("700", "711") {
    ///     println!("Name: {:?}", name);
    /// }
    /// ```
    fn names_in_range(&self, start_tag: &str, end_tag: &str) -> Vec<&Field>;

    /// Find all authors and extract their birth/death dates from field 100/700 subfield 'd'.
    ///
    /// Returns tuples of (name, dates) for each author with date information.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// for (name, dates) in record.authors_with_dates() {
    ///     println!("{}: {}", name, dates);
    /// }
    /// ```
    fn authors_with_dates(&self) -> Vec<(&str, &str)>;

    /// Find all subjects with a particular note in subfield 'x' (general subdivision).
    ///
    /// # Arguments
    ///
    /// * `subdivision` - The subdivision text to search for (partial match)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// for subject in record.subjects_with_note("Medieval") {
    ///     println!("Subject: {:?}", subject);
    /// }
    /// ```
    fn subjects_with_note(&self, subdivision: &str) -> Vec<&Field> {
        let mut results = Vec::new();
        for tag in crate::record_helpers::SUBJECT_TAGS {
            let query = SubfieldValueQuery::partial(*tag, 'x', subdivision);
            results.extend(self.fields_matching_value(&query));
        }
        results
    }
}
