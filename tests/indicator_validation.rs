//! Comprehensive test suite for MARC21 field indicator validation.
//!
//! This test suite validates both the syntax (allowed values) and semantics
//! (meaning of values) of indicators across all common MARC21 fields.

use mrrc::validation::IndicatorValidator;

#[test]
fn test_validate_personal_names() {
    let validator = IndicatorValidator::new();

    // 100 - Main entry -- Personal name
    assert!(validator.validate_indicators("100", '0', '#').is_ok());
    assert!(validator.validate_indicators("100", '1', ' ').is_ok());
    assert!(validator.validate_indicators("100", '3', '#').is_ok());
    assert!(validator.validate_indicators("100", '2', '#').is_err());

    // 600 - Subject added entry -- Personal name
    assert!(validator.validate_indicators("600", '0', '0').is_ok());
    assert!(validator.validate_indicators("600", '1', '7').is_ok());
    assert!(validator.validate_indicators("600", '3', '2').is_ok());
    assert!(validator.validate_indicators("600", '0', '8').is_err()); // ind2 out of range

    // 700 - Added entry -- Personal name
    assert!(validator.validate_indicators("700", '0', '#').is_ok());
    assert!(validator.validate_indicators("700", '1', ' ').is_ok());
    assert!(validator.validate_indicators("700", '3', '2').is_ok());
}

#[test]
fn test_validate_corporate_names() {
    let validator = IndicatorValidator::new();

    // 110 - Main entry -- Corporate name
    assert!(validator.validate_indicators("110", '1', '#').is_ok());
    assert!(validator.validate_indicators("110", '2', ' ').is_ok());
    assert!(validator.validate_indicators("110", '0', '#').is_err());

    // 610 - Subject added entry -- Corporate name
    assert!(validator.validate_indicators("610", '1', '0').is_ok());
    assert!(validator.validate_indicators("610", '2', '7').is_ok());
    assert!(validator.validate_indicators("610", '0', '0').is_err()); // ind1 invalid

    // 710 - Added entry -- Corporate name
    assert!(validator.validate_indicators("710", '1', '#').is_ok());
    assert!(validator.validate_indicators("710", '2', ' ').is_ok());
    assert!(validator.validate_indicators("710", '2', '2').is_ok());
}

#[test]
fn test_validate_meeting_names() {
    let validator = IndicatorValidator::new();

    // 111 - Main entry -- Meeting name
    assert!(validator.validate_indicators("111", '0', '#').is_ok());
    assert!(validator.validate_indicators("111", '1', ' ').is_ok());
    assert!(validator.validate_indicators("111", '2', '#').is_ok());
    assert!(validator.validate_indicators("111", '3', '#').is_err());

    // 611 - Subject added entry -- Meeting name
    assert!(validator.validate_indicators("611", '0', '0').is_ok());
    assert!(validator.validate_indicators("611", '1', '7').is_ok());
    assert!(validator.validate_indicators("611", '2', '2').is_ok());

    // 711 - Added entry -- Meeting name
    assert!(validator.validate_indicators("711", '0', '#').is_ok());
    assert!(validator.validate_indicators("711", '1', ' ').is_ok());
    assert!(validator.validate_indicators("711", '2', '2').is_ok());
}

#[test]
fn test_validate_title_fields() {
    let validator = IndicatorValidator::new();

    // 130 - Main entry -- Uniform title
    assert!(validator.validate_indicators("130", '0', '#').is_ok());
    assert!(validator.validate_indicators("130", '9', ' ').is_ok());
    assert!(validator.validate_indicators("130", 'a', '#').is_err()); // ind1 not a digit

    // 240 - Uniform title
    assert!(validator.validate_indicators("240", '0', '0').is_ok());
    assert!(validator.validate_indicators("240", '1', '9').is_ok());
    assert!(validator.validate_indicators("240", '2', '0').is_err()); // ind1 invalid

    // 245 - Title statement
    assert!(validator.validate_indicators("245", '0', '0').is_ok());
    assert!(validator.validate_indicators("245", '1', '9').is_ok());
    assert!(validator.validate_indicators("245", '2', '0').is_err()); // ind1 invalid

    // 740 - Added entry -- Uncontrolled related/analytical title
    assert!(validator.validate_indicators("740", '0', '#').is_ok());
    assert!(validator.validate_indicators("740", '9', '2').is_ok());
    assert!(validator.validate_indicators("740", 'a', '#').is_err()); // ind1 not a digit
}

