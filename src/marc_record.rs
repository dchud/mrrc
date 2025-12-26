//! Core trait for all MARC record types.
//!
//! This module defines the `MarcRecord` trait that provides a common interface for
//! all MARC record types (bibliographic, authority, and holdings records).

use crate::leader::Leader;
use crate::record::Field;

/// Common trait for all MARC record types.
///
/// This trait defines the operations that all MARC records must support:
/// - Leader management
/// - Control field operations (fields 000-009)
/// - Data field operations (fields 010+)
///
/// This trait enables generic code that works with any MARC record type.
///
/// # Examples
///
/// ```ignore
/// use mrrc::{MarcRecord, Record, AuthorityRecord, Leader};
///
/// fn print_record_type<T: MarcRecord>(record: &T) {
///     println!("Record type: {}", record.leader().record_type);
/// }
///
/// let bib_record = Record::new(Leader::default());
/// let auth_record = AuthorityRecord::new(Leader::default());
///
/// print_record_type(&bib_record);
/// print_record_type(&auth_record);
/// ```
pub trait MarcRecord {
    /// Get a reference to the record's leader (24-byte header).
    fn leader(&self) -> &Leader;

    /// Get a mutable reference to the record's leader.
    fn leader_mut(&mut self) -> &mut Leader;

    /// Add or replace a control field (000-009).
    ///
    /// Control fields are single-valued fields containing fixed-length or
    /// variable-length data without subfields.
    ///
    /// # Arguments
    ///
    /// * `tag` - The field tag (e.g., "001", "003", "008")
    /// * `value` - The field value
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mrrc::{MarcRecord, Record, Leader};
    ///
    /// let mut record = Record::new(Leader::default());
    /// record.add_control_field("001", "12345");
    /// ```
    fn add_control_field(&mut self, tag: impl Into<String>, value: impl Into<String>);

    /// Get the value of a control field.
    ///
    /// Returns `None` if the field does not exist.
    ///
    /// # Arguments
    ///
    /// * `tag` - The field tag to look up
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mrrc::{MarcRecord, Record, Leader};
    ///
    /// let mut record = Record::new(Leader::default());
    /// record.add_control_field("001", "12345");
    /// assert_eq!(record.get_control_field("001"), Some("12345"));
    /// ```
    fn get_control_field(&self, tag: &str) -> Option<&str>;

    /// Iterate over all control fields.
    ///
    /// Returns an iterator of (tag, value) tuples for all control fields
    /// in tag order.
    fn control_fields_iter(&self) -> Box<dyn Iterator<Item = (&str, &str)> + '_>;

    /// Get all fields with a given tag.
    ///
    /// Returns a slice of all fields matching the tag, or `None` if no fields exist.
    #[must_use]
    fn get_fields(&self, tag: &str) -> Option<&[Field]>;

    /// Get the first field with a given tag.
    ///
    /// Returns the first field matching the tag, or `None` if no fields exist.
    #[must_use]
    fn get_field(&self, tag: &str) -> Option<&Field>;
}
