//! Converting MARC records between different formats
//!
//! This example demonstrates how to convert MARC records to various serialization formats:
//! JSON, MARCJSON, XML, CSV, Dublin Core, and MODS.

use mrrc::{Field, Leader, Record};

fn main() {
    let record = create_sample_record();

    println!("\n=== Original Record ===\n");
    println!("Title: {}", record.title().unwrap_or("Unknown"));
    println!("Authors: {:?}", record.authors());

    // JSON format
    println!("\n=== JSON Format ===\n");
    if let Ok(json) = mrrc::json::record_to_json(&record) {
        println!("{json}");
    }

    // MARCJSON format
    println!("\n=== MARCJSON Format ===\n");
    if let Ok(json) = mrrc::marcjson::record_to_marcjson(&record) {
        println!("{json}");
    }

    // XML format
    println!("\n=== XML Format ===\n");
    if let Ok(xml) = mrrc::xml::record_to_xml(&record) {
        println!("{xml}");
    }

    // CSV format
    println!("\n=== CSV Format ===\n");
    if let Ok(csv) = mrrc::csv::record_to_csv(&record) {
        println!("{csv}");
    }

    // Dublin Core format
    println!("\n=== Dublin Core Format ===\n");
    if let Ok(dc) = mrrc::dublin_core::record_to_dublin_core(&record) {
        println!("Title: {:?}", dc.title);
        println!("Creator: {:?}", dc.creator);
        println!("Subject: {:?}", dc.subject);
        println!("Date: {:?}", dc.date);
    }

    // MODS format
    println!("\n=== MODS Format ===\n");
    if let Ok(mods) = mrrc::mods::record_to_mods_xml(&record) {
        println!("{mods}");
    }
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

fn create_sample_record() -> Record {
    Record::builder(default_leader())
        .control_field_str("001", "ocm12345678")
        .control_field_str("008", "200101s2020    xxu||||||||||||||||eng||")
        .field(
            Field::builder("245".to_string(), '1', '0')
                .subfield_str('a', "Rust systems programming /")
                .subfield_str('c', "Prepared by Jane Smith and Bob Jones.")
                .build(),
        )
        .field(
            Field::builder("100".to_string(), '1', ' ')
                .subfield_str('a', "Smith, Jane,")
                .subfield_str('d', "1975-")
                .subfield_str('e', "author.")
                .build(),
        )
        .field(
            Field::builder("700".to_string(), '1', ' ')
                .subfield_str('a', "Jones, Bob,")
                .subfield_str('d', "1980-")
                .subfield_str('e', "author.")
                .build(),
        )
        .field(
            Field::builder("260".to_string(), ' ', ' ')
                .subfield_str('a', "San Francisco :")
                .subfield_str('b', "O'Reilly Media,")
                .subfield_str('c', "2020.")
                .build(),
        )
        .field(
            Field::builder("300".to_string(), ' ', ' ')
                .subfield_str('a', "xv, 450 pages ;")
                .subfield_str('c', "24 cm.")
                .build(),
        )
        .field(
            Field::builder("020".to_string(), ' ', ' ')
                .subfield_str('a', "9781491927285")
                .build(),
        )
        .field(
            Field::builder("650".to_string(), ' ', '0')
                .subfield_str('a', "Rust (Computer program language)")
                .build(),
        )
        .field(
            Field::builder("650".to_string(), ' ', '0')
                .subfield_str('a', "Systems programming (Computer science)")
                .build(),
        )
        .field(
            Field::builder("650".to_string(), ' ', '0')
                .subfield_str('a', "C (Computer program language)")
                .build(),
        )
        .field(
            Field::builder("655".to_string(), ' ', '7')
                .subfield_str('a', "Handbooks and manuals.")
                .subfield_str('2', "lcgft")
                .build(),
        )
        .build()
}
