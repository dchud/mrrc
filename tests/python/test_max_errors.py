"""Tests for the ``max_errors`` kwarg on ``mrrc.MARCReader``.

Mirrors the Rust unit tests in ``src/reader.rs`` covering recovery-cap
trip in lenient mode, ``max_errors=0`` disabling the cap, and the cap
being inert in strict mode (strict propagates the first error before
the cap can accumulate).
"""

from __future__ import annotations

from pathlib import Path

import pytest

import mrrc

_REPO_ROOT = Path(__file__).resolve().parents[2]
_FIXTURES = _REPO_ROOT / "tests" / "data" / "error_fixtures"


def _bad_record_bytes() -> bytes:
    """A single malformed ISO 2709 record that trips E101 in lenient mode."""
    return (_FIXTURES / "e101_directory_non_digit_length.bin").read_bytes()


def test_max_errors_caps_lenient_iteration_with_fatal_reader_error() -> None:
    """With ``max_errors=1`` and a stream of N+2 malformed records, the
    parser captures the first error on ``record.errors`` and then
    raises ``FatalReaderError`` (E099) on the next read."""
    bad = _bad_record_bytes()
    stream = bad * 3  # 3 records, each will trip E101 in lenient mode
    reader = mrrc.MARCReader(stream, recovery_mode="lenient", max_errors=1)
    with pytest.raises(mrrc.FatalReaderError) as excinfo:
        list(reader)
    assert excinfo.value.code == "E099"
    assert excinfo.value.slug == "fatal_reader_error"


def test_max_errors_zero_disables_cap() -> None:
    """``max_errors=0`` matches the Rust core contract: no cap, every
    record is yielded with its error captured, no FatalReaderError."""
    bad = _bad_record_bytes()
    stream = bad * 3
    reader = mrrc.MARCReader(stream, recovery_mode="lenient", max_errors=0)
    records = list(reader)
    assert len(records) == 3
    for record in records:
        assert record is not None
        codes = [e.code for e in record.errors]
        assert "E101" in codes


def test_max_errors_inert_in_strict_mode() -> None:
    """The cap accumulates recovered errors; strict mode propagates the
    first error before any recovery can land on the cap, so the kwarg
    is observationally inert (the strict error fires regardless of the
    cap value)."""
    bad = _bad_record_bytes()
    reader = mrrc.MARCReader(bad, recovery_mode="strict", max_errors=1)
    with pytest.raises(mrrc.MrrcException) as excinfo:
        list(reader)
    # Strict surfaces the underlying E101, not the cap's E099.
    assert excinfo.value.code == "E101"
