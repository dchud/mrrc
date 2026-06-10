"""pymarc iteration-shape parity.

Asserts that, over a mixed-quality corpus, mrrc's permissive iteration
produces the same per-record record-vs-None shape as pymarc 5.3.1.

pymarc is an optional comparison target, not a runtime or default-test
dependency, so this is guarded by ``pytest.importorskip``. Run it with
pymarc installed, e.g.:

    uv run --with 'pymarc==5.3.1' python -m pytest tests/python/test_pymarc_parity.py

The corpus keeps each record's leader ``record_length`` intact so both
readers advance one record at a time on the same byte boundaries; the
malformed records corrupt the directory region only, which both libraries
surface as a yielded ``None`` under permissive iteration. That makes the
per-position shape — not just the count — directly comparable.
"""

from __future__ import annotations

import io
from pathlib import Path

import pytest

import mrrc

_REPO_ROOT = Path(__file__).resolve().parents[2]
_VALID = _REPO_ROOT / "tests" / "data" / "simple_book.mrc"

_TOTAL = 100
_MALFORMED_EVERY = 10  # every 10th record is malformed -> 10 malformed, 90 valid


def _build_corpus() -> tuple[bytes, list[bool]]:
    """Return (stream, expected_shape) for a 100-record mixed corpus.

    expected_shape[i] is True where record i is well-formed (should yield a
    record) and False where it is malformed (should yield None).
    """
    valid = _VALID.read_bytes()
    # Corrupt the directory region (bytes 24..36) with non-digit 'X' while
    # leaving the 24-byte leader — and therefore record_length — intact, so
    # the record boundary stays clean and both readers yield None for it.
    malformed = bytearray(valid)
    for i in range(24, 36):
        malformed[i] = ord("X")
    malformed = bytes(malformed)

    records: list[bytes] = []
    expected: list[bool] = []
    for i in range(_TOTAL):
        is_malformed = i % _MALFORMED_EVERY == (_MALFORMED_EVERY - 1)
        records.append(malformed if is_malformed else valid)
        expected.append(not is_malformed)
    return b"".join(records), expected


def test_permissive_iteration_shape_matches_pymarc() -> None:
    """mrrc(permissive=True) and pymarc(permissive=True) yield the same
    record-vs-None shape over a 100-record, 10%-malformed corpus."""
    pymarc = pytest.importorskip("pymarc")

    stream, expected = _build_corpus()

    mrrc_shape = [r is not None for r in mrrc.MARCReader(stream, permissive=True)]
    pymarc_shape = [
        r is not None for r in pymarc.MARCReader(io.BytesIO(stream), permissive=True)
    ]

    assert len(mrrc_shape) == _TOTAL, f"mrrc iterated {len(mrrc_shape)}, expected {_TOTAL}"
    assert len(pymarc_shape) == _TOTAL, (
        f"pymarc iterated {len(pymarc_shape)}, expected {_TOTAL}"
    )
    # Both match the intended shape, and therefore each other.
    assert mrrc_shape == expected, "mrrc shape diverged from the corpus pattern"
    assert pymarc_shape == expected, "pymarc shape diverged from the corpus pattern"
    assert mrrc_shape == pymarc_shape
