# Architectural Decisions: Python Wrapper

**Date:** 2025-12-28  
**Task:** mrrc-9ic.6  
**Status:** AWAITING DECISION

This document captures the three critical architectural decisions needed before Phase 1 can begin.

---

## Decision 1: Package Name & PyPI Publication Strategy

### Question
What name should the Python package be published under on PyPI?

### Options

#### **Option A: `mrrc` (RECOMMENDED)**
**Package name on PyPI:** `mrrc`  
**Import statement:** `from mrrc import MARCReader, Record, Field`  
**Python module name:** `mrrc._mrrc` (native extension)  
**Directory structure:** Single `mrrc/` package

**Pros:**
- ✅ Single, unified brand across Rust and Python ecosystems
- ✅ Users type `pip install mrrc` for Python, `cargo add mrrc` for Rust
- ✅ Cleaner mental model: one library, two languages
- ✅ No naming confusion (vs `pymarc`, `pymrrc`, etc.)
- ✅ Easy to discover: searching "mrrc" finds both

**Cons:**
- ⚠️ PyPI will show it as a compiled extension (not pure Python)
- ⚠️ Users may initially expect pure Python and be surprised by wheels
- ⚠️ Rust crate and Python package share version number (couples releases)

**Mitigation:**
- Add prominent badge: "🦀 Rust-backed Python library"
- Include platform info in PyPI description
- Document in README that it's a compiled extension

#### **Option B: `pymrrc`**
**Package name on PyPI:** `pymrrc`  
**Import statement:** `from pymrrc import MARCReader`  
**Benefits:** Clear Python-specific branding, matches `pymarc` naming convention

**Pros:**
- ✅ Clear that it's a Python package (convention: `pyXXX`)
- ✅ Doesn't compete with or shadow the Rust `mrrc` on PyPI
- ✅ Familiar pattern for Python users

**Cons:**
- ❌ Two separate brands, confusing ecosystem
- ❌ Different import name than Rust users expect
- ❌ Extra cognitive load during migration from `pymarc`
- ❌ Harder to discover if searching "mrrc"

### Recommendation
**Choose Option A (`mrrc` on PyPI)**

**Rationale:** The unified brand is worth the minor documentation overhead. The Rust crate is already established on crates.io as `mrrc`. Publishing the Python package under the same name creates a single, discoverable ecosystem and aligns with the library's identity.

**Action:** Document clearly in README that it's a Rust-backed library available in both Rust and Python.

---

## Decision 2: Python Version Support & MSRV

### Question
What is the minimum and target Python version for the wrapper?

### Options

#### **Option A: Python 3.9+ (RECOMMENDED)**
**MSRV:** Python 3.9  
**Target:** Python 3.12  
**CI Matrix:** 3.9, 3.10, 3.11, 3.12  
**pyproject.toml:** `python_requires = ">=3.9"`

**Context:**
- Python 3.9 released Oct 2020; EOL Oct 2025
- Python 3.10 released Oct 2021; EOL Oct 2026
- Python 3.11 released Oct 2022; EOL Oct 2027
- Python 3.12 released Oct 2023; EOL Oct 2028

**Pros:**
- ✅ Broadest compatibility, reaches academic + enterprise users
- ✅ 3.9 still widely used in legacy systems
- ✅ PyO3 supports 3.7+; no technical barrier
- ✅ Helps users migrate gradually from `pymarc`

**Cons:**
- ⚠️ Must maintain wheel builds for 3.9 for next ~6 months
- ⚠️ Can't use type hints like `list[X]` (Python 3.10+) in type stubs
- ⚠️ Python 3.9 EOL approaching (Oct 2025, ~9 months away)
- ⚠️ CI matrix is larger (4 versions × 3 platforms = 12 jobs)

#### **Option B: Python 3.10+**
**MSRV:** Python 3.10  
**Target:** Python 3.12  
**CI Matrix:** 3.10, 3.11, 3.12  
**pyproject.toml:** `python_requires = ">=3.10"`

**Context:**
- Modern default for most development
- Type hints fully supported (PEP 604: `int | str`)
- Good balance of compatibility and modernity

**Pros:**
- ✅ Modern, reasonable MSRV
- ✅ Can use PEP 604 union syntax in type hints
- ✅ Still covers 95% of active Python users
- ✅ Smaller CI matrix (3 versions × 3 platforms = 9 jobs)
- ✅ EOL Oct 2026 (good runway)

**Cons:**
- ⚠️ Excludes some enterprise systems still on 3.9
- ⚠️ Slightly narrower reach than 3.9

#### **Option C: Python 3.12+ only**
**MSRV:** Python 3.12  
**Target:** Python 3.12+  
**CI Matrix:** 3.12, (future 3.13+)  
**pyproject.toml:** `python_requires = ">=3.12"`

**Context:**
- Latest stable, newest features

**Pros:**
- ✅ Smallest CI matrix
- ✅ Latest language features
- ✅ Best performance

**Cons:**
- ❌ Excludes most current users; too restrictive
- ❌ Breaks forward compatibility for enterprise users
- ❌ Too limiting for a library targeting migration from `pymarc`

### Recommendation
**Choose Option A (Python 3.9+)**

**Rationale:** 
- Broadest compatibility helps adoption from `pymarc` users
- 3.9 EOL is ~9 months away; can drop it in v0.3.0 (Q3 2026)
- Type hints using `from __future__ import annotations` work in 3.9
- Enterprise/academic users need this support

**Action:** 
- Set `python_requires = ">=3.9"` in `pyproject.toml`
- Test against 3.9, 3.10, 3.11, 3.12 in CI
- Plan deprecation of 3.9 for v0.3.0 (after its EOL)

---

