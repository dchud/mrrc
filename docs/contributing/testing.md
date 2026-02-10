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

# Quick benchmark comparison
pytest tests/python/test_benchmark_comparison.py -v
```

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
#[test]
fn test_parse_simple_record() {
    let data = include_bytes!("../tests/data/simple_book.mrc");
    let record = Record::from_marc21(data).unwrap();
    assert_eq!(record.title(), Some("Test Title".to_string()));
}
```

### Python Test Example

```python
import pytest
from mrrc import MARCReader

def test_read_simple_record():
    with open("tests/data/simple_book.mrc", "rb") as f:
        reader = MARCReader(f)
        record = next(reader)
        assert record.title() is not None
```

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
