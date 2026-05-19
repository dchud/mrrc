//! Shared field-access surface for all MARC record types.
//!
//! [`Record`](crate::Record), [`AuthorityRecord`](crate::AuthorityRecord),
//! and [`HoldingsRecord`](crate::HoldingsRecord) all store data fields in
//! the same shape — `IndexMap<String, Vec<Field>>` keyed by tag — and all
//! expose a control-field accessor. The [`FieldAccess`] trait centralizes
//! the three derived accessors (`get_field`, `get_field_or_err`,
//! `get_fields`) so the three record types share a single implementation
//! and behave identically across all of them. Implementors provide just
//! two raw hooks: storage access and the record's control number.
//!
//! The trait is re-exported at the crate root; bring it into scope to
//! call the accessors:
//!
//! ```ignore
//! use mrrc::{FieldAccess, Record};
//! let record: Record = /* ... */;
//! let title = record.get_field("245");
//! ```
//!
//! Code that does `use mrrc::*` picks up the trait automatically.

use indexmap::IndexMap;

use crate::error::{MarcError, Result};
use crate::record::Field;

/// Tag-keyed field accessors shared across all MARC record types.
///
/// Implementors provide `fields_map` (storage borrow) and
/// `record_control_number` (the value to populate on
/// [`FieldNotFound`](MarcError::FieldNotFound) errors). The three
/// derived accessors are provided by default impls and behave
/// identically across all record types.
pub trait FieldAccess {
    /// Borrow the data-field storage map.
    fn fields_map(&self) -> &IndexMap<String, Vec<Field>>;

    /// Read the record's control number for error context — typically
    /// the value of the 001 control field, if present. Used to populate
    /// `record_control_number` on
    /// [`FieldNotFound`](MarcError::FieldNotFound) errors raised by
    /// [`get_field_or_err`](Self::get_field_or_err).
    fn record_control_number(&self) -> Option<String>;

    /// Get all fields with the given tag (returns slice when present).
    #[must_use]
    fn get_fields(&self, tag: &str) -> Option<&[Field]> {
        self.fields_map().get(tag).map(Vec::as_slice)
    }

    /// Get the first field with the given tag.
    #[must_use]
    fn get_field(&self, tag: &str) -> Option<&Field> {
        self.fields_map().get(tag).and_then(|v| v.first())
    }

    /// Get the first field with the given tag, returning
    /// [`MarcError::FieldNotFound`] (E105) when the tag is not present.
    ///
    /// [`get_field`](Self::get_field) returns `Option<&Field>` for
    /// pymarc-compatible callers that want a `None` sentinel; this
    /// `*_or_err` variant is for callers who want the typed E105 error
    /// with `record_control_number` and `field_tag` populated for
    /// diagnostic context.
    ///
    /// # Errors
    ///
    /// Returns [`MarcError::FieldNotFound`] when no field with `tag` is
    /// present in the record.
    fn get_field_or_err(&self, tag: &str) -> Result<&Field> {
        self.get_field(tag).ok_or_else(|| MarcError::FieldNotFound {
            record_index: None,
            record_control_number: self.record_control_number(),
            field_tag: tag.to_string(),
        })
    }
}
