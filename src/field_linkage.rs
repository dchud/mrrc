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
    /// The 3-digit field tag from the linkage (e.g., "880", "245")
    pub tag: String,

    /// Occurrence number (01-999) linking fields together
    pub occurrence: String,

    /// Script identification code (e.g., "(2" for Hebrew, "(3" for Arabic,
    /// "$1" for CJK, "(N" for Cyrillic, "(S" for Greek)
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
        // Pattern: TAG-OCC[/SCRIPT][/r]
        // TAG = 3-digit field tag (e.g., 880, 100, 245)
        // OCC = 2-3 digit occurrence number (e.g., 01, 02, 001)
        // SCRIPT = optional MARC script identification code:
        //   - Parenthesized: (2 (Hebrew), (3 (Arabic), (B (Latin),
        //     (N (Cyrillic), (S (Greek), (4 (Devanagari), etc.
        //   - Dollar-sign: $1 (CJK)
        // /r = optional field orientation (right-to-left)
        let pattern = Regex::new(r"^(\d{3})-(\d{2,3})(?:/([\(\$][A-Za-z0-9]))?(?:/r)?$").ok()?;

        let caps = pattern.captures(value)?;

        let tag = caps.get(1)?.as_str().to_string();
        let occurrence = caps.get(2)?.as_str().to_string();
        let script_id = caps
            .get(3)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let is_reverse = value.ends_with("/r");

        Some(LinkageInfo {
            tag,
            occurrence,
            script_id,
            is_reverse,
        })
    }

    /// Get the linked field tag (e.g., "880", "245").
    #[must_use]
    pub fn tag(&self) -> &str {
        &self.tag
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

    // ------------------------------------------------------------------
    // Basic parsing
    // ------------------------------------------------------------------

    #[test]
    fn test_parse_basic_linkage() {
        let info = LinkageInfo::parse("100-01").unwrap();
        assert_eq!(info.tag, "100");
        assert_eq!(info.occurrence, "01");
        assert_eq!(info.script_id, "");
        assert!(!info.is_reverse);
    }

    #[test]
    fn test_parse_with_reverse_flag() {
        let info = LinkageInfo::parse("100-01/r").unwrap();
        assert_eq!(info.tag, "100");
        assert_eq!(info.occurrence, "01");
        assert!(info.is_reverse);
    }

    #[test]
    fn test_parse_different_occurrences() {
        let info = LinkageInfo::parse("245-02").unwrap();
        assert_eq!(info.tag, "245");
        assert_eq!(info.occurrence, "02");

        let info = LinkageInfo::parse("650-03").unwrap();
        assert_eq!(info.tag, "650");
        assert_eq!(info.occurrence, "03");
    }

    #[test]
    fn test_parse_880_tag() {
        let info = LinkageInfo::parse("880-01").unwrap();
        assert_eq!(info.tag, "880");
        assert_eq!(info.occurrence, "01");
    }

    // ------------------------------------------------------------------
    // Real MARC script identification codes
    // ------------------------------------------------------------------

    #[test]
    fn test_parse_hebrew_script_code() {
        // (2 = Hebrew, /r = right-to-left
        let info = LinkageInfo::parse("245-01/(2/r").unwrap();
        assert_eq!(info.tag, "245");
        assert_eq!(info.occurrence, "01");
        assert_eq!(info.script_id, "(2");
        assert!(info.is_reverse);
    }

    #[test]
    fn test_parse_arabic_script_code() {
        // (3 = Arabic, /r = right-to-left
        let info = LinkageInfo::parse("100-02/(3/r").unwrap();
        assert_eq!(info.tag, "100");
        assert_eq!(info.occurrence, "02");
        assert_eq!(info.script_id, "(3");
        assert!(info.is_reverse);
    }

    #[test]
    fn test_parse_cjk_script_code() {
        // $1 = CJK (Chinese, Japanese, Korean)
        let info = LinkageInfo::parse("245-01/$1").unwrap();
        assert_eq!(info.tag, "245");
        assert_eq!(info.occurrence, "01");
        assert_eq!(info.script_id, "$1");
        assert!(!info.is_reverse);
    }

    #[test]
    fn test_parse_cyrillic_script_code() {
        // (N = Cyrillic
        let info = LinkageInfo::parse("245-01/(N").unwrap();
        assert_eq!(info.tag, "245");
        assert_eq!(info.occurrence, "01");
        assert_eq!(info.script_id, "(N");
        assert!(!info.is_reverse);
    }

    #[test]
    fn test_parse_greek_script_code() {
        // (S = Greek
        let info = LinkageInfo::parse("245-01/(S").unwrap();
        assert_eq!(info.tag, "245");
        assert_eq!(info.occurrence, "01");
        assert_eq!(info.script_id, "(S");
        assert!(!info.is_reverse);
    }

    #[test]
    fn test_parse_latin_script_code() {
        // (B = Latin
        let info = LinkageInfo::parse("245-01/(B").unwrap();
        assert_eq!(info.tag, "245");
        assert_eq!(info.occurrence, "01");
        assert_eq!(info.script_id, "(B");
        assert!(!info.is_reverse);
    }

    #[test]
    fn test_parse_script_code_without_reverse() {
        // Script code present but no /r
        let info = LinkageInfo::parse("260-03/(2").unwrap();
        assert_eq!(info.tag, "260");
        assert_eq!(info.occurrence, "03");
        assert_eq!(info.script_id, "(2");
        assert!(!info.is_reverse);
    }

    // ------------------------------------------------------------------
    // Invalid formats
    // ------------------------------------------------------------------

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

    // ------------------------------------------------------------------
    // Occurrence handling
    // ------------------------------------------------------------------

    #[test]
    fn test_parse_leading_zeros() {
        let info = LinkageInfo::parse("100-01").unwrap();
        assert_eq!(info.occurrence, "01");

        let info = LinkageInfo::parse("650-99").unwrap();
        assert_eq!(info.occurrence, "99");
    }

    #[test]
    fn test_parse_three_digit_occurrence() {
        let info = LinkageInfo::parse("100-001").unwrap();
        assert_eq!(info.occurrence, "001");
    }

    // ------------------------------------------------------------------
    // Accessors and identity
    // ------------------------------------------------------------------

    #[test]
    fn test_for_reverse_link() {
        let info = LinkageInfo::parse("100-01").unwrap();
        assert_eq!(info.for_reverse_link(), "01");
    }

    #[test]
    fn test_accessors() {
        let info = LinkageInfo::parse("245-02/(2/r").unwrap();
        assert_eq!(info.tag(), "245");
        assert_eq!(info.occurrence(), "02");
        assert_eq!(info.script_id(), "(2");
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
