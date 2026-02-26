//! Creating MARC records with the builder API
//!
//! This example demonstrates the recommended patterns for creating MARC records using the builder API.
//! The builder pattern provides a fluent, idiomatic Rust interface for record construction.

use mrrc::{Field, Leader, Record, RecordHelpers};

fn main() {
    // Example 1: Creating a simple bibliographic record
    simple_record();

    // Example 2: Creating a record with multiple subjects
    record_with_subjects();

    // Example 3: Creating a record with complex field structures
    complex_record();
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

fn simple_record() {
    println!("\n=== Simple Bibliographic Record ===\n");

    let record = Record::builder(default_leader())
        .control_field_str("001", "9780061120084")
        .control_field_str("008", "051029s2005    xxu||||||||||||||||eng||")
        .field(
            Field::builder("245".to_string(), '1', '0')
                .subfield_str('a', "To Kill a Mockingbird /")
                .subfield_str('c', "Harper Lee.")
                .build(),
        )
        .field(
            Field::builder("100".to_string(), '1', ' ')
                .subfield_str('a', "Lee, Harper,")
                .subfield_str('d', "1926-2016,")
                .subfield_str('e', "author.")
                .build(),
        )
        .field(
            Field::builder("650".to_string(), ' ', '0')
                .subfield_str('a', "Psychological fiction.")
                .build(),
        )
        .field(
            Field::builder("650".to_string(), ' ', '0')
                .subfield_str('a', "Legal stories.")
                .build(),
        )
        .build();

    println!("Record Type: {}", record.leader.record_type);
    println!(
        "Control Number: {}",
        record.get_control_field("001").unwrap_or("N/A")
    );

    if let Some(title) = record.title() {
        println!("Title: {title}");
    }

    if let Some(author) = record.author() {
        println!("Author: {author}");
    }

    println!("Subjects:");
    for subject in record.subjects() {
        println!("  - {subject}");
    }
}

fn record_with_subjects() {
    println!("\n=== Record with Multiple Subjects ===\n");

    let record = Record::builder(default_leader())
        .control_field_str("001", "12345678")
        .field(
            Field::builder("245".to_string(), '1', '0')
                .subfield_str('a', "Introduction to quantum mechanics")
                .build(),
        )
        .field(
            Field::builder("650".to_string(), ' ', '0')
                .subfield_str('a', "Quantum mechanics")
                .subfield_str('v', "Textbooks.")
                .build(),
        )
        .field(
            Field::builder("650".to_string(), ' ', '0')
                .subfield_str('a', "Physics")
                .subfield_str('x', "Study and teaching")
                .subfield_str('z', "Higher.")
                .build(),
        )
        .field(
            Field::builder("650".to_string(), ' ', '0')
                .subfield_str('a', "Quantum theory.")
                .build(),
        )
        .build();

    println!("Subjects with subdivisions:");
    for field in record.fields_by_tag("650") {
        if let Some(main) = field.get_subfield('a') {
            println!("  Main: {main}");

            if let Some(subdivision) = field.get_subfield('x') {
                println!("    Topical subdivision: {subdivision}");
            }
            if let Some(subdivision) = field.get_subfield('z') {
                println!("    Geographic subdivision: {subdivision}");
            }
            if let Some(form) = field.get_subfield('v') {
                println!("    Form subdivision: {form}");
            }
        }
    }
}

fn complex_record() {
    println!("\n=== Complex Bibliographic Record ===\n");

    let record = Record::builder(default_leader())
        .control_field_str("001", "ocm00123456")
        .control_field_str("005", "20051229123456.0")
        .control_field_str("008", "051229s2005    xxu||||||||||||||||eng||")
        // Main entry - personal name
        .field(
            Field::builder("100".to_string(), '1', ' ')
                .subfield_str('a', "Doe, John,")
                .subfield_str('d', "1950-")
                .subfield_str('e', "author.")
                .build(),
        )
        // Title statement
        .field(
            Field::builder("245".to_string(), '1', '4')
                .subfield_str('a', "The guide to advanced Rust programming /")
                .subfield_str('c', "John Doe.")
                .build(),
        )
        // Publication, distribution
        .field(
            Field::builder("260".to_string(), ' ', ' ')
                .subfield_str('a', "New York :") // Place of publication
                .subfield_str('b', "O'Reilly Media,") // Publisher
                .subfield_str('c', "2005.") // Date of publication
                .build(),
        )
        // Physical description
        .field(
            Field::builder("300".to_string(), ' ', ' ')
                .subfield_str('a', "xix, 400 pages :") // Extent
                .subfield_str('b', "color illustrations ;") // Other physical details
                .subfield_str('c', "24 cm") // Dimensions
                .build(),
        )
        // ISBN
        .field(
            Field::builder("020".to_string(), ' ', ' ')
                .subfield_str('a', "9780596004957")
                .build(),
        )
        // Subject headings
        .field(
            Field::builder("650".to_string(), ' ', '0')
                .subfield_str('a', "Rust (Computer program language)")
                .build(),
        )
        .field(
            Field::builder("650".to_string(), ' ', '0')
                .subfield_str('a', "Programming languages")
                .subfield_str('x', "Handbooks, manuals, etc.")
                .build(),
        )
        // Added entry - personal name
        .field(
            Field::builder("700".to_string(), '1', ' ')
                .subfield_str('a', "Smith, Jane,")
                .subfield_str('d', "1960-")
                .subfield_str('e', "editor.")
                .build(),
        )
        .build();

    println!("Title: {}", record.title().unwrap_or("Unknown"));
    println!("Author: {}", record.author().unwrap_or("Unknown"));

    if let Some(publication) = record.publication_info() {
        println!(
            "Published: {} in {}",
            publication.date.as_deref().unwrap_or("unknown date"),
            publication.place.as_deref().unwrap_or("unknown place"),
        );
        if let Some(publisher) = &publication.publisher {
            println!("Publisher: {publisher}");
        }
    }

    println!("\nSubjects:");
    for subject in record.subjects() {
        println!("  {subject}");
    }

    println!("\nISBN:");
    for isbn in record.isbns() {
        println!("  {isbn}");
    }
}
