//! Advanced field query patterns for MARC records.
//!
//! This module provides domain-specific query patterns for finding fields based on
//! complex criteria such as indicators, tag ranges, subfield patterns, and regex matching.
//!
//! # Examples
//!
//! ## Query by indicators
//!
//! ```ignore
//! use mrrc::field_query::FieldQuery;
//!
//! // Find all 650 fields with indicator2 = '0' (LCSH)
//! for field in record.fields_by_indicator("650", None, Some('0')) {
//!     println!("Subject: {:?}", field);
//! }
//! ```
//!
//! ## Query by tag range
//!
//! ```ignore
//! // Find all subject-related fields (600-699)
//! for field in record.fields_in_range("600", "699") {
//!     println!("Subject field: {}", field.tag);
//! }
//! ```
//!
//! ## Query with builder pattern
//!
//! ```ignore
//! let query = FieldQuery::new()
//!     .tag("650")
//!     .indicator2('0')
//!     .has_subfield('a');
//!
//! for field in record.matching_fields(&query) {
//!     println!("LCSH heading: {:?}", field);
//! }
//! ```

use crate::record::Field;
use regex::Regex;

/// A query builder for finding fields matching complex criteria.
///
/// `FieldQuery` uses the builder pattern to construct complex field queries
/// that can match on tags, indicators, and subfield presence.
///
/// # Examples
///
/// ```ignore
/// let query = FieldQuery::new()
///     .tag("650")
///     .indicator1(None)  // Match any character
///     .indicator2(Some('0'))  // Match only '0'
///     .has_subfield('a');
///
/// for field in record.fields_matching(&query) {
///     // Process field
/// }
/// ```
#[derive(Debug, Clone)]
pub struct FieldQuery {
    /// Optional tag filter. If None, matches all tags.
    pub tag: Option<String>,
    /// Optional first indicator filter. None = wildcard (match any)
    pub indicator1: Option<char>,
    /// Optional second indicator filter. None = wildcard (match any)
    pub indicator2: Option<char>,
    /// Required subfield codes (AND logic)
    pub required_subfields: Vec<char>,
}

impl Default for FieldQuery {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldQuery {
    /// Create a new query that matches all fields.
    #[must_use]
    pub fn new() -> Self {
        FieldQuery {
            tag: None,
            indicator1: None,
            indicator2: None,
            required_subfields: Vec::new(),
        }
    }

    /// Restrict query to fields with a specific tag.
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// Restrict query to fields with a specific first indicator.
    ///
    /// Passing `None` creates a wildcard that matches any character.
    #[must_use]
    pub fn indicator1(mut self, indicator: Option<char>) -> Self {
        self.indicator1 = indicator;
        self
    }

    /// Restrict query to fields with a specific second indicator.
    ///
    /// Passing `None` creates a wildcard that matches any character.
    #[must_use]
    pub fn indicator2(mut self, indicator: Option<char>) -> Self {
        self.indicator2 = indicator;
        self
    }

    /// Require the field to have a subfield with the given code.
    ///
    /// Multiple calls add additional required subfields (AND logic).
    #[must_use]
    pub fn has_subfield(mut self, code: char) -> Self {
        if !self.required_subfields.contains(&code) {
            self.required_subfields.push(code);
        }
        self
    }

    /// Require the field to have all of the given subfield codes.
    #[must_use]
    pub fn has_subfields(mut self, codes: &[char]) -> Self {
        for &code in codes {
            if !self.required_subfields.contains(&code) {
                self.required_subfields.push(code);
            }
        }
        self
    }

    /// Match fields in a tag range (inclusive).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Match all subject fields (600-699)
    /// let query = FieldQuery::new().tag_range("600", "699");
    /// ```
    #[must_use]
    pub fn tag_range(self, start_tag: &str, end_tag: &str) -> TagRangeQuery {
        TagRangeQuery {
            start_tag: start_tag.to_string(),
            end_tag: end_tag.to_string(),
            indicator1: self.indicator1,
            indicator2: self.indicator2,
            required_subfields: self.required_subfields,
        }
    }

