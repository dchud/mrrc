//! LOC MODS-to-MARC XSLT conformance tests.
//!
//! These tests verify that `mods_xml_to_record()` produces MARC fields that match
//! the output of the LOC MODS 3.8 → MARCXML XSLT stylesheet for representative
//! MODS test fixtures. Indicator values are allowed to differ since MODS does not
//! preserve MARC indicators.

use mrrc::mods::mods_xml_to_record;

/// Helper: assert a data field exists with given tag and subfield value.
fn assert_subfield(record: &mrrc::Record, tag: &str, code: char, expected: &str) {
    let fields = record
        .get_fields(tag)
        .unwrap_or_else(|| panic!("Expected field {tag} not found"));
    let found = fields.iter().any(|f| {
        f.subfields
            .iter()
            .any(|sf| sf.code == code && sf.value == expected)
    });
    assert!(
        found,
        "Field {tag}${code} with value \"{expected}\" not found. Fields: {fields:?}",
    );
}

/// Simple monograph: title, personal name, originInfo, physicalDescription,
/// abstract, subjects, ISBN, language.
///
/// LOC XSLT reference output:
///   245 $a "Rust Systems Programming" $b "A Practical Guide"
///   100 $a "Smith, Jane " $d "1975-" $e "creator"
///   260 $a "San Francisco" $b "O'Reilly Media" $c "2020"
///   300 $a "xv, 450 pages"
///   520 $a "An introduction to systems programming..."
///   650 $a "Rust (Computer program language)"
///   650 $a "Systems programming (Computer science)"
///   020 $a "9781491927285"
#[test]
fn test_conformance_simple_monograph() {
    let xml = include_str!("data/mods/simple_monograph.xml");
    let record = mods_xml_to_record(xml).expect("Failed to parse simple_monograph");

    // Title
    assert_subfield(&record, "245", 'a', "Rust Systems Programming");
    assert_subfield(&record, "245", 'b', "A Practical Guide");

    // Personal name as creator → 100
    assert_subfield(&record, "100", 'a', "Smith, Jane");
    assert_subfield(&record, "100", 'd', "1975-");
    assert_subfield(&record, "100", 'e', "creator");

    // Origin info → 260
    assert_subfield(&record, "260", 'a', "San Francisco");
    assert_subfield(&record, "260", 'b', "O'Reilly Media");
    assert_subfield(&record, "260", 'c', "2020");

    // Physical description → 300
    assert_subfield(&record, "300", 'a', "xv, 450 pages");
    assert_subfield(&record, "300", 'c', "24 cm");

    // Abstract → 520
    assert_subfield(
        &record,
        "520",
        'a',
        "An introduction to systems programming using the Rust language.",
    );

    // Subjects → 650
    assert_subfield(&record, "650", 'a', "Rust (Computer program language)");
    assert_subfield(
        &record,
        "650",
        'a',
        "Systems programming (Computer science)",
    );

    // ISBN → 020
    assert_subfield(&record, "020", 'a', "9781491927285");

    // Language → 041
    assert_subfield(&record, "041", 'a', "eng");

    // Leader record type should be 'a' (text)
    assert_eq!(record.leader.record_type, 'a');
}

/// Multi-author work: multiple personal names, corporate name, edition,
/// ISBN, LCCN, classifications (LCC and DDC).
///
/// LOC XSLT reference output:
///   245 $a "Advanced Database Systems"
///   100 $a "Garcia, Maria " $e "creator"
///   700 $a "Chen, Wei " $e "author"
///   710 $a "University Press " $e "publisher"
///   260 $a "Cambridge" $b "University Press" $c "2021"
///   250 $a "2nd ed."
///   020 $a "9780521168120"
///   010 $a "2021012345"
///   050 $a "QA76.9.D3"
///   082 $a "005.74"
#[test]
fn test_conformance_multi_author() {
    let xml = include_str!("data/mods/multi_author.xml");
    let record = mods_xml_to_record(xml).expect("Failed to parse multi_author");

    // Title
    assert_subfield(&record, "245", 'a', "Advanced Database Systems");

    // First creator → 100
    assert_subfield(&record, "100", 'a', "Garcia, Maria");

    // Second author → 700 (100 already taken)
    assert_subfield(&record, "700", 'a', "Chen, Wei");

    // Corporate name (not a creator) → 710
    assert_subfield(&record, "710", 'a', "University Press");

    // Origin info
    assert_subfield(&record, "260", 'a', "Cambridge");
    assert_subfield(&record, "260", 'b', "University Press");
    assert_subfield(&record, "260", 'c', "2021");

    // Edition → 250
    assert_subfield(&record, "250", 'a', "2nd ed.");

    // Identifiers
    assert_subfield(&record, "020", 'a', "9780521168120");
    assert_subfield(&record, "010", 'a', "2021012345");

    // Classifications
    assert_subfield(&record, "050", 'a', "QA76.9.D3");
    assert_subfield(&record, "082", 'a', "005.74");
}

