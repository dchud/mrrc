# Querying Fields (Rust)

Learn to use MRRC's Query DSL for complex field searching.

## Basic Field Access

```rust
use mrrc::MarcReader;
use std::fs::File;

fn main() -> mrrc::Result<()> {
    let file = File::open("records.mrc")?;
    let mut reader = MarcReader::new(file);

    while let Some(record) = reader.read_record()? {
        // Get all fields with a tag
        for field in record.fields_by_tag("650") {
            if let Some(subject) = field.subfield("a") {
                println!("{}", subject);
            }
        }
    }
    Ok(())
}
```

## FieldQuery (Tag Matching)

```rust
use mrrc::FieldQuery;

// Find fields by tag
let query = FieldQuery::tag("245");
let fields = query.find_all(&record);

for field in fields {
    println!("{:?}", field.subfield("a"));
}
```

## Tag Range Query

Find fields within a tag range:

```rust
use mrrc::FieldQuery;

// Find all subject fields (600-699)
let query = FieldQuery::tag_range("600", "699");
let subjects = query.find_all(&record);

for field in subjects {
    println!("{}: {:?}", field.tag(), field.subfield("a"));
}
```

## Filtering by Indicators

```rust
// Manual filtering by indicators
let lcsh_subjects: Vec<_> = record
    .fields_by_tag("650")
    .into_iter()
    .filter(|f| f.indicator2() == '0')  // LCSH
    .collect();

for field in lcsh_subjects {
    if let Some(subject) = field.subfield("a") {
        println!("LCSH: {}", subject);
    }
}
```

## Subfield Pattern Query

Match subfield values with patterns:

```rust
use mrrc::SubfieldPatternQuery;
use regex::Regex;

// Find ISBN-13s (start with 978 or 979)
let pattern = Regex::new(r"^97[89]").unwrap();

for field in record.fields_by_tag("020") {
    if let Some(isbn) = field.subfield("a") {
        if pattern.is_match(isbn) {
            println!("ISBN-13: {}", isbn);
        }
    }
}
```

## Subfield Value Query

Match exact or partial values:

```rust
// Exact match
for field in record.fields_by_tag("650") {
    if let Some(subject) = field.subfield("a") {
        if subject == "History" {
            println!("Found exact match");
        }
    }
}

// Partial match (contains)
for field in record.fields_by_tag("650") {
    if let Some(subject) = field.subfield("a") {
        if subject.to_lowercase().contains("history") {
            println!("Found: {}", subject);
        }
    }
}
```

## Combining Queries

```rust
fn find_lcsh_subjects_with_subdivision(record: &Record) -> Vec<String> {
    let mut results = Vec::new();

    for field in record.fields_by_tag("650") {
        // Must be LCSH (indicator2 = 0)
        if field.indicator2() != '0' {
            continue;
        }

        // Must have both $a and $x
        let main_subject = field.subfield("a");
        let subdivision = field.subfield("x");

        if let (Some(main), Some(sub)) = (main_subject, subdivision) {
            results.push(format!("{} -- {}", main, sub));
        }
    }

    results
}
```

## Complete Example

```rust
use mrrc::{MarcReader, Record};
use std::fs::File;

fn find_records_about(path: &str, topic: &str) -> mrrc::Result<Vec<String>> {
    let file = File::open(path)?;
    let mut reader = MarcReader::new(file);
    let mut results = Vec::new();

    let topic_lower = topic.to_lowercase();

    while let Some(record) = reader.read_record()? {
        // Check LCSH subjects
        for field in record.fields_by_tag("650") {
            if field.indicator2() != '0' {
                continue;
            }

            if let Some(subject) = field.subfield("a") {
                if subject.to_lowercase().contains(&topic_lower) {
                    if let Some(title) = record.title() {
                        results.push(format!("{}: {}", title, subject));
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
        println!("{}", result);
    }

    Ok(())
}
```

## Performance Tips

- Use specific tags when possible (faster than ranges)
- Filter by tag first, then by indicator, then by content
- For large datasets, consider parallel processing with Rayon

## Next Steps

- [Reading Records](reading-records.md) - Basic record access
- [Concurrency](concurrency.md) - Parallel processing
- [Rust API Reference](../../reference/rust-api.md) - Full API documentation
