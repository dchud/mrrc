//! Integration tests for the `RecordHelpers` trait
//!
//! Validates that `RecordHelpers` trait extension is available and working
//! across all record types (`Record`, `AuthorityRecord`, `HoldingsRecord`).

mod common;

use common::create_test_leader;
use mrrc::{AuthorityRecord, Field, HoldingsRecord, Record, RecordHelpers, Subfield};

fn create_test_record_with_title() -> Record {
    let mut record = Record::new(create_test_leader());
    record.add_control_field("001".to_string(), "12345".to_string());

    let mut title_field = Field::new("245".to_string(), '1', '0');
    title_field.subfields.push(Subfield {
        code: 'a',
        value: "The Great Gatsby".to_string(),
    });
    title_field.subfields.push(Subfield {
        code: 'c',
        value: "F. Scott Fitzgerald".to_string(),
    });
    record.add_field(title_field);

    record
}

#[test]
fn test_record_helpers_on_bibliographic_record() {
    let record = create_test_record_with_title();

    // Test title() helper
    assert_eq!(record.title(), Some("The Great Gatsby"));

    // Test control_number() helper
    assert_eq!(record.control_number(), Some("12345"));

    // Test is_book() helper
    assert!(record.is_book());

    // Test is_serial() helper
    assert!(!record.is_serial());
}

#[test]
fn test_record_helpers_title_with_responsibility() {
    let record = create_test_record_with_title();

    let (title, responsibility) = record.title_with_responsibility();
    assert_eq!(title, Some("The Great Gatsby"));
    assert_eq!(responsibility, Some("F. Scott Fitzgerald"));
}

#[test]
fn test_record_helpers_with_isbn() {
    let mut record = Record::new(create_test_leader());

    // Add ISBN field 020
    let mut isbn_field = Field::new("020".to_string(), ' ', ' ');
    isbn_field.subfields.push(Subfield {
        code: 'a',
        value: "978-0-7432-7356-5".to_string(),
    });
    record.add_field(isbn_field);

    assert_eq!(record.isbn(), Some("978-0-7432-7356-5"));

    // Test isbns() for multiple ISBNs
    let mut second_isbn = Field::new("020".to_string(), ' ', ' ');
    second_isbn.subfields.push(Subfield {
        code: 'a',
        value: "0-7432-7356-1".to_string(),
    });
    record.add_field(second_isbn);

    let all_isbns = record.isbns();
    assert_eq!(all_isbns.len(), 2);
    assert!(all_isbns.contains(&"978-0-7432-7356-5"));
    assert!(all_isbns.contains(&"0-7432-7356-1"));
}

#[test]
fn test_record_helpers_with_subjects() {
    let mut record = Record::new(create_test_leader());

    // Add subject headings
    let mut subject1 = Field::new("650".to_string(), ' ', '0');
    subject1.subfields.push(Subfield {
        code: 'a',
        value: "Literary fiction".to_string(),
    });
    record.add_field(subject1);

    let mut subject2 = Field::new("650".to_string(), ' ', '0');
    subject2.subfields.push(Subfield {
        code: 'a',
        value: "American fiction, 20th century".to_string(),
    });
    record.add_field(subject2);

    let subjects = record.subjects();
    assert_eq!(subjects.len(), 2);
    assert!(subjects.contains(&"Literary fiction"));
    assert!(subjects.contains(&"American fiction, 20th century"));
}

#[test]
fn test_record_helpers_with_authors() {
    let mut record = Record::new(create_test_leader());

    // Primary author in 100
    let mut author_100 = Field::new("100".to_string(), '1', ' ');
    author_100.subfields.push(Subfield {
        code: 'a',
        value: "Fitzgerald, F. Scott".to_string(),
    });
    record.add_field(author_100);

    // Secondary authors in 700
    let mut author_700_1 = Field::new("700".to_string(), '1', ' ');
    author_700_1.subfields.push(Subfield {
        code: 'a',
        value: "Smith, John".to_string(),
    });
    record.add_field(author_700_1);

    let mut author_700_2 = Field::new("700".to_string(), '1', ' ');
    author_700_2.subfields.push(Subfield {
        code: 'a',
        value: "Doe, Jane".to_string(),
    });
    record.add_field(author_700_2);

    assert_eq!(record.author(), Some("Fitzgerald, F. Scott"));

    let all_authors = record.authors();
    assert_eq!(all_authors.len(), 2);
    assert!(all_authors.contains(&"Smith, John"));
    assert!(all_authors.contains(&"Doe, Jane"));
}

#[test]
fn test_record_helpers_with_publication_info() {
    let mut record = Record::new(create_test_leader());

    // Add publication field 260
    let mut pub_field = Field::new("260".to_string(), ' ', '1');
    pub_field.subfields.push(Subfield {
        code: 'a',
        value: "New York :".to_string(),
    });
    pub_field.subfields.push(Subfield {
        code: 'b',
        value: "Scribner,".to_string(),
    });
    pub_field.subfields.push(Subfield {
        code: 'c',
        value: "2004.".to_string(),
    });
    record.add_field(pub_field);

    assert_eq!(record.publisher(), Some("Scribner,"));
    assert_eq!(record.place_of_publication(), Some("New York :"));

    if let Some(info) = record.publication_info() {
        assert_eq!(info.place, Some("New York :".to_string()));
        assert_eq!(info.publisher, Some("Scribner,".to_string()));
    } else {
        panic!("publication_info should return Some");
    }
}

