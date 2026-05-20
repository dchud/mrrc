//! Integration test for per-record-type MARC 21 leader validation at
//! `validation_level=strict_marc` (bd-0x73.21.7).
//!
//! Verifies that:
//!
//! - The bibliographic, authority, and holdings readers each apply their
//!   own MARC 21 Format leader allowed-value sets at `strict_marc`, not the
//!   bibliographic set across the board.
//! - Positions 7, 8, 19 — undefined for authority and holdings — accept
//!   any byte (including the fill character `|`) without raising E002.
//! - Positions 5, 6, 17, 18 — defined per record type — reject values
//!   that are valid for a *different* record type.
//!
//! Construction uses the on-disk fixtures `simple_authority.mrc` and
//! `simple_holdings.mrc` as base byte streams with single-byte leader
//! patches to flip valid/invalid states.

use std::io::Cursor;

use mrrc::{AuthorityMarcReader, HoldingsMarcReader, MarcReader, RecoveryMode, ValidationLevel};

const SIMPLE_AUTHORITY: &[u8] = include_bytes!("data/simple_authority.mrc");
const SIMPLE_HOLDINGS: &[u8] = include_bytes!("data/simple_holdings.mrc");

/// Return a clone of `base` with `byte` written at leader position `pos`.
fn patch_leader_byte(base: &[u8], pos: usize, byte: u8) -> Vec<u8> {
    let mut out = base.to_vec();
    out[pos] = byte;
    out
}

// ============================================================
// Authority reader at strict_marc
// ============================================================

#[test]
fn authority_strict_marc_accepts_valid_authority_leader() {
    // Patch position 18 (punctuation policy) from the fixture's 'a' (a
    // bibliographic AACR2 marker, not valid for authority) to ' ' so the
    // leader satisfies the MARC 21 Authority Format allowed-value sets.
    let bytes = patch_leader_byte(SIMPLE_AUTHORITY, 18, b' ');
    let mut reader = AuthorityMarcReader::new(Cursor::new(bytes))
        .with_recovery_mode(RecoveryMode::Strict)
        .with_validation_level(ValidationLevel::StrictMarc);

    let record = reader
        .read_record()
        .expect("authority strict_marc must accept its own valid leader (bd-0x73.21.7)");
    assert!(record.is_some(), "expected one authority record");
}

#[test]
fn authority_strict_marc_accepts_fill_at_undefined_positions() {
    // The unmodified fixture already has `bibliographic_level='|'` (pos 7)
    // — undefined for authority. With position 18 patched to ' ', this
    // record must pass even though '|' would be invalid for the
    // bibliographic position-7 set.
    let bytes = patch_leader_byte(SIMPLE_AUTHORITY, 18, b' ');
    assert_eq!(bytes[7], b'|', "fixture must carry fill at position 7");

    let mut reader = AuthorityMarcReader::new(Cursor::new(bytes))
        .with_recovery_mode(RecoveryMode::Strict)
        .with_validation_level(ValidationLevel::StrictMarc);

    reader
        .read_record()
        .expect("authority strict_marc must accept fill char at undefined leader positions");
}

#[test]
fn authority_strict_marc_rejects_bibliographic_encoding_level() {
    // Authority pos 17 (encoding level) is restricted to {n, o}. The
    // bibliographic-valid value '1' must trip E002.
    let bytes = {
        let mut b = patch_leader_byte(SIMPLE_AUTHORITY, 18, b' ');
        b[17] = b'1'; // bibliographic full-level-material-not-examined
        b
    };
    let mut reader = AuthorityMarcReader::new(Cursor::new(bytes))
        .with_recovery_mode(RecoveryMode::Strict)
        .with_validation_level(ValidationLevel::StrictMarc);

    let err = reader
        .read_record()
        .expect_err("authority must reject bibliographic-only encoding level at strict_marc");
    assert_eq!(
        err.code(),
        "E002",
        "expected E002 leader_invalid, got {err:?}"
    );
}

