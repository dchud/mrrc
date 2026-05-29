#!/usr/bin/env python3
"""Reconcile the error-code sources of truth.

The v0.8 audit found wiring gaps shipped because nothing asserted "every
documented error code maps to a real MarcError variant with a production
call site and a wired coverage case." This script is the standing guard
against that class of drift — it runs in `.cargo/check.sh` and catches a
gap the error_coverage harness can't (a new code with no manifest case).

It reconciles three sources of truth and fails (exit 1) on any disagreement:

  1. `src/error.rs`            — the MarcError variants and their codes
                                 (the `code()` match arms).
  2. `docs/reference/error-codes.md` — the documented error codes.
  3. `tests/error_coverage.toml`     — the per-code coverage manifest.

Checks:
  A. Every MarcError code is documented in error-codes.md.
  B. Every documented E-code is a real MarcError code (W-codes are warnings,
     not MarcError variants, and are listed separately).
  C. Every MarcError code has at least one `wired = true` manifest case.
  D. No `wired = false` rows exist in the manifest.
  E. Every MarcError variant is constructed at least once in non-test code
     (the unwired-variant audit).

Run from the repo root: `python scripts/verify_error_docs.py`.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
ERROR_RS = REPO / "src" / "error.rs"
DOC = REPO / "docs" / "reference" / "error-codes.md"
MANIFEST = REPO / "tests" / "error_coverage.toml"
# Directories scanned for production (non-test) variant construction sites.
SRC_DIRS = [REPO / "src", REPO / "src-python" / "src"]


def variant_to_code() -> dict[str, str]:
    """Parse the `MarcError::Variant { .. } => "ENNN"` arms in error.rs."""
    text = ERROR_RS.read_text(encoding="utf-8")
    pairs = re.findall(r"MarcError::(\w+)\s*\{\s*\.\.\s*\}\s*=>\s*\"(E\d+)\"", text)
    if not pairs:
        sys.exit(f"FATAL: no code() arms parsed from {ERROR_RS}")
    return {variant: code for variant, code in pairs}


def documented_codes() -> tuple[set[str], set[str]]:
    """Return (E-codes, W-codes) documented in error-codes.md via `{ #CODE }`."""
    text = DOC.read_text(encoding="utf-8")
    anchors = set(re.findall(r"\{\s*#(E\d+|W\d+)\s*\}", text))
    e_codes = {c for c in anchors if c.startswith("E")}
    w_codes = {c for c in anchors if c.startswith("W")}
    if not e_codes:
        sys.exit(f"FATAL: no error-code anchors parsed from {DOC}")
    return e_codes, w_codes


def manifest_codes() -> tuple[dict[str, int], list[str]]:
    """Return ({code: wired_true_case_count}, [unwired_case_ids]).

    Parses `[[case]]` blocks for `code`, `id`, and `wired`. A regex parse
    (not tomllib) keeps this working on Python 3.10.
    """
    text = MANIFEST.read_text(encoding="utf-8")
    blocks = text.split("[[case]]")[1:]
    wired_count: dict[str, int] = {}
    unwired: list[str] = []
    for block in blocks:
        code_m = re.search(r'^\s*code\s*=\s*"(E\d+)"', block, re.MULTILINE)
        wired_m = re.search(r"^\s*wired\s*=\s*(true|false)", block, re.MULTILINE)
        id_m = re.search(r'^\s*id\s*=\s*"([^"]+)"', block, re.MULTILINE)
        if not code_m or not wired_m:
            continue
        code = code_m.group(1)
        if wired_m.group(1) == "true":
            wired_count[code] = wired_count.get(code, 0) + 1
        else:
            unwired.append(id_m.group(1) if id_m else f"{code} (unknown id)")
    return wired_count, unwired


def construction_sites(variants: set[str]) -> dict[str, int]:
    """Count literal `MarcError::Variant {` / `(` constructions in src code.

    Excludes match arms (`MarcError::Variant { .. }`) and test files. Every
    variant is constructed by a literal somewhere — directly or inside an
    `err_*` helper — so a zero count flags a genuinely unwired variant.
    """
    counts = {v: 0 for v in variants}
    for root in SRC_DIRS:
        for path in root.rglob("*.rs"):
            text = path.read_text(encoding="utf-8")
            is_test_file = path.name.endswith("_test.rs") or "/tests/" in str(path)
            if is_test_file:
                continue
            for variant in variants:
                # A struct construction has a field initializer (`name:`)
                # before its closing brace: `MarcError::V { record_index: ... }`.
                # A match arm uses bare bindings (`{ found, .. }` / `{ .. }`) with
                # no colon, so it is correctly excluded. `(?s)` lets the field
                # span newlines; `[^}]*?` stops at the first `}` if none matches.
                struct_ctor = re.findall(
                    rf"(?s)MarcError::{variant}\s*\{{[^}}]*?\w+:", text
                )
                tuple_ctor = re.findall(rf"MarcError::{variant}\s*\(", text)
                counts[variant] += len(struct_ctor) + len(tuple_ctor)
    return counts


def main() -> int:
    v2c = variant_to_code()
    codes = set(v2c.values())
    doc_e, doc_w = documented_codes()
    wired, unwired = manifest_codes()
    ctors = construction_sites(set(v2c))

    failures: list[str] = []

    # A. Every MarcError code documented.
    missing_doc = codes - doc_e
    if missing_doc:
        failures.append(f"A: MarcError codes not documented in error-codes.md: {sorted(missing_doc)}")

    # B. Every documented E-code is a real MarcError code.
    extra_doc = doc_e - codes
    if extra_doc:
        failures.append(f"B: error-codes.md documents E-codes with no MarcError variant: {sorted(extra_doc)}")

    # C. Every MarcError code has a wired manifest case.
    unwired_codes = codes - set(wired)
    if unwired_codes:
        failures.append(f"C: MarcError codes with no wired=true manifest case: {sorted(unwired_codes)}")

    # D. No wired=false rows.
    if unwired:
        failures.append(f"D: manifest has wired=false cases: {unwired}")

    # E. Every variant constructed in non-test code.
    zero_ctor = sorted(v for v, n in ctors.items() if n == 0)
    if zero_ctor:
        failures.append(f"E: MarcError variants with no non-test construction site: {zero_ctor}")

    # Printable table.
    print(f"{'CODE':6} {'VARIANT':22} {'DOC':4} {'WIRED':6} {'CTORS':6}")
    print("-" * 50)
    for variant, code in sorted(v2c.items(), key=lambda kv: kv[1]):
        print(
            f"{code:6} {variant:22} "
            f"{'yes' if code in doc_e else 'NO':4} "
            f"{wired.get(code, 0):<6} {ctors.get(variant, 0):<6}"
        )
    print("-" * 50)
    print(
        f"{len(codes)} MarcError codes | {len(doc_e)} documented E-codes "
        f"| {len(doc_w)} warning codes ({sorted(doc_w)}) "
        f"| {sum(wired.values())} wired manifest cases"
    )

    if failures:
        print("\nERROR-CODE RECONCILIATION: FAIL")
        for f in failures:
            print(f"  - {f}")
        return 1
    print("\nERROR-CODE RECONCILIATION: PASS — error-code sources of truth reconcile.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
