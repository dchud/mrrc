//! Byte-source parse fuzz harness.
//!
//! Drives the in-memory byte-source entry points — `parse_record_from_bytes`
//! and `parse_record_from_shared_bytes` — over arbitrary bytes in every
//! recovery mode and validation level. These reach
//! `parse_iso2709_record_from_bytes` (the leader-length guard, the
//! truncated-body copy, and the `LEADER_LEN..LEADER_LEN + expected_data_len`
//! slice) — divergent code the reader-driven targets (`parse_record`,
//! `roundtrip_binary`, `recovery_mode_consistency`) never exercise.
//! `parse_record_from_shared_bytes` is the production Python read path (the
//! src-python batched reader), so a panic here is a panic users could hit.
//!
//! An `Err(MarcError)` on malformed input is correct behavior, not a bug, so
//! the results are discarded — libfuzzer only flags panics, OOMs, and
//! timeouts.
//!
//! See `docs/contributing/fuzzing.md` for triage.

#![no_main]

use libfuzzer_sys::fuzz_target;
use mrrc::{
    RecoveryMode, ValidationLevel, parse_record_from_bytes,
    parse_record_from_shared_bytes,
};
use std::sync::Arc;

const MODES: [RecoveryMode; 3] = [
    RecoveryMode::Strict,
    RecoveryMode::Lenient,
    RecoveryMode::Permissive,
];

const LEVELS: [ValidationLevel; 2] =
    [ValidationLevel::Structural, ValidationLevel::StrictMarc];

fuzz_target!(|data: &[u8]| {
    // Share one buffer across the borrowing entry point — the same shape the
    // Python read path uses (parser and diagnostics context both borrow it).
    let shared = Arc::new(data.to_vec());
    for mode in MODES {
        for level in LEVELS {
            let _ = parse_record_from_shared_bytes(&shared, mode, level);
            // The owning entry point moves a fresh buffer in each call.
            let _ = parse_record_from_bytes(data.to_vec(), mode, level);
        }
    }
});
