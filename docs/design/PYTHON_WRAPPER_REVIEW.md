# Technical Review: Python Wrapper Proposal for MRRC

**Date:** 2025-12-28  
**Reviewer:** AI Agent  
**Status:** DRAFT - Ready for team discussion

---

## Executive Summary

The proposal is solid and well-structured. The use of PyO3/Maturin is the correct modern approach, and the phased implementation plan is sensible. However, several critical details are underdeveloped or missing, particularly around error handling, Python version support, type hints, and the publishing strategy.

**Recommendation:** Proceed with the proposal, but resolve the architectural decisions and gaps outlined below before Phase 1.

---

## 1. STRENGTHS

### 1.1 Technology Stack
- ✅ **PyO3/Maturin** is the industry standard for Rust↔Python
- ✅ Avoids C/C++ FFI complexity
- ✅ Handles memory management and GIL automatically
- ✅ Rich ecosystem (type hints, async support, Python exception integration)

### 1.2 Architecture
- ✅ Workspace structure keeps core library clean
- ✅ Stateful wrapper pattern is proven and maintainable
- ✅ Modular design allows independent testing of each wrapper layer

### 1.3 Build & Distribution
- ✅ Maturin-action handles multi-platform wheel building
- ✅ manylinux_2_28 is modern and widely compatible
- ✅ CI/CD workflow is standard

### 1.4 Testing Strategy
- ✅ Benchmarking approach is comprehensive (pytest-benchmark + codspeed)
- ✅ Three-way comparison (pymarc vs mrrc vs pymrrc) provides good performance context

---

## 2. CRITICAL GAPS & RECOMMENDATIONS

### 2.1 Error Handling Mapping

**Gap:** The proposal does not address how Rust `Result` types and `MarcError` exceptions map to Python.

**Problem:**
- `mrrc` uses `Result<T>` throughout; Python uses exceptions
- `pymarc` has specific exception types; we need compatibility
- PyO3 can auto-convert, but custom mapping is cleaner

**Recommendation:**
```rust
// In src-python/src/errors.rs
#[pyclass(extends = Exception)]
pub struct MarcException;

impl From<mrrc::error::MarcError> for PyErr {
    fn from(err: mrrc::error::MarcError) -> Self {
        PyErr::new::<MarcException>(err.to_string())
    }
}
```

**Action Items:**
- Define Python exception hierarchy matching `pymarc` (if applicable)
- Create error conversion layer
- Add tests for exception propagation

---

### 2.2 Python Version Support & MSRV

**Gap:** No stated minimum Python version or maximum supported Python versions.

**Problem:**
- PyO3 supports Python 3.7+, but newer versions are better
- maturin auto-detects, but we should document
- Different Python versions have different ABI incompatibilities

**Recommendation:**
- **Target:** Python 3.9+ (end-of-life: Oct 2025, but still widely used)
  - Python 3.10+: preferred (modern type hints, better performance)
  - Python 3.12+: testing (latest stability)
- Add `python_requires = ">=3.9"` to `pyproject.toml`
- Test in CI against multiple versions (3.9, 3.10, 3.11, 3.12)

---

### 2.3 GIL & Threading Considerations

**Gap:** Proposal doesn't mention GIL implications for I/O-heavy workloads.

**Problem:**
- `MARCReader` reads from files in Rust; during native code execution, GIL is released
- But returning Python objects re-acquires GIL
- No mention of async support or thread safety

**Recommendation:**
- Use `#[pyo3(text_signature = "(...)")]` for documentation
- Explicitly document that `MARCReader.read_record()` **releases the GIL** during file I/O
  - This allows true parallelism (e.g., with `multiprocessing` or `concurrent.futures`)
- Consider `#[pyo3(signature = (...))]` for complex signatures
- **Not necessary for Phase 1**, but worth documenting now

---

### 2.4 Publishing Strategy (Critical Decision)

**Gap:** Proposal is unclear: publish as `mrrc` or `pymrrc`?

**Current State:**
- Rust crate is already `mrrc` on crates.io
- Python package name on PyPI must be decided

**Options:**

| Option | Pros | Cons |
|--------|------|------|
| **`mrrc`** on PyPI | Single brand, users import `mrrc` for both | Risk: users expect Python library, get Rust library docs |
| **`pymrrc`** on PyPI | Clear Python package, matches `pymarc` naming | Extra brand, different import statement |
| **Dual publication** | Both available, gradual migration path | Maintenance burden, confusion |

**Recommendation:**
- **Primary:** Publish as `mrrc` on PyPI
  - Python users: `pip install mrrc`, `from mrrc import MARCReader`
  - Rust users: `cargo add mrrc`
