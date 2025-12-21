//! Indicator and field validation for MARC records.
//!
//! This module provides validation of MARC field indicators according to MARC21 standards.
//! Indicators are the two characters following a field tag that specify how the field should
//! be interpreted.

use crate::error::{MarcError, Result};
use crate::record::Field;
use std::collections::HashMap;

/// Validation rules for a field's indicators
#[derive(Debug, Clone)]
pub struct IndicatorRules {
    /// Tag this rule applies to
    pub tag: String,
    /// Validation for first indicator
    pub indicator1: IndicatorValidation,
    /// Validation for second indicator
    pub indicator2: IndicatorValidation,
}

/// Validation rule for a single indicator
#[derive(Debug, Clone)]
pub enum IndicatorValidation {
    /// Indicator is undefined; blank (#) is required
    Undefined,
    /// Indicator can be any single character
    Any,
    /// Indicator must be one of the specified values
    Values(Vec<char>),
    /// Indicator must be a digit (0-9) within the specified range
    DigitRange {
        /// Minimum digit value (0-9)
        min: u8,
        /// Maximum digit value (0-9)
        max: u8,
    },
    /// Indicator is not defined in current standard (deprecated)
    Obsolete,
}

impl IndicatorValidation {
    /// Check if the given character is valid for this indicator
    #[must_use]
    pub fn is_valid(&self, c: char) -> bool {
        match self {
            IndicatorValidation::Undefined => c == '#' || c == ' ',
            IndicatorValidation::Any | IndicatorValidation::Obsolete => true,
            IndicatorValidation::Values(values) => values.contains(&c),
            IndicatorValidation::DigitRange { min, max } => {
                if let Some(digit) = c.to_digit(10) {
                    #[allow(clippy::cast_possible_truncation)]
                    let d = digit as u8;
                    d >= *min && d <= *max
                } else {
                    false
                }
            },
        }
    }
}

/// Validator for MARC field indicators
#[derive(Debug)]
pub struct IndicatorValidator {
    rules: HashMap<String, IndicatorRules>,
}

impl IndicatorValidator {
    /// Create a new validator with MARC21 standard rules
    #[must_use]
    pub fn new() -> Self {
        let rules = Self::build_standard_rules();
        IndicatorValidator { rules }
    }

