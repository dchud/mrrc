//! Working with MARC Authority and Holdings records
//!
//! This example demonstrates how to create and work with Authority records (Type Z)
//! and Holdings records (Types x/y/v/u), which are specialized record types for
//! maintaining authority data and item holdings information.

use mrrc::{AuthorityRecord, Field, HoldingsRecord, Leader};

fn main() {
    println!("\n=== Authority Records ===\n");
    working_with_authority_records();

    println!("\n=== Holdings Records ===\n");
    working_with_holdings_records();
}

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

fn working_with_authority_records() {
    // Create a sample authority record for a personal name
    let mut leader = default_leader();
    leader.record_type = 'z'; // Type z = Authority record

    let record = AuthorityRecord::builder(leader)
        .control_field("001".to_string(), "n79021850".to_string())
        .control_field(
            "008".to_string(),
            "840117n| acannaabn          |a ana".to_string(),
        )
        // Heading - main entry (1XX)
        .heading(
            Field::builder("100".to_string(), '1', ' ')
                .subfield_str('a', "Twain, Mark,")
                .subfield_str('d', "1835-1910.")
                .build(),
        )
        // See from tracings - variant names (4XX)
        .add_see_from(
            Field::builder("400".to_string(), '1', ' ')
                .subfield_str('a', "Clemens, Samuel Langhorne,")
                .subfield_str('d', "1835-1910.")
                .build(),
        )
        // See also from tracings - related headings (5XX)
        .add_see_also(
            Field::builder("500".to_string(), '1', ' ')
                .subfield_str('a', "Clemens, Olivia Langdon,")
                .subfield_str('d', "1845-1904.")
                .subfield_str('e', "Related name.")
                .build(),
        )
        // Source data found note (670)
        .add_note(
            Field::builder("670".to_string(), ' ', ' ')
                .subfield_str('a', "His Tom Sawyer abroad, 1894.")
                .build(),
        )
        .add_note(
            Field::builder("670".to_string(), ' ', ' ')
                .subfield_str('a', "His Following the equator, 1897.")
                .build(),
        )
        // General note (680)
        .add_note(
            Field::builder("680".to_string(), ' ', ' ')
                .subfield_str('a', "American writer and humorist.")
                .build(),
        )
        .build();

    println!("Authority Record Type: {}", record.leader.record_type);

    // Access the main heading
    if let Some(heading) = record.heading() {
        if let Some(name) = heading.get_subfield('a') {
            println!("Main Heading: {name}");
        }
        if let Some(dates) = heading.get_subfield('d') {
            println!("Dates: {dates}");
        }
    }

    // Access see-from tracings (variant forms)
    println!("\nVariant Forms (See From Tracings):");
    for field in record.see_from_tracings() {
        if let Some(name) = field.get_subfield('a') {
            println!("  {name}");
        }
    }

    // Access see-also tracings (related headings)
    println!("\nRelated Headings (See Also Tracings):");
    for field in record.see_also_tracings() {
        if let Some(name) = field.get_subfield('a') {
            println!("  {name}");
        }
    }

    // Access notes
    println!("\nNotes:");
    for field in record.source_data_found() {
        if let Some(note) = field.get_subfield('a') {
            println!("  Source found: {note}");
        }
    }

    for field in record.notes() {
        if let Some(note) = field.get_subfield('a') {
            println!("  General note: {note}");
        }
    }
}

fn working_with_holdings_records() {
    // Create a sample holdings record
    let mut leader = default_leader();
    leader.record_type = 'x'; // Type x = Single-part holdings

    let record = HoldingsRecord::builder(leader)
        .control_field("001".to_string(), "ocm00012345".to_string())
        .control_field(
            "008".to_string(),
            "050101c        8   ||| ||||||||||||eng||".to_string(),
        )
        // Holding institution (852)
        .add_field(
            Field::builder("852".to_string(), ' ', ' ')
                .subfield_str('a', "University Library")
                .subfield_str('b', "General Stacks")
                .subfield_str('c', "PS1302")
                .subfield_str('h', "Twain, Mark")
                .subfield_str('i', "Tom Sawyer")
                .build(),
        )
        // Item information (876)
        .add_field(
            Field::builder("876".to_string(), ' ', ' ')
                .subfield_str('a', "Item 1")
                .subfield_str('p', "Copy 1")
                .subfield_str('q', "1")
                .subfield_str('x', "On shelves")
                .build(),
        )
        // Textual holdings (866)
        .add_field(
            Field::builder("866".to_string(), ' ', ' ')
                .subfield_str('a', "General note about the holdings.")
                .build(),
        )
        .build();

    println!("Holdings Record Type: {}", record.leader.record_type);

    // Access location and call number (852 field)
    if let Some(fields) = record.get_fields("852") {
        if let Some(field) = fields.first() {
            println!("\nLocation Information:");
            if let Some(inst) = field.get_subfield('a') {
                println!("  Institution: {inst}");
            }
            if let Some(location) = field.get_subfield('b') {
                println!("  Location: {location}");
            }
            if let Some(call_num) = field.get_subfield('c') {
                println!("  Call number: {call_num}");
            }
        }
    }

    // Access item information (876 field)
    println!("\nItem Information:");
    if let Some(fields) = record.get_fields("876") {
        for field in fields {
            if let Some(id) = field.get_subfield('a') {
                println!("  Item ID: {id}");
            }
            if let Some(copy) = field.get_subfield('p') {
                println!("  Copy: {copy}");
            }
            if let Some(status) = field.get_subfield('x') {
                println!("  Status: {status}");
            }
        }
    }

    // Access textual notes (866 field)
    if let Some(fields) = record.get_fields("866") {
        if let Some(field) = fields.first() {
            if let Some(note) = field.get_subfield('a') {
                println!("\nHoldings Note: {note}");
            }
        }
    }
}
