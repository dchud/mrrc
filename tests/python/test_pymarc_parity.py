"""pymarc parity oracle.

Asserts that mrrc matches pymarc on iteration shape over a mixed-quality
corpus and on values (titles, field formatting, serialization bytes) over
valid corpora.

pymarc is managed through the ``oracle`` extra (locked in uv.lock,
bumped by Dependabot), so these tests execute in any environment synced
with ``uv sync --all-extras`` and in the CI oracle step. The importorskip
guard only covers environments installed without the extra, such as the
wheel-test matrix; CI verifies pymarc is importable before running the
oracle so the suite cannot silently skip there.

When a pymarc release changes behavior, these tests fail on the
Dependabot bump PR. Adjudicate there: adopt the new pymarc behavior, or
document the deliberate divergence in the migration guide and pin the
expectation here with a comment.

The mixed-quality corpus keeps each record's leader ``record_length``
intact so both readers advance one record at a time on the same byte
boundaries; the malformed records corrupt the directory region only,
which both libraries surface as a yielded ``None`` under permissive
iteration. That makes the per-position shape — not just the count —
directly comparable.
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


_FIXTURE_1K = _REPO_ROOT / "tests" / "data" / "fixtures" / "1k_records.mrc"


def _read_both(data: bytes):
    """Read a valid corpus through both libraries, asserting equal counts."""
    pymarc = pytest.importorskip("pymarc")
    mrrc_records = list(mrrc.MARCReader(data))
    pymarc_records = list(pymarc.MARCReader(io.BytesIO(data)))
    assert len(mrrc_records) == len(pymarc_records)
    assert all(r is not None for r in mrrc_records)
    assert all(r is not None for r in pymarc_records)
    return mrrc_records, pymarc_records


def _assert_record_values_match(mrrc_record, pymarc_record, where: str) -> None:
    assert str(mrrc_record.leader) == str(pymarc_record.leader), f"{where}: leader"
    assert mrrc_record.title == pymarc_record.title, f"{where}: title"
    mrrc_fields = mrrc_record.fields()
    pymarc_fields = pymarc_record.fields
    assert [f.tag for f in mrrc_fields] == [f.tag for f in pymarc_fields], (
        f"{where}: field tag sequence"
    )
    for mrrc_field, pymarc_field in zip(mrrc_fields, pymarc_fields, strict=False):
        tag = mrrc_field.tag
        assert mrrc_field.format_field() == pymarc_field.format_field(), (
            f"{where} field {tag}: format_field()"
        )
        assert mrrc_field.value() == pymarc_field.value(), f"{where} field {tag}: value()"
    assert mrrc_record.as_marc() == pymarc_record.as_marc(), f"{where}: as_marc() bytes"


def test_value_level_parity_simple_book() -> None:
    """Titles, field values/formatting, and serialization bytes match
    pymarc on a single known-good record."""
    mrrc_records, pymarc_records = _read_both(_VALID.read_bytes())
    _assert_record_values_match(mrrc_records[0], pymarc_records[0], "simple_book")


def test_value_level_parity_1k_corpus() -> None:
    """Value-level parity with pymarc across the full 1k-record fixture:
    title, per-field format_field() and value(), and as_marc()
    byte-equality for every record."""
    if not _FIXTURE_1K.exists():
        pytest.skip(f"Fixture not found: {_FIXTURE_1K}")
    mrrc_records, pymarc_records = _read_both(_FIXTURE_1K.read_bytes())
    for i, (mrrc_record, pymarc_record) in enumerate(
        zip(mrrc_records, pymarc_records, strict=False)
    ):
        _assert_record_values_match(mrrc_record, pymarc_record, f"record {i}")


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
