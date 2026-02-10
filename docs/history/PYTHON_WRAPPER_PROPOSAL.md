# Proposal: Python Wrapper API for MRRC

**Status:** Draft
**Date:** 2025-12-26
**Target:** Building a `pymarc`-compatible Python package backed by `mrrc`.

## 1. Executive Summary

This proposal outlines the strategy to create a high-performance Python extension module for `mrrc` that offers near 100% API compatibility with the existing `pymarc` library. By leveraging Rust's performance for parsing and data manipulation, we expect significant speedups for IO-bound and compute-heavy MARC processing tasks in Python.

## 2. Technology Stack

We will use the standard "modern" stack for Rust-Python integration:

*   **[PyO3](https://github.com/PyO3/pyo3)**: The Rust crate for writing native Python modules. It handles the FFI (Foreign Function Interface), reference counting, and type conversion.
*   **[Maturin](https://github.com/PyO3/maturin)**: The build system and packaging tool. It acts as a replacement for `setuptools`/`Poetry` for Rust-based extensions, capable of building `manylinux` wheels for distribution.

## 3. Architecture

### 3.1 Workspace Structure
We recommend restructuring the repository into a Cargo workspace to keep the core library clean and separate from the Python bindings:

```text
mrrc/                 # Root
├── Cargo.toml        # Workspace definition
├── pyproject.toml    # Python package metadata (Maturin)
├── src/              # (Existing) Core Rust library
├── src-python/       # [NEW] Python extension crate (cdylib)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs    # PyO3 module definition
│   │   ├── record.rs # Python class wrappers
│   │   └── ...
└── tests/            # Integration tests
```

### 3.2 Wrapping Strategy
We will implement a "stateful wrapper" pattern.

1.  **Core Types**: Create Python classes (`struct` annotated with `#[pyclass]`) in `src-python` that wrap the underlying `mrrc` types.
    ```rust
    // src-python/src/record.rs
    #[pyclass(name = "Record")]
    pub struct PyRecord {
        inner: mrrc::Record,
    }
    ```
2.  **API Compatibility Layer**:
    *   Implement methods on `PyRecord` that match `pymarc.Record`'s signature exactly (e.g., `title()`, `author()`, `add_field()`).
    *   Use `#[pymethods]` to expose them to Python.
    *   Handle Python usage patterns (e.g., `__iter__`, `__getitem__`, `__str__`).

### 3.3 Dependencies
**Rust Dependencies (in `src-python/Cargo.toml`):**
*   `pyo3` (features = ["extension-module"])
*   `mrrc` (path = "../")

**Build System:**
*   `maturin` (installed via pip or cargo)

## 4. Build & Distribution

### 4.1 Local Development
Developers can build and install the package effectively in their virtual environment:
```bash
maturin develop --release
```

### 4.2 CI/CD Workflows
We will add a new GitHub Workflow `python-release.yml` using `PyO3/maturin-action`.

**Workflow Steps:**
1.  **Build Wheels**: usage of `maturin-action` to build wheels for:
    *   Linux (manylinux_2_28) - x86_64, aarch64
    *   macOS - x86_64, arm64 (Apple Silicon)
    *   Windows - x86_64
2.  **Test**: Install generated wheels and run `pytest`.
3.  **Publish**: Upload to PyPI (on tag push).

## 5. Testing & Benchmarking

### 5.1 Correctness Testing (`tox` / `pytest`)
To guarantee compatibility:
1.  **Unit Tests**: Replicate `pymarc`'s unit test suite in `tests/python/`.
2.  **Compliance Suite**: Create a test harness that runs the *actual* `pymarc` test suite against our `mrrc` wrapper. Be prepared for minor deviations (e.g., stricter validation in Rust).

### 5.2 Benchmarking Strategy
We will use **[pytest-benchmark](https://github.com/ionelmc/pytest-benchmark)** and **[codspeed](https://codspeed.io/)** for continuous performance tracking.

**Scenarios to Benchmark:**
1.  **Parsing**: Read 100k MARC records (ISO 2709).
    *   Compare: `pymarc.MARCReader` vs `mrrc.MARCReader`.
2.  **Serialization**: Write 100k records to disk.
3.  **Field Access**: Iterate over all fields and subfields (heavy object creation overhead check).
4.  **Memory Usage**: Measure peak RSS during processing of large files.

**Comparison Matrix:**
*   `pymarc` (Pure Python baseline)
*   `mrrc` (Rust, raw speed limit - measured via Cargo bench)
*   `pymrrc` (The wrapper - overhead measurement)

## 6. Implementation Stages

1.  **Phase 1: Skeleton**: Set up Workspace, Maturin, and a "Hello World" `Record` class.
2.  **Phase 2: Core Data Model**: Implement `Record`, `Field`, `Leader` wrappers.
3.  **Phase 3: Readers/Writers**: Implement `MARCReader` and `MARCWriter` using `mrrc`'s efficient parsing.
4.  **Phase 4: Polish**: Implement "pythonic" magic methods (`__iter__`, `__len__`, etc.) and ensure `pymarc` compatibility.

## 7. Migration Guide
Once built, users should only need to change their import:
```python
# from pymarc import MARCReader
from pymrrc import MARCReader
```
(Or we can publish as `mrrc` and provide a top-level API that matches).
