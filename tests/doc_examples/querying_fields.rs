//! Code for `docs/tutorials/rust/querying-fields.md`.

use mrrc::Record;

mod basic_field_access {
    // --8<-- [start:basic-field-access]
    use mrrc::MarcReader;
    use std::fs::File;

    fn main() -> mrrc::Result<()> {
        let file = File::open("records.mrc")?;
        let mut reader = MarcReader::new(file);

        while let Some(record) = reader.read_record()? {
            // Get all fields with a tag
            for field in record.fields_by_tag("650") {
                if let Some(subject) = field.get_subfield('a') {
                    println!("{subject}");
                }
            }
        }
        Ok(())
    }
    // --8<-- [end:basic-field-access]
}

fn field_query(record: Record) {
    // --8<-- [start:field-query]
    use mrrc::FieldQuery;

    // Find fields by tag
    let query = FieldQuery::new().tag("245");

    for field in record.fields_matching(&query) {
        println!("{:?}", field.get_subfield('a'));
    }
    // --8<-- [end:field-query]
}

fn tag_range_query(record: Record) {
    // --8<-- [start:tag-range-query]
    use mrrc::FieldQuery;

    // Find all subject fields (600-699)
    let query = FieldQuery::new().tag_range("600", "699");

    for field in record.fields_matching_range(&query) {
        println!("{}: {:?}", field.tag, field.get_subfield('a'));
    }
    // --8<-- [end:tag-range-query]
}

fn filtering_by_indicators(record: Record) {
    // --8<-- [start:filtering-by-indicators]
    // Manual filtering by indicators
    let lcsh_subjects: Vec<_> = record
        .fields_by_tag("650")
        .filter(|f| f.indicator2 == '0') // LCSH
        .collect();

    for field in lcsh_subjects {
        if let Some(subject) = field.get_subfield('a') {
            println!("LCSH: {subject}");
        }
    }
    // --8<-- [end:filtering-by-indicators]
}

fn subfield_pattern_query(record: Record) {
    // --8<-- [start:subfield-pattern-query]
    use mrrc::SubfieldPatternQuery;

    // Find ISBN-13s: the $a of an 020 field starting with 978 or 979
    let query = SubfieldPatternQuery::new("020", 'a', r"^97[89]").expect("valid regex");

    for field in record.fields_matching_pattern(&query) {
        if let Some(isbn) = field.get_subfield('a') {
            println!("ISBN-13: {isbn}");
        }
    }
    // --8<-- [end:subfield-pattern-query]
}

fn subfield_value_query(record: Record) {
    // --8<-- [start:subfield-value-query]
    use mrrc::SubfieldValueQuery;

    // Exact match: 650 $a equal to "History"
    let exact = SubfieldValueQuery::new("650", 'a', "History");
    for field in record.fields_matching_value(&exact) {
        println!("Exact: {:?}", field.get_subfield('a'));
    }

    // Partial match: 650 $a containing "History"
    let partial = SubfieldValueQuery::partial("650", 'a', "History");
    for field in record.fields_matching_value(&partial) {
        println!("Partial: {:?}", field.get_subfield('a'));
    }
    // --8<-- [end:subfield-value-query]
}

mod combining_queries {
    // --8<-- [start:combining-queries]
    use mrrc::Record;

    fn find_lcsh_subjects_with_subdivision(record: &Record) -> Vec<String> {
        let mut results = Vec::new();

        for field in record.fields_by_tag("650") {
            // Must be LCSH (indicator2 = 0)
            if field.indicator2 != '0' {
                continue;
            }

            // Must have both $a and $x
            let main_subject = field.get_subfield('a');
            let subdivision = field.get_subfield('x');

            if let (Some(main), Some(sub)) = (main_subject, subdivision) {
                results.push(format!("{main} -- {sub}"));
            }
        }

        results
    }
    // --8<-- [end:combining-queries]

    #[test]
    fn runs_against_fixtures() {
        for record in crate::fixtures::records() {
            find_lcsh_subjects_with_subdivision(&record);
        }
    }
}

mod complete_example {
    // --8<-- [start:complete-example]
    use mrrc::{MarcReader, RecordHelpers};
    use std::fs::File;

    fn find_records_about(path: &str, topic: &str) -> mrrc::Result<Vec<String>> {
        let file = File::open(path)?;
        let mut reader = MarcReader::new(file);
        let mut results = Vec::new();

        let topic_lower = topic.to_lowercase();

        while let Some(record) = reader.read_record()? {
            // Check LCSH subjects
            for field in record.fields_by_tag("650") {
                if field.indicator2 != '0' {
                    continue;
                }

                if let Some(subject) = field.get_subfield('a') {
                    if subject.to_lowercase().contains(&topic_lower) {
                        if let Some(title) = record.title() {
                            results.push(format!("{title}: {subject}"));
                        }
                        break;
                    }
                }
            }
        }

        Ok(results)
    }

    fn main() -> mrrc::Result<()> {
        let results = find_records_about("library.mrc", "computers")?;

        for result in results {
            println!("{result}");
        }

        Ok(())
    }
    // --8<-- [end:complete-example]

    #[test]
    fn finds_fixture_records() {
        let results = find_records_about(crate::fixtures::RECORDS_PATH, "fiction").unwrap();
        assert!(!results.is_empty());
    }
}

#[test]
fn fragments_run_against_fixtures() {
    for record in crate::fixtures::records() {
        field_query(record.clone());
        tag_range_query(record.clone());
        filtering_by_indicators(record.clone());
        subfield_pattern_query(record.clone());
        subfield_value_query(record);
    }
}
