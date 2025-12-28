# Python Wrapper Implementation Strategies

**Date:** 2025-12-28  
**Status:** DRAFT - Strategy Documents  
**Task:** mrrc-9ic.9

This document codifies pre-implementation strategies for the Python wrapper to ensure Phase 1 can proceed smoothly without discovering critical gaps during implementation.

---

## 1. Type Hint & IDE Support Strategy

### 1.1 Overview
Python users expect IDE autocomplete and type checking support. PyO3 generates `.pyi` stub files; we must ensure they're discoverable and correct.

### 1.2 Approach

#### Step 1: Configure Maturin for .pyi Generation
```toml
# pyproject.toml
[tool.maturin]
python-packages = ["mrrc"]
module-name = "mrrc._mrrc"  # Native module name
```

#### Step 2: Expose Python Types via __init__.py
```python
# src-python/mrrc/__init__.py
from mrrc._mrrc import (
    Record,
    Field,
    Leader,
    MARCReader,
    MARCWriter,
    MarcException,
)

__all__ = [
    "Record",
    "Field",
    "Leader",
    "MARCReader",
    "MARCWriter",
    "MarcException",
]
```

#### Step 3: Add py.typed Marker (PEP 561)
Create empty file `src-python/mrrc/py.typed` (no content needed)
- Signals to type checkers that this package has type information
- Must be included in wheel distribution

#### Step 4: Validate Type Hints
```bash
# Install type checking tools
pip install mypy pyright pytest

# Test with mypy
mypy tests/python/ --strict

# Test with pyright
pyright tests/python/
```

### 1.3 Configuration

```toml
# pyproject.toml additions
[tool.mypy]
python_version = "3.9"
warn_return_any = true
warn_unused_configs = true
disallow_untyped_defs = true
disallow_incomplete_defs = true

[tool.pyright]
pythonVersion = "3.9"
typeCheckingMode = "strict"
```

### 1.4 Docstring Format (for IDE inference)
Use Google-style docstrings with type hints:

```rust
/// Read the next MARC record from the input stream.
///
/// # Returns
/// A Record if more data is available, None if EOF reached.
///
/// # Errors
/// Returns an error if the MARC data is malformed or encoding fails.
///
/// # Example
/// ```python
/// reader = MARCReader(open("records.mrc", "rb"))
/// record = reader.read_record()
/// if record:
///     print(record.title())
/// ```
#[pyo3(text_signature = "(self) -> Optional[Record]")]
pub fn read_record(&mut self) -> PyResult<Option<PyRecord>> {
    // ...
}
```

### 1.5 Testing Type Hints
```python
# tests/python/test_types.py
from mrrc import Record, Field, MARCReader
from typing import Optional

def test_reader_returns_record() -> None:
    """Verify type hints are correct."""
    reader: MARCReader = MARCReader(open("test.mrc", "rb"))
    record: Optional[Record] = reader.read_record()
    assert isinstance(record, Record) or record is None
```

### 1.6 Build Configuration
```bash
# In GitHub Actions or CI:
# Step 1: Build wheels (maturin will generate .pyi files)
maturin build --release

