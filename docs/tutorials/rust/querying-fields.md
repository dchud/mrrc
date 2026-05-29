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
            if let Some(subject) = field.get_subfield('a') {
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
let query = FieldQuery::new().tag("245");

for field in record.fields_matching(&query) {
    println!("{:?}", field.get_subfield('a'));
}
```

## Tag Range Query

Find fields within a tag range:

```rust
use mrrc::FieldQuery;

// Find all subject fields (600-699)
let query = FieldQuery::new().tag_range("600", "699");

for field in record.fields_matching_range(&query) {
    println!("{}: {:?}", field.tag, field.get_subfield('a'));
}
```

## Filtering by Indicators

```rust
// Manual filtering by indicators
let lcsh_subjects: Vec<_> = record
    .fields_by_tag("650")
    .filter(|f| f.indicator2 == '0')  // LCSH
    .collect();

for field in lcsh_subjects {
    if let Some(subject) = field.get_subfield('a') {
        println!("LCSH: {}", subject);
    }
}
```

## Subfield Pattern Query

`SubfieldPatternQuery` matches a subfield against a regular expression, so you
don't have to wire up `regex` yourself. Construction returns a `Result` (the
error is a `regex::Error`) because the pattern is compiled up front:

```rust
use mrrc::SubfieldPatternQuery;

// Find ISBN-13s: the $a of an 020 field starting with 978 or 979
let query = SubfieldPatternQuery::new("020", 'a', r"^97[89]")
    .expect("valid regex");

for field in record.fields_matching_pattern(&query) {
    if let Some(isbn) = field.get_subfield('a') {
        println!("ISBN-13: {}", isbn);
    }
}
```

## Subfield Value Query

`SubfieldValueQuery` matches a subfield's value directly. Use `new` for an exact
match and `partial` for a substring (both case-sensitive):

```rust
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
```

To test a single field instead of scanning the whole record, call
`query.matches(&field)`, which returns a `bool`.

## Combining Queries

```rust
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
            results.push(format!("{} -- {}", main, sub));
        }
    }

    results
}
```

## Complete Example

```rust
use mrrc::{MarcReader, Record, RecordHelpers};
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
