//! Bibliographic helper utilities for MARC records.
//!
//! This module provides utilities for validating and parsing common
//! bibliographic identifiers and data found in MARC records.

/// ISBN (International Standard Book Number) validator and parser
#[derive(Debug)]
pub struct IsbnValidator;

impl IsbnValidator {
    /// Validate an ISBN-10 checksum
    ///
    /// ISBN-10 uses a weighted checksum where digits are multiplied by 10-9 and mod 11.
    /// The check digit can be 0-9 or 'X' (representing 10).
    ///
    /// # Examples
    ///
    /// ```
    /// use mrrc::IsbnValidator;
    ///
    /// // Valid ISBN-10
    /// assert!(IsbnValidator::validate_isbn10("0306406152"));
    /// // Invalid ISBN-10
    /// assert!(!IsbnValidator::validate_isbn10("0306406153"));
    /// ```
    #[must_use]
    pub fn validate_isbn10(isbn: &str) -> bool {
        let clean = isbn.replace(['-', ' '], "");

        if clean.len() != 10 {
            return false;
        }

        let mut sum = 0;
        for (i, ch) in clean.chars().enumerate() {
            let digit = if i == 9 && ch == 'X' {
                10
            } else if let Some(d) = ch.to_digit(10) {
                d
            } else {
                return false;
            };

            sum += digit * (10 - u32::try_from(i).unwrap_or(0));
        }

        sum % 11 == 0
    }

    /// Validate an ISBN-13 checksum
    ///
    /// ISBN-13 uses a weighted checksum where odd positions are multiplied by 1
    /// and even positions by 3, summed mod 10.
    ///
    /// # Examples
    ///
    /// ```
    /// use mrrc::IsbnValidator;
    ///
    /// // Valid ISBN-13
    /// assert!(IsbnValidator::validate_isbn13("9780306406157"));
    /// // Invalid ISBN-13
    /// assert!(!IsbnValidator::validate_isbn13("9780306406158"));
    /// ```
    #[must_use]
    pub fn validate_isbn13(isbn: &str) -> bool {
        let clean = isbn.replace(['-', ' '], "");

        if clean.len() != 13 {
            return false;
        }

        // Must start with 978 or 979
        if !clean.starts_with("978") && !clean.starts_with("979") {
            return false;
        }

        let mut sum = 0;
        for (i, ch) in clean.chars().enumerate() {
            if let Some(digit) = ch.to_digit(10) {
                let weight = if i % 2 == 0 { 1 } else { 3 };
                sum += digit * weight;
            } else {
                return false;
            }
        }

        (10 - (sum % 10)) % 10 == 0
    }

    /// Validate an ISBN (auto-detect ISBN-10 or ISBN-13)
    ///
    /// # Examples
    ///
    /// ```
    /// use mrrc::IsbnValidator;
    ///
    /// assert!(IsbnValidator::validate("0306406152")); // ISBN-10
    /// assert!(IsbnValidator::validate("9780306406157")); // ISBN-13
    /// ```
    #[must_use]
    pub fn validate(isbn: &str) -> bool {
        let clean = isbn.replace(['-', ' '], "");
        match clean.len() {
            10 => Self::validate_isbn10(&clean),
            13 => Self::validate_isbn13(&clean),
            _ => false,
        }
    }

    /// Extract the ISBN without dashes or spaces
    ///
    /// # Examples
    ///
    /// ```
    /// use mrrc::IsbnValidator;
    ///
    /// assert_eq!(IsbnValidator::normalize("978-0-306-40615-7"), "9780306406157");
    /// assert_eq!(IsbnValidator::normalize("0-306-40615-2"), "0306406152");
    /// ```
    #[must_use]
    pub fn normalize(isbn: &str) -> String {
        isbn.replace(['-', ' '], "")
    }
}

/// Publication information parser
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationInfo {
    /// Place of publication (field 260, subfield 'a')
    pub place: Option<String>,
    /// Publisher (field 260, subfield 'b')
    pub publisher: Option<String>,
    /// Publication date (field 260, subfield 'c')
    pub date: Option<String>,
}

impl PublicationInfo {
    /// Create a new `PublicationInfo`
    #[must_use]
    pub fn new(place: Option<String>, publisher: Option<String>, date: Option<String>) -> Self {
        PublicationInfo {
            place,
            publisher,
            date,
        }
    }

    /// Extract publication year from the date field
    ///
    /// Attempts to parse a 4-digit year from the publication date.
    /// Looks for the first sequence of 4 digits.
    ///
    /// # Examples
    ///
    /// ```
    /// use mrrc::PublicationInfo;
    ///
    /// let info = PublicationInfo::new(None, None, Some("New York : Springer, 2015.".to_string()));
    /// assert_eq!(info.publication_year(), Some(2015));
    ///
    /// let info = PublicationInfo::new(None, None, Some("[s.l. : s.n.], 1999".to_string()));
    /// assert_eq!(info.publication_year(), Some(1999));
    /// ```
    #[must_use]
    pub fn publication_year(&self) -> Option<u32> {
        if let Some(date_str) = &self.date {
            // Look for first 4-digit sequence
            let mut digits = String::new();
            for ch in date_str.chars() {
                if ch.is_ascii_digit() {
                    digits.push(ch);
                    if digits.len() == 4 {
                        return digits.parse().ok();
                    }
                } else if !digits.is_empty() {
                    // Reset if we hit a non-digit after collecting some
                    digits.clear();
                }
            }

            // Try one more time if we never hit 4 digits
            if digits.is_empty() {
                None
            } else {
                while digits.len() < 4 {
                    digits.push('0');
                }
                digits.parse().ok()
            }
        } else {
            None
        }
    }

