//! Code for `docs/tutorials/rust/reading-records.md`.

mod basic_reading {
    // --8<-- [start:basic-reading]
    use mrrc::{MarcReader, RecordHelpers};
    use std::fs::File;

    fn main() -> mrrc::Result<()> {
        let file = File::open("records.mrc")?;
        let mut reader = MarcReader::new(file);

        while let Some(record) = reader.read_record()? {
            if let Some(title) = record.title() {
                println!("Title: {title}");
            }
        }
        Ok(())
    }
    // --8<-- [end:basic-reading]
}

mod reading_from_memory {
    // --8<-- [start:reading-from-memory]
    use mrrc::{MarcReader, RecordHelpers};
    use std::io::Cursor;

    fn main() -> mrrc::Result<()> {
        let data = std::fs::read("records.mrc")?;
        let cursor = Cursor::new(data);
        let mut reader = MarcReader::new(cursor);

        while let Some(record) = reader.read_record()? {
            println!("{:?}", record.title());
        }
        Ok(())
    }
    // --8<-- [end:reading-from-memory]
}

mod accessing_fields {
    // --8<-- [start:accessing-fields]
    use mrrc::Record;

    fn process_record(record: &Record) {
        // Get first field by tag
        if let Some(field) = record.get_field("245") {
            if let Some(title) = field.get_subfield('a') {
                println!("Title: {title}");
            }
        }

        // Get all fields with a tag
        for field in record.fields_by_tag("650") {
            if let Some(subject) = field.get_subfield('a') {
                println!("Subject: {subject}");
            }
        }

        // Iterate over all fields
        for field in record.fields() {
            println!("Field {}: {} subfields", field.tag, field.subfields.len());
        }
    }
    // --8<-- [end:accessing-fields]

    #[test]
    fn runs_against_fixtures() {
        for record in crate::fixtures::records() {
            process_record(&record);
        }
    }
}

mod control_fields {
    // --8<-- [start:control-fields]
    use mrrc::Record;

    fn read_control_fields(record: &Record) {
        if let Some(control_num) = record.get_control_field("001") {
            println!("Control number: {control_num}");
        }

        if let Some(fixed) = record.get_control_field("008") {
            // Parse fixed-length data elements
            let pub_year = &fixed[7..11];
            let language = &fixed[35..38];
            println!("Published: {pub_year}, Language: {language}");
        }
    }
    // --8<-- [end:control-fields]

    #[test]
    fn runs_against_fixtures() {
        for record in crate::fixtures::records() {
            read_control_fields(&record);
        }
    }
}

mod convenience_methods {
    // --8<-- [start:convenience-methods]
    use mrrc::{Record, RecordHelpers};

    fn extract_metadata(record: &Record) {
        if let Some(title) = record.title() {
            println!("Title: {title}");
        }

        if let Some(author) = record.author() {
            println!("Author: {author}");
        }

        // Get all ISBNs
        for isbn in record.isbns() {
            println!("ISBN: {isbn}");
        }

        // Get all subjects
        for subject in record.subjects() {
            println!("Subject: {subject}");
        }
    }
    // --8<-- [end:convenience-methods]

    #[test]
    fn runs_against_fixtures() {
        for record in crate::fixtures::records() {
            extract_metadata(&record);
        }
    }
}

mod working_with_subfields {
    // --8<-- [start:working-with-subfields]
    use mrrc::Field;

    fn process_field(field: &Field) {
        // Get first subfield value
        if let Some(value) = field.get_subfield('a') {
            println!("$a: {value}");
        }

        // Get all values for a subfield code
        let values: Vec<&str> = field.subfields_by_code('a').collect();

        // Iterate over all subfields
        for subfield in field.subfields() {
            println!("${}: {}", subfield.code, subfield.value);
        }
    }
    // --8<-- [end:working-with-subfields]

    #[test]
    fn runs_against_fixtures() {
        for record in crate::fixtures::records() {
            for field in record.fields() {
                process_field(field);
            }
        }
    }
}

mod working_with_indicators {
    // --8<-- [start:working-with-indicators]
    use mrrc::Field;

    fn check_indicators(field: &Field) {
        let ind1 = field.indicator1;
        let ind2 = field.indicator2;

        // For 245: ind2 = nonfiling characters
        if field.tag == "245" {
            let skip = ind2.to_digit(10).unwrap_or(0) as usize;
            if let Some(title) = field.get_subfield('a') {
                // Skip characters, not bytes: titles are not all ASCII
                let filing_title: String = title.chars().skip(skip).collect();
                println!("Filing title: {filing_title}");
            }
        }
    }
    // --8<-- [end:working-with-indicators]

    #[test]
    fn runs_against_fixtures() {
        for record in crate::fixtures::records() {
            for field in record.fields() {
                check_indicators(field);
            }
        }
    }
}

mod error_handling {
    // --8<-- [start:error-handling]
    use mrrc::{MarcError, MarcReader, Result};

    fn process_file(path: &str) -> Result<usize> {
        let file = std::fs::File::open(path)?;
        let mut reader = MarcReader::new(file);
        let mut count = 0;

        while let Some(record) = reader.read_record()? {
            count += 1;
        }

        Ok(count)
    }

    fn main() {
        match process_file("records.mrc") {
            Ok(count) => println!("Processed {count} records"),
            Err(MarcError::IoError { cause, .. }) => eprintln!("I/O error: {cause}"),
            Err(MarcError::InvalidLeader { message, .. }) => {
                eprintln!("Invalid leader: {message}");
            },
            Err(e) => eprintln!("Error: {e}"),
        }
    }
    // --8<-- [end:error-handling]

    #[test]
    fn counts_fixture_records() {
        assert_eq!(process_file(crate::fixtures::RECORDS_PATH).unwrap(), 105);
    }
}

mod complete_example {
    // --8<-- [start:complete-example]
    use mrrc::{MarcReader, RecordHelpers};
    use std::fs::File;

    fn main() -> mrrc::Result<()> {
        let file = File::open("library.mrc")?;
        let mut reader = MarcReader::new(file);

        let mut books = 0;
        let mut serials = 0;

        while let Some(record) = reader.read_record()? {
            match record.leader.bibliographic_level {
                'm' => books += 1,
                's' => serials += 1,
                _ => {},
            }

            if let Some(title) = record.title() {
                println!("{title}");
            }
        }

        println!("\nSummary: {books} books, {serials} serials");
        Ok(())
    }
    // --8<-- [end:complete-example]
}