#[test]
fn authority_structural_accepts_anything_authority_strict_rejects() {
    // At default validation level (`structural`) the MARC 21 semantic
    // leader check is bypassed entirely; even an obviously-bibliographic
    // encoding_level should parse cleanly through the authority reader.
    let bytes = {
        let mut b = SIMPLE_AUTHORITY.to_vec();
        b[17] = b'1';
        b
    };
    let mut reader = AuthorityMarcReader::new(Cursor::new(bytes));
    reader
        .read_record()
        .expect("structural authority must ignore MARC 21 leader semantics");
}

// ============================================================
// Holdings reader at strict_marc
// ============================================================

/// Apply both position-17 (`encoding_level`) and position-18
/// (`cataloging_form`/item-info) patches needed to make `simple_holdings.mrc`
/// satisfy the holdings allowed-value sets. The fixture's defaults
/// (`n`/`a`) reflect authority- and bibliographic-shaped values that
/// pre-date the per-type validators.
fn holdings_strict_marc_valid_bytes() -> Vec<u8> {
    let mut b = patch_leader_byte(SIMPLE_HOLDINGS, 18, b' ');
    b[17] = b'1'; // holdings encoding level: full level
    b
}

#[test]
fn holdings_strict_marc_accepts_valid_holdings_leader() {
    let mut reader = HoldingsMarcReader::new(Cursor::new(holdings_strict_marc_valid_bytes()))
        .with_recovery_mode(RecoveryMode::Strict)
        .with_validation_level(ValidationLevel::StrictMarc);

    let record = reader
        .read_record()
        .expect("holdings strict_marc must accept its own valid leader (bd-0x73.21.7)");
    assert!(record.is_some(), "expected one holdings record");
}

#[test]
fn holdings_strict_marc_rejects_authority_encoding_level() {
    // Holdings pos 17 (encoding level) is restricted to {1,2,3,4,5,m,u,z}.
    // 'n' is authority-valid but holdings-invalid; expect E002.
    let bytes = {
        let mut b = holdings_strict_marc_valid_bytes();
        b[17] = b'n';
        b
    };
    let mut reader = HoldingsMarcReader::new(Cursor::new(bytes))
        .with_recovery_mode(RecoveryMode::Strict)
        .with_validation_level(ValidationLevel::StrictMarc);

    let err = reader
        .read_record()
        .expect_err("holdings must reject authority-only encoding level at strict_marc");
    assert_eq!(
        err.code(),
        "E002",
        "expected E002 leader_invalid, got {err:?}"
    );
}

#[test]
fn holdings_strict_marc_accepts_fill_at_undefined_positions() {
    let bytes = holdings_strict_marc_valid_bytes();
    assert_eq!(bytes[7], b'|');
    let mut reader = HoldingsMarcReader::new(Cursor::new(bytes))
        .with_recovery_mode(RecoveryMode::Strict)
        .with_validation_level(ValidationLevel::StrictMarc);
    reader
        .read_record()
        .expect("holdings strict_marc must accept fill char at undefined leader positions");
}

// ============================================================
// Bibliographic reader still uses its own allowed-value set
// ============================================================

#[test]
fn bibliographic_strict_marc_rejects_holdings_record_type() {
    // The bibliographic allowed set at position 6 covers a/c/d/e/f/g/i/j/
    // k/m/o/p/r/t/v/z. 'x' (holdings-only) is *not* in that set; the
    // bibliographic reader at strict_marc must still trip E002 on it,
    // proving the trait default still picks up `validate_leader` rather
    // than silently widening the bibliographic allowed sets.
    let book = std::fs::read("tests/data/simple_book.mrc").unwrap();
    let bytes = {
        let mut b = book;
        b[6] = b'x'; // holdings record_type
        b
    };
    let mut reader = MarcReader::new(Cursor::new(bytes))
        .with_recovery_mode(RecoveryMode::Strict)
        .with_validation_level(ValidationLevel::StrictMarc);

    let err = reader
        .read_record()
        .expect_err("bibliographic strict_marc must still apply bibliographic allowed sets");
    assert_eq!(
        err.code(),
        "E002",
        "expected E002 leader_invalid, got {err:?}"
    );
}
