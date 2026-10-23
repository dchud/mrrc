#!/usr/bin/env python3
"""Fail when a Rust code block on the documentation site is not compiled.

Every ```rust fence on a site page must hold only snippet include lines
(``--8<-- "file.rs:section"``) that pull code from a compiled file under
tests/: a top-level ``tests/*.rs`` test target, or a ``tests/doc_examples/``
file that ``tests/doc_examples/main.rs`` declares as a module. mkdocs reads
the includes through pymdownx.snippets (``base_path: [tests]``), and
``cargo test`` compiles the code, so the code on the site cannot go stale
without failing the build.

The lint checks, for the pages mkdocs builds (``docs/**/*.md`` minus the
``exclude_docs`` patterns in mkdocs.yml):

- a ``rust`` (or ``rs``) fence holds nothing but include lines;
- each include names a compiled file and, if given, a section that the file
  defines, with matching ``[start:name]`` / ``[end:name]`` markers;
- every file in ``tests/doc_examples/`` is declared in ``main.rs``;
- every section marked in ``tests/doc_examples/`` or a top-level test file
  is shown on some page.

Code that is deliberately not compilable Rust (a signature listing,
pseudo-code, a compiler error message) belongs in a ``text`` fence.

Usage: python3 scripts/lint_doc_examples.py
Exits non-zero (and prints offenders) on any violation.
"""

from __future__ import annotations

import fnmatch
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / "docs"
SNIPPET_BASE = ROOT / "tests"
EXAMPLES = SNIPPET_BASE / "doc_examples"

FENCE_OPEN = re.compile(
    r"^(?P<indent>[ \t]*)(?P<fence>`{3,}|~{3,})(?P<info>.*)$"
)
INCLUDE = re.compile(
    r"""^[ \t]*-+8<-+[ \t]+(["'])(?P<target>[^"']+)\1[ \t]*$"""
)
TARGET = re.compile(
    r"^(?P<path>[^:]+\.rs)(?::(?P<section>[a-z][-_0-9a-z]*))?$"
)
MARKER = re.compile(
    r"-+8<-+[ \t]+\[[ \t]*(?P<kind>start|end)[ \t]*:[ \t]*"
    r"(?P<name>[a-z][-_0-9a-z]*)[ \t]*\]"
)
MOD_DECL = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(?P<name>\w+)\s*;", re.M
)


def excluded_patterns() -> list[str]:
    """The gitignore-style patterns under ``exclude_docs:`` in mkdocs.yml."""
    lines = (ROOT / "mkdocs.yml").read_text(encoding="utf-8").splitlines()
    patterns: list[str] = []
    for i, line in enumerate(lines):
        if re.match(r"^exclude_docs:\s*\|", line):
            for follow in lines[i + 1 :]:
                if follow and not follow[0].isspace():
                    break
                if follow.strip() and not follow.strip().startswith("#"):
                    patterns.append(follow.strip())
            break
    return patterns


def site_pages() -> list[Path]:
    patterns = excluded_patterns()
    pages = []
    for page in sorted(DOCS.rglob("*.md")):
        rel = page.relative_to(DOCS).as_posix()
        if any(
            rel.startswith(p.lstrip("/"))
            if p.endswith("/")
            else fnmatch.fnmatch(rel, p.lstrip("/"))
            for p in patterns
        ):
            continue
        pages.append(page)
    return pages


def rust_fences(lines: list[str]):
    """Yield (line number, body lines) for each rust fence, skipping fences
    nested inside other fences (a ```rust line inside a ````markdown block is
    content, not a fence)."""
    i = 0
    while i < len(lines):
        m = FENCE_OPEN.match(lines[i])
        if not m:
            i += 1
            continue
        fence = m.group("fence")
        close = re.compile(
            rf"^[ \t]*{re.escape(fence[0])}{{{len(fence)},}}[ \t]*$"
        )
        j = i + 1
        while j < len(lines) and not close.match(lines[j]):
            j += 1
        language = re.split(
            r"[\s,{}]", m.group("info").strip().lstrip("{.") + " "
        )[0]
        if language.lower() in ("rust", "rs"):
            yield i + 1, lines[i + 1 : j]
        i = j + 1


