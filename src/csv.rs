//! CSV serialization of MARC records.
//!
//! This module provides conversion of MARC records to CSV (Comma-Separated Values) format,
//! suitable for import into spreadsheet applications and data analysis tools.
//!
//! # API Patterns
//!
//! - **Single record**: [`record_to_csv`] - Converts a single `Record` to CSV
//! - **Batch records**: [`records_to_csv`] - Converts a slice of `Record`s to CSV with combined output
//! - **Filtered batch**: [`records_to_csv_filtered`] - Converts records to CSV with field filtering
//!
//! # Examples
//!
//! Single record:
//! ```ignore
//! use mrrc::{Record, Field, Leader, csv};
//!
//! let mut record = Record::new(Leader::default());
//! let mut field = Field::new("245".to_string(), '1', '0');
//! field.add_subfield('a', "Title".to_string());
//! record.add_field(field);
//!
//! let csv = csv::record_to_csv(&record)?;
//! println!("{}", csv);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Batch records:
//! ```ignore
//! use mrrc::{Record, Field, Leader, csv};
//!
//! let mut record = Record::new(Leader::default());
//! let mut field = Field::new("245".to_string(), '1', '0');
//! field.add_subfield('a', "Title".to_string());
//! record.add_field(field);
//!
//! let csv = csv::records_to_csv(&[record])?;
//! println!("{}", csv);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::fmt::Write;

use crate::error::Result;
use crate::record::Record;

/// Convert a single MARC record to CSV format.
///
/// Produces a CSV with one row per field/subfield combination, with columns:
/// - `tag`: The MARC field tag
/// - `ind1`: First indicator (or empty for control fields)
/// - `ind2`: Second indicator (or empty for control fields)
/// - `subfield_code`: The subfield code (or empty for control fields)
/// - `value`: The field or subfield value
///
/// Control fields (001-009) are output with empty indicator and subfield columns.
///
/// # Examples
///
/// ```ignore
/// use mrrc::{Record, Field, Leader, csv};
///
/// let mut record = Record::new(Leader::default());
/// record.add_control_field("001".to_string(), "12345".to_string());
///
/// let mut field = Field::new("245".to_string(), '1', '0');
/// field.add_subfield('a', "Title".to_string());
/// field.add_subfield('b', "Subtitle".to_string());
/// record.add_field(field);
///
/// let csv = csv::record_to_csv(&record)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns an error if the CSV cannot be written.
pub fn record_to_csv(record: &Record) -> Result<String> {
    records_to_csv(std::slice::from_ref(record))
}

/// Convert multiple MARC records to CSV format.
///
/// Produces a CSV with one row per field/subfield combination across all records, with columns:
/// - `tag`: The MARC field tag
/// - `ind1`: First indicator (or empty for control fields)
/// - `ind2`: Second indicator (or empty for control fields)
/// - `subfield_code`: The subfield code (or empty for control fields)
/// - `value`: The field or subfield value
///
/// Control fields (001-009) are output with empty indicator and subfield columns.
///
/// # Examples
///
/// ```ignore
/// use mrrc::{Record, Field, Leader, csv};
///
/// let mut record = Record::new(Leader::default());
/// record.add_control_field("001".to_string(), "12345".to_string());
///
/// let mut field = Field::new("245".to_string(), '1', '0');
/// field.add_subfield('a', "Title".to_string());
/// field.add_subfield('b', "Subtitle".to_string());
/// record.add_field(field);
///
/// let csv = csv::records_to_csv(&[record])?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns an error if the CSV cannot be written.
pub fn records_to_csv(records: &[Record]) -> Result<String> {
    let mut output = String::new();

    // Write header
    writeln!(output, "tag,ind1,ind2,subfield_code,value").ok();

    for record in records {
        // Write control fields
        for (tag, value) in &record.control_fields {
            // Escape value if needed
            let escaped_value = escape_csv_value(value);
            writeln!(output, "{tag},,,{escaped_value}").ok();
        }

        // Write data fields with subfields
        for (tag, field_list) in &record.fields {
            for field in field_list {
                if field.subfields.is_empty() {
                    // Write field row without subfields
                    writeln!(output, "{tag},{},{},", field.indicator1, field.indicator2).ok();
                } else {
                    // Write one row per subfield
                    for subfield in &field.subfields {
                        let escaped_value = escape_csv_value(&subfield.value);
                        writeln!(
                            output,
                            "{tag},{},{},{},{}",
                            field.indicator1, field.indicator2, subfield.code, escaped_value
                        )
                        .ok();
                    }
                }
            }
        }
    }

    Ok(output)
}