    /// Format publication info as a complete statement
    ///
    /// # Examples
    ///
    /// ```
    /// use mrrc::PublicationInfo;
    ///
    /// let info = PublicationInfo::new(
    ///     Some("London".to_string()),
    ///     Some("Routledge".to_string()),
    ///     Some("2020".to_string()),
    /// );
    /// assert_eq!(
    ///     info.format_statement(),
    ///     "London : Routledge, 2020"
    /// );
    /// ```
    #[must_use]
    pub fn format_statement(&self) -> String {
        let mut parts = Vec::new();

        if let Some(place) = &self.place {
            if !place.is_empty() {
                parts.push(place.clone());
            }
        }

        if let Some(publisher) = &self.publisher {
            if !publisher.is_empty() {
                parts.push(publisher.clone());
            }
        }

        let base = if parts.is_empty() {
            String::new()
        } else {
            parts.join(" : ")
        };

        if let Some(date) = &self.date {
            if date.is_empty() {
                base
            } else if base.is_empty() {
                date.clone()
            } else {
                format!("{base}, {date}")
            }
        } else {
            base
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_isbn10_valid() {
        assert!(IsbnValidator::validate_isbn10("0306406152"));
        assert!(IsbnValidator::validate_isbn10("043942089X"));
    }

    #[test]
    fn test_validate_isbn10_invalid() {
        assert!(!IsbnValidator::validate_isbn10("0306406153"));
        assert!(!IsbnValidator::validate_isbn10("123"));
        assert!(!IsbnValidator::validate_isbn10("abcd123456"));
    }

    #[test]
    fn test_validate_isbn10_with_dashes() {
        assert!(IsbnValidator::validate_isbn10("0-306-40615-2"));
        assert!(IsbnValidator::validate_isbn10("0-439-42089-X"));
    }

    #[test]
    fn test_validate_isbn13_valid() {
        assert!(IsbnValidator::validate_isbn13("9780306406157"));
        assert!(IsbnValidator::validate_isbn13("9780201379624")); // Valid ISBN-13
    }

    #[test]
    fn test_validate_isbn13_invalid() {
        assert!(!IsbnValidator::validate_isbn13("9780306406158"));
        assert!(!IsbnValidator::validate_isbn13("1234567890123"));
        assert!(!IsbnValidator::validate_isbn13("123"));
    }

    #[test]
    fn test_validate_isbn13_with_dashes() {
        assert!(IsbnValidator::validate_isbn13("978-0-306-40615-7"));
    }

    #[test]
    fn test_validate_auto_detect() {
        assert!(IsbnValidator::validate("0306406152")); // ISBN-10
        assert!(IsbnValidator::validate("9780306406157")); // ISBN-13
        assert!(!IsbnValidator::validate("123"));
    }

    #[test]
    fn test_normalize() {
        assert_eq!(
            IsbnValidator::normalize("978-0-306-40615-7"),
            "9780306406157"
        );
        assert_eq!(IsbnValidator::normalize("0-306-40615-2"), "0306406152");
        assert_eq!(
            IsbnValidator::normalize("978 0 306 40615 7"),
            "9780306406157"
        );
    }

    #[test]
    fn test_publication_year_extraction() {
        let info = PublicationInfo::new(None, None, Some("2020".to_string()));
        assert_eq!(info.publication_year(), Some(2020));

        let info = PublicationInfo::new(None, None, Some("New York : Springer, 2015.".to_string()));
        assert_eq!(info.publication_year(), Some(2015));

        let info = PublicationInfo::new(None, None, Some("[s.l. : s.n.], 1999".to_string()));
        assert_eq!(info.publication_year(), Some(1999));

        let info = PublicationInfo::new(None, None, None);
        assert_eq!(info.publication_year(), None);
    }

    #[test]
    fn test_format_statement_complete() {
        let info = PublicationInfo::new(
            Some("London".to_string()),
            Some("Routledge".to_string()),
            Some("2020".to_string()),
        );
        assert_eq!(info.format_statement(), "London : Routledge, 2020");
    }

    #[test]
    fn test_format_statement_partial() {
        let info =
            PublicationInfo::new(Some("New York".to_string()), None, Some("1995".to_string()));
        assert_eq!(info.format_statement(), "New York, 1995");
    }

    #[test]
    fn test_format_statement_empty() {
        let info = PublicationInfo::new(None, None, None);
        assert_eq!(info.format_statement(), "");
    }
}
