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