    /// Check if a field matches all criteria in this query.
    #[must_use]
    pub fn matches(&self, field: &Field) -> bool {
        // Check tag
        if let Some(ref tag) = self.tag {
            if field.tag != *tag {
                return false;
            }
        }

        // Check first indicator
        if let Some(ind1) = self.indicator1 {
            if field.indicator1 != ind1 {
                return false;
            }
        }

        // Check second indicator
        if let Some(ind2) = self.indicator2 {
            if field.indicator2 != ind2 {
                return false;
            }
        }

        // Check required subfields
        for &required_code in &self.required_subfields {
            if field.get_subfield(required_code).is_none() {
                return false;
            }
        }

        true
    }
}

/// Query for fields within a tag range.
///
/// This is returned by `FieldQuery::tag_range()` and enables range-based queries.
#[derive(Debug, Clone)]
pub struct TagRangeQuery {
    /// Start of tag range (inclusive)
    pub start_tag: String,
    /// End of tag range (inclusive)
    pub end_tag: String,
    /// Optional first indicator filter
    pub indicator1: Option<char>,
    /// Optional second indicator filter
    pub indicator2: Option<char>,
    /// Required subfield codes (AND logic)
    pub required_subfields: Vec<char>,
}

impl TagRangeQuery {
    /// Check if a tag is within this range (inclusive).
    #[must_use]
    pub fn tag_in_range(&self, tag: &str) -> bool {
        tag >= self.start_tag.as_str() && tag <= self.end_tag.as_str()
    }

    /// Check if a field matches this range query.
    #[must_use]
    pub fn matches(&self, field: &Field) -> bool {
        if !self.tag_in_range(&field.tag) {
            return false;
        }

        if let Some(ind1) = self.indicator1 {
            if field.indicator1 != ind1 {
                return false;
            }
        }

        if let Some(ind2) = self.indicator2 {
            if field.indicator2 != ind2 {
                return false;
            }
        }

        for &required_code in &self.required_subfields {
            if field.get_subfield(required_code).is_none() {
                return false;
            }
        }

        true
    }
}

/// Query for fields with subfield values matching a regex pattern.
///
/// This enables finding fields where a specific subfield's value matches
/// a regular expression pattern.
///
/// # Examples
///
/// ```ignore
/// // Find all ISBNs that start with 978
/// let query = SubfieldPatternQuery::new("020", 'a', "^978-.*");
/// for field in record.fields_matching_pattern(&query) {
///     println!("ISBN: {:?}", field);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SubfieldPatternQuery {
    /// Tag to match
    pub tag: String,
    /// Subfield code to match
    pub subfield_code: char,
    /// Regex pattern for subfield value
    pattern: Regex,
}

impl SubfieldPatternQuery {
    /// Create a new subfield pattern query.
    ///
    /// # Arguments
    ///
    /// * `tag` - The field tag to search in
    /// * `subfield_code` - The subfield code to match against
    /// * `pattern` - A regex pattern string
    ///
    /// # Returns
    ///
    /// `Ok(SubfieldPatternQuery)` if the pattern is valid regex, or `Err` if invalid.
    ///
    /// # Errors
    ///
    /// Returns a `regex::Error` if the pattern is not a valid regular expression.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let query = SubfieldPatternQuery::new("650", 'a', r"^[A-Z]")?;
    /// ```
    pub fn new(
        tag: impl Into<String>,
        subfield_code: char,
        pattern: &str,
    ) -> Result<Self, regex::Error> {
        Ok(SubfieldPatternQuery {
            tag: tag.into(),
            subfield_code,
            pattern: Regex::new(pattern)?,
        })
    }

    /// Check if a field matches this pattern query.
    #[must_use]
    pub fn matches(&self, field: &Field) -> bool {
        if field.tag != self.tag {
            return false;
        }

        field
            .get_subfield(self.subfield_code)
            .is_some_and(|value| self.pattern.is_match(value))
    }
}