- **Rationale:** Single brand, cleaner ecosystem, matches what users want
- **Caveat:** Ensure `pyproject.toml` and README make it clear this is the Python package

---

### 2.5 Type Hints & IDE Support

**Gap:** No mention of type stubs or Python typing support.

**Problem:**
- IDE autocomplete won't work without type information
- PyO3 generates `pyi` files, but must be configured
- Requires `py.typed` marker file

**Recommendation:**
```toml
# pyproject.toml
[tool.maturin]
python-packages = ["mrrc"]
include = ["mrrc/py.typed"]  # PEP 561 marker

[tool.mypy]
namespace_packages = true
```

**Action Items:**
- Enable `maturin` to generate `.pyi` stub files
- Add `py.typed` marker file
- Test with mypy/pyright

---

### 2.6 Documentation Generation

**Gap:** No mention of how Python API documentation is built/maintained.

**Problem:**
- Rust doc comments don't automatically translate to Python docs
- Sphinx + autodoc won't work with compiled modules
- Need separate Python docstrings

**Recommendation:**
```rust
/// Read a MARC record from the input stream.
///
/// Returns None if no more records are available.
///
/// # Example
/// ```python
/// reader = MARCReader(open("records.mrc", "rb"))
/// while record := reader.read_record():
///     print(record.title())
/// ```
#[pyo3(text_signature = "(self)")]
fn read_record(&mut self) -> PyResult<Option<PyRecord>> {
    // ...
}
```

**Action Items:**
- Document Python API in docstrings using Python examples
- Generate HTML docs with `pydoc` or Sphinx (manual build)
- Publish to docs.rs or a separate docs site

---

### 2.7 Performance Benchmarking Setup

**Gap:** Benchmarking plan is good, but setup details are missing.

**Problem:**
- Requires `pymarc` as a test dependency
- Needs large test data files (100k records)
- Needs GitHub Actions secrets for codspeed (optional)

**Recommendation:**
```bash
# tests/python/conftest.py
@pytest.fixture(scope="session")
def sample_records():
    # Generate or load 100k sample records
    # Use tests/data/multi_records.mrc as base, replicate 1000x
    pass

# tests/python/test_bench_reader.py
@pytest.mark.benchmark
def test_read_100k_records(benchmark):
    result = benchmark(MARCReader(...).read_all)
    assert len(result) == 100000
