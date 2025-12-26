//! Field collection traits for managing MARC record field collections.
//!
//! This module provides traits that standardize field collection management
//! across different record types, reducing code duplication.

use crate::record::Field;

/// A trait for managing a collection of MARC fields.
///
/// This trait defines the operations needed to manage field collections
/// in a consistent way across different record types. Each implementation
/// manages a specific `Vec<Field>` collection.
///
/// # Examples
///
/// ```ignore
/// impl FieldCollection for AuthorityRecord {
///     fn add_field_to_collection(&mut self, field: Field, name: &str) {
///         match name {
///             "see_from" => self.tracings_see_from.push(field),
///             "see_also" => self.tracings_see_also.push(field),
///             _ => {}
///         }
///     }
///
///     fn get_collection(&self, name: &str) -> Option<&[Field]> {
///         match name {
///             "see_from" => Some(&self.tracings_see_from),
///             "see_also" => Some(&self.tracings_see_also),
///             _ => None
///         }
///     }
/// }
/// ```
pub trait FieldCollection {
    /// Add a field to a named collection.
    fn add_field_to_collection(&mut self, field: Field, name: &str);

    /// Get a reference to a named collection.
    fn get_collection(&self, name: &str) -> Option<&[Field]>;
}

/// Helper to reduce boilerplate for simple field accessor pairs.
///
/// This provides a standardized way to implement simple add/get pairs
/// for field collections.
#[derive(Debug)]
pub struct SimpleFieldCollectionHelper;

impl SimpleFieldCollectionHelper {
    /// Add a field to a mutable vector reference.
    #[inline]
    pub fn add_field(collection: &mut Vec<Field>, field: Field) {
        collection.push(field);
    }

    /// Get a slice reference from a vector.
    #[inline]
    #[must_use]
    pub fn get_collection(collection: &[Field]) -> &[Field] {
        collection
    }
}
