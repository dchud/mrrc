//! Reading MARC records and querying fields using the advanced DSL
//!
//! This example demonstrates reading MARC records and using the powerful field query API
//! to find and extract specific information from records.

use mrrc::{Field, Leader, Record};

fn main() {
    // Create a sample record for demonstration
    let record = create_sample_record();

    println!("\n=== Basic Field Access ===\n");
    basic_field_access(&record);

    println!("\n=== Filtering by Indicators ===\n");
    filter_by_indicators(&record);

    println!("\n=== Working with Subfields ===\n");
    working_with_subfields(&record);

    println!("\n=== Advanced Queries ===\n");
    advanced_queries(&record);
}

/// Helper function to create a default bibliographic leader
fn default_leader() -> Leader {
    Leader {
        record_length: 0,
        record_status: 'n',
        record_type: 'a',
        bibliographic_level: 'm',
        control_record_type: ' ',
        character_coding: ' ',
        indicator_count: 2,
        subfield_code_count: 2,
        data_base_address: 0,
        encoding_level: ' ',
        cataloging_form: 'a',
        multipart_level: ' ',
        reserved: "4500".to_string(),
    }
}

fn create_sample_record() -> Record {
    Record::builder(default_leader())
        .control_field_str("001", "ocm12345678")
        .control_field_str("008", "200101s2020    xxu||||||||||||||||eng||")
        .field(
            Field::builder("245".to_string(), '1', '0')
                .subfield_str('a', "Advanced Rust patterns /")
                .subfield_str('c', "Jane Smith.")
                .build(),
        )
        .field(
            Field::builder("650".to_string(), ' ', '0')
                .subfield_str('a', "Rust (Computer program language)")
                .build(),
        )
        .field(
            Field::builder("650".to_string(), ' ', '0')
                .subfield_str('a', "Programming languages")
                .subfield_str('x', "Design and construction.")
                .build(),
        )
        .field(
            Field::builder("650".to_string(), ' ', '1')
                .subfield_str('a', "Software engineering.")
                .build(),
        )
        .field(
            Field::builder("700".to_string(), '1', ' ')
                .subfield_str('a', "Jones, Bob,")
                .subfield_str('e', "editor.")
                .build(),
        )
        .build()
}

fn basic_field_access(record: &Record) {
    println!("Getting fields by tag:");

    // Get all 650 (subject) fields
    if let Some(subjects) = record.get_fields("650") {
        println!("Found {} subject fields", subjects.len());
        for field in subjects {
            if let Some(value) = field.get_subfield('a') {
                println!("  Subject: {value}");
            }
        }
    }

    // Get the first 245 field
    if let Some(field) = record.get_field("245") {
        println!(
            "\nTitle field indicators: {}{}",
            field.indicator1, field.indicator2
        );
        for subfield in field.subfields() {
            println!("  ${}: {}", subfield.code, subfield.value);
        }
    }
}

fn filter_by_indicators(record: &Record) {
    println!("Filtering 650 fields by second indicator:\n");

    // Get only LCSH (indicator 2 = '0') subjects
    let lcsh_subjects: Vec<_> = record
        .fields_by_tag("650")
        .filter(|f| f.indicator2 == '0')
        .collect();

    println!("LCSH subjects (indicator2 = '0'): {}", lcsh_subjects.len());
    for field in lcsh_subjects {
        if let Some(value) = field.get_subfield('a') {
            println!("  {value}");
        }
    }

    // Get non-LCSH subjects
    let other_subjects: Vec<_> = record
        .fields_by_tag("650")
        .filter(|f| f.indicator2 != '0')
        .collect();

    println!(
        "\nOther subjects (indicator2 != '0'): {}",
        other_subjects.len()
    );
    for field in other_subjects {
        if let Some(value) = field.get_subfield('a') {
            println!("  {value}");
        }
    }
}

fn working_with_subfields(record: &Record) {
    println!("Subfield operations:\n");

    // Get all values for subfield 'a' in 650 fields
    let all_subjects: Vec<&str> = record
        .fields_by_tag("650")
        .filter_map(|f| f.get_subfield('a'))
        .collect();

    println!("All subfield 'a' values in 650 fields:");
    for subject in all_subjects {
        println!("  {subject}");
    }

    // Get 650 fields with subfield 'x' (topical subdivision)
    println!("\n650 fields with topical subdivisions:");
    for field in record.fields_by_tag("650") {
        if let Some(main) = field.get_subfield('a') {
            if let Some(subd) = field.get_subfield('x') {
                println!("  {main} -- {subd}");
            }
        }
    }
}

fn advanced_queries(record: &Record) {
    println!("Using the advanced field query API:\n");

    // Get 650 fields with specific subfield 'a'
    println!("650 fields containing subfield 'a':");
    for field in record.fields_with_subfield("650", 'a') {
        let value = field.get_subfield('a').unwrap_or("N/A");
        println!("  {}: {}", field.tag, value);
    }

    // Get multiple fields by range
    println!("\nAccess points (1XX, 6XX, 7XX range):");
    for field in record.fields_in_range("100", "799") {
        println!("  {}: {}", field.tag, field.indicator1);
    }

    // Count fields by tag
    println!("\nField count summary:");
    let mut tag_counts: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    for field in record.fields() {
        *tag_counts.entry(field.tag.clone()).or_insert(0) += 1;
    }

    for (tag, count) in tag_counts.iter().take(10) {
        println!("  {tag}: {count} field(s)");
    }
}
