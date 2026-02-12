# Project Layout

This document describes how the MRRC codebase is organized and how its
components relate to each other. For implementation details (GIL release
strategy, parser internals, concurrency model), see
[Architecture](architecture.md).

## How the Pieces Fit Together

MRRC is a **Rust library with Python bindings**. The core parsing,
serialization, and record manipulation logic lives in pure Rust. A
separate crate wraps that logic in Python-callable functions and classes
using [PyO3](https://pyo3.rs/), and [maturin](https://www.maturin.rs/)
packages everything into a Python wheel that users install with `pip`.

```
┌─────────────────────────────────────────────────────┐
│  Python user code                                   │
│    import mrrc                                      │
│    reader = mrrc.MARCReader("file.mrc")             │
├─────────────────────────────────────────────────────┤
│  mrrc/  (Python package)                            │
│    __init__.py  — re-exports, pymarc-compat wrappers│
│    _mrrc.pyi    — type stubs for IDE support        │
│    formats/     — pure-Python format helpers         │
├─────────────────────────────────────────────────────┤
│  src-python/  (PyO3 bindings crate, "mrrc-python")  │
│    Compiled to _mrrc.cpython-*.so by maturin        │
│    Wraps Rust types as Python classes                │
│    Manages GIL release during parsing               │
├─────────────────────────────────────────────────────┤
│  src/  (core Rust library crate, "mrrc")            │
│    Pure Rust, no Python dependency                  │
│    Parsing, serialization, queries, encoding        │
└─────────────────────────────────────────────────────┘
```

When a user calls `mrrc.MARCReader("file.mrc")`, the call flows:

1. `mrrc/__init__.py` imports from the compiled `_mrrc` extension module
2. `_mrrc` is the shared library built from `src-python/`, linked against
   the core `mrrc` crate
3. The core crate does the actual parsing work

## Directory Structure

### `src/` — Core Rust Library

The `mrrc` crate. Pure Rust with no Python dependency. This is where
parsing, serialization, encoding, and query logic lives.

Key modules:

| Module | Purpose |
|--------|---------|
| `reader.rs` | ISO 2709 binary record reader |
| `writer.rs` | ISO 2709 binary record writer |
| `record.rs` | Record, Field, Subfield data structures |
| `leader.rs` | MARC leader (24-byte header) parsing |
| `encoding.rs` | MARC-8 ↔ UTF-8 character conversion |
| `field_query.rs` | Query DSL for searching fields |
| `json.rs`, `xml.rs`, `marcjson.rs` | Serialization formats |
| `dublin_core.rs`, `mods.rs` | Metadata crosswalk formats |
| `bibframe/` | BIBFRAME RDF conversion |
| `csv.rs` | CSV export |
| `boundary_scanner.rs` | Fast record boundary detection |
| `rayon_parser_pool.rs` | Parallel parsing with Rayon |
| `producer_consumer_pipeline.rs` | Threaded producer-consumer reader |

This crate is also usable as a standalone Rust library (`cargo add mrrc`),
independent of Python.

### `src-python/` — PyO3 Bindings

The `mrrc-python` crate. Depends on the core `mrrc` crate (via
`mrrc = { path = ".." }` in its `Cargo.toml`) and on PyO3 for the
Python ↔ Rust bridge.

This crate:

- Wraps Rust types as Python classes (`PyRecord`, `PyField`, `PyMARCReader`, etc.)
- Implements the three-phase GIL release pattern (see [Architecture](architecture.md))
- Handles type detection for `MARCReader` inputs (file paths, bytes, file objects)
- Compiles to a `cdylib` shared library (`_mrrc.cpython-*.so`)

Key modules:

| Module | Purpose |
|--------|---------|
| `lib.rs` | PyO3 module definition, exports all Python-visible types |
| `unified_reader.rs` | `MARCReader` — dispatches to the right backend |
| `backend.rs` | `ReaderBackend` enum (RustFile, Cursor, PythonFile) |
| `wrappers.rs` | `Record`, `Field`, `Leader` Python wrappers |
| `writers.rs` | `MARCWriter` Python wrapper |
| `query.rs` | Query DSL Python interface |
| `formats.rs` | Format conversion function exports |
| `bibframe.rs` | BIBFRAME Python wrappers |

### `mrrc/` — Python Package

The installable Python package. Uses maturin's
[mixed Rust/Python layout](https://www.maturin.rs/project_layout#mixed-rustpython-project):
pure Python code lives here alongside the compiled extension.

| File | Purpose |
|------|---------|
| `__init__.py` | Re-exports from `_mrrc`, adds pymarc-compatible wrappers (e.g. `Record` subclass with kwargs) |
| `_mrrc.pyi` | Type stubs so IDEs and type checkers understand the Rust extension |
| `py.typed` | PEP 561 marker — tells type checkers this package ships inline types |
| `formats/` | Pure-Python format helper classes |
| `rayon_parser_pool.py` | Python-side helpers for Rayon parallel parsing |

### `tests/` — Test Suites

```
tests/
├── *.rs              # 17 Rust integration test files (bibframe, mods, field query, etc.)
├── common/           # Shared Rust test utilities
├── data/             # MARC test fixtures (.mrc files, BIBFRAME baselines, MODS samples)
│   └── fixtures/     # Large benchmark fixtures (1k, 5k, 10k records)
└── python/           # 27 Python test files (pytest)
```

Rust unit tests live inline in `src/` (`#[cfg(test)]` modules).
Rust integration tests live in `tests/*.rs`.
Python tests live in `tests/python/` and are run with pytest.

### Other Directories

| Directory | Purpose |
|-----------|---------|
| `benches/` | Rust benchmarks (Criterion/Codspeed) |
| `examples/` | Example code in both Rust and Python |
| `scripts/` | Profiling, benchmarking, and fixture generation scripts |
| `docs/` | Documentation (mkdocs) |
| `.cargo/` | Cargo config and `check.sh` (local CI script) |
| `.githooks/` | Optional git hooks (pre-push runs check.sh) |
| `.github/workflows/` | CI workflows (lint, test, build, benchmark, etc.) |

## Build System

### How maturin builds the package

[maturin](https://www.maturin.rs/) is the build backend (declared in
`pyproject.toml`). When you run `maturin develop` or `pip install`:

1. maturin reads `pyproject.toml` to find the manifest path
   (`src-python/Cargo.toml`) and the Python package name (`mrrc._mrrc`)
2. Cargo compiles `src-python/` as a `cdylib`, linking against the core
   `mrrc` crate from `src/`
3. The resulting shared library is placed at `mrrc/_mrrc.cpython-*.so`
4. maturin bundles this `.so` with the pure Python files in `mrrc/` into
   a wheel

### Cargo workspace

The root `Cargo.toml` defines a workspace with two members:

- `.` (root) — the `mrrc` core library crate
- `src-python` — the `mrrc-python` PyO3 bindings crate

This means `cargo test`, `cargo clippy`, etc. operate on both crates.
The bindings crate depends on the core crate, so changes to `src/` are
picked up automatically when rebuilding the Python extension.

### Development build commands

```bash
# Rebuild the Python extension after Rust changes (debug, fast)
uv run maturin develop

# Rebuild with optimizations (for benchmarking)
uv run maturin develop --release

# Run all local checks (fmt, clippy, docs, audit, build, tests, ruff)
.cargo/check.sh

# Quick checks (skip docs, audit, maturin build)
.cargo/check.sh --quick
```

## Configuration Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Rust workspace + core crate config |
| `src-python/Cargo.toml` | PyO3 bindings crate config |
| `pyproject.toml` | Python package metadata, maturin settings, pytest/mypy/ruff config |
| `rustfmt.toml` | Rust formatting rules |
| `clippy.toml` | Clippy lint thresholds |
| `codecov.yml` | Code coverage settings |
| `mkdocs.yml` | Documentation site config |

## Common Development Workflows

**Adding a new Rust feature exposed to Python:**

1. Implement the feature in `src/` (core crate)
2. Write Rust unit tests inline and/or integration tests in `tests/*.rs`
3. Add PyO3 wrapper in `src-python/src/`
4. Export from `src-python/src/lib.rs`
5. Re-export from `mrrc/__init__.py`
6. Add type stub to `mrrc/_mrrc.pyi`
7. Write Python tests in `tests/python/`
8. Run `.cargo/check.sh`

**Adding a pure-Python feature (no Rust changes):**

1. Add to `mrrc/__init__.py` or a new file in `mrrc/`
2. Write Python tests in `tests/python/`
3. Run `.cargo/check.sh --quick`

**Changing only Rust internals (no API change):**

1. Edit files in `src/`
2. Run `cargo test` for fast feedback
3. Run `.cargo/check.sh` before pushing