#[test]
fn test_validate_topical_subjects() {
    let validator = IndicatorValidator::new();

    // 650 - Subject added entry -- Topical term
    assert!(validator.validate_indicators("650", '#', '0').is_ok());
    assert!(validator.validate_indicators("650", ' ', '7').is_ok());
    assert!(validator.validate_indicators("650", '0', '0').is_err()); // ind1 must be undefined

    // 651 - Subject added entry -- Geographic name
    assert!(validator.validate_indicators("651", '#', '0').is_ok());
    assert!(validator.validate_indicators("651", ' ', '7').is_ok());
    assert!(validator.validate_indicators("651", '0', '0').is_err()); // ind1 must be undefined
}

#[test]
fn test_validate_series() {
    let validator = IndicatorValidator::new();

    // 490 - Series statement
    assert!(validator.validate_indicators("490", '0', '#').is_ok());
    assert!(validator.validate_indicators("490", '1', ' ').is_ok());
    assert!(validator.validate_indicators("490", '2', '#').is_err()); // ind1 invalid
}

#[test]
fn test_validate_control_fields() {
    let validator = IndicatorValidator::new();

    // Control fields typically have undefined indicators
    assert!(validator.validate_indicators("010", '#', '#').is_ok());
    assert!(validator.validate_indicators("010", ' ', ' ').is_ok());
    assert!(validator.validate_indicators("010", '0', '#').is_err()); // ind1 must be undefined

    assert!(validator.validate_indicators("020", '#', '#').is_ok());
    assert!(validator.validate_indicators("020", ' ', ' ').is_ok());
    assert!(validator.validate_indicators("020", 'a', '#').is_err()); // ind1 must be undefined
}

#[test]
fn test_semantic_meanings_title_fields() {
    let validator = IndicatorValidator::new();

    // 245 - Title statement
    assert_eq!(
        validator.get_indicator_meaning("245", 1, '0'),
        Some("No title added entry".to_string())
    );
    assert_eq!(
        validator.get_indicator_meaning("245", 1, '1'),
        Some("Title added entry".to_string())
    );

    // 130 and 240 have nonfiling characters (numeric), not semantic meanings
    assert_eq!(validator.get_indicator_meaning("130", 1, '5'), None);
    assert_eq!(validator.get_indicator_meaning("240", 2, '0'), None);
}

#[test]
fn test_semantic_meanings_name_fields() {
    let validator = IndicatorValidator::new();

    // 100 - Personal name
    assert_eq!(
        validator.get_indicator_meaning("100", 1, '0'),
        Some("Forename".to_string())
    );
    assert_eq!(
        validator.get_indicator_meaning("100", 1, '1'),
        Some("Surname".to_string())
    );
    assert_eq!(
        validator.get_indicator_meaning("100", 1, '3'),
        Some("Family name".to_string())
    );

    // 110 - Corporate name
    assert_eq!(
        validator.get_indicator_meaning("110", 1, '1'),
        Some("Jurisdiction".to_string())
    );
    assert_eq!(
        validator.get_indicator_meaning("110", 1, '2'),
        Some("Name in direct order".to_string())
    );

    // 111 - Meeting name
    assert_eq!(
        validator.get_indicator_meaning("111", 1, '0'),
        Some("Inverted name".to_string())
    );
    assert_eq!(
        validator.get_indicator_meaning("111", 1, '1'),
        Some("Jurisdiction".to_string())
    );
    assert_eq!(
        validator.get_indicator_meaning("111", 1, '2'),
        Some("Name in direct order".to_string())
    );
}

