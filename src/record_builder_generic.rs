//! Generic builder for all MARC record types.
//!
//! This module provides a unified `GenericRecordBuilder<T>` that works with any
//! MARC record type implementing the `MarcRecord` trait, eliminating code duplication
//! across `RecordBuilder`, `AuthorityRecordBuilder`, and `HoldingsRecordBuilder`.

use crate::marc_record::MarcRecord;

/// Generic builder for constructing MARC records of any type.
///
/// This builder uses the `MarcRecord` trait to provide a unified interface for
/// building any MARC record type (bibliographic, authority, or holdings).
/// It accepts a function to create a new record and provides fluent methods for
/// adding control fields and data fields.
///
/// # Examples
///
/// ```ignore
/// use mrrc::{GenericRecordBuilder, Record, AuthorityRecord, Leader, Field};
///
/// // Build a bibliographic Record
/// let record = GenericRecordBuilder::new(Record::new(Leader::default()))
///     .control_field("001", "12345")
///     .control_field("003", "OCoLC")
///     .build();
///
/// // Build an AuthorityRecord - same interface!
/// let auth = GenericRecordBuilder::new(AuthorityRecord::new(Leader::default()))
///     .control_field("001", "auth001")
///     .control_field("003", "DLC")
///     .build();
/// ```
#[derive(Debug)]
pub struct GenericRecordBuilder<T: MarcRecord> {
    record: T,
}

impl<T: MarcRecord> GenericRecordBuilder<T> {
    /// Create a new builder with an existing record.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mrrc::{GenericRecordBuilder, Record, Leader};
    ///
    /// let record = Record::new(Leader::default());
    /// let builder = GenericRecordBuilder::new(record);
    /// ```
    #[must_use]
    pub fn new(record: T) -> Self {
        GenericRecordBuilder { record }
    }

    /// Add or replace a control field (000-009).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mrrc::{GenericRecordBuilder, Record, Leader};
    ///
    /// let builder = GenericRecordBuilder::new(Record::new(Leader::default()))
    ///     .control_field("001", "12345")
    ///     .control_field("003", "OCoLC");
    /// ```
    #[must_use]
    pub fn control_field(mut self, tag: impl Into<String>, value: impl Into<String>) -> Self {
        self.record.add_control_field(tag, value);
        self
    }

    /// Get a mutable reference to the underlying record.
    ///
    /// This allows modifying the record directly before building it.
    /// Useful for accessing record-type-specific methods.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mrrc::{GenericRecordBuilder, Record, Leader, Field};
    ///
    /// let field = Field::new("245".to_string(), '1', '0');
    /// let mut builder = GenericRecordBuilder::new(Record::new(Leader::default()))
    ///     .control_field("001", "12345");
    /// builder.record_mut().add_field(field);
    /// ```
    pub fn record_mut(&mut self) -> &mut T {
        &mut self.record
    }

    /// Get a reference to the underlying record.
    pub fn record(&self) -> &T {
        &self.record
    }

    /// Build and return the record.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mrrc::{GenericRecordBuilder, Record, Leader};
    ///
    /// let record = GenericRecordBuilder::new(Record::new(Leader::default()))
    ///     .control_field("001", "12345")
    ///     .build();
    /// ```
    #[must_use]
    pub fn build(self) -> T {
        self.record
    }
}