    /// Build MARC21 standard indicator validation rules
    #[allow(clippy::too_many_lines)]
    fn build_standard_rules() -> HashMap<String, IndicatorRules> {
        let mut rules = HashMap::new();

        // Control fields (010-039): Generally undefined indicators
        let undefined_undefined = IndicatorRules {
            tag: String::new(),
            indicator1: IndicatorValidation::Undefined,
            indicator2: IndicatorValidation::Undefined,
        };

        for tag in &["010", "020", "024", "028", "030", "035", "037", "040"] {
            let mut rule = undefined_undefined.clone();
            rule.tag = (*tag).to_string();
            rules.insert((*tag).to_string(), rule);
        }

        // 100 - Main entry -- Personal name
        // Ind1: 0=Forename, 1=Surname, 3=Family name
        // Ind2: undefined (obsolete, was 0-2)
        rules.insert(
            "100".to_string(),
            IndicatorRules {
                tag: "100".to_string(),
                indicator1: IndicatorValidation::Values(vec!['0', '1', '3']),
                indicator2: IndicatorValidation::Undefined,
            },
        );

        // 110 - Main entry -- Corporate name
        // Ind1: 1=Jurisdiction, 2=Name in direct order
        // Ind2: undefined
        rules.insert(
            "110".to_string(),
            IndicatorRules {
                tag: "110".to_string(),
                indicator1: IndicatorValidation::Values(vec!['1', '2']),
                indicator2: IndicatorValidation::Undefined,
            },
        );

        // 111 - Main entry -- Meeting name
        // Ind1: 0=Inverted name, 1=Jurisdiction, 2=Name in direct order
        // Ind2: undefined
        rules.insert(
            "111".to_string(),
            IndicatorRules {
                tag: "111".to_string(),
                indicator1: IndicatorValidation::Values(vec!['0', '1', '2']),
                indicator2: IndicatorValidation::Undefined,
            },
        );

        // 130 - Main entry -- Uniform title
        // Ind1: 0-9 = nonfiling characters
        // Ind2: undefined (obsolete)
        rules.insert(
            "130".to_string(),
            IndicatorRules {
                tag: "130".to_string(),
                indicator1: IndicatorValidation::DigitRange { min: 0, max: 9 },
                indicator2: IndicatorValidation::Undefined,
            },
        );

        // 240 - Uniform title
        // Ind1: 0=Not printed, 1=Printed
        // Ind2: 0-9 = nonfiling characters
        rules.insert(
            "240".to_string(),
            IndicatorRules {
                tag: "240".to_string(),
                indicator1: IndicatorValidation::Values(vec!['0', '1']),
                indicator2: IndicatorValidation::DigitRange { min: 0, max: 9 },
            },
        );

        // 245 - Title statement
        // Ind1: 0=No title added entry, 1=Title added entry
        // Ind2: 0-9 = nonfiling characters
        rules.insert(
            "245".to_string(),
            IndicatorRules {
                tag: "245".to_string(),
                indicator1: IndicatorValidation::Values(vec!['0', '1']),
                indicator2: IndicatorValidation::DigitRange { min: 0, max: 9 },
            },
        );

        // 490 - Series statement
        // Ind1: 0=Not traced, 1=Traced
        // Ind2: undefined
        rules.insert(
            "490".to_string(),
            IndicatorRules {
                tag: "490".to_string(),
                indicator1: IndicatorValidation::Values(vec!['0', '1']),
                indicator2: IndicatorValidation::Undefined,
            },
        );

        // 5XX - Notes (generally undefined)
        for tag in &[
            "500", "501", "502", "504", "505", "506", "508", "511", "520", "521", "522", "524",
            "525", "526", "530", "533", "535", "538", "541", "546", "547", "550", "552", "555",
            "556", "561", "562", "563", "565", "567", "570", "580", "581", "583", "586", "588",
        ] {
            let mut rule = undefined_undefined.clone();
            rule.tag = (*tag).to_string();
            rules.insert((*tag).to_string(), rule);
        }

        // 600 - Subject added entry -- Personal name
        // Ind1: 0=Forename, 1=Surname, 3=Family name
        // Ind2: 0=LCSH, 1=LCSH, 2=mesh, 3=nal, 4=source not specified, 5=Canadian subjects, 6=rvm, 7=Source in $2
        rules.insert(
            "600".to_string(),
            IndicatorRules {
                tag: "600".to_string(),
                indicator1: IndicatorValidation::Values(vec!['0', '1', '3']),
                indicator2: IndicatorValidation::Values(vec![
                    '0', '1', '2', '3', '4', '5', '6', '7',
                ]),
            },
        );

        // 610 - Subject added entry -- Corporate name
        // Ind1: 1=Jurisdiction, 2=Name in direct order
        // Ind2: 0-7 = thesaurus
        rules.insert(
            "610".to_string(),
            IndicatorRules {
                tag: "610".to_string(),
                indicator1: IndicatorValidation::Values(vec!['1', '2']),
                indicator2: IndicatorValidation::DigitRange { min: 0, max: 7 },
            },
        );

        // 611 - Subject added entry -- Meeting name
        // Ind1: 0=Inverted name, 1=Jurisdiction, 2=Name in direct order
        // Ind2: 0-7 = thesaurus
        rules.insert(
            "611".to_string(),
            IndicatorRules {
                tag: "611".to_string(),
                indicator1: IndicatorValidation::Values(vec!['0', '1', '2']),
                indicator2: IndicatorValidation::DigitRange { min: 0, max: 7 },
            },
        );

        // 650 - Subject added entry -- Topical term
        // Ind1: undefined
        // Ind2: 0-7 = thesaurus
        rules.insert(
            "650".to_string(),
            IndicatorRules {
                tag: "650".to_string(),
                indicator1: IndicatorValidation::Undefined,
                indicator2: IndicatorValidation::DigitRange { min: 0, max: 7 },
            },
        );

        // 651 - Subject added entry -- Geographic name
        // Ind1: undefined
        // Ind2: 0-7 = thesaurus
        rules.insert(
            "651".to_string(),
            IndicatorRules {
                tag: "651".to_string(),
                indicator1: IndicatorValidation::Undefined,
                indicator2: IndicatorValidation::DigitRange { min: 0, max: 7 },
            },
        );

        // 700 - Added entry -- Personal name
        // Ind1: 0=Forename, 1=Surname, 3=Family name
        // Ind2: #=No info, 2=Analytical entry
        rules.insert(
            "700".to_string(),
            IndicatorRules {
                tag: "700".to_string(),
                indicator1: IndicatorValidation::Values(vec!['0', '1', '3']),
                indicator2: IndicatorValidation::Values(vec!['#', ' ', '2']),
            },
        );

        // 710 - Added entry -- Corporate name
        // Ind1: 1=Jurisdiction, 2=Name in direct order
        // Ind2: #=No info, 2=Analytical entry
        rules.insert(
            "710".to_string(),
            IndicatorRules {
                tag: "710".to_string(),
                indicator1: IndicatorValidation::Values(vec!['1', '2']),
                indicator2: IndicatorValidation::Values(vec!['#', ' ', '2']),
            },
        );

        // 711 - Added entry -- Meeting name
        // Ind1: 0=Inverted, 1=Jurisdiction, 2=Name in direct order
        // Ind2: #=No info, 2=Analytical entry
        rules.insert(
            "711".to_string(),
            IndicatorRules {
                tag: "711".to_string(),
                indicator1: IndicatorValidation::Values(vec!['0', '1', '2']),
                indicator2: IndicatorValidation::Values(vec!['#', ' ', '2']),
            },
        );

        // 740 - Added entry -- Uncontrolled related/analytical title
        // Ind1: 0-9 = nonfiling characters
        // Ind2: #=No info, 2=Analytical entry
        rules.insert(
            "740".to_string(),
            IndicatorRules {
                tag: "740".to_string(),
                indicator1: IndicatorValidation::DigitRange { min: 0, max: 9 },
                indicator2: IndicatorValidation::Values(vec!['#', ' ', '2']),
            },
        );

        // 8XX - Added entries and notes for series, related works, etc
        for tag in &["800", "810", "811", "830", "840", "856"] {
            let mut rule = undefined_undefined.clone();
            rule.tag = (*tag).to_string();
            rules.insert((*tag).to_string(), rule);
        }

        rules
    }

