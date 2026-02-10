# Writing Records (Rust)

Learn to create MARC records from scratch using builder patterns.

## Creating a Record

Use the builder pattern for clean record construction:

```rust
use mrrc::{Record, Field, Leader};

let record = Record::builder()
    .leader(Leader::default())
    .control_field("001", "12345")
    .control_field("008", "200101s2020    xxu||||||||||||||||eng||")
    .field(
        Field::builder("245", '1', '0')
            .subfield('a', "The Great Gatsby /")
            .subfield('c', "F. Scott Fitzgerald.")
            .build()
    )
    .build();
```

## Building Fields

```rust
use mrrc::Field;

// Using builder
let title = Field::builder("245", '1', '0')
    .subfield('a', "Main title :")
    .subfield('b', "subtitle /")
    .subfield('c', "by Author.")
    .build();

// Direct construction
let mut author = Field::new("100".to_string(), '1', ' ');
author.add_subfield('a', "Fitzgerald, F. Scott,".to_string());
author.add_subfield('d', "1896-1940.".to_string());
```

## Configuring the Leader

```rust
use mrrc::Leader;

let mut leader = Leader::default();
leader.record_status = 'n';           // New record
leader.record_type = 'a';             // Language material
leader.bibliographic_level = 'm';     // Monograph
leader.character_coding = 'a';        // UTF-8

let record = Record::builder()
    .leader(leader)
    // ... fields
    .build();
```

## Writing to Files

```rust
use mrrc::MarcWriter;
use std::fs::File;

fn main() -> mrrc::Result<()> {
    let file = File::create("output.mrc")?;
    let mut writer = MarcWriter::new(file);

    writer.write_record(&record)?;

    Ok(())
}
```

## Write Multiple Records

```rust
use mrrc::{MarcReader, MarcWriter};
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
```

## Complete Example

```rust
use mrrc::{Record, Field, Leader, MarcWriter};
use std::fs::File;

fn create_book_record(
    title: &str,
    author: &str,
    isbn: &str,
    subjects: &[&str],
) -> Record {
    let mut leader = Leader::default();
    leader.record_status = 'n';
    leader.record_type = 'a';
    leader.bibliographic_level = 'm';
    leader.character_coding = 'a';

    let mut builder = Record::builder()
        .leader(leader)
        .control_field("001", &format!("mrrc-{}", isbn))
        .control_field("008", "200101s2020    xxu||||||||||||||||eng||");

    // ISBN
    builder = builder.field(
        Field::builder("020", ' ', ' ')
            .subfield('a', isbn)
            .build()
    );

    // Author
    if !author.is_empty() {
        builder = builder.field(
            Field::builder("100", '1', ' ')
                .subfield('a', author)
                .build()
        );
    }

    // Title
    let ind1 = if author.is_empty() { '0' } else { '1' };
    let mut title_field = Field::builder("245", ind1, '0')
        .subfield('a', title);
    if !author.is_empty() {
        title_field = title_field.subfield('c', &format!("by {}", author));
    }
    builder = builder.field(title_field.build());

    // Subjects
    for subject in subjects {
        builder = builder.field(
            Field::builder("650", ' ', '0')
                .subfield('a', subject)
                .build()
        );
    }

    builder.build()
}

fn main() -> mrrc::Result<()> {
    let record = create_book_record(
        "Introduction to MARC",
        "Smith, John",
        "9780123456789",
        &["MARC format", "Library science"],
    );

    let file = File::create("new_record.mrc")?;
    let mut writer = MarcWriter::new(file);
    writer.write_record(&record)?;

    println!("Record created successfully");
    Ok(())
}
```

## Next Steps

- [Reading Records](reading-records.md) - Read and access records
- [Format Conversion](format-conversion.md) - Convert to other formats
- [Rust API Reference](../../reference/rust-api.md) - Full API documentation
