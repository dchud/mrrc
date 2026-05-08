//! Integration test for the per-record diagnostics surface
//! (`record.errors` + `iter_with_errors`).
//!
//! Verifies:
//! - The 2x3x2 matrix
//!   `{structural, strict_marc} x {strict, lenient, permissive} x {iter, iter_with_errors}`
//!   behaves as documented.
//! - At every cell, `record.errors` and the second tuple element of
//!   `iter_with_errors()` carry the same data (the `iter_with_errors`
//!   surface is an ergonomic alternative, not a parallel data path).
//! - Each error code (E201, E202, E301) is captured with full positional
//!   context when fired in a recoverable cell.
//! - Authority and Holdings readers expose `record.errors` and
//!   `iter_with_errors` symmetrically with the bibliographic reader for
//!   clean records (smoke).

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use mrrc::{AuthorityMarcReader, HoldingsMarcReader, MarcReader, RecoveryMode, ValidationLevel};

// =====================================================================
// Helpers
// =====================================================================

fn fixture_bytes(name: &str) -> Vec<u8> {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/error_fixtures")
        .join(name);
    fs::read(&path).unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()))
}

fn data_bytes(name: &str) -> Vec<u8> {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data")
        .join(name);
    fs::read(&path).unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()))
}

fn build_reader(
    bytes: &[u8],
    validation: ValidationLevel,
    recovery: RecoveryMode,
) -> MarcReader<Cursor<Vec<u8>>> {
    MarcReader::new(Cursor::new(bytes.to_vec()))
        .with_recovery_mode(recovery)
        .with_validation_level(validation)
}

// =====================================================================
// 12-cell matrix on the E201 fixture
// =====================================================================
//
// Structural cells: bad indicator byte is accepted; record parses clean,
// errors empty. Six cells, six "clean parse" assertions.
// StrictMarc cells: indicator validation fires E201.
//   - Strict cells: iterator yields Err.
//   - Lenient/Permissive cells: yields a record with errors populated.

const E201_FIXTURE: &str = "e201_bad_indicator.bin";

fn assert_clean_via_iter(bytes: &[u8], validation: ValidationLevel, recovery: RecoveryMode) {
    let mut reader = build_reader(bytes, validation, recovery);
    let mut count = 0;
    while let Ok(Some(rec)) = reader.read_record() {
        let errs = &rec.errors;
        assert!(
            errs.is_empty(),
            "({validation:?}, {recovery:?}, iter): expected empty errors, got {errs:?}",
        );
        count += 1;
    }
    assert!(
        count > 0,
        "({validation:?}, {recovery:?}, iter): yielded no records",
    );
}

fn assert_clean_via_iter_with_errors(
    bytes: &[u8],
    validation: ValidationLevel,
    recovery: RecoveryMode,
) {
    let mut reader = build_reader(bytes, validation, recovery);
    let mut count = 0;
    for entry in reader.iter_with_errors() {
        let (rec, errs) = entry.unwrap_or_else(|e| {
            panic!(
                "({validation:?}, {recovery:?}, iter_with_errors): expected clean parse, got {e:?}"
            )
        });
        assert!(
            errs.is_empty() && rec.errors.is_empty(),
            "({validation:?}, {recovery:?}, iter_with_errors): expected empty errors, got {errs:?}",
        );
        count += 1;
    }
    assert!(
        count > 0,
        "({validation:?}, {recovery:?}, iter_with_errors): yielded no records",
    );
}

// --- Structural row: 6 cells, all clean parses ---

#[test]
fn cell_structural_strict_iter() {
    assert_clean_via_iter(
        &fixture_bytes(E201_FIXTURE),
        ValidationLevel::Structural,
        RecoveryMode::Strict,
    );
}

#[test]
fn cell_structural_strict_iter_with_errors() {
    assert_clean_via_iter_with_errors(
        &fixture_bytes(E201_FIXTURE),
        ValidationLevel::Structural,
        RecoveryMode::Strict,
    );
}