```

**Action Items:**
- Set up `tests/python/` directory
- Create benchmark fixtures
- Add pytest-benchmark configuration to `pyproject.toml`
- Conditionally import `pymarc` (may not be installed)

---

### 2.8 CI/CD Configuration

**Gap:** Workflow details are high-level; specifics missing.

**Problem:**
- Python tests need to run *after* wheel building
- Different Python versions need different wheels
- Need to handle build failures for some platforms

**Recommendation:**
```yaml
# .github/workflows/python-release.yml
name: Python Build & Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build-wheels:
    strategy:
      matrix:
        python-version: ["3.9", "3.10", "3.11", "3.12"]
        platform: [ubuntu-latest, macos-latest, windows-latest]
    uses: PyO3/maturin-action@v1
    with:
      python-version: ${{ matrix.python-version }}
      manylinux: auto
      
  test-wheels:
    needs: build-wheels
    strategy:
      matrix:
        python-version: ["3.9", "3.10", "3.11", "3.12"]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/setup-python@v4
        with:
          python-version: ${{ matrix.python-version }}
      - run: pip install dist/*.whl pytest pytest-benchmark
      - run: pytest tests/python/
      
  publish:
    needs: test-wheels
    runs-on: ubuntu-latest
    if: success()
    steps:
      - uses: pypa/gh-action-pypi-publish@release/v1
```

**Action Items:**
- Create `python-release.yml` workflow
- Add Python test matrix
- Document PyPI token setup in CONTRIBUTING.md

---

### 2.9 Backwards Compatibility & Migration Strategy

**Gap:** Unclear how to handle differences between `pymarc` and `mrrc` APIs.

**Problem:**
- `pymarc` may have quirks or undocumented behaviors
- `mrrc` is stricter (e.g., validation)
- Users may rely on internal APIs

**Recommendation:**
- Create a compatibility matrix: `pymarc` method → `mrrc` method
- Document known differences in a migration guide
- Provide adapters/shims for deprecated patterns
- Example compatibility file:
  ```rust
  // src-python/src/compat.rs
  /// Deprecated: Use field.add_subfield_str() instead
  #[deprecated(since = "0.1.0", note = "use Field::add_subfield_str")]
  pub fn add_subfield(&mut self, code: char, value: String) {
      self.inner.add_subfield(code, value);
  }
  ```

**Action Items:**
- Document API differences vs `pymarc`
- Create migration guide in docs/
- Provide deprecation shims where needed

---

### 2.10 Format Conversion Support

**Gap:** No mention of JSON/XML/MARCJSON serialization in Python.

**Problem:**
- `mrrc` has JSON, XML, MARCJSON modules
- Python users may need these
- Not mentioned in the proposal

**Recommendation:**
- Phase 4 extension: Add format conversion methods
```python
record = MARCReader(...).read_record()
json_str = record.to_json()
xml_str = record.to_xml()
marcjson = record.to_marcjson()
```

**Action Items:**
- Add to Phase 4 scope (or separate Phase 5)
- Implement once Phase 2 (core model) is done

---

## 3. ARCHITECTURAL DECISIONS NEEDED

### Decision 1: Package Name & Import Path
- **Option A:** `mrrc` on PyPI, import `from mrrc import MARCReader` ✅ Recommended
- **Option B:** `pymrrc` on PyPI, import `from pymrrc import MARCReader`
- **Decide before:** Phase 1 (workspace setup)

### Decision 2: Python Version Support
- **Option A:** Python 3.9+ (broader compatibility) ✅ Recommended
- **Option B:** Python 3.10+ (modern, GH Actions default)
- **Option C:** Python 3.12+ only (latest)
- **Decide before:** Phase 1

### Decision 3: Error Handling Strategy
- **Option A:** Auto-convert `MarcError` to generic Python `Exception`
- **Option B:** Create custom `MarcException` hierarchy matching `pymarc`
- **Option C:** Create custom exceptions without matching `pymarc`
- **Decide before:** Phase 2

---

## 4. TIMELINE & RISK ASSESSMENT

### Risk Matrix

| Risk | Severity | Mitigation |
|------|----------|-----------|
| PyO3 build failures on some architectures | Medium | Test on all 3 platforms early (Phase 1) |
| GIL contention with heavy I/O | Low | Document & benchmark; file I/O releases GIL |
| Type hint generation issues | Low | Use pyo3-stub-gen; test with mypy |
| `pymarc` API is undocumented | Medium | Reverse-engineer from source & tests |
| Python 3.9 EOL (Oct 2025) | Low | Plan for upgrade to 3.10+ in 2026 |

### Estimated Effort
- **Phase 1:** 1-2 days (workspace, boilerplate)
- **Phase 2:** 3-5 days (core model, error handling)
- **Phase 3:** 3-5 days (readers/writers, I/O)
- **Phase 4:** 2-3 days (polish, tests, documentation)
- **Total:** 1.5-2 weeks of focused work

---

## 5. CHECKLIST BEFORE PROCEEDING

- [ ] Resolve Decision 1 (package name)
- [ ] Resolve Decision 2 (Python version)
- [ ] Resolve Decision 3 (error handling)
- [ ] Create error mapping module (src-python/src/errors.rs)
- [ ] Document type hint strategy (.pyi generation, py.typed)
- [ ] Create benchmarking fixtures & data
- [ ] Draft CI/CD workflow (python-release.yml)
- [ ] Draft Python docstring conventions
- [ ] Create migration guide template
- [ ] Review pymarc source for API surface

---

## 6. NEXT STEPS

1. **Discuss & Decide** (team meeting):
   - Package name (mrrc vs pymrrc)
   - Python version support (3.9+, 3.10+, 3.12+?)
   - Error handling strategy

2. **Create Follow-up Tasks**:
   - mrrc-9ic.2: Set up workspace & Cargo/pyproject.toml
   - mrrc-9ic.3: Implement error handling layer
   - mrrc-9ic.4: Build Phase 1 skeleton (Hello World Record)
   - mrrc-9ic.5: Comprehensive pymarc API audit
   - mrrc-9ic.6: CI/CD workflow setup

3. **Phase 1 Kickoff**:
   - Once decisions are made, proceed to skeleton setup
   - Early focus on cross-platform builds (Linux, macOS, Windows)

---

## Appendix A: Reference Links

- [PyO3 Best Practices](https://pyo3.rs/main/)
- [Maturin Documentation](https://maturin.rs/)
- [Python Packaging Standards (PEP 517, 518, 561)](https://packaging.python.org/)
- [pymarc on GitLab](https://gitlab.com/pymarc/pymarc)
- [Python Benchmark Tooling](https://pytest-benchmark.readthedocs.io/)