#[test]
fn test_record_helpers_with_language() {
    let mut record = Record::new(create_test_leader());

    // Add 008 with language at positions 35-37
    // 008 field is 40 characters long: positions 0-39
    // Pos 0-5: date (6), Pos 6: type (1), Pos 7-10: more (4), Pos 11-34: padding (24),
    // Pos 35-37: language (3), Pos 38-39: final (2)
    let field_008 = "000101naeng                        eng  ".to_string();
    assert_eq!(
        field_008.len(),
        40,
        "008 field must be exactly 40 characters"
    );
    record.add_control_field("008".to_string(), field_008);

    assert_eq!(record.language(), Some("eng"));
}

#[test]
fn test_record_helpers_with_authority_record() {
    let mut auth_record = AuthorityRecord::new(create_test_leader());
    auth_record.add_control_field("001".to_string(), "auth12345".to_string());

    // Even though AuthorityRecord doesn't have the same field structure,
    // the trait methods should still be callable (they'll return None if fields don't exist)
    assert_eq!(auth_record.control_number(), Some("auth12345"));
    assert_eq!(auth_record.title(), None);
    assert!(!auth_record.is_serial()); // All non-'s' should be false
}

#[test]
fn test_record_helpers_with_holdings_record() {
    let mut holdings_leader = create_test_leader();
    // Holdings records typically have record_type 'x', 'y', 'v', or 'u' (Leader/06)
    holdings_leader.record_type = 'x';

    let mut holdings = HoldingsRecord::new(holdings_leader);
    holdings.add_control_field("001".to_string(), "hold12345".to_string());

    // HoldingsRecord should also support RecordHelpers methods
    assert_eq!(holdings.control_number(), Some("hold12345"));
    assert_eq!(holdings.title(), None);
    // Holdings records have record_type != 'a', so is_book should be false
    assert!(!holdings.is_book());
}

#[test]
fn test_record_helpers_is_music() {
    let mut music_leader = create_test_leader();
    music_leader.record_type = 'c'; // Music

    let record = Record::new(music_leader);
    assert!(record.is_music());
    assert!(!record.is_book());
}

#[test]
fn test_record_helpers_is_audiovisual() {
    let mut av_leader = create_test_leader();
    av_leader.record_type = 'g'; // Audiovisual

    let record = Record::new(av_leader);
    assert!(record.is_audiovisual());
    assert!(!record.is_book());
}

#[test]
fn test_record_helpers_physical_description() {
    let mut record = Record::new(create_test_leader());

    let mut phys_field = Field::new("300".to_string(), ' ', ' ');
    phys_field.subfields.push(Subfield {
        code: 'a',
        value: "416 pages :".to_string(),
    });
    record.add_field(phys_field);

    assert_eq!(record.physical_description(), Some("416 pages :"));
}

#[test]
fn test_record_helpers_series() {
    let mut record = Record::new(create_test_leader());

    let mut series_field = Field::new("490".to_string(), '1', ' ');
    series_field.subfields.push(Subfield {
        code: 'a',
        value: "The Classic Library".to_string(),
    });
    record.add_field(series_field);

    assert_eq!(record.series(), Some("The Classic Library"));
}

#[test]
fn test_record_helpers_issn() {
    let mut record = Record::new(create_test_leader());

    let mut issn_field = Field::new("022".to_string(), ' ', ' ');
    issn_field.subfields.push(Subfield {
        code: 'a',
        value: "0028-0836".to_string(),
    });
    record.add_field(issn_field);

    assert_eq!(record.issn(), Some("0028-0836"));
}

#[test]
fn test_record_helpers_lccn() {
    let mut record = Record::new(create_test_leader());

    let mut lccn_field = Field::new("010".to_string(), ' ', ' ');
    lccn_field.subfields.push(Subfield {
        code: 'a',
        value: "2004051930".to_string(),
    });
    record.add_field(lccn_field);

    assert_eq!(record.lccn(), Some("2004051930"));
}

#[test]
fn test_record_helpers_trait_available_on_all_types() {
    // This test verifies that the trait is available on all record types
    // by calling multiple methods on each type

    let mut bib = Record::new(create_test_leader());
    let mut auth = AuthorityRecord::new(create_test_leader());
    let mut hold = HoldingsRecord::new(create_test_leader());

    bib.add_control_field("001".to_string(), "bib001".to_string());
    auth.add_control_field("001".to_string(), "auth001".to_string());
    hold.add_control_field("001".to_string(), "hold001".to_string());

    // All should have RecordHelpers methods available
    assert_eq!(bib.control_number(), Some("bib001"));
    assert_eq!(auth.control_number(), Some("auth001"));
    assert_eq!(hold.control_number(), Some("hold001"));

    assert!(!bib.is_serial());
    assert!(!auth.is_serial());
    assert!(!hold.is_serial());
}