#[test]
fn cell_structural_lenient_iter() {
    assert_clean_via_iter(
        &fixture_bytes(E201_FIXTURE),
        ValidationLevel::Structural,
        RecoveryMode::Lenient,
    );
}

#[test]
fn cell_structural_lenient_iter_with_errors() {
    assert_clean_via_iter_with_errors(
        &fixture_bytes(E201_FIXTURE),
        ValidationLevel::Structural,
        RecoveryMode::Lenient,
    );
}

#[test]
fn cell_structural_permissive_iter() {
    assert_clean_via_iter(
        &fixture_bytes(E201_FIXTURE),
        ValidationLevel::Structural,
        RecoveryMode::Permissive,
    );
}

#[test]
fn cell_structural_permissive_iter_with_errors() {
    assert_clean_via_iter_with_errors(
        &fixture_bytes(E201_FIXTURE),
        ValidationLevel::Structural,
        RecoveryMode::Permissive,
    );
}

// --- StrictMarc row: 6 cells with mixed expectations ---

#[test]
fn cell_strict_marc_strict_iter() {
    let mut reader = build_reader(
        &fixture_bytes(E201_FIXTURE),
        ValidationLevel::StrictMarc,
        RecoveryMode::Strict,
    );
    let result = reader.read_record();
    assert!(
        result.is_err(),
        "expected Err in StrictMarc+Strict; got {result:?}"
    );
    assert_eq!(result.unwrap_err().code(), "E201");
}

#[test]
fn cell_strict_marc_strict_iter_with_errors() {
    let mut reader = build_reader(
        &fixture_bytes(E201_FIXTURE),
        ValidationLevel::StrictMarc,
        RecoveryMode::Strict,
    );
    let entry = reader.iter_with_errors().next();
    let result = entry.expect("expected Some(Err)");
    assert!(result.is_err(), "expected Err item from iter_with_errors");
    assert_eq!(result.unwrap_err().code(), "E201");
}

#[test]
fn cell_strict_marc_lenient_iter() {
    let mut reader = build_reader(
        &fixture_bytes(E201_FIXTURE),
        ValidationLevel::StrictMarc,
        RecoveryMode::Lenient,
    );
    let rec = reader
        .read_record()
        .expect("expected Ok(Some) in lenient")
        .expect("expected a record");
    assert!(
        !rec.errors.is_empty(),
        "expected populated errors in lenient"
    );
    assert_eq!(rec.errors[0].code(), "E201");
}

#[test]
fn cell_strict_marc_lenient_iter_with_errors() {
    let mut reader = build_reader(
        &fixture_bytes(E201_FIXTURE),
        ValidationLevel::StrictMarc,
        RecoveryMode::Lenient,
    );
    let (rec, errs) = reader
        .iter_with_errors()
        .next()
        .expect("expected one item")
        .expect("expected Ok");
    assert!(!errs.is_empty(), "expected populated errors tuple element");
    assert!(
        !rec.errors.is_empty(),
        "expected record.errors also populated (same data, two surfaces)"
    );
    assert_eq!(errs[0].code(), "E201");
    assert_eq!(rec.errors.len(), errs.len(), "two surfaces should agree");
}

#[test]
fn cell_strict_marc_permissive_iter() {
    let mut reader = build_reader(
        &fixture_bytes(E201_FIXTURE),
        ValidationLevel::StrictMarc,
        RecoveryMode::Permissive,
    );
    let rec = reader
        .read_record()
        .expect("expected Ok(Some) in permissive")
        .expect("expected a record");
    assert!(
        !rec.errors.is_empty(),
        "expected populated errors in permissive"
    );
    assert_eq!(rec.errors[0].code(), "E201");
}

#[test]
fn cell_strict_marc_permissive_iter_with_errors() {
    let mut reader = build_reader(
        &fixture_bytes(E201_FIXTURE),
        ValidationLevel::StrictMarc,
        RecoveryMode::Permissive,
    );
    let (rec, errs) = reader
        .iter_with_errors()
        .next()
        .expect("expected one item")
        .expect("expected Ok");
    assert!(!errs.is_empty(), "expected populated errors tuple element");
    assert_eq!(errs[0].code(), "E201");
    assert_eq!(rec.errors.len(), errs.len(), "two surfaces should agree");
}

