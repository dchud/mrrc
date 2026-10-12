# Testing

This guide covers running the MRRC test suite.

## Quick Start

```bash
# Run all pre-push checks (~30 seconds)
.cargo/check.sh

# This runs:
# - rustfmt (formatting)
# - clippy (linting)
# - cargo doc (documentation)
# - cargo audit (security)
# - maturin build
# - pytest (Python tests, excluding benchmarks)
```

## Test Categories

### Rust Tests

```bash
# All Rust tests
cargo test

# Specific test
cargo test test_name

# Tests with output
cargo test -- --nocapture

# Documentation tests
cargo test --doc
```

### Python Tests

```bash
# All Python tests (excluding benchmarks)
pytest tests/python/ -m "not benchmark"

# Specific test file
pytest tests/python/test_reader.py

# Specific test
pytest tests/python/test_reader.py::test_read_simple_record

# With verbose output
pytest -v tests/python/

# With coverage
pytest --cov=mrrc tests/python/
```

### Benchmarks

```bash
# Rust benchmarks (Criterion)
cargo bench

# Python benchmarks
pytest tests/python/ -m benchmark
```

### Cross-library parity (pymarc)

`tests/python/test_pymarc_parity.py` compares mrrc's permissive
iteration shape (record-vs-`None` per record) against pymarc over a
mixed-quality corpus. pymarc is an optional comparison target, not a
project dependency, so the test is guarded by `pytest.importorskip`
and **skips silently** unless pymarc is installed. To actually run it,
install pymarc for the invocation:

```bash
uv run --with 'pymarc==5.3.1' python -m pytest tests/python/test_pymarc_parity.py -q
```

Worth running when changing reader iteration, recovery-mode defaults,
or anything affecting how malformed records are surfaced.

`tests/python/test_pymarc_compat_accessors.py` carries a similar
pymarc-guarded parity check for the `current_exception` /
`current_chunk` accessors.

## Test Fixtures

Test data is located in `tests/data/`:

| File | Description | Records |
|------|-------------|---------|
| `simple_book.mrc` | Basic bibliographic record | 1 |
| `multi_records.mrc` | Multiple records | 3 |
| `simple_authority.mrc` | Authority record | 1 |
| `simple_holdings.mrc` | Holdings record | 1 |
| `with_control_fields.mrc` | Record with 008 field | 1 |

Benchmark fixtures in `tests/data/fixtures/`:

| File | Size | Records |
|------|------|---------|
| `1k_records.mrc` | 257 KB | 1,000 |
| `10k_records.mrc` | 2.5 MB | 10,000 |

## Test Organization

```
tests/
├── doc_examples/               # Rust code shown on the documentation site
├── python/
│   ├── test_reader.py          # MARCReader tests
│   ├── test_writer.py          # MARCWriter tests
│   ├── test_record.py          # Record manipulation
│   ├── test_query_dsl.py       # Query DSL tests
│   ├── test_formats.py         # Format conversion
│   ├── test_pymarc_compat.py   # pymarc compatibility
│   └── test_benchmark_*.py     # Performance tests
└── data/
    └── *.mrc                   # Test fixtures
```

## Writing Tests

### Rust Test Example

```rust
--8<-- "doc_examples/testing.rs:rust-test-example"
```

### Python Test Example

```python
import pytest
from mrrc import MARCReader

def test_read_simple_record():
    with open("tests/data/simple_book.mrc", "rb") as f:
        reader = MARCReader(f)
        record = next(reader)
        assert record.title is not None
```

## Rust Examples on the Documentation Site

