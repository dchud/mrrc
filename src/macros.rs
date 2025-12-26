//! Macros for code generation in MARC record types.
//!
//! This module provides macros to reduce boilerplate in record type implementations,
//! particularly for field accessor pairs and collection management.

/// Macro to generate add/get accessor methods for a field collection.
///
/// This macro is designed to be used inside impl blocks and generates two methods:
/// - `add_<add_method_name>()` - adds a field to the collection (mutable)
/// - `<get_method_name>()` - gets the collection (immutable)
///
/// Both methods are marked with `#[must_use]` for the getter where appropriate.
///
/// # Example
///
/// ```ignore
/// pub struct MyRecord {
///     my_fields: Vec<Field>,
/// }
///
/// impl MyRecord {
///     // Use the macro to generate add_my_field() and my_fields() methods
///     define_field_accessors!(my_fields, add_my_field, my_fields);
/// }
/// ```
#[macro_export]
macro_rules! define_field_accessors {
    ($field_name:ident, $add_method:ident, $get_method:ident) => {
        /// Add a field to this collection.
        pub fn $add_method(&mut self, field: $crate::record::Field) {
            self.$field_name.push(field);
        }

        /// Get all fields from this collection.
        #[must_use]
        pub fn $get_method(&self) -> &[$crate::record::Field] {
            &self.$field_name
        }
    };
}

/// Macro to generate filtered accessor methods for a field collection by tag.
///
/// This generates a method that filters and returns fields matching a specific tag.
///
/// # Example
///
/// ```ignore
/// filtered_field_accessor!(notes, "670", source_data_found);
/// ```
#[macro_export]
macro_rules! filtered_field_accessor {
    ($field_name:ident, $tag:expr, $method_name:ident) => {
        /// Get fields filtered by tag.
        #[must_use]
        pub fn $method_name(&self) -> Vec<&$crate::record::Field> {
            self.$field_name.iter().filter(|f| f.tag == $tag).collect()
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::record::Field;

    struct TestRecord {
        fields: Vec<Field>,
    }

    impl TestRecord {
        define_field_accessors!(fields, add_field_test, fields_test);
    }

    #[test]
    fn test_define_field_accessors_macro() {
        let mut record = TestRecord { fields: Vec::new() };

        let field = Field::new("245".to_string(), '1', '0');
        record.add_field_test(field);

        assert_eq!(record.fields_test().len(), 1);
    }
}
