//! Recovery-mode cross-consistency fuzz harness.
//!
//! Drives the same input bytes through `MarcReader` in all three
//! recovery modes (`Strict`, `Lenient`, `Permissive`) at
//! `ValidationLevel::StrictMarc` and asserts that the modes agree on
//! what a record-shape input means. Strict's verdict is the ground
//! truth; lenient and permissive are derived recovery surfaces and
//! must stay consistent with it.
//!
//! Asserted contract on the **first** outcome each reader produces:
//!
//! - `strict = Ok(clean record)` → both `lenient` and `permissive`
//!   yield a record with no per-record errors and the same field
//!   count. Recovery modes must not flag a record strict accepted
//!   (no false-positive errors during recovery).
//! - `strict = Ok(None)` (EOF before any record) → both other modes
//!   must also be EOF. Recovery has nothing to do on an empty stream.
//! - `strict.errors.is_empty()` is an invariant: strict mode never
//!   returns a record carrying per-record diagnostics (it propagates
//!   them as `Err` instead). A violation is a wiring inconsistency.
//! - `strict = Err(_)` is **not** asserted to imply recovery-mode
//!   error reporting. Lenient and permissive intentionally relax
//!   some byte-level checks (e.g., the `RECORD_TERMINATOR` check
//!   only fires in strict — see `CHANGELOG` for v0.8.1), so they
//!   may yield a clean record where strict raised. That's by
//!   design, not a contract violation.
//!
//! What this catches that `parse_record`, `roundtrip_binary`, and
//! `error_classification` do not:
//!
//! - Per-mode divergence in the **acceptance** direction — a parser
//!   bug where lenient or permissive rejects (or flags with errors)
//!   a record strict accepts wouldn't show up as a panic in any
//!   single mode, but surfaces here as a cross-mode disagreement.
//! - Invariant drift on `record.errors` in strict mode — if a future
//!   change accidentally lets diagnostics leak into a strict-mode
//!   record, this target fails.
//!
//! See `docs/contributing/fuzzing.md` for triage.

#![no_main]

use libfuzzer_sys::fuzz_target;
use mrrc::{MarcReader, RecoveryMode, ValidationLevel};
use std::io::Cursor;

#[derive(Debug)]
enum Outcome {
    /// Reader yielded a record. `has_errors` is true iff the per-
    /// record diagnostics list is non-empty; `field_count` is the
    /// number of data fields the record carries.
    Record { has_errors: bool, field_count: usize },
    /// Reader returned `Ok(None)` — end of stream before any record.
    Eof,
    /// Reader propagated an error.
    Err,
}

fn first_outcome(data: &[u8], mode: RecoveryMode) -> Outcome {
    let mut reader = MarcReader::new(Cursor::new(data))
        .with_recovery_mode(mode)
        .with_validation_level(ValidationLevel::StrictMarc);
    match reader.read_record() {
        Ok(Some(record)) => Outcome::Record {
            has_errors: !record.errors.is_empty(),
            field_count: record.fields().count(),
        },
        Ok(None) => Outcome::Eof,
        Err(_) => Outcome::Err,
    }
}

fuzz_target!(|data: &[u8]| {
    let strict = first_outcome(data, RecoveryMode::Strict);
    let lenient = first_outcome(data, RecoveryMode::Lenient);
    let permissive = first_outcome(data, RecoveryMode::Permissive);

    match strict {
        Outcome::Record {
            has_errors: false,
            field_count,
        } => {
            // Strict accepted a clean record. Lenient and permissive
            // must both yield a clean record with the same field count.
            match &lenient {
                Outcome::Record {
                    has_errors: false,
                    field_count: lf,
                } if *lf == field_count => {},
                other => panic!(
                    "strict accepted clean record (fields={field_count}) but lenient diverged: {other:?}"
                ),
            }
            match &permissive {
                Outcome::Record {
                    has_errors: false,
                    field_count: pf,
                } if *pf == field_count => {},
                other => panic!(
                    "strict accepted clean record (fields={field_count}) but permissive diverged: {other:?}"
                ),
            }
        },
        Outcome::Record { has_errors: true, .. } => {
            // Strict mode never returns a record with diagnostics —
            // errors propagate as Err. A violation is a wiring bug.
            panic!("strict mode yielded record with errors (invariant violation)");
        },
        Outcome::Err => {
            // Strict rejected. Recovery modes are intentionally more
            // lenient about byte-level checks (terminator, etc.), so
            // lenient/permissive may yield a clean record here without
            // it being a contract violation. The non-panic invariant
            // is the only guarantee, and it's already covered by
            // `parse_record`. Asserting tighter cross-mode behavior
            // here would require strengthening recovery-mode parsers
            // to mirror strict's byte-level checks via record.errors
            // — out of scope for this target.
        },
        Outcome::Eof => {
            // EOF before any record bytes — recovery modes have nothing
            // to recover. All three must agree.
            assert!(
                matches!(lenient, Outcome::Eof),
                "strict reached EOF but lenient yielded {lenient:?}"
            );
            assert!(
                matches!(permissive, Outcome::Eof),
                "strict reached EOF but permissive yielded {permissive:?}"
            );
        },
    }
});
