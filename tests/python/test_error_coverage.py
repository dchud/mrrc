"""Error-handling coverage harness (Python side).

Reads ``tests/error_coverage.toml`` and runs one assertion bundle per
``[[case]]`` entry through :class:`mrrc.MARCReader`. The Rust harness
(:file:`tests/error_coverage.rs`) does the same on the Rust side; both
consume the same manifest so the documentation-as-spec contract is
enforced uniformly across the FFI.

For each ``wired = true`` case, asserts that the documented exception
class fires when the parser reads the fixture bytes, with matching
:attr:`code`, :attr:`slug`, and the documented positional-context
attributes populated. For each ``wired = false`` case, the test is
skipped with the manifest's reason so the gap between docs and
implementation is visible in CI output.

The harness exercises strict mode for now. Lenient and permissive
contracts depend on per-record diagnostic surfaces that are tracked
separately; cases declare which modes their contract covers so the
harness can extend in place when those surfaces exist.
"""

from __future__ import annotations

import tomllib
from pathlib import Path

import pytest

import mrrc

_REPO_ROOT = Path(__file__).resolve().parents[2]
_MANIFEST_PATH = _REPO_ROOT / "tests" / "error_coverage.toml"


def _load_manifest() -> dict:
    with _MANIFEST_PATH.open("rb") as f:
        return tomllib.load(f)


_MANIFEST = _load_manifest()
_CASES = _MANIFEST["case"]


def _drive_strict(case: dict) -> mrrc.MrrcException:
    """Drive ``MARCReader`` over the case's fixture in strict mode and
    return the raised exception. Fails the test if no exception fires
    or if the fixture is unreadable."""
    fixture = (_REPO_ROOT / case["trigger_fixture"]).read_bytes()
    reader = mrrc.MARCReader(fixture, recovery_mode="strict")
    try:
        for _ in reader:
            pass
    except mrrc.MrrcException as e:
        return e
    pytest.fail(
        f"{case['id']} ({case['code']} / {case['variant']}): "
        f"expected {case['code']} error in strict mode, got clean iteration"
    )


@pytest.mark.parametrize("case", _CASES, ids=lambda c: c["id"])
def test_documented_error_fires(case: dict) -> None:
    if not case["wired"]:
        pytest.skip(case.get("skip_reason", "unwired"))

    exc = _drive_strict(case)

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
    for case in _CASES:
        case_id = case["id"]
        assert case_id not in seen_ids, f"duplicate case id {case_id}"
        seen_ids.add(case_id)

        fixture_path = _REPO_ROOT / case["trigger_fixture"]
        assert fixture_path.exists(), (
            f"case {case_id}: fixture {fixture_path} does not exist"
        )

        if not case["wired"]:
            assert case.get("skip_reason"), (
                f"case {case_id} is unwired but has no skip_reason"
            )


def test_coverage_tally(capsys: pytest.CaptureFixture[str]) -> None:
    """Emits ``error coverage: X/Y wired (skipped: Z)`` so the count is
    visible in CI output. Failure is delegated to
    ``test_documented_error_fires``; this test only prints."""
    wired = sum(1 for c in _CASES if c["wired"])
    total = len(_CASES)
    skipped = total - wired
    with capsys.disabled():
        print(f"\n[error_coverage] wired: {wired}/{total} (skipped: {skipped})")
