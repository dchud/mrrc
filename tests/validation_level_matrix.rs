//! Integration test for the `validation_level × recovery_mode` matrix.
//!
//! `validation_level` controls *what counts as an error*; `recovery_mode`
//! controls *what to do when one fires*. Bd-0x73.10 introduced the
//! orthogonal axis; this test verifies each of the six cells behaves as
//! the bead's "Done when" says it should.
//!
//! Single rule under test: `structural` is lossy across every reader,
//! `strict_marc` is strict across every reader. At `Structural`,
//! E201/E202/E301 do not fire on the bad-byte fixtures. At
//! `StrictMarc`, they do, and the recovery axis selects how the parser
//! responds (raise / recover / swallow).
//!
//! The fixtures are reused from the bd-0x73.8 coverage harness so this
//! test exercises real malformed bytes, not synthetic errors.
//!
//! Limitations: lenient/permissive paths *recover* and *swallow*
//! respectively today; per-record diagnostic surfaces (bd-0x73.11) are
//! a future bead, so the assertions verify the iteration shape (no
//! raised error, record yielded or not) rather than the full
//! diagnostic payload.

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
