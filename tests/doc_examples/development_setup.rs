//! Code for `docs/contributing/development-setup.md`.

use mrrc::Result;

// The template's body is a placeholder that cannot fail.
#[allow(clippy::unnecessary_wraps)]
// --8<-- [start:doc-comment-template]
/// Brief one-line description.
///
/// More detail: what it does, when to use it, and any notable behavior.
///
/// # Examples
///
/// ```
/// # use mrrc::{Record, Leader};
/// let record = Record::builder(Leader::default()).build();
/// assert_eq!(record.fields().count(), 0);
/// ```
///
/// # Errors
///
/// Returns an error if [condition].
///
/// # Panics
///
/// Panics if [unlikely condition].
pub fn my_function() -> Result<()> {
    // ...
    Ok(())
}
// --8<-- [end:doc-comment-template]