# Step 2: Unpack wheel and verify .pyi files exist
unzip dist/mrrc-*.whl -d /tmp/wheel_check
ls /tmp/wheel_check/mrrc/*.pyi

# Step 3: Type check against generated stubs
mypy tests/python/
```

---

## 2. Python Documentation Strategy

### 2.1 Overview
Rust doc comments don't generate Python documentation. We need a clear approach for building Python API docs.

### 2.2 Docstring Convention
Use **Google-style** docstrings in Rust code (PyO3 will include them in `__doc__`):

```rust
#[pyclass(name = "Record")]
pub struct PyRecord {
    inner: mrrc::Record,
}

#[pymethods]
impl PyRecord {
    /// Create a new MARC record with the given leader.
    ///
    /// Args:
    ///     leader: A Leader object defining record type and encoding.
    ///
    /// Returns:
    ///     A new Record instance.
    ///
    /// Example:
    ///     >>> from mrrc import Record, Leader
    ///     >>> leader = Leader(...)
    ///     >>> record = Record(leader)
    #[new]
    pub fn new(leader: PyLeader) -> Self {
        PyRecord {
            inner: mrrc::Record::new(leader.inner.clone()),
        }
    }

    /// Get the title of the record.
    ///
    /// Extracts the main title from field 245 subfield 'a'.
    ///
    /// Returns:
    ///     The title string, or None if not present.
    pub fn title(&self) -> Option<String> {
        self.inner.title()
    }
}
```

### 2.3 Documentation Generation

#### Option A: Sphinx with `sphinx-autodoc`
```bash
# Generate HTML docs from docstrings
pip install sphinx sphinx-rtd-theme

# Create docs/conf.py
# Run: sphinx-build -b html docs docs/_build/
```

#### Option B: mkdocs (Simpler)
```yaml
# mkdocs.yml
site_name: mrrc Python API
theme:
  name: material
nav:
  - Home: index.md
  - API Reference:
    - Record: api/record.md
    - Field: api/field.md
    - MARCReader: api/reader.md
```

#### Option C: Manual Docs (Starting Point)
```markdown
# docs/python_api.md
## Record Class

### Record(leader)
Create a new MARC record.

**Parameters:**
- `leader` (Leader): Record leader

**Returns:** Record instance

**Example:**
```python
from mrrc import Record, Leader
record = Record(leader)
```
```

**Recommendation:** Start with Option C (manual), move to Option B (mkdocs) once stable.

### 2.4 Publishing Docs
- Build as part of CI (not PyPI release)
- Publish to GitHub Pages (`docs/` branch)
- Include README quick-start examples

---

## 3. Benchmarking Framework Strategy

### 3.1 Overview
We want to quantify performance gains over `pymarc`. Need reproducible, isolated benchmarks.

### 3.2 Test Data Generation

```python
# tests/python/conftest.py
import pytest
from pathlib import Path
from mrrc import Record, Field, Leader

@pytest.fixture(scope="session")
def sample_records_100k():
    """Generate 100k sample MARC records for benchmarking."""
    records = []
    for i in range(100000):
        leader = Leader(
            record_type='a',
            bibliographic_level='m',
            ...
        )
        record = Record(leader)
        record.add_control_field("001", f"00000{i:05d}")
        record.add_field(
            Field("245", '1', '0')
                .add_subfield('a', f"Record {i} /")
                .add_subfield('c', "Author Name.")
        )
        records.append(record)
    return records

@pytest.fixture(scope="session")
def sample_mrc_file_100k(tmp_path_factory, sample_records_100k):
    """Write 100k records to a temporary .mrc file."""
    mrc_file = tmp_path_factory.mktemp("data") / "records_100k.mrc"
    writer = MARCWriter(open(mrc_file, "wb"))
    for record in sample_records_100k:
        writer.write_record(record)
    return mrc_file
```

### 3.3 Benchmark Suite

```python
# tests/python/test_benchmark_reader.py
import pytest
from mrrc import MARCReader

@pytest.mark.benchmark(group="reader")
def test_read_100k_records(benchmark, sample_mrc_file_100k):
    """Benchmark reading 100k MARC records."""
    def read_all():
        records = []
        reader = MARCReader(open(sample_mrc_file_100k, "rb"))
        while record := reader.read_record():
            records.append(record)
        return records
    
    result = benchmark(read_all)
    assert len(result) == 100_000

@pytest.mark.benchmark(group="reader")
def test_read_and_extract_titles(benchmark, sample_mrc_file_100k):
    """Benchmark reading and extracting field data."""
    def read_and_extract():
        titles = []
        reader = MARCReader(open(sample_mrc_file_100k, "rb"))
        while record := reader.read_record():
            if title := record.title():
                titles.append(title)
        return titles
    
    result = benchmark(read_and_extract)
    assert len(result) > 0

@pytest.mark.benchmark(group="writer")
def test_write_100k_records(benchmark, sample_records_100k, tmp_path):
    """Benchmark writing 100k MARC records."""
    def write_all():
        output = tmp_path / "output.mrc"
        writer = MARCWriter(open(output, "wb"))
        for record in sample_records_100k:
            writer.write_record(record)
    
    benchmark(write_all)
```

### 3.4 Running Benchmarks

```bash
# Run with pytest-benchmark
pytest tests/python/test_benchmark_*.py -v --benchmark-only

# Generate HTML report
pytest tests/python/ --benchmark-json=.benchmarks/results.json
pytest-benchmark compare .benchmarks/results.json

# Compare with pymarc (if available)
pip install pymarc
pytest tests/python/test_benchmark_comparison.py --benchmark-compare
```

### 3.5 Comparison Metrics

Store results for comparison:
- **Throughput** (records/sec)
- **Memory** (RSS peak MB)
- **Variance** (std dev %)
- **Baseline** (pymarc, if available)

---

## 4. CI/CD Workflow Strategy

### 4.1 Build Matrix

```yaml
# .github/workflows/python-build.yml
name: Python Build & Test

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build-wheels:
    name: Build wheels
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        python-version: ["3.9", "3.10", "3.11", "3.12"]
    steps:
      - uses: actions/checkout@v3
      - uses: PyO3/maturin-action@v1
        with:
          python-version: ${{ matrix.python-version }}
          manylinux: auto
          args: --release
      - uses: actions/upload-artifact@v3
        with:
          name: wheels-${{ matrix.os }}-${{ matrix.python-version }}
          path: dist

  test-wheels:
    name: Test wheels
    needs: build-wheels
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        python-version: ["3.9", "3.10", "3.11", "3.12"]
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-python@v4
        with:
          python-version: ${{ matrix.python-version }}
      - uses: actions/download-artifact@v3
        with:
          name: wheels-${{ matrix.os }}-${{ matrix.python-version }}
          path: dist
      - run: |
          pip install dist/*.whl
          pip install pytest pytest-benchmark mypy pyright
      - run: pytest tests/python/ -v
      - run: mypy tests/python/ --strict
```

### 4.2 Release Workflow

```yaml
# .github/workflows/python-release.yml
name: Python Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build-release:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        python-version: ["3.9", "3.10", "3.11", "3.12"]
    steps:
      - uses: actions/checkout@v3
      - uses: PyO3/maturin-action@v1
        with:
          python-version: ${{ matrix.python-version }}
          manylinux: auto
          args: --release
      - uses: actions/upload-artifact@v3
        with:
          name: wheels
          path: dist

  publish:
    needs: build-release
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/download-artifact@v3
        with:
          name: wheels
          path: dist
      - uses: pypa/gh-action-pypi-publish@release/v1
        with:
          password: ${{ secrets.PYPI_API_TOKEN }}
```

---

## 5. GIL Behavior Documentation

### 5.1 GIL Release Policy

**File I/O operations release the GIL:**
```rust
// src-python/src/reader.rs
#[pyo3(text_signature = "(self)")]
pub fn read_record(&mut self) -> PyResult<Option<PyRecord>> {
    // The native Rust code runs without the GIL
    // This allows Python threads to run concurrently
    let result = self.inner.read_record()
        .map_err(|e| e.into())?
        .map(|r| PyRecord { inner: r });
    Ok(result)
}
```

### 5.2 Threading Examples

```python
# docs/threading.md
## Using mrrc with Threading

mrrc's I/O operations release the Python GIL, allowing true parallelism:

### Example: Parallel Reading with multiprocessing
```python
from multiprocessing import Pool
from mrrc import MARCReader

def process_records(filename):
    records = []
    reader = MARCReader(open(filename, "rb"))
    while record := reader.read_record():
        records.append(record.title() or "Unknown")
    return records

if __name__ == "__main__":
    with Pool(4) as pool:
        results = pool.map(process_records, [
            "file1.mrc",
            "file2.mrc",
            "file3.mrc",
            "file4.mrc",
        ])
```

### 5.3 Performance Notes
- **GIL released:** File read/write, MARC parsing, field access
- **GIL held:** Python exception creation, type conversions
- **Implication:** I/O-bound workloads see near-linear speedup with threading

---

## 6. Error Handling Strategy (Decision Required)

**See mrrc-9ic.6 for the following decisions:**

### Option A: Auto-Conversion (Simplest)
```rust
// PyO3 auto-converts Result errors to Python exceptions
#[pymethods]
impl PyRecord {
    pub fn add_field(&mut self, field: PyField) -> PyResult<()> {
        self.inner.add_field(field.inner.clone())
            .map_err(|e| PyErr::new::<PyException>(e.to_string()))
    }
}
```

### Option B: Custom Exception (Recommended)
```rust
// Define custom Python exception hierarchy
#[pyclass(extends = PyException)]
pub struct MarcException;

#[pyclass(extends = MarcException)]
pub struct MarcEncodingError;

impl From<mrrc::error::MarcError> for PyErr {
    fn from(err: mrrc::error::MarcError) -> Self {
        match err {
            mrrc::error::MarcError::EncodingError(_) => 
                PyErr::new::<MarcEncodingError>(err.to_string()),
            _ => PyErr::new::<MarcException>(err.to_string()),
        }
    }
}
```

**Decision maker:** mrrc-9ic.6 task

---

## 7. Implementation Checklist (Before Phase 1)

- [ ] Decisions resolved in mrrc-9ic.6 (package name, Python version, error handling)
- [ ] Type hint strategy confirmed (mypy/pyright config written)
- [ ] Documentation approach chosen (Sphinx, mkdocs, or manual)
- [ ] Test data generation script written (conftest.py)
- [ ] Benchmark skeleton created (test_benchmark_*.py)
- [ ] CI/CD workflows drafted (.github/workflows/)
- [ ] GIL behavior documented for users
- [ ] pymarc API audit completed (mrrc-9ic.7)

---

## 8. References

- [PyO3 Rust/Python Integration](https://pyo3.rs/)
- [Maturin Build System](https://maturin.rs/)
- [PEP 561: Type Hints](https://www.python.org/dev/peps/pep-0561/)
- [Google Python Style Guide: Docstrings](https://google.github.io/styleguide/pyguide.html#38-comments-and-docstrings)
- [pytest-benchmark](https://pytest-benchmark.readthedocs.io/)
- [Python GIL & Threading](https://realpython.com/python-gil/)
