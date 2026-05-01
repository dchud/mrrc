//! Error-handling coverage harness.
//!
//! Reads `tests/error_coverage.toml` and runs one assertion bundle per
//! `[[case]]` entry. For each `wired = true` case, asserts that the
//! documented [`MarcError`](mrrc::MarcError) variant, code, slug, and
//! positional context fire when the parser reads the fixture bytes.
//! For each `wired = false` case, prints the skip reason so the gap
//! between documentation and implementation is visible in CI output.
//!
//! The final assertion fails on any wired-case violation and prints
//! `wired: X/Y (skipped: Z)`. The numbers are monotonic over time:
//! when detection lands for a previously-unwired trigger, flipping
//! `wired = false` → `wired = true` in the manifest is sufficient and
//! the assertion picks up the new case automatically.
//!
//! The harness exercises strict mode for now. Lenient and permissive
//! mode contracts depend on per-record diagnostic surfaces
//! (`record.errors` / `iter_with_errors` / `last_exception`) that are
//! tracked separately; cases declare which modes their contract
//! covers so the harness can extend in place when those surfaces
//! exist.

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use mrrc::{MarcReader, RecoveryMode};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Manifest {
    #[allow(dead_code)]
    schema_version: u32,
    case: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    id: String,
    code: String,
    variant: String,
    slug: String,
    trigger_fixture: String,
    #[allow(dead_code)]
    description: String,
    expected_context: Vec<String>,
    recovery_modes: Vec<String>,
    wired: bool,
    #[serde(default)]
    skip_reason: Option<String>,
}

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/error_coverage.toml")
}

fn load_manifest() -> Manifest {
    let text = fs::read_to_string(manifest_path()).expect("read error_coverage.toml");
    toml::from_str(&text).expect("parse error_coverage.toml")
}

fn fixture_bytes(case: &Case) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(&case.trigger_fixture);
    fs::read(&path).unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()))
}

fn recovery_mode_from_str(s: &str) -> RecoveryMode {
    match s {
        "strict" => RecoveryMode::Strict,
        "lenient" => RecoveryMode::Lenient,
        "permissive" => RecoveryMode::Permissive,
        other => panic!("unknown recovery_mode {other:?} in manifest"),
    }
}

/// Drive the reader on `bytes` in `mode` and return the first error
/// encountered, if any. Successful records and EOF (`Ok(None)`) both
/// return `None`.
fn first_error(bytes: &[u8], mode: RecoveryMode) -> Option<mrrc::MarcError> {
    let mut reader = MarcReader::new(Cursor::new(bytes)).with_recovery_mode(mode);
    loop {
        match reader.read_record() {
            Ok(Some(_)) => {},
            Ok(None) => return None,
            Err(e) => return Some(e),
        }
    }
}

/// Map a positional-context field name from the manifest to the JSON
/// key produced by [`MarcError::to_json_value`]. Most names are
/// identical; `bytes_near` and `found` are surfaced as `bytes_near_hex`
/// / `found_hex` because the JSON uses a `_hex`-suffix convention for
/// raw byte fields.
fn json_probe_key(field: &str) -> &str {
    match field {
        "bytes_near" => "bytes_near_hex",
        "found" => "found_hex",
        other => other,
    }
}

/// Run one wired case. Returns `Ok(())` on success or `Err(reason)` on
/// the first violation so the caller can collect failures across all
/// cases without short-circuiting.
fn run_wired(case: &Case) -> Result<(), String> {
    let bytes = fixture_bytes(case);
    for mode_str in &case.recovery_modes {
        let mode = recovery_mode_from_str(mode_str);
        if mode != RecoveryMode::Strict {
            // Strict mode is the only one the harness asserts today.
            // Lenient / permissive contracts require per-record
            // diagnostic surfaces that don't exist yet; the manifest
            // declares the modes so this branch can extend without
            // schema churn.
            continue;
        }
        let err = first_error(&bytes, mode).ok_or_else(|| {
            format!(
                "{} ({} / {}): expected {} error in {:?} mode, got Ok",
                case.id, case.code, case.variant, case.code, mode
            )
        })?;
        if err.code() != case.code {
            return Err(format!(
                "{} ({}): expected code {}, got {} ({:?})",
                case.id,
                case.variant,
                case.code,
                err.code(),
                err
            ));
        }
        if err.slug() != case.slug {
            return Err(format!(
                "{} ({}): expected slug {:?}, got {:?}",
                case.id,
                case.variant,
                case.slug,
                err.slug()
            ));
        }
        let dict = err.to_json_value();
        for field in &case.expected_context {
            let key = json_probe_key(field);
            let present = dict.get(key).is_some_and(|v| !v.is_null());
            if !present {
                return Err(format!(
                    "{} ({}): expected_context field {} not populated (probed via {:?}); error JSON: {}",
                    case.id, case.variant, field, key, dict
                ));
            }
        }
    }
    Ok(())
}

#[test]
fn manifest_is_well_formed() {
    let manifest = load_manifest();
    assert_eq!(manifest.schema_version, 1, "schema_version drift");
    assert!(!manifest.case.is_empty(), "manifest has no cases");

    let mut ids = std::collections::HashSet::new();
    for case in &manifest.case {
        let inserted = ids.insert(case.id.clone());
        assert!(inserted, "duplicate case id {}", case.id);

        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(&case.trigger_fixture);
        assert!(
            path.exists(),
            "case {}: fixture {} does not exist",
            case.id,
            path.display()
        );

        if !case.wired {
            assert!(
                case.skip_reason.is_some(),
                "case {} is unwired but has no skip_reason",
                case.id
            );
        }
    }
}

#[test]
fn coverage_assertions() {
    let manifest = load_manifest();
    let mut failures: Vec<String> = Vec::new();
    let mut wired_count = 0usize;
    let mut skipped: Vec<(String, String)> = Vec::new();

    for case in &manifest.case {
        if case.wired {
            wired_count += 1;
            if let Err(reason) = run_wired(case) {
                failures.push(reason);
            }
        } else {
            let reason = case
                .skip_reason
                .clone()
                .unwrap_or_else(|| "unwired (no reason provided)".into());
            skipped.push((case.id.clone(), reason));
        }
    }

    for (id, reason) in &skipped {
        eprintln!("[error_coverage] SKIP {id}: {reason}");
    }
    eprintln!(
        "[error_coverage] wired: {}/{} (skipped: {})",
        wired_count,
        manifest.case.len(),
        skipped.len()
    );

    assert!(
        failures.is_empty(),
        "wired-case failures:\n  - {}",
        failures.join("\n  - ")
    );
}
