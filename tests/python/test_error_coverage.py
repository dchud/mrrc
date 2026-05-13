"""Error-handling coverage harness (Python side).

Reads ``tests/error_coverage.toml`` and runs one assertion bundle per
``[[case]]`` entry through the relevant mrrc Python entry point. The
Rust harness (:file:`tests/error_coverage.rs`) does the same on the
Rust side; both consume the same manifest so the
documentation-as-spec contract is enforced uniformly across the FFI.

For each ``wired = true`` case whose ``trigger_kind`` this harness
supports, asserts that the documented exception class fires when the
parser exercises the trigger, with matching :attr:`code`,
:attr:`slug`, and the documented positional-context attributes
populated. Other cases are skipped with reasons drawn from the
manifest (for unwired cases) or from this harness (for cases whose
``trigger_kind`` is not yet implemented here).

Currently supported ``trigger_kind`` values:
    * ``parse_iso2709`` — feed bytes to :class:`mrrc.MARCReader` in
      strict mode and capture the raised exception.
    * ``parse_marcxml`` — feed text to :func:`mrrc.marcxml_to_record`.
    * ``parse_marcjson`` — feed text to :func:`mrrc.marcjson_to_record`.
    * ``writer`` — construct a record whose serialized length exceeds
      the ISO 2709 99999-byte limit and call
      :meth:`mrrc.MARCWriter.write_record`, capturing the
      :class:`mrrc.WriterError`.

Skipped on the Python side:

* ``io_error`` — E007 is documented to raise built-in ``OSError``,
  not a typed ``mrrc.IoError`` (pymarc-compat). The Rust harness
  asserts the typed variant.
* ``recovery_cap`` — the Rust core supports ``max_errors`` but the
  Python ``MARCReader`` binding does not yet expose it.
* ``accessor`` — cases without a per-case branch.
"""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

import pytest

import mrrc

if sys.version_info >= (3, 11):
    import tomllib
else:
    import tomli as tomllib  # type: ignore[no-redef]

_REPO_ROOT = Path(__file__).resolve().parents[2]
_MANIFEST_PATH = _REPO_ROOT / "tests" / "error_coverage.toml"


def _load_manifest() -> dict[str, Any]:
    with _MANIFEST_PATH.open("rb") as f:
        return tomllib.load(f)


_MANIFEST = _load_manifest()
_CASES: list[dict[str, Any]] = _MANIFEST["case"]


# Cases the Rust core handles correctly but where the Python binding's
# Rust→Python error conversion currently drops or fails to populate
# positional-context fields. Empty when no such regression is in flight;
# entries list a technical reason for each documented gap so the skip
# is visible in CI output until the binding-level fix lands.
_PYTHON_BINDING_REGRESSIONS: dict[str, str] = {}


def _fixture_path(case: dict[str, Any]) -> Path:
    rel = case.get("trigger_fixture")
    assert rel, f"case {case['id']}: trigger_kind requires a trigger_fixture but none set"
    return _REPO_ROOT / rel