/// Query for fields with a subfield value matching a specific string.
///
/// Supports exact matches, partial matches, and wildcards.
#[derive(Debug, Clone)]
pub struct SubfieldValueQuery {
    /// Tag to match
    pub tag: String,
    /// Subfield code to match
    pub subfield_code: char,
    /// Value to match
    pub value: String,
    /// If true, match substrings (contains); if false, exact match
    pub partial: bool,
}

impl SubfieldValueQuery {
    /// Create a new exact subfield value query.
    #[must_use]
    pub fn new(tag: impl Into<String>, subfield_code: char, value: impl Into<String>) -> Self {
        SubfieldValueQuery {
            tag: tag.into(),
            subfield_code,
            value: value.into(),
            partial: false,
        }
    }

    /// Create a new partial/substring subfield value query.
    #[must_use]
    pub fn partial(tag: impl Into<String>, subfield_code: char, value: impl Into<String>) -> Self {
        SubfieldValueQuery {
            tag: tag.into(),
            subfield_code,
            value: value.into(),
            partial: true,
        }
    }

    /// Check if a field matches this value query.
    #[must_use]
    pub fn matches(&self, field: &Field) -> bool {
        if field.tag != self.tag {
            return false;
        }

        field.get_subfield(self.subfield_code).is_some_and(|value| {
            if self.partial {
                value.contains(self.value.as_str())
            } else {
                value == self.value.as_str()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::Field;

    fn create_test_field(tag: &str, ind1: char, ind2: char, subfields: &[(char, &str)]) -> Field {
        let mut field = Field::new(tag.to_string(), ind1, ind2);
        for &(code, value) in subfields {
            field.add_subfield_str(code, value);
        }
        field
    }

    #[test]
    fn test_query_matches_tag() {
        let field = create_test_field("650", ' ', '0', &[('a', "Subject")]);
        let query = FieldQuery::new().tag("650");
        assert!(query.matches(&field));

        let query = FieldQuery::new().tag("651");
        assert!(!query.matches(&field));
    }

    #[test]
    fn test_query_matches_indicator1() {
        let field = create_test_field("245", '1', '0', &[('a', "Title")]);

        let query = FieldQuery::new().indicator1(Some('1'));
        assert!(query.matches(&field));

        let query = FieldQuery::new().indicator1(Some('0'));
        assert!(!query.matches(&field));

        // None is wildcard
        let query = FieldQuery::new().indicator1(None);
        assert!(query.matches(&field));
    }

    #[test]
    fn test_query_matches_indicator2() {
        let field = create_test_field("650", ' ', '0', &[('a', "Subject")]);

        let query = FieldQuery::new().indicator2(Some('0'));
        assert!(query.matches(&field));

        let query = FieldQuery::new().indicator2(Some('1'));
        assert!(!query.matches(&field));
    }

    #[test]
    fn test_query_matches_has_subfield() {
        let field = create_test_field("650", ' ', '0', &[('a', "Subject"), ('x', "History")]);

        let query = FieldQuery::new().has_subfield('a');
        assert!(query.matches(&field));

        let query = FieldQuery::new().has_subfield('b');
        assert!(!query.matches(&field));
    }

    #[test]
    fn test_query_matches_multiple_subfields() {
        let field = create_test_field("650", ' ', '0', &[('a', "Subject"), ('x', "History")]);

        let query = FieldQuery::new().has_subfield('a').has_subfield('x');
        assert!(query.matches(&field));

        let query = FieldQuery::new().has_subfield('a').has_subfield('b');
        assert!(!query.matches(&field));
    }

    #[test]
    fn test_query_combines_criteria() {
        let field = create_test_field("650", ' ', '0', &[('a', "Subject"), ('x', "History")]);

        let query = FieldQuery::new()
            .tag("650")
            .indicator2(Some('0'))
            .has_subfield('a');
        assert!(query.matches(&field));

        let query = FieldQuery::new()
            .tag("651")
            .indicator2(Some('0'))
            .has_subfield('a');
        assert!(!query.matches(&field));

        let query = FieldQuery::new()
            .tag("650")
            .indicator2(Some('1'))
            .has_subfield('a');
        assert!(!query.matches(&field));
    }

    #[test]
    fn test_tag_range_query_in_range() {
        let query = TagRangeQuery {
            start_tag: "600".to_string(),
            end_tag: "699".to_string(),
            indicator1: None,
            indicator2: None,
            required_subfields: Vec::new(),
        };

        assert!(query.tag_in_range("600"));
        assert!(query.tag_in_range("650"));
        assert!(query.tag_in_range("699"));
        assert!(!query.tag_in_range("599"));
        assert!(!query.tag_in_range("700"));
    }

    #[test]
    fn test_tag_range_query_matches() {
        let field = create_test_field("650", ' ', '0', &[('a', "Subject")]);
        let query = TagRangeQuery {
            start_tag: "600".to_string(),
            end_tag: "699".to_string(),
            indicator1: None,
            indicator2: Some('0'),
            required_subfields: vec!['a'],
        };

        assert!(query.matches(&field));

        let field = create_test_field("700", ' ', '0', &[('a', "Name")]);
        assert!(!query.matches(&field));
    }

    #[test]
    fn test_field_query_default() {
        let query = FieldQuery::default();
        let field = create_test_field("650", ' ', '0', &[('a', "Subject")]);
        assert!(query.matches(&field));
    }

    #[test]
    fn test_subfield_pattern_query_matches_isbn() {
        let field = create_test_field("020", ' ', ' ', &[('a', "978-0-12345-678-9")]);
        let query = SubfieldPatternQuery::new("020", 'a', r"^978-.*").unwrap();
        assert!(query.matches(&field));

        let query = SubfieldPatternQuery::new("020", 'a', r"^979-.*").unwrap();
        assert!(!query.matches(&field));
    }

    #[test]
    fn test_subfield_pattern_query_different_tag() {
        let field = create_test_field("020", ' ', ' ', &[('a', "978-0-12345-678-9")]);
        let query = SubfieldPatternQuery::new("022", 'a', r"^978-.*").unwrap();
        assert!(!query.matches(&field));
    }

    #[test]
    fn test_subfield_pattern_query_no_matching_subfield() {
        let field = create_test_field("020", ' ', ' ', &[('a', "978-0-12345-678-9")]);
        let query = SubfieldPatternQuery::new("020", 'b', r"^978-.*").unwrap();
        assert!(!query.matches(&field));
    }

    #[test]
    fn test_subfield_pattern_query_complex_pattern() {
        let field = create_test_field("100", ' ', ' ', &[('a', "Smith, John"), ('d', "1873-1944")]);
        // Match years in format YYYY-YYYY
        let query = SubfieldPatternQuery::new("100", 'd', r"\d{4}-\d{4}").unwrap();
        assert!(query.matches(&field));
    }

    #[test]
    fn test_subfield_value_query_exact_match() {
        let field = create_test_field("650", ' ', '0', &[('a', "History")]);
        let query = SubfieldValueQuery::new("650", 'a', "History");
        assert!(query.matches(&field));

        let query = SubfieldValueQuery::new("650", 'a', "history");
        assert!(!query.matches(&field)); // Case sensitive
    }

    #[test]
    fn test_subfield_value_query_partial_match() {
        let field = create_test_field("650", ' ', '0', &[('a', "World History")]);
        let query = SubfieldValueQuery::partial("650", 'a', "History");
        assert!(query.matches(&field));

        let query = SubfieldValueQuery::partial("650", 'a', "World");
        assert!(query.matches(&field));

        let query = SubfieldValueQuery::partial("650", 'a', "Medieval");
        assert!(!query.matches(&field));
    }

    #[test]
    fn test_subfield_value_query_different_tag() {
        let field = create_test_field("650", ' ', '0', &[('a', "History")]);
        let query = SubfieldValueQuery::new("651", 'a', "History");
        assert!(!query.matches(&field));
    }

    #[test]
    fn test_subfield_value_query_no_subfield() {
        let field = create_test_field("650", ' ', '0', &[('a', "History")]);
        let query = SubfieldValueQuery::new("650", 'x', "Subdivision");
        assert!(!query.matches(&field));
    }
}