def sections(path: Path) -> tuple[dict[str, int], list[str]]:
    """Section name -> start line for one file, and any marker problems."""
    starts: dict[str, int] = {}
    ended: set[str] = set()
    problems = []
    for lineno, line in enumerate(
        path.read_text(encoding="utf-8").splitlines(), 1
    ):
        m = MARKER.search(line)
        if not m:
            continue
        name = m.group("name")
        if m.group("kind") == "start":
            if name in starts:
                problems.append(
                    f"{path.relative_to(ROOT)}:{lineno}: section "
                    f"'{name}' starts twice"
                )
            starts[name] = lineno
        elif name not in starts or name in ended:
            problems.append(
                f"{path.relative_to(ROOT)}:{lineno}: section "
                f"'{name}' ends without a start"
            )
        else:
            ended.add(name)
    for name, lineno in starts.items():
        if name not in ended:
            problems.append(
                f"{path.relative_to(ROOT)}:{lineno}: section "
                f"'{name}' has no end marker"
            )
    return starts, problems


def main() -> int:
    problems: list[str] = []

    declared = set(
        MOD_DECL.findall((EXAMPLES / "main.rs").read_text(encoding="utf-8"))
    )
    example_files = sorted(
        p for p in EXAMPLES.glob("*.rs") if p.name != "main.rs"
    )
    for path in example_files:
        if path.stem not in declared:
            problems.append(
                f"{path.relative_to(ROOT)}: not declared with "
                f"`mod {path.stem};` in tests/doc_examples/main.rs, "
                "so it is never compiled"
            )
    if any(EXAMPLES.rglob("*/*.rs")):
        problems.append(
            "tests/doc_examples/: keep example files at the top "
            "level, one per page"
        )

    section_cache: dict[Path, dict[str, int]] = {}

    def file_sections(path: Path) -> dict[str, int]:
        if path not in section_cache:
            found, marker_problems = sections(path)
            section_cache[path] = found
            problems.extend(marker_problems)
        return section_cache[path]

    shown: set[tuple[Path, str]] = set()
    for page in site_pages():
        rel_page = page.relative_to(ROOT)
        lines = page.read_text(encoding="utf-8").splitlines()
        for lineno, body in rust_fences(lines):
            where = f"{rel_page}:{lineno}"
            includes = [line for line in body if line.strip()]
            if not includes:
                problems.append(f"{where}: empty rust code block")
            for line in includes:
                m = INCLUDE.match(line)
                if not m:
                    problems.append(
                        f"{where}: inline Rust code; move it into a section "
                        "of a tests/doc_examples/ file and include it (see "
                        "docs/contributing/testing.md)"
                    )
                    break
                target = m.group("target").strip()
                t = TARGET.match(target)
                if not t:
                    problems.append(
                        f"{where}: include '{target}' must name a "
                        ".rs file and, optionally, a section "
                        "(file.rs:section-name)"
                    )
                    continue
                path = (SNIPPET_BASE / t.group("path")).resolve()
                compiled = path.parent == SNIPPET_BASE or (
                    path.parent == EXAMPLES
                    and (path.name == "main.rs" or path.stem in declared)
                )
                if not path.is_file():
                    problems.append(
                        f"{where}: include '{target}': no file "
                        f"tests/{t.group('path')}"
                    )
                    continue
                section = t.group("section")
                if section is not None:
                    shown.add((path, section))
                if not compiled:
                    problems.append(
                        f"{where}: include '{target}' is not a "
                        "compiled file (a top-level tests/*.rs "
                        "file, or a tests/doc_examples/ file "
                        "declared in main.rs)"
                    )
                    continue
                if section is not None and section not in file_sections(path):
                    problems.append(
                        f"{where}: include '{target}': no section "
                        f"'{section}' in tests/{t.group('path')}"
                    )

    # Sections exist only to be shown, in the example files and in any
    # top-level test file that marks one.
    for path in example_files + sorted(SNIPPET_BASE.glob("*.rs")):
        for name, lineno in file_sections(path.resolve()).items():
            if (path.resolve(), name) not in shown:
                problems.append(
                    f"{path.relative_to(ROOT)}:{lineno}: section "
                    f"'{name}' is not shown on any page"
                )

    if not problems:
        print("✓ doc example lint passed")
        return 0

    print(
        "✗ doc example lint failed: every Rust code block on the site must "
        "be included from compiled code.",
        file=sys.stderr,
    )
    for problem in problems:
        print(f"  {problem}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