def _exercise_strict(case: dict[str, Any]) -> mrrc.MrrcException:
    """Exercise the case's trigger and return the raised exception.
    Skips the test if this harness does not support the case's
    ``trigger_kind``; fails the test if no exception fires."""
    kind = case.get("trigger_kind", "parse_iso2709")

    if kind == "parse_iso2709":
        bytes_ = _fixture_path(case).read_bytes()
        reader = mrrc.MARCReader(
            bytes_,
            recovery_mode="strict",
            validation_level=case.get("validation_level", "structural"),
        )
        try:
            for _ in reader:
                pass
        except mrrc.MrrcException as e:
            return e
        pytest.fail(
            f"{case['id']} ({case['code']} / {case['variant']}): "
            f"expected {case['code']} error in strict mode, got clean iteration"
        )
    elif kind == "parse_marcxml":
        text = _fixture_path(case).read_text()
        try:
            mrrc.xml_to_record(text)
        except mrrc.MrrcException as e:
            return e
        pytest.fail(
            f"{case['id']} ({case['code']}): expected {case['code']} error from "
            f"xml_to_record, got clean parse"
        )
    elif kind == "parse_marcjson":
        text = _fixture_path(case).read_text()
        try:
            mrrc.marcjson_to_record(text)
        except mrrc.MrrcException as e:
            return e
        pytest.fail(
            f"{case['id']} ({case['code']}): expected {case['code']} error from "
            f"marcjson_to_record, got clean parse"
        )
    elif kind == "io_error":
        pytest.skip(
            "E007 on the Python side raises built-in OSError, not a typed "
            "mrrc.IoError (documented in docs/reference/error-codes.md#E007 "
            "as pymarc-compat). The Rust harness asserts the typed variant; "
            "this typed-class framework cannot assert built-in OSError."
        )
    elif kind == "recovery_cap":
        pytest.skip(
            "trigger_kind=recovery_cap exercises max_errors / FatalReaderError; "
            "the Rust core supports it but the Python MARCReader binding does "
            "not yet expose max_errors as a kwarg"
        )
    elif kind == "accessor":
        bytes_ = _fixture_path(case).read_bytes()
        reader = mrrc.MARCReader(bytes_, recovery_mode="strict")
        try:
            record = next(iter(reader))
        except StopIteration:
            pytest.fail(
                f"{case['id']} ({case['code']}): fixture parsed to no records; "
                "accessor cannot be exercised"
            )
        except mrrc.MrrcException as e:
            pytest.fail(
                f"{case['id']} ({case['code']}): fixture failed to parse cleanly "
                f"({e}); accessor cannot be exercised"
            )
        # Per-case branch: accessor names + arguments aren't yet expressed
        # in the manifest schema, so each accessor case wires its trigger
        # here. New accessor cases need a branch added.
        if case["id"] == "e105_field_not_found":
            try:
                record.get_field_or_err("999")
            except mrrc.MrrcException as e:
                return e
            pytest.fail(
                f"{case['id']} ({case['code']}): get_field_or_err('999') returned "
                "a field on simple_book.mrc; expected FieldNotFound"
            )
        pytest.skip(
            f"{case['id']}: trigger_kind=accessor case has no harness branch; "
            "add one in test_error_coverage.py"
        )
    elif kind == "writer":
        record = mrrc.Record()
        record.add_field(
            mrrc.Field(
                tag="999",
                indicator1=" ",
                indicator2=" ",
                subfields=[mrrc.Subfield("a", "x" * 100_000)],
            )
        )
        import io

        buf = io.BytesIO()
        writer = mrrc.MARCWriter(buf)
        try:
            writer.write_record(record)
        except mrrc.MrrcException as e:
            return e
        pytest.fail(
            f"{case['id']} ({case['code']}): expected {case['code']} error from "
            "MARCWriter.write_record on an oversize record, got success"
        )
    else:
        pytest.fail(f"{case['id']}: unknown trigger_kind {kind!r}")


@pytest.mark.parametrize("case", _CASES, ids=lambda c: c["id"])
def test_documented_error_fires(case: dict[str, Any]) -> None:
    if not case["wired"]:
        pytest.skip(case.get("skip_reason", "unwired"))

    if "strict" not in case["recovery_modes"]:
        pytest.skip(
            "case contract does not cover strict mode; non-strict "
            "assertions pending"
        )

    if case["id"] in _PYTHON_BINDING_REGRESSIONS:
        pytest.skip(
            f"python-binding regression: {_PYTHON_BINDING_REGRESSIONS[case['id']]}"
        )

    exc = _exercise_strict(case)

    assert exc.code == case["code"], (
        f"{case['id']}: expected code {case['code']}, "
        f"got {exc.code} ({type(exc).__name__})"
    )
    assert exc.slug == case["slug"], (
        f"{case['id']}: expected slug {case['slug']!r}, got {exc.slug!r}"
    )

    for field in case["expected_context"]:
        value = getattr(exc, field, None)
        assert value is not None, (
            f"{case['id']}: expected_context field {field!r} not populated; "
            f"exception attrs: {exc.to_dict()}"
        )


def test_manifest_is_well_formed() -> None:
    assert _MANIFEST["schema_version"] == 1, "schema_version drift"
    assert _CASES, "manifest has no cases"

    seen_ids: set[str] = set()
    parse_kinds = {"parse_iso2709", "parse_marcxml", "parse_marcjson"}
    for case in _CASES:
        case_id = case["id"]
        assert case_id not in seen_ids, f"duplicate case id {case_id}"
        seen_ids.add(case_id)

        kind = case.get("trigger_kind", "parse_iso2709")
        if kind in parse_kinds:
            assert "trigger_fixture" in case, (
                f"case {case_id}: trigger_kind {kind!r} requires a trigger_fixture"
            )
            fixture_path = _REPO_ROOT / case["trigger_fixture"]
            assert fixture_path.exists(), (
                f"case {case_id}: fixture {fixture_path} does not exist"
            )

        if not case["wired"]:
            assert case.get("skip_reason"), (
                f"case {case_id} is unwired but has no skip_reason"
            )


def test_coverage_tally(capsys: pytest.CaptureFixture[str]) -> None:
    """Emits ``wired in manifest: W/T`` to make the coverage state
    visible in CI output. Per-case pass/fail/skip is reported by
    :func:`test_documented_error_fires`."""
    wired = sum(1 for c in _CASES if c["wired"])
    total = len(_CASES)
    skipped = total - wired
    with capsys.disabled():
        print(f"\n[error_coverage] wired in manifest: {wired}/{total} (unwired: {skipped})")