    /// Validate a field's indicators
    ///
    /// # Errors
    ///
    /// Returns `Err` if indicators don't meet validation rules for this field tag.
    pub fn validate_field(&self, field: &Field) -> Result<()> {
        self.validate_indicators(&field.tag, field.indicator1, field.indicator2)
    }

    /// Validate indicators for a specific tag
    ///
    /// # Errors
    ///
    /// Returns `Err` if indicators don't meet validation rules.
    pub fn validate_indicators(&self, tag: &str, indicator1: char, indicator2: char) -> Result<()> {
        if let Some(rules) = self.rules.get(tag) {
            if !rules.indicator1.is_valid(indicator1) {
                return Err(MarcError::InvalidField(format!(
                    "Invalid indicator1 '{}' for field {}: expected {:?}",
                    indicator1, tag, rules.indicator1
                )));
            }
            if !rules.indicator2.is_valid(indicator2) {
                return Err(MarcError::InvalidField(format!(
                    "Invalid indicator2 '{}' for field {}: expected {:?}",
                    indicator2, tag, rules.indicator2
                )));
            }
        }
        // Fields without rules are assumed to accept any indicators

        Ok(())
    }

    /// Get the validation rules for a tag
    #[must_use]
    pub fn get_rules(&self, tag: &str) -> Option<&IndicatorRules> {
        self.rules.get(tag)
    }
}

impl Default for IndicatorValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undefined_indicator_validation() {
        let validation = IndicatorValidation::Undefined;
        assert!(validation.is_valid('#'));
        assert!(validation.is_valid(' '));
        assert!(!validation.is_valid('0'));
        assert!(!validation.is_valid('1'));
    }

    #[test]
    fn test_values_indicator_validation() {
        let validation = IndicatorValidation::Values(vec!['0', '1', '3']);
        assert!(validation.is_valid('0'));
        assert!(validation.is_valid('1'));
        assert!(validation.is_valid('3'));
        assert!(!validation.is_valid('2'));
        assert!(!validation.is_valid('#'));
    }

    #[test]
    fn test_digit_range_validation() {
        let validation = IndicatorValidation::DigitRange { min: 0, max: 9 };
        assert!(validation.is_valid('0'));
        assert!(validation.is_valid('5'));
        assert!(validation.is_valid('9'));
        assert!(!validation.is_valid('a'));
        assert!(!validation.is_valid('#'));
    }

    #[test]
    fn test_field_100_validation() {
        let validator = IndicatorValidator::new();
        assert!(validator.validate_indicators("100", '1', '#').is_ok());
        assert!(validator.validate_indicators("100", '0', ' ').is_ok());
        assert!(validator.validate_indicators("100", '3', '#').is_ok());
        assert!(validator.validate_indicators("100", '2', '#').is_err()); // Invalid
        assert!(validator.validate_indicators("100", '1', '0').is_err()); // Ind2 should be undefined
    }

    #[test]
    fn test_field_245_validation() {
        let validator = IndicatorValidator::new();
        assert!(validator.validate_indicators("245", '1', '0').is_ok());
        assert!(validator.validate_indicators("245", '0', '4').is_ok());
        assert!(validator.validate_indicators("245", '1', '9').is_ok());
        assert!(validator.validate_indicators("245", '2', '0').is_err()); // Invalid ind1
        assert!(validator.validate_indicators("245", '1', 'a').is_err()); // Invalid ind2
    }

    #[test]
    fn test_field_650_validation() {
        let validator = IndicatorValidator::new();
        assert!(validator.validate_indicators("650", '#', '0').is_ok());
        assert!(validator.validate_indicators("650", ' ', '7').is_ok());
        assert!(validator.validate_indicators("650", '#', '2').is_ok()); // Valid 0-7 range
        assert!(validator.validate_indicators("650", '0', '0').is_err()); // Invalid ind1
        assert!(validator.validate_indicators("650", '#', '8').is_err()); // Invalid ind2 (out of range)
    }

    #[test]
    fn test_field_700_validation() {
        let validator = IndicatorValidator::new();
        assert!(validator.validate_indicators("700", '1', '#').is_ok());
        assert!(validator.validate_indicators("700", '1', ' ').is_ok());
        assert!(validator.validate_indicators("700", '3', '2').is_ok());
        assert!(validator.validate_indicators("700", '1', '0').is_err()); // Invalid ind2
    }
}