## Decision 3: Error Handling Strategy

### Question
How should Rust `MarcError` types be converted to Python exceptions?

### Options

#### **Option A: Auto-Conversion to Generic Exception (Simplest)**
**Approach:** Let PyO3 auto-convert all `Result` errors to `PyException`

```rust
#[pymethods]
impl PyRecord {
    pub fn add_field(&mut self, field: PyField) -> PyResult<()> {
        self.inner.add_field(field.inner.clone())
            .map_err(|e| PyErr::new::<PyException>(e.to_string()))
    }
}
```

**Pros:**
- ✅ Simplest implementation (1-2 lines per method)
- ✅ No custom exception class needed
- ✅ Fast to implement, no ongoing maintenance
- ✅ Works for Phase 2 prototype

**Cons:**
- ❌ All errors are the same type; hard to catch specific errors
- ❌ Not `pymarc`-compatible (breaks error handling code)
- ❌ Poor ergonomics for users: `except Exception` catches everything
- ❌ Can't distinguish encoding errors from validation errors

**Example (bad):**
```python
try:
    record = read_record(data)
except Exception as e:  # Too broad
    print(f"Some error: {e}")
```

#### **Option B: Custom Exception Hierarchy (RECOMMENDED)**
**Approach:** Create custom exception classes matching common error categories

```rust
// src-python/src/errors.rs
#[pyclass(extends = PyException)]
pub struct MarcException;

#[pyclass(extends = MarcException)]
pub struct MarcEncodingError;

#[pyclass(extends = MarcException)]
pub struct MarcValidationError;

impl From<mrrc::error::MarcError> for PyErr {
    fn from(err: mrrc::error::MarcError) -> Self {
        match err {
            mrrc::error::MarcError::EncodingError(msg) => {
                PyErr::new::<MarcEncodingError>(msg)
            }
            mrrc::error::MarcError::InvalidField(_) => {
                PyErr::new::<MarcValidationError>(err.to_string())
            }
            _ => PyErr::new::<MarcException>(err.to_string()),
        }
    }
}
```

**Pros:**
- ✅ Better ergonomics; allows `except MarcEncodingError`
- ✅ Users can handle different error types appropriately
- ✅ Matches patterns users know from other libraries
- ✅ More maintainable (errors map explicitly)
- ✅ Friendly to error recovery strategies
- ✅ Enables `pymarc`-compatible error handling *eventually*

**Cons:**
- ⚠️ More code upfront (~50 lines of Rust)
- ⚠️ Requires maintaining mapping as `mrrc` errors evolve
- ⚠️ Python exceptions must be properly defined

**Example (good):**
```python
try:
    record = read_record(data)
except mrrc.MarcEncodingError as e:
    print(f"Encoding issue: {e}")
    # Try fallback encoding
except mrrc.MarcValidationError as e:
    print(f"Invalid MARC: {e}")
    # Skip record
except mrrc.MarcException as e:
    print(f"Other error: {e}")
```

#### **Option C: Match `pymarc` Exception Hierarchy**
**Approach:** Create exceptions that exactly match `pymarc`'s error types

**Problem:** `pymarc` doesn't have a formal exception hierarchy; it mostly uses generic exceptions. No benefit over Option B.

**Not Recommended** — Go with Option B instead.

### Recommendation
**Choose Option B (Custom Exception Hierarchy)**

**Rationale:**
- Better error handling experience for users
- Not much harder than Option A (~50 lines of code)
- Enables future `pymarc` compatibility without rework
- Industry best practice for library design
- Sets foundation for user feedback on error cases

**Exceptions to Create:**
1. `MarcException` — Base exception (extends `PyException`)
2. `MarcEncodingError` — Character encoding issues
3. `MarcValidationError` — Field/record validation failures
4. `MarcIOError` — File I/O problems

**Action:** Create `src-python/src/errors.rs` with all 4 exception types before Phase 2 starts.

---

## Summary Table

| Decision | Option | Choice | Blocking |
|----------|--------|--------|----------|
| **Package Name** | A (mrrc) vs B (pymrrc) | **A: `mrrc`** | Phase 1 |
| **Python Versions** | A (3.9+) vs B (3.10+) vs C (3.12+) | **A: 3.9+** | Phase 1 |
| **Error Handling** | A (generic) vs B (custom hierarchy) vs C (pymarc match) | **B: custom hierarchy** | Phase 2 |

---

## Implementation Checklist

- [ ] **Decision 1 Confirmed:** Package published as `mrrc` on PyPI
  - [ ] Update `pyproject.toml` with package name
  - [ ] Document in README: "🦀 Rust-backed Python library"
  
- [ ] **Decision 2 Confirmed:** Python 3.9+ support
  - [ ] Set `python_requires = ">=3.9"` in `pyproject.toml`
  - [ ] Configure CI matrix for 3.9, 3.10, 3.11, 3.12
  - [ ] Document Python version support in README
  
- [ ] **Decision 3 Confirmed:** Custom exception hierarchy
  - [ ] Create `src-python/src/errors.rs`
  - [ ] Define 4 exception classes with proper extends
  - [ ] Implement `From<mrrc::error::MarcError>` mapping
  - [ ] Add tests for exception propagation

---

## Next Steps

1. **Review & Approve** these decisions (team/stakeholder sign-off)
2. **Document Decisions** in this file (mark DECIDED)
3. **Proceed to Phase 1** once all confirmed

---

## References

- PYTHON_WRAPPER_REVIEW.md (Gap #2, #4, #3)
- PYTHON_WRAPPER_STRATEGIES.md (Section 6: Error Handling)
- Python EOL Schedule: https://devguide.python.org/versions/
- PyO3 Documentation: https://pyo3.rs/