#[test]
fn test_semantic_meanings_subject_fields() {
    let validator = IndicatorValidator::new();

    // 600 - Subject -- Personal name (ind2 = thesaurus)
    assert_eq!(
        validator.get_indicator_meaning("600", 2, '0'),
        Some("LCSH".to_string())
    );
    assert_eq!(
        validator.get_indicator_meaning("600", 2, '2'),
        Some("MeSH".to_string())
    );
    assert_eq!(
        validator.get_indicator_meaning("600", 2, '7'),
        Some("Source in $2".to_string())
    );

    // 610 - Subject -- Corporate name (ind2 = thesaurus)
    assert_eq!(
        validator.get_indicator_meaning("610", 2, '0'),
        Some("LCSH".to_string())
    );
    assert_eq!(
        validator.get_indicator_meaning("610", 2, '5'),
        Some("Canadian subjects".to_string())
    );

    // 611 - Subject -- Meeting name (ind2 = thesaurus)
    assert_eq!(
        validator.get_indicator_meaning("611", 2, '0'),
        Some("LCSH".to_string())
    );
    assert_eq!(
        validator.get_indicator_meaning("611", 2, '6'),
        Some("RVM".to_string())
    );

    // 650 - Subject -- Topical term (ind2 = thesaurus)
    assert_eq!(
        validator.get_indicator_meaning("650", 2, '0'),
        Some("LCSH".to_string())
    );
    assert_eq!(
        validator.get_indicator_meaning("650", 2, '3'),
        Some("NAL".to_string())
    );

    // 651 - Subject -- Geographic name (ind2 = thesaurus)
    assert_eq!(
        validator.get_indicator_meaning("651", 2, '1'),
        Some("LCSH (conflict)".to_string())
    );
    assert_eq!(
        validator.get_indicator_meaning("651", 2, '4'),
        Some("Source not specified".to_string())
    );
}

#[test]
fn test_semantic_meanings_added_entries() {
    let validator = IndicatorValidator::new();

    // 700 - Added entry -- Personal name (ind2 = type)
    assert_eq!(
        validator.get_indicator_meaning("700", 1, '0'),
        Some("Forename".to_string())
    );
    assert_eq!(
        validator.get_indicator_meaning("700", 2, '#'),
        Some("No additional information".to_string())
    );
    assert_eq!(
        validator.get_indicator_meaning("700", 2, ' '),
        Some("No additional information".to_string())
    );
    assert_eq!(
        validator.get_indicator_meaning("700", 2, '2'),
        Some("Analytical entry".to_string())
    );

    // 710 - Added entry -- Corporate name (ind2 = type)
    assert_eq!(
        validator.get_indicator_meaning("710", 1, '1'),
        Some("Jurisdiction".to_string())
    );
    assert_eq!(
        validator.get_indicator_meaning("710", 2, '#'),
        Some("No additional information".to_string())
    );

    // 711 - Added entry -- Meeting name (ind2 = type)
    assert_eq!(
        validator.get_indicator_meaning("711", 1, '0'),
        Some("Inverted".to_string())
    );
    assert_eq!(
        validator.get_indicator_meaning("711", 2, '2'),
        Some("Analytical entry".to_string())
    );
}

#[test]
fn test_get_all_meanings_100() {
    let validator = IndicatorValidator::new();
    let meanings = validator.get_indicator_meanings("100", 1);

    assert!(!meanings.is_empty());
    assert_eq!(meanings.len(), 3);

    // Check all three personal name types are present
    assert!(meanings.iter().any(|m| m.value == '0'));
    assert!(meanings.iter().any(|m| m.value == '1'));
    assert!(meanings.iter().any(|m| m.value == '3'));
}

