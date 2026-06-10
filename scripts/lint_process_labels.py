#!/usr/bin/env python3
"""Fail when persistent artifacts embed implementation-process labels.

Source, tests, user-facing docs, and workflows should describe the system in
timeless terms. Provenance belongs in git history; version context belongs in
the CHANGELOG. Bead IDs, PR/issue numbers, phase markers, and release-tagged
claims rot: they get renumbered, squashed, or become noise once readers are
past that version.

Scope: tracked files under src/, src-python/src/, mrrc/, tests/, docs/
(excluding the archival docs/history/ and docs/design/profiling/), .github/,
and README.md. The CHANGELOG is the load-bearing exception and is excluded.

A line containing the marker ``noqa: process-label`` is exempt, for the rare
genuine reference (e.g. a third-party dependency version).

Usage: python3 scripts/lint_process_labels.py
Exits non-zero (and prints offenders) on any unexcused match.
"""
from __future__ import annotations

import re
import subprocess
import sys

EXEMPT_MARKER = "noqa: process-label"

# (label, compiled pattern). Patterns are deliberately conservative: each
# targets a shape that is almost always a process label, not prose. We do NOT
# flag bare version triples (toolchain pins, dependency and example versions are
# legitimate) or the word "phase" (the GIL 3-phase model is real architecture) —
# only version references carrying a release-context cue are process labels.
PATTERNS = [
    ("bead id", re.compile(r"\bbd-[a-z0-9]{4}\b")),
    ("PR reference", re.compile(r"(?i)\bPR\s*#?\s*\d+\b")),
    ("issue/PR number", re.compile(r"(?<![\w&#/])#\d+\b")),
    ("release-context cue", re.compile(
        r"(?i)\b(?:since|as of|flipped in)\s+(?:version\s+)?\d+\.\d+\b")),
]

# Directories/files to sweep, and archival/exempt paths to skip.
INCLUDE_PREFIXES = (
    "src/", "src-python/src/", "mrrc/", "tests/", "docs/", ".github/",
)
INCLUDE_FILES = ("README.md",)
# docs/history/ and docs/design/ are archival investigation and proposal
# material, not published reference; they legitimately carry planning language.
EXCLUDE_PREFIXES = ("docs/history/", "docs/design/")
EXCLUDE_FILES = ("CHANGELOG.md",)


def tracked_files() -> list[str]:
    out = subprocess.run(
        ["git", "ls-files"], capture_output=True, text=True, check=True
    ).stdout.splitlines()
    files = []
    for path in out:
        if path in EXCLUDE_FILES or path.startswith(EXCLUDE_PREFIXES):
            continue
        if path in INCLUDE_FILES or path.startswith(INCLUDE_PREFIXES):
            files.append(path)
    return files


def main() -> int:
    offenders = []
    for path in tracked_files():
        try:
            with open(path, encoding="utf-8") as fh:
                lines = fh.readlines()
        except (UnicodeDecodeError, FileNotFoundError):
            continue
        for lineno, line in enumerate(lines, 1):
            if EXEMPT_MARKER in line:
                continue
            for label, pattern in PATTERNS:
                m = pattern.search(line)
                if m:
                    offenders.append((path, lineno, label, m.group(0),
                                      line.rstrip()))

    if not offenders:
        print("✓ process-label lint passed")
        return 0

    print("✗ process-label lint failed: persistent artifacts must not embed "
          "process labels (bead IDs, PR/issue numbers, phases, version-tagged "
          "claims).", file=sys.stderr)
    print(f"  Put provenance in git/CHANGELOG, or add '{EXEMPT_MARKER}' to a "
          "line that is a genuine reference.\n", file=sys.stderr)
    for path, lineno, label, match, text in offenders:
        snippet = text if len(text) <= 100 else text[:97] + "..."
        print(f"  {path}:{lineno}: [{label}] {match!r}", file=sys.stderr)
        print(f"      {snippet.strip()}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