/// Record with subjects, classification, recordInfo, accessCondition,
/// tableOfContents, targetAudience.
///
/// LOC XSLT reference output:
///   245 $a "History of the American West"
///   100 $a "Johnson, Robert " $e "creator"
///   655 $a "History"
///   650 $a "Frontier and pioneer life"
///   651 $a "West (U.S.)"
///   650 $a "Native Americans"
///   020 $a "9780195071221"
///   001 "99887766"  003 "`OCoLC`"  040 $a "DLC"
///   506 $a "Open access"
///   505 $a "Chapter 1: The frontier -- Chapter 2: Expansion"
///   521 $a "General"
#[test]
fn test_conformance_with_subjects() {
    let xml = include_str!("data/mods/with_subjects.xml");
    let record = mods_xml_to_record(xml).expect("Failed to parse with_subjects");

    assert_subfield(&record, "245", 'a', "History of the American West");
    assert_subfield(&record, "100", 'a', "Johnson, Robert");

    // Genre → 655
    assert_subfield(&record, "655", 'a', "History");

    // Subjects
    assert_subfield(&record, "650", 'a', "Frontier and pioneer life");
    assert_subfield(&record, "651", 'a', "West (U.S.)");
    assert_subfield(&record, "650", 'a', "Native Americans");

    // ISBN
    assert_subfield(&record, "020", 'a', "9780195071221");

    // Record info → control fields
    assert_eq!(record.get_control_field("001"), Some("99887766"));
    assert_eq!(record.get_control_field("003"), Some("OCoLC"));
    assert_subfield(&record, "040", 'a', "DLC");

    // Access condition (restriction) → 506
    assert_subfield(&record, "506", 'a', "Open access");

    // Table of contents → 505
    assert_subfield(
        &record,
        "505",
        'a',
        "Chapter 1: The frontier -- Chapter 2: Expansion",
    );

    // Target audience → 521
    assert_subfield(&record, "521", 'a', "General");
}

/// Record with relatedItem (host, series), location, accessCondition.
///
/// LOC XSLT reference output:
///   245 $a "Machine Learning Fundamentals"
///   100 $a "Lee, David " $e "creator"
///   260 $a "New York" $b "Springer" $c "2022"
///   300 $a "xii, 380 pages" $b "print" $c "25 cm"  (form may vary)
///   020 $a "9783030597849"
///   773 $t "Graduate Texts in Computer Science"
///   830 $a "Springer AI Series"
///   852 $a "MIT Libraries"
///   540 $a "Copyright 2022 Springer"
#[test]
fn test_conformance_with_related_items() {
    let xml = include_str!("data/mods/with_related_items.xml");
    let record = mods_xml_to_record(xml).expect("Failed to parse with_related_items");

    assert_subfield(&record, "245", 'a', "Machine Learning Fundamentals");
    assert_subfield(&record, "100", 'a', "Lee, David");

    // Origin info
    assert_subfield(&record, "260", 'a', "New York");
    assert_subfield(&record, "260", 'b', "Springer");
    assert_subfield(&record, "260", 'c', "2022");

    // Physical description
    assert_subfield(&record, "300", 'a', "xii, 380 pages");
    assert_subfield(&record, "300", 'c', "25 cm");

    // ISBN
    assert_subfield(&record, "020", 'a', "9783030597849");

    // Related items
    assert_subfield(&record, "773", 't', "Graduate Texts in Computer Science");
    assert_subfield(&record, "830", 'a', "Springer AI Series");

    // Location → 852
    assert_subfield(&record, "852", 'a', "MIT Libraries");

    // Access condition (use and reproduction) → 540
    assert_subfield(&record, "540", 'a', "Copyright 2022 Springer");
}