/// Convert MARC records to CSV format using a custom field selector.
///
/// Allows filtering which fields are exported to CSV. Only fields matching the
/// provided filter function will be included.
///
/// # Arguments
///
/// * `records` - The MARC records to export
/// * `filter` - A function that returns `true` for fields to include
///
/// # Examples
///
/// ```ignore
/// use mrrc::{Record, Field, Leader, csv};
///
/// let mut record = Record::new(Leader::default());
/// let mut field = Field::new("245".to_string(), '1', '0');
/// field.add_subfield('a', "Title".to_string());
/// record.add_field(field);
///
/// // Only export 245 field
/// let csv = csv::records_to_csv_filtered(&[record], |tag| tag == "245")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns an error if the CSV cannot be written.
pub fn records_to_csv_filtered<F>(records: &[Record], filter: F) -> Result<String>
where
    F: Fn(&str) -> bool,
{
    let mut output = String::new();

    // Write header
    writeln!(output, "tag,ind1,ind2,subfield_code,value").ok();

    for record in records {
        // Write control fields
        for (tag, value) in &record.control_fields {
            if filter(tag) {
                let escaped_value = escape_csv_value(value);
                writeln!(output, "{tag},,,{escaped_value}").ok();
            }
        }

        // Write data fields with subfields
        for (tag, field_list) in &record.fields {
            if filter(tag) {
                for field in field_list {
                    if field.subfields.is_empty() {
                        writeln!(output, "{tag},{},{},", field.indicator1, field.indicator2).ok();
                    } else {
                        for subfield in &field.subfields {
                            let escaped_value = escape_csv_value(&subfield.value);
                            writeln!(
                                output,
                                "{tag},{},{},{},{}",
                                field.indicator1, field.indicator2, subfield.code, escaped_value
                            )
                            .ok();
                        }
                    }
                }
            }
        }
    }

    Ok(output)
}

/// Escape a value for CSV output.
///
/// Wraps values in quotes if they contain commas, quotes, or newlines.
/// Quotes within the value are doubled.
fn escape_csv_value(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{Field, Record};
    use crate::Leader;

    fn make_test_leader() -> Leader {
        Leader {
            record_length: 1000,
            record_status: 'n',
            record_type: 'a',
            bibliographic_level: 'm',
            control_record_type: ' ',
            character_coding: 'a',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 100,
            encoding_level: ' ',
            cataloging_form: 'a',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        }
    }

    #[test]
    fn test_control_field_csv() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "12345".to_string());

        let csv = records_to_csv(&[record]).expect("Failed to generate CSV");

        assert!(csv.contains("001,,,12345"));
        assert!(csv.starts_with("tag,ind1,ind2,subfield_code,value\n"));
    }

    #[test]
    fn test_data_field_with_subfields() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Title".to_string());
        field.add_subfield('b', "Subtitle".to_string());
        record.add_field(field);

        let csv = records_to_csv(&[record]).expect("Failed to generate CSV");

        assert!(csv.contains("245,1,0,a,Title"));
        assert!(csv.contains("245,1,0,b,Subtitle"));
    }

    #[test]
    fn test_csv_escaping() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Title, with comma".to_string());
        record.add_field(field);

        let csv = records_to_csv(&[record]).expect("Failed to generate CSV");

        assert!(csv.contains("245,1,0,a,\"Title, with comma\""));
    }

    #[test]
    fn test_csv_quote_escaping() {
        let mut record = Record::new(make_test_leader());
        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Title \"quoted\"".to_string());
        record.add_field(field);

        let csv = records_to_csv(&[record]).expect("Failed to generate CSV");

        assert!(csv.contains("245,1,0,a,\"Title \"\"quoted\"\"\""));
    }

    #[test]
    fn test_csv_filtered() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "12345".to_string());

        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Title".to_string());
        record.add_field(field);

        let mut field2 = Field::new("650".to_string(), ' ', '0');
        field2.add_subfield('a', "Subject".to_string());
        record.add_field(field2);

        // Only include 245 field
        let csv =
            records_to_csv_filtered(&[record], |tag| tag == "245").expect("Failed to generate CSV");

        assert!(csv.contains("245,1,0,a,Title"));
        assert!(!csv.contains("650"));
        assert!(!csv.contains("001"));
    }

    #[test]
    fn test_multiple_records() {
        let mut record1 = Record::new(make_test_leader());
        record1.add_control_field("001".to_string(), "11111".to_string());

        let mut record2 = Record::new(make_test_leader());
        record2.add_control_field("001".to_string(), "22222".to_string());

        let csv = records_to_csv(&[record1, record2]).expect("Failed to generate CSV");

        assert!(csv.contains("001,,,11111"));
        assert!(csv.contains("001,,,22222"));
    }
}