Every Rust code block on the documentation site is compiled. The code lives in
`tests/doc_examples/`, an integration test target, and the pages include it
with the
[snippets](https://facelessuser.github.io/pymdown-extensions/extensions/snippets/)
extension. When an API change breaks an example, `cargo test` and Clippy fail,
and fixing the code in `tests/doc_examples/` fixes the page.

**Where the code lives.** The code for `docs/<section>/<page>.md` is in
`tests/doc_examples/<page>.rs`, with hyphens in the page name written as
underscores, and `tests/doc_examples/main.rs` declares the file with
`mod <page>;`. Each code block on the page is a named section of that file: the
lines between `// --8<-- [start:<name>]` and `// --8<-- [end:<name>]`. The page
shows a section with an include line in a `rust` fence:

````markdown
```rust
;--8<-- "doc_examples/reading_records.rs:basic-reading"
```
````

Include paths are relative to `tests/`, so a page can also show a section of a
top-level test file, as the [fuzzing guide](fuzzing.md) does with
`fuzz_regressions.rs:harness`.

**Code around a section.** Only the lines between the markers appear on the
page, with their common indentation removed. The code around them supplies what
the page's prose defines. A fragment that uses a `record` (or a `leader`, a
`graph`, a `file`) sits in a function that takes it as a parameter, returning
`mrrc::Result<()>` when the fragment uses `?`. A complete program, with its own
`use` lines and `fn main`, sits in a module of its own:

```text
fn access_fields(record: Record) {
    // ;--8<-- [start:access-fields]
    use mrrc::RecordHelpers;

    println!("Title: {:?}", record.title());
    // ;--8<-- [end:access-fields]
}

mod read_records {
    // ;--8<-- [start:read-records]
    use mrrc::{MarcReader, RecordHelpers};
    use std::fs::File;

    fn main() -> mrrc::Result<()> {
        ...
    }
    // ;--8<-- [end:read-records]
}
```

Put the `use` lines a reader needs inside the section, so the code on the page
can be copied as it stands. Code outside the sections (fixtures, attributes,
helper functions) stays off the page.

**Running the examples.** `cargo test --test doc_examples` compiles every
example and runs the ones that can run against the test fixtures: fragments
that take a record are called with each record in
`tests/data/fixtures/fidelity_test_100.mrc`. Complete programs that open files
such as `records.mrc` are compiled but not run. Clippy checks the examples with
the workspace's pedantic lints; `main.rs` allows the few that page-shaped code
trips, with the reason for each.

**The lint.** `scripts/lint_doc_examples.py` runs in `.cargo/check.sh` and in
the Docs Build Check workflow. It fails when:

- a `rust` (or `rs`) fence on a page the site builds holds anything but include
  lines;
- an include names a file that is not compiled (a top-level `tests/*.rs` file,
  or a `tests/doc_examples/` file declared in `main.rs`), or a section the file
  does not define;
- a section has no matching end marker;
- a file in `tests/doc_examples/` is not declared in `main.rs`;
- a section is not shown on any page.

There is no exemption marker. Code that is deliberately not compilable Rust,
such as a signature listing, pseudo-code, or a compiler error message, goes in a
`text` fence. With `check_paths` set in `mkdocs.yml`, `mkdocs build` also fails
on a missing file or section.

**To add or change an example:**

1. Write or edit the section in `tests/doc_examples/<page>.rs` (create the file
   and declare it in `main.rs` for a page that has no Rust code yet).
2. Put the include line in the page.
3. Run `cargo test --test doc_examples`, `python3
   scripts/lint_doc_examples.py`, and `uv run mkdocs build --strict` (all part of
   `.cargo/check.sh`), and check the page with `uv run mkdocs serve`.

## Memory safety checks

`.cargo/check.sh --memory-checks` runs the library tests under AddressSanitizer
(ASAN) to catch use-after-free, leaks, heap buffer overflows, and data races. It
needs a nightly toolchain; known or expected reports are suppressed via
`.cargo/asan_suppressions.txt`.

Run it after changes to memory-critical code, when updating dependencies, or as
part of pre-release validation. CI runs the same ASAN pass nightly via
`.github/workflows/memory-safety.yml`.

## CI Integration

Tests run automatically on:
- Every push to any branch
- Every pull request

CI workflows:
- `test.yml` - Cargo tests
- `python-build.yml` - Python build and tests
- `lint.yml` - Format and lint checks

## Troubleshooting

### Tests fail with import error

Rebuild Python bindings:
```bash
uv run maturin develop
```

### Benchmark tests are slow

Skip them for quick iteration:
```bash
pytest tests/python/ -m "not benchmark"
```

### Test fixtures not found

Ensure you're running from the repository root:
```bash
cd /path/to/mrrc
pytest tests/python/
```

## See Also

- [Development Setup](development-setup.md) - Setting up the environment
- [Architecture](architecture.md) - Understanding the codebase
