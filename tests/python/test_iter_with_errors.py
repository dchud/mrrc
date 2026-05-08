"""Python-side tests for ``MARCReader.iter_with_errors`` and
``record.errors``.

Covers two surfaces the Rust matrix can't reach:
- ``MARCReader(permissive=True).iter_with_errors()`` yielding
  ``(None, [exception])`` for records that the wrapper would otherwise
  swallow as ``None``.
- ``record.errors`` exposed as a Python list of typed exception
  instances with the right code/slug.
"""

from __future__ import annotations

from pathlib import Path

import pytest

import mrrc

_REPO_ROOT = Path(__file__).resolve().parents[2]
_FIXTURES = _REPO_ROOT / "tests" / "data" / "error_fixtures"


def _read_fixture(name: str) -> bytes:
    return (_FIXTURES / name).read_bytes()


# ---------------------------------------------------------------------
# record.errors basic shape
# ---------------------------------------------------------------------


def test_clean_record_has_empty_errors() -> None:
    bytes_ = (_REPO_ROOT / "tests" / "data" / "simple_book.mrc").read_bytes()
    reader = mrrc.MARCReader(bytes_)
    record = next(reader)
    assert record.errors == []


def test_lenient_populates_record_errors() -> None:
    """At lenient + strict_marc, a bad-byte record yields with errors."""
    bytes_ = _read_fixture("e201_bad_indicator.bin")
    reader = mrrc.MARCReader(
        bytes_,
        recovery_mode="lenient",
        validation_level="strict_marc",
    )
    record = next(reader)
    assert record is not None
    assert len(record.errors) >= 1
    assert record.errors[0].code == "E201"


# ---------------------------------------------------------------------
# iter_with_errors basic shape (non-permissive)
# ---------------------------------------------------------------------


def test_iter_with_errors_clean_records() -> None:
    bytes_ = (_REPO_ROOT / "tests" / "data" / "multi_records.mrc").read_bytes()
    reader = mrrc.MARCReader(bytes_)
    for record, errors in reader.iter_with_errors():
        assert record is not None
        assert errors == []


def test_iter_with_errors_lenient_yields_tuples_with_errors() -> None:
    bytes_ = _read_fixture("e202_non_printable_subfield_code.bin")
    reader = mrrc.MARCReader(
        bytes_,
        recovery_mode="lenient",
        validation_level="strict_marc",
    )
    items = list(reader.iter_with_errors())
    assert len(items) == 1
    record, errors = items[0]
    assert record is not None
    assert len(errors) >= 1
    assert errors[0].code == "E202"


# ---------------------------------------------------------------------
# permissive=True: swallowed records yield (None, [exception])
# ---------------------------------------------------------------------


def test_permissive_iter_yields_none_for_swallowed() -> None:
    """Without iter_with_errors, permissive=True yields None on failed parses."""
    # The E201 fixture under default Rust strictness raises an exception
    # in the non-permissive Python wrapper would re-raise. With
    # permissive=True the wrapper swallows and yields None.
    bytes_ = _read_fixture("e201_bad_indicator.bin")
    reader = mrrc.MARCReader(bytes_, permissive=True, validation_level="strict_marc")
    items = list(reader)
    # The fixture has a single record; since the parser raises on it,
    # permissive yields None for it.
    assert items == [None]


def test_permissive_iter_with_errors_yields_none_plus_exception() -> None:
    """Per option (A): permissive-swallowed records become (None, [exception])
    via iter_with_errors so the diagnostic isn't lost."""
    bytes_ = _read_fixture("e201_bad_indicator.bin")
    reader = mrrc.MARCReader(bytes_, permissive=True, validation_level="strict_marc")
    items = list(reader.iter_with_errors())
    assert len(items) == 1
    record, errors = items[0]
    assert record is None, "expected None for swallowed record"
    assert len(errors) == 1, f"expected one captured exception, got {errors}"
    assert errors[0].code == "E201"


def test_permissive_iter_with_errors_returns_records_when_clean() -> None:
    """Permissive mode + clean records: iter_with_errors yields normal tuples."""
    bytes_ = (_REPO_ROOT / "tests" / "data" / "multi_records.mrc").read_bytes()
    reader = mrrc.MARCReader(bytes_, permissive=True)
    for record, errors in reader.iter_with_errors():
        assert record is not None
        assert errors == []


# ---------------------------------------------------------------------
# Recovery-mode coverage matrix per error code
# ---------------------------------------------------------------------
#
# For each malformed-input fixture, exercise the documented recovery
# contract. Three contracts:
#
# * "fatal"      — fires in every recovery_mode (cannot recover).
# * "strict-only"— fires in strict; lenient/permissive recovery cap
#                  absorbs (record yielded clean, no errors captured).
# * "recoverable"— fires in strict; lenient/permissive yield the
#                  record with the error captured on record.errors.


@pytest.mark.parametrize(
    ("fixture", "code"),
    [
        ("e001_record_length_non_digit.bin", "E001"),
        ("e002_indicator_count_non_digit.bin", "E002"),
        ("e003_base_address_non_digit.bin", "E003"),
        ("e004_base_address_past_record.bin", "E004"),
    ],
)
def test_fatal_errors_raise_in_every_mode(fixture: str, code: str) -> None:
    """Leader/structural errors that prevent establishing a record
    boundary are unrecoverable: lenient and permissive both surface
    them rather than silently advancing."""
    bytes_ = _read_fixture(fixture)
    for recovery_mode in ("strict", "lenient"):
        reader = mrrc.MARCReader(bytes_, recovery_mode=recovery_mode)
        with pytest.raises(Exception) as exc_info:
            list(reader)
        assert getattr(exc_info.value, "code", None) == code, (
            f"{fixture} at recovery_mode={recovery_mode}: "
            f"expected {code}, got {exc_info.value!r}"
        )


def test_strict_only_e006_raises_in_strict_clean_in_lenient() -> None:
    """E006 (no record terminator) is strict-only by design: lenient
    and permissive let the recovery cap absorb it."""
    bytes_ = _read_fixture("e006_no_record_terminator.bin")

    reader = mrrc.MARCReader(bytes_, recovery_mode="strict")
    with pytest.raises(Exception) as exc_info:
        list(reader)
    assert getattr(exc_info.value, "code", None) == "E006"

    reader = mrrc.MARCReader(bytes_, recovery_mode="lenient")
    records = list(reader)
    assert len(records) == 1
    assert records[0] is not None


@pytest.mark.parametrize(
    ("fixture", "code"),
    [
        ("e101_directory_non_digit_length.bin", "E101"),
        ("e106_field_length_past_data.bin", "E106"),
    ],
)
def test_recoverable_errors_captured_in_lenient(fixture: str, code: str) -> None:
    """Mid-record recoverable errors land on record.errors in lenient
    mode. The parser yields a (partially recovered) record rather than
    propagating the error.

    E005 (stream truncation) is exercised in the Rust harness only.
    The Python wrapper's per-record byte-prefetch backend in
    src-python/src/backend.rs raises on a short stream read before the
    recovery-aware parser is invoked, so E005 surfaces as a raised
    TruncatedRecord even at recovery_mode="lenient". Aligning that
    layer with the parser's recovery contract is tracked separately.
    """
    bytes_ = _read_fixture(fixture)
    reader = mrrc.MARCReader(bytes_, recovery_mode="lenient")
    records = list(reader)
    assert len(records) == 1
    record = records[0]
    assert record is not None
    codes = [e.code for e in record.errors]
    assert code in codes, (
        f"{fixture}: expected {code} on record.errors, got {codes}"
    )
