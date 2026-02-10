//! Field linkage support for MARC 880 (Alternate Graphical Representation) fields.
//!
//! This module provides utilities for working with linked fields in MARC records,
//! particularly for handling romanized and vernacular text pairs.
//!
//! # Background: MARC 880 Fields
//!
//! The 880 field is used to provide an alternate graphical representation of data
//! that appears in another field. Common use cases:
//! - Original script (e.g., Arabic, Hebrew, Chinese) paired with romanized form
//! - Romanized title paired with original script version
//!
//! Linkage is established through **subfield 6** (Linkage), which contains:
//! - An occurrence number (001-999) that links the original field with its 880 counterpart
//! - A script identification code
//! - An optional reverse script flag
//!
//! # Examples
//!
//! Subfield 6 format: `NNN-XX` or `NNN-XX/r`
//! - `100: $6 880-01$a Smith, John` (original field with linkage)
//! - `880: $6 100-01$a سميث، جون` (880 field with reverse linkage)
//!
//! The occurrence numbers match to link the fields together.

use regex::Regex;

/// Information extracted from MARC subfield 6 (Linkage).
///
/// This structure represents the parsed linkage information that connects
/// an original field with its alternate graphical representation (usually
/// in field 880).
///
/// # Examples
///
/// ```ignore
/// use mrrc::field_linkage::LinkageInfo;
///
/// let info = LinkageInfo::parse("100-01").unwrap();
/// assert_eq!(info.occurrence, "01");
/// assert_eq!(info.script_id, "");
/// assert!(!info.is_reverse);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkageInfo {
    /// Occurrence number (001-999) linking fields together
    pub occurrence: String,

    /// Script identification code (e.g., "01" for Hebrew, "02" for Arabic)
    pub script_id: String,

    /// Whether reverse script is flagged (after `/r`)
    pub is_reverse: bool,
}

impl LinkageInfo {
    /// Parse a MARC subfield 6 value into linkage information.
    ///
    /// # Format
    ///
    /// Expected format: `TAG-OCC[/r]` or `TAG-OCC[/script][/r]`
    /// - `TAG`: Three-digit field tag (first part)
    /// - `-OCC`: Occurrence number separator (2-3 digits)
    /// - `/script`: Optional script identification code
    /// - `/r`: Optional reverse script flag
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Standard linkage (field 100, occurrence 01)
    /// let info = LinkageInfo::parse("100-01").unwrap();
    /// assert_eq!(info.occurrence, "01");
    ///
    /// // With script code
    /// let info = LinkageInfo::parse("245-02/r").unwrap();
    /// assert_eq!(info.occurrence, "02");
    /// assert!(info.is_reverse);
    /// ```
    ///
    /// # Returns
    ///
    /// `Some(LinkageInfo)` if the value matches the expected format,
    /// `None` if the format is invalid.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        // Pattern: TAG-OCC[/script][/r]
        // TAG = 3 digit field tag (e.g., 880, 100, 245)
        // OCC = 2-3 digit occurrence number (e.g., 01, 02, 001)
        // /script = optional script id code
        // /r = optional reverse flag
        let pattern = Regex::new(r"^(\d{3})-(\d{2,3})(?:/[a-z]{2})?(?:/r)?$").ok()?;

        let caps = pattern.captures(value)?;

        let _tag = caps.get(1)?.as_str(); // First part (field tag)
        let occurrence = caps.get(2)?.as_str().to_string(); // The occurrence number

        // Extract script id if present (between first and second /)
        // For now we keep it simple - just track occurrence number
        let script_id = String::new();
        let is_reverse = value.contains("/r");

        Some(LinkageInfo {
            occurrence,
            script_id,
            is_reverse,
        })
    }

    /// Get the occurrence number as a string.
    #[must_use]
    pub fn occurrence(&self) -> &str {
        &self.occurrence
    }

    /// Get the script identification code.
    #[must_use]
    pub fn script_id(&self) -> &str {
        &self.script_id
    }

    /// Check if reverse script flag is set.
    #[must_use]
    pub fn is_reverse(&self) -> bool {
        self.is_reverse
    }

    /// Get the reverse linkage occurrence for finding the paired field.
    ///
    /// In 880 linking, both fields have the same occurrence number.
    /// This returns the occurrence for finding the paired field.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let info = LinkageInfo::parse("100-01").unwrap();
    /// let occurrence = info.for_reverse_link();
    /// // occurrence = "01"
    /// // Would match 880 field with "880-01" linkage
    /// ```
    #[must_use]
    pub fn for_reverse_link(&self) -> String {
        self.occurrence.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_linkage() {
        let info = LinkageInfo::parse("100-01").unwrap();
        assert_eq!(info.occurrence, "01");
        assert!(!info.is_reverse);
    }

    #[test]
    fn test_parse_with_reverse_flag() {
        let info = LinkageInfo::parse("100-01/r").unwrap();
        assert_eq!(info.occurrence, "01");
        assert!(info.is_reverse);
    }

    #[test]
    fn test_parse_different_occurrences() {
        let info = LinkageInfo::parse("245-02").unwrap();
        assert_eq!(info.occurrence, "02");

        let info = LinkageInfo::parse("650-03").unwrap();
        assert_eq!(info.occurrence, "03");
    }

    #[test]
    fn test_parse_invalid_format_no_dash() {
        assert!(LinkageInfo::parse("10001").is_none());
    }

    #[test]
    fn test_parse_invalid_format_wrong_digits() {
        assert!(LinkageInfo::parse("10-01").is_none());
        assert!(LinkageInfo::parse("1000-01").is_none());
    }

    #[test]
    fn test_parse_invalid_format_wrong_occurrence() {
        assert!(LinkageInfo::parse("100-1").is_none()); // Occurrence too short
        assert!(LinkageInfo::parse("100-").is_none()); // No occurrence
    }

    #[test]
    fn test_parse_empty_string() {
        assert!(LinkageInfo::parse("").is_none());
    }

    #[test]
    fn test_parse_leading_zeros() {
        let info = LinkageInfo::parse("100-01").unwrap();
        assert_eq!(info.occurrence, "01");

        let info = LinkageInfo::parse("650-99").unwrap();
        assert_eq!(info.occurrence, "99");
    }

    #[test]
    fn test_for_reverse_link() {
        let info = LinkageInfo::parse("100-01").unwrap();
        assert_eq!(info.for_reverse_link(), "01");
    }

    #[test]
    fn test_accessors() {
        let info = LinkageInfo::parse("245-02/r").unwrap();
        assert_eq!(info.occurrence(), "02");
        assert!(info.is_reverse());
    }

    #[test]
    fn test_equality() {
        let info1 = LinkageInfo::parse("100-01").unwrap();
        let info2 = LinkageInfo::parse("100-01").unwrap();
        let info3 = LinkageInfo::parse("100-02").unwrap();

        assert_eq!(info1, info2);
        assert_ne!(info1, info3);
    }

    #[test]
    fn test_clone() {
        let info1 = LinkageInfo::parse("100-01/r").unwrap();
        let info2 = info1.clone();

        assert_eq!(info1, info2);
    }
}
