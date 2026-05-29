//! Integration test for the `validation_level × recovery_mode` matrix.
//!
//! `validation_level` controls *what counts as an error*; `recovery_mode`
//! controls *what to do when one fires*. This test verifies each of the
//! six cells of the cross-product behaves as documented.
//!
//! Single rule under test: `structural` is lossy across every reader,
//! `strict_marc` is strict across every reader. At `Structural`,
//! E201/E202/E301 do not fire on the bad-byte fixtures. At
//! `StrictMarc`, they do, and the recovery axis selects how the parser
//! responds (raise / recover / swallow).
//!
//! The fixtures are reused from the error-coverage harness
//! (`tests/error_coverage.rs` / `tests/error_coverage.toml`) so this
//! test exercises real malformed bytes, not synthetic errors.
//!
//! The lenient/permissive cases assert the iteration shape only (no
//! raised error). Per-record diagnostic surfaces — letting a caller
//! inspect *which* record failed and how during a recovered
//! iteration — are a separate piece of the error-handling work and
//! are not exercised here.

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use mrrc::{MarcReader, RecoveryMode, ValidationLevel};

/// One entry in the 2x3 matrix. `expect_strict_error_code` is `Some(code)`
/// when `(strict_marc, strict)` is expected to raise — set per-fixture
/// to the documented error code for the trigger.
struct Cell {
    validation: ValidationLevel,
    recovery: RecoveryMode,
    /// `Some(code)` if iteration should raise an error with this code,
    /// `None` if iteration should complete cleanly.
    expected_error: Option<&'static str>,
}

fn fixture_bytes(name: &str) -> Vec<u8> {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/error_fixtures")
        .join(name);
    fs::read(&path).unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()))
}

/// Iterate the reader to exhaustion. Returns `Ok(record_count)` if no
/// error fired, or `Err(code)` for the first error.
fn drain(
    bytes: &[u8],
    validation: ValidationLevel,
    recovery: RecoveryMode,
) -> Result<usize, String> {
    let mut reader = MarcReader::new(Cursor::new(bytes.to_vec()))
        .with_recovery_mode(recovery)
        .with_validation_level(validation);
    let mut count = 0usize;
    loop {
        match reader.read_record() {
            Ok(Some(_)) => count += 1,
            Ok(None) => return Ok(count),
            Err(e) => return Err(e.code().to_string()),
        }
    }
}

/// Run the matrix against one fixture for an error that fires only at
/// `RecoveryMode::Strict`. `lenient`/`permissive` must NOT surface it —
/// the recovery cap absorbs it — so those cells assert clean iteration.
/// Independent of `validation_level`: both levels behave identically.
fn run_strict_only_matrix(fixture_name: &str, error_code: &'static str) {
    let bytes = fixture_bytes(fixture_name);
    for validation in [ValidationLevel::Structural, ValidationLevel::StrictMarc] {
        for (recovery, expect_fire) in [
            (RecoveryMode::Strict, true),
            (RecoveryMode::Lenient, false),
            (RecoveryMode::Permissive, false),
        ] {
            let outcome = drain(&bytes, validation, recovery);
            match (expect_fire, outcome) {
                (true, Err(actual)) => assert_eq!(
                    actual, error_code,
                    "fixture {fixture_name} at ({validation:?}, {recovery:?}): expected error {error_code}, got {actual}",
                ),
                (true, Ok(n)) => panic!(
                    "fixture {fixture_name} at ({validation:?}, {recovery:?}): expected error {error_code}, got clean iteration ({n} records)",
                ),
                (false, Ok(_)) => {},
                (false, Err(actual)) => panic!(
                    "fixture {fixture_name} at ({validation:?}, {recovery:?}): expected clean iteration (recovery cap absorbs), got error {actual}",
                ),
            }
        }
    }
}