#[test]
fn test_get_all_meanings_650_ind2() {
    let validator = IndicatorValidator::new();
    let meanings = validator.get_indicator_meanings("650", 2);

    assert!(!meanings.is_empty());
    assert_eq!(meanings.len(), 8); // 0-7 thesaurus codes

    // Verify all thesaurus codes are present
    for digit in 0..=7 {
        let digit_char = char::from_digit(digit, 10).unwrap();
        assert!(meanings.iter().any(|m| m.value == digit_char));
    }
}

#[test]
fn test_get_all_meanings_700_ind2() {
    let validator = IndicatorValidator::new();
    let meanings = validator.get_indicator_meanings("700", 2);

    assert!(!meanings.is_empty());
    assert_eq!(meanings.len(), 3); // '#', ' ', '2'

    assert!(meanings.iter().any(|m| m.value == '#'));
    assert!(meanings.iter().any(|m| m.value == ' '));
    assert!(meanings.iter().any(|m| m.value == '2'));
}

#[test]
fn test_no_meanings_for_nonfiling_chars() {
    let validator = IndicatorValidator::new();

    // Numeric indicators (nonfiling characters) don't have discrete meanings
    let meanings = validator.get_indicator_meanings("130", 1);
    assert!(meanings.is_empty());

    let meanings = validator.get_indicator_meanings("245", 2);
    assert!(meanings.is_empty());

    let meanings = validator.get_indicator_meanings("240", 2);
    assert!(meanings.is_empty());
}

#[test]
fn test_no_meanings_for_undefined_indicators() {
    let validator = IndicatorValidator::new();

    // Undefined indicators have no meanings
    let meanings = validator.get_indicator_meanings("100", 2);
    assert!(meanings.is_empty());

    let meanings = validator.get_indicator_meanings("650", 1);
    assert!(meanings.is_empty());

    let meanings = validator.get_indicator_meanings("010", 1);
    assert!(meanings.is_empty());
}

#[test]
fn test_invalid_indicator_numbers() {
    let validator = IndicatorValidator::new();

    // Only indicators 1 and 2 are valid
    assert_eq!(validator.get_indicator_meaning("100", 0, '0'), None);
    assert_eq!(validator.get_indicator_meaning("100", 3, '0'), None);

    assert!(validator.get_indicator_meanings("100", 0).is_empty());
    assert!(validator.get_indicator_meanings("100", 3).is_empty());
}

#[test]
fn test_unknown_field_tags() {
    let validator = IndicatorValidator::new();

    // Unknown tags should not validate or have meanings
    let meanings = validator.get_indicator_meanings("999", 1);
    assert!(meanings.is_empty());

    assert_eq!(validator.get_indicator_meaning("999", 1, '0'), None);

    // But fields without rules in the validator are assumed to accept any indicators
    assert!(validator.validate_indicators("999", '0', 'x').is_ok());
}

#[test]
fn test_series_added_entries() {
    let validator = IndicatorValidator::new();

    // 800 - Series added entry -- Personal name
    assert!(validator.validate_indicators("800", '#', '#').is_ok());
    assert!(validator.validate_indicators("800", '0', '0').is_err());

    // 810 - Series added entry -- Corporate name
    assert!(validator.validate_indicators("810", ' ', ' ').is_ok());
    assert!(validator.validate_indicators("810", '1', '1').is_err());

    // 811 - Series added entry -- Meeting name
    assert!(validator.validate_indicators("811", '#', '#').is_ok());
    assert!(validator.validate_indicators("811", '2', '2').is_err());
}

#[test]
fn test_all_rules_have_validator() {
    let validator = IndicatorValidator::new();

    // Verify that we can get rules for all defined tags
    let defined_tags = vec![
        "010", "020", "100", "110", "111", "130", "240", "245", "490", "600", "610", "611", "650",
        "651", "700", "710", "711", "740", "800", "810", "811", "830", "840", "856",
    ];

    for tag in defined_tags {
        let rules = validator.get_rules(tag);
        assert!(rules.is_some(), "Expected validation rules for field {tag}");
    }
}
