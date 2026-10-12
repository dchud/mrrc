//! Code for `docs/tutorials/rust/writing-records.md`.

use mrrc::Record;

fn creating_a_record() -> mrrc::Result<Record> {
    // --8<-- [start:creating-a-record]
    use mrrc::{Field, Leader, Record};

    let record = Record::builder(Leader::from_bytes(b"00000nam a2200000 i 4500")?)
        .control_field_str("001", "12345")
        .control_field_str("008", "200101s2020    xxu||||||||||||||||eng||")
        .field(
            Field::builder("245".to_string(), '1', '0')
                .subfield_str('a', "The Great Gatsby /")
                .subfield_str('c', "F. Scott Fitzgerald.")
                .build(),
        )
        .build();
    // --8<-- [end:creating-a-record]
    Ok(record)
}

fn building_fields() {
    // --8<-- [start:building-fields]
    use mrrc::Field;

    // Using builder
    let title = Field::builder("245".to_string(), '1', '0')
        .subfield_str('a', "Main title :")
        .subfield_str('b', "subtitle /")
        .subfield_str('c', "by Author.")
        .build();

    // Direct construction
    let mut author = Field::new("100".to_string(), '1', ' ');
    author.add_subfield('a', "Fitzgerald, F. Scott,".to_string());
    author.add_subfield('d', "1896-1940.".to_string());
    // --8<-- [end:building-fields]
}

fn configuring_the_leader() -> mrrc::Result<Record> {
    // --8<-- [start:configuring-the-leader]
    use mrrc::{Leader, Record};

    let mut leader = Leader::from_bytes(b"00000nam a2200000 i 4500")?;
    leader.record_status = 'n'; // New record
    leader.record_type = 'a'; // Language material
    leader.bibliographic_level = 'm'; // Monograph
    leader.character_coding = 'a'; // UTF-8

    let record = Record::builder(leader)
        // ... fields
        .build();
    // --8<-- [end:configuring-the-leader]
    Ok(record)
}

fn writing_to_files(record: Record) -> mrrc::Result<()> {
    // --8<-- [start:writing-to-files]
    use mrrc::MarcWriter;
    use std::fs::File;

    let file = File::create("output.mrc")?;
    let mut writer = MarcWriter::new(file);

    writer.write_record(&record)?;
    // --8<-- [end:writing-to-files]
    Ok(())
}

mod write_multiple_records {
    // --8<-- [start:write-multiple-records]
    use mrrc::{MarcReader, MarcWriter, RecordHelpers};
    use std::fs::File;

    fn main() -> mrrc::Result<()> {
        let input = File::open("input.mrc")?;
        let output = File::create("output.mrc")?;

        let mut reader = MarcReader::new(input);
        let mut writer = MarcWriter::new(output);

        while let Some(record) = reader.read_record()? {
            if record.title().is_some() {
                writer.write_record(&record)?;
            }
        }

        Ok(())
    }
    // --8<-- [end:write-multiple-records]
}

mod complete_example {
    // --8<-- [start:complete-example]
    use mrrc::{Field, Leader, MarcWriter, Record};
    use std::fs::File;

    fn create_book_record(
        title: &str,
        author: &str,
        isbn: &str,
        subjects: &[&str],
    ) -> mrrc::Result<Record> {
        let mut leader = Leader::from_bytes(b"00000nam a2200000 i 4500")?;
        leader.record_status = 'n';
        leader.record_type = 'a';
        leader.bibliographic_level = 'm';
        leader.character_coding = 'a';

        let mut builder = Record::builder(leader)
            .control_field_str("001", &format!("mrrc-{isbn}"))
            .control_field_str("008", "200101s2020    xxu||||||||||||||||eng||");

        // ISBN
        builder = builder.field(
            Field::builder("020".to_string(), ' ', ' ')
                .subfield_str('a', isbn)
                .build(),
        );

        // Author
        if !author.is_empty() {
            builder = builder.field(
                Field::builder("100".to_string(), '1', ' ')
                    .subfield_str('a', author)
                    .build(),
            );
        }

        // Title
        let ind1 = if author.is_empty() { '0' } else { '1' };
        let mut title_field = Field::builder("245".to_string(), ind1, '0').subfield_str('a', title);
        if !author.is_empty() {
            title_field = title_field.subfield_str('c', &format!("by {author}"));
        }
        builder = builder.field(title_field.build());

        // Subjects
        for subject in subjects {
            builder = builder.field(
                Field::builder("650".to_string(), ' ', '0')
                    .subfield_str('a', subject)
                    .build(),
            );
        }

        Ok(builder.build())
    }

    fn main() -> mrrc::Result<()> {
        let record = create_book_record(
            "Introduction to MARC",
            "Smith, John",
            "9780123456789",
            &["MARC format", "Library science"],
        )?;

        let file = File::create("new_record.mrc")?;
        let mut writer = MarcWriter::new(file);
        writer.write_record(&record)?;

        println!("Record created successfully");
        Ok(())
    }
    // --8<-- [end:complete-example]

    #[test]
    fn builds_and_writes_a_record() {
        use mrrc::{MarcReader, RecordHelpers};

        let record = create_book_record(
            "Introduction to MARC",
            "Smith, John",
            "9780123456789",
            &["MARC format", "Library science"],
        )
        .unwrap();
        let mut bytes = Vec::new();
        MarcWriter::new(&mut bytes).write_record(&record).unwrap();
        let parsed = MarcReader::new(&bytes[..]).read_record().unwrap().unwrap();
        assert_eq!(parsed.title(), Some("Introduction to MARC"));
        assert_eq!(parsed.fields_by_tag("650").count(), 2);
    }
}

#[test]
fn fragments_run() {
    let record = creating_a_record().unwrap();
    assert_eq!(record.get_control_field("001"), Some("12345"));
    building_fields();
    let record = configuring_the_leader().unwrap();
    assert_eq!(record.leader.bibliographic_level, 'm');
}
