//! The Rust code shown on the documentation site.
//!
//! Every Rust code block on a page under `docs/` is included from a named
//! section of a file in this directory (or from a top-level test file), so
//! the code on the site is compiled, and where it can run against the test
//! fixtures, run, by `cargo test`. Each file holds the code for one page:
//! `docs/<dir>/<page>.md` has its code in `<page>.rs` here, with hyphens in
//! the page name written as underscores.
//!
//! A section is the code between `// --8<-- [start:name]` and
//! `// --8<-- [end:name]`, and a page includes it with
//! `--8<-- "doc_examples/<page>.rs:name"` inside a `rust` code fence. The
//! code around a section supplies what the page's prose provides: a
//! fragment that uses a `record` sits in a function taking `record: Record`,
//! and a complete program sits in its own module. Complete programs that
//! open files such as `records.mrc` are compiled but not run.
//!
//! See `docs/contributing/testing.md` for the workflow.

// Complete programs are compiled but never called; fragments bind values only
// to show the call that produces them; and a fragment's function takes the
// values the page's prose defines (`record`, `leader`, `graph`) by value, as
// the page has them, even where the fragment only borrows them.
#![allow(dead_code, unused_variables, clippy::needless_pass_by_value)]
// The examples return `mrrc::Result` as readers' code does (the library allows
// this lint for the same type), and they nest `if let` rather than use let
// chains so that they compile in edition 2021 crates.
#![allow(clippy::result_large_err, clippy::collapsible_if)]

mod bibframe_conversion;
mod concurrency;
mod development_setup;
mod encoding;
mod error_codes;
mod formal_methods;
mod format_conversion;
mod formats;
mod index;
mod marc_primer;
mod querying_fields;
mod quickstart_rust;
mod reading_records;
mod rust_api;
mod testing;
mod validators;
mod working_with_large_files;
mod writing_records;

/// Fixture records the fragments run against.
mod fixtures {
    use mrrc::{MarcReader, Record};
    use std::fs::File;

    /// Path of a fixture file of 105 varied bibliographic records.
    pub const RECORDS_PATH: &str = "tests/data/fixtures/fidelity_test_100.mrc";

    /// Every record in [`RECORDS_PATH`].
    pub fn records() -> Vec<Record> {
        let file = File::open(RECORDS_PATH).expect("open fixture file");
        let mut reader = MarcReader::new(file);
        let mut records = Vec::new();
        while let Some(record) = reader.read_record().expect("read fixture record") {
            records.push(record);
        }
        records
    }
}