/// Run the matrix against one fixture, asserting `error_code` fires in
/// every cell of the 2×3 grid. Used for errors that are unconditional —
/// i.e., not gated on `validation_level` and not recoverable in
/// `lenient`/`permissive`. Leader/structural errors that prevent the
/// parser from establishing a record boundary (E001, E002 structural,
/// E003, E004) are fatal in this sense: the recovery cap has nothing
/// to absorb because there's no valid boundary to skip past.
fn run_fatal_matrix(fixture_name: &str, error_code: &'static str) {
    let bytes = fixture_bytes(fixture_name);
    for validation in [ValidationLevel::Structural, ValidationLevel::StrictMarc] {
        for recovery in [
            RecoveryMode::Strict,
            RecoveryMode::Lenient,
            RecoveryMode::Permissive,
        ] {
            match drain(&bytes, validation, recovery) {
                Err(actual) => assert_eq!(
                    actual, error_code,
                    "fixture {fixture_name} at ({validation:?}, {recovery:?}): expected fatal error {error_code}, got {actual}",
                ),
                Ok(n) => panic!(
                    "fixture {fixture_name} at ({validation:?}, {recovery:?}): expected fatal error {error_code}, got clean iteration ({n} records)",
                ),
            }
        }
    }
}

/// Run the matrix against one fixture. `strict_error_code` is the error
/// code expected to fire under `(StrictMarc, Strict)`.
fn run_matrix(fixture_name: &str, strict_error_code: &'static str) {
    let bytes = fixture_bytes(fixture_name);
    let cells = [
        Cell {
            validation: ValidationLevel::Structural,
            recovery: RecoveryMode::Strict,
            expected_error: None,
        },
        Cell {
            validation: ValidationLevel::Structural,
            recovery: RecoveryMode::Lenient,
            expected_error: None,
        },
        Cell {
            validation: ValidationLevel::Structural,
            recovery: RecoveryMode::Permissive,
            expected_error: None,
        },
        Cell {
            validation: ValidationLevel::StrictMarc,
            recovery: RecoveryMode::Strict,
            expected_error: Some(strict_error_code),
        },
        Cell {
            validation: ValidationLevel::StrictMarc,
            recovery: RecoveryMode::Lenient,
            expected_error: None,
        },
        Cell {
            validation: ValidationLevel::StrictMarc,
            recovery: RecoveryMode::Permissive,
            expected_error: None,
        },
    ];

    for cell in &cells {
        let outcome = drain(&bytes, cell.validation, cell.recovery);
        match (cell.expected_error, outcome) {
            (None, Ok(_)) => {},
            (Some(code), Err(actual)) => assert_eq!(
                actual, code,
                "fixture {fixture_name} at ({:?}, {:?}): expected error code {code}, got {actual}",
                cell.validation, cell.recovery
            ),
            (None, Err(actual)) => panic!(
                "fixture {fixture_name} at ({:?}, {:?}): expected clean iteration, got error {actual}",
                cell.validation, cell.recovery
            ),
            (Some(code), Ok(n)) => panic!(
                "fixture {fixture_name} at ({:?}, {:?}): expected error {code}, got clean iteration ({n} records)",
                cell.validation, cell.recovery
            ),
        }
    }
}

#[test]
fn matrix_e201_bad_indicator() {
    run_matrix("e201_bad_indicator.bin", "E201");
}

#[test]
fn matrix_e202_bad_subfield_code() {
    run_matrix("e202_non_printable_subfield_code.bin", "E202");
}

#[test]
fn matrix_e301_invalid_utf8() {
    run_matrix("e301_invalid_utf8_in_subfield.bin", "E301");
}

#[test]
fn matrix_e201_per_tag_indicator_245() {
    run_matrix("e201_per_tag_indicator_245.bin", "E201");
}

#[test]
fn matrix_e002_invalid_record_status() {
    run_matrix("e002_invalid_record_status.bin", "E002");
}

// Fatal errors: fire in every cell regardless of validation_level or
// recovery_mode. These assert the contract that a leader the parser
// cannot construct or trust is unrecoverable — the recovery cap has
// nothing to absorb because there's no valid record boundary to skip
// past.

#[test]
fn fatal_e001_record_length_non_digit() {
    run_fatal_matrix("e001_record_length_non_digit.bin", "E001");
}

#[test]
fn fatal_e002_indicator_count_non_digit() {
    run_fatal_matrix("e002_indicator_count_non_digit.bin", "E002");
}

#[test]
fn fatal_e003_base_address_non_digit() {
    run_fatal_matrix("e003_base_address_non_digit.bin", "E003");
}

#[test]
fn fatal_e004_base_address_past_record() {
    run_fatal_matrix("e004_base_address_past_record.bin", "E004");
}

// E006: strict-only, level-independent. The lenient/permissive recovery
// cap absorbs the disagreement via existing directory/field paths.
#[test]
fn strict_only_e006_no_record_terminator() {
    run_strict_only_matrix("e006_no_record_terminator.bin", "E006");
}