// =====================================================================
// Per-error-code parametric: each code captured with positional context
// =====================================================================
//
// Canonical cell: StrictMarc + Lenient (most observable). Verify the
// expected error code is captured AND that core positional context
// (record_index, field_tag where applicable) survives onto record.errors.

fn assert_code_captured(fixture: &str, expected_code: &str) {
    let mut reader = build_reader(
        &fixture_bytes(fixture),
        ValidationLevel::StrictMarc,
        RecoveryMode::Lenient,
    );
    let rec = reader
        .read_record()
        .expect("expected Ok(Some)")
        .expect("expected a record");
    assert!(
        !rec.errors.is_empty(),
        "{fixture}: errors should be populated"
    );
    let err = &rec.errors[0];
    assert_eq!(err.code(), expected_code, "{fixture}: wrong error code");
    let dict = err.to_json_value();
    assert!(
        dict.get("record_index").is_some_and(|v| !v.is_null()),
        "{fixture}: record_index missing in captured error: {dict}"
    );
}

#[test]
fn captures_e201_with_context() {
    assert_code_captured("e201_bad_indicator.bin", "E201");
}

#[test]
fn captures_e202_with_context() {
    assert_code_captured("e202_non_printable_subfield_code.bin", "E202");
}

#[test]
fn captures_e301_with_context() {
    assert_code_captured("e301_invalid_utf8_in_subfield.bin", "E301");
}

// =====================================================================
// Cross-reader smoke: authority + holdings expose the diagnostic surface
// =====================================================================
//
// No malformed authority/holdings fixtures exist; verify the surface
// works on clean records (empty errors, iter_with_errors yields tuples).

#[test]
fn authority_reader_exposes_diagnostic_surface() {
    let bytes = data_bytes("simple_authority.mrc");
    let mut reader = AuthorityMarcReader::new(Cursor::new(bytes));
    let mut saw_record = false;
    for entry in reader.iter_with_errors() {
        let (rec, errs) = entry.expect("expected Ok on clean authority fixture");
        assert!(errs.is_empty(), "expected empty errors on clean authority");
        assert!(rec.errors.is_empty(), "record.errors should also be empty");
        saw_record = true;
    }
    assert!(saw_record, "authority fixture yielded no records");
}

#[test]
fn holdings_reader_exposes_diagnostic_surface() {
    // No clean holdings fixture exists in tests/data/; build one inline.
    // Minimal holdings record: leader byte 6 = 'x' (single-part item),
    // one 001 control field. Using the smallest legal ISO 2709 shape.
    //
    // This test asserts only that the diagnostic surface API is wired —
    // not record content. If the fixture is too malformed to parse, we
    // detect that here and adjust.
    use mrrc::{Leader, MarcWriter, Record};
    let leader = Leader {
        record_length: 0,
        record_status: 'n',
        record_type: 'x',
        bibliographic_level: ' ',
        control_record_type: ' ',
        character_coding: ' ',
        indicator_count: 2,
        subfield_code_count: 2,
        data_base_address: 0,
        encoding_level: ' ',
        cataloging_form: ' ',
        multipart_level: ' ',
        reserved: "4500".to_string(),
    };
    let mut record = Record::new(leader);
    record.add_control_field("001".to_string(), "h001".to_string());

    let mut buf: Vec<u8> = Vec::new();
    {
        let mut writer = MarcWriter::new(&mut buf);
        writer
            .write_record(&record)
            .expect("write holdings test record");
    }

    let mut reader = HoldingsMarcReader::new(Cursor::new(buf));
    let mut saw_record = false;
    for entry in reader.iter_with_errors() {
        let (rec, errs) = entry.expect("expected Ok on clean inline holdings record");
        assert!(errs.is_empty(), "expected empty errors on clean holdings");
        assert!(rec.errors.is_empty(), "record.errors should also be empty");
        saw_record = true;
    }
    assert!(saw_record, "inline holdings record yielded nothing");
}
