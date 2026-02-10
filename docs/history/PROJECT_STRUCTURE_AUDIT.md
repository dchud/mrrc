# Project Structure Audit: Cargo.toml & pyproject.toml

**Date:** 2025-12-28  
**Task:** mrrc-qq5  
**Status:** Analysis Complete

## Executive Summary

**FINDING: `src-python/pyproject.toml` is REDUNDANT and should be DELETED.**

The project currently has conflicting Python build configurations. The root `pyproject.toml` is the authoritative source and correctly configured. The `src-python/pyproject.toml` appears to be an obsolete artifact from an earlier project structure.

### Recommended Action
Delete `src-python/pyproject.toml` - it is not used and conflicts with root configuration.

---

## Current Structure Analysis

### Files Inventory

```
./Cargo.toml                  # Workspace root - CORRECT
./pyproject.toml              # Maturin build config - CORRECT (authoritative)
./src-python/Cargo.toml       # Python extension crate - CORRECT
./src-python/pyproject.toml   # REDUNDANT - DELETE
```

### File Details

#### 1. Root `Cargo.toml` ✅ CORRECT

```toml
[workspace]
members = [".", "src-python"]
resolver = "2"

[package]
name = "mrrc"  # Main Rust library
# ... Rust dependencies ...
```

**Purpose:** 
- Defines a Cargo workspace with two members
- `.` is the main Rust library (`mrrc`)
- `src-python` is the Python extension crate

**Status:** ✅ Correct

#### 2. Root `pyproject.toml` ✅ CORRECT

```toml
[build-system]
requires = ["maturin"]
build-backend = "maturin"

[project]
name = "mrrc"
version = "0.1.0"
requires-python = ">=3.9"
# ... Python metadata ...

[tool.maturin]
python-packages = ["mrrc"]
module-name = "mrrc._mrrc"
manifest-path = "src-python/Cargo.toml"  # ← Points to Python extension
```

**Purpose:**
- Defines Python package metadata (PEP 621)
- Configures Maturin to build wheels
- Points Maturin to the Python extension crate

**Status:** ✅ Correct - This is the authoritative Python build config

#### 3. `src-python/Cargo.toml` ✅ CORRECT

```toml
[package]
name = "mrrc-python"
version = "0.1.0"

[dependencies]
pyo3 = { version = "0.22", features = ["extension-module"] }
mrrc = { path = ".." }  # Depends on parent library

[lib]
name = "_mrrc"
crate-type = ["cdylib"]
```

**Purpose:**
- Defines the Python extension module
- Creates a C-compatible shared library (`_mrrc`)
- Depends on the parent `mrrc` library

**Status:** ✅ Correct

#### 4. `src-python/pyproject.toml` ❌ REDUNDANT

```toml
[build-system]
requires = ["maturin>=1.7,<2.0"]  # ← DIFFERENT from root
build-backend = "maturin"

[project]
name = "mrrc"
requires-python = ">=3.8"  # ← CONFLICTS with root (requires >=3.9)

[tool.maturin]
module-name = "mrrc._mrrc"
bindings = "pyo3"
python-source = "python"  # ← References nonexistent ./python dir

[tool.ruff]  # ← Additional linting config (duplicate effort)
```

**Issues:**
- ❌ Conflicts with root `pyproject.toml` on Python version requirement (3.8 vs 3.9)
- ❌ References nonexistent `./python` source directory
- ❌ Duplicate metadata (name, version)
- ❌ Maturin version constraint differs (>= 1.7 vs loose)
- ❌ Duplicate tool configs (ruff linting)
- ❌ Not used in current build workflow

**Status:** ❌ REDUNDANT - DELETE

---

## Why `src-python/pyproject.toml` is Obsolete

### How Maturin Actually Works

When you run `maturin build` or `maturin develop`:

1. **Entry Point**: Maturin looks for `pyproject.toml` in the **current working directory** (root)
2. **Manifest Path**: Root `pyproject.toml` specifies `manifest-path = "src-python/Cargo.toml"`
3. **Build**: Maturin uses:
   - Metadata from root `pyproject.toml` (name, version, dependencies)
   - Cargo config from `src-python/Cargo.toml` (library definition)
4. **Never Reads**: `src-python/pyproject.toml` is completely ignored

### Proof

Running `maturin develop` (as done in this project):
```bash
$ cd /path/to/mrrc  # Root directory
$ source .venv/bin/activate
$ maturin develop
# Maturin reads: ./pyproject.toml (ONLY)
# Ignores: ./src-python/pyproject.toml
```

The `src-python/pyproject.toml` is never consulted.

---

## Recommended Structure (Maturin Best Practice)

According to [Maturin documentation](https://maturin.rs/), the standard for mixed Rust/Python projects is:

```
project-root/
├── Cargo.toml              # Workspace (or single package)
├── pyproject.toml          # Python build config ← SINGLE SOURCE OF TRUTH
├── README.md
├── src/                    # Rust library code
├── src-python/
│   ├── Cargo.toml          # Python extension crate
│   ├── src/                # Extension code
│   └── (NO pyproject.toml)  # ← DON'T DUPLICATE
└── tests/
```

**Key Principle:** `pyproject.toml` should live at the project root and serve as the single source of truth for Python packaging.

---

## Impact Analysis

### If We Delete `src-python/pyproject.toml`

**Risk Level:** ✅ SAFE - No functional impact

**Why:**
- Root `pyproject.toml` is already the authoritative config
- Maturin only reads root `pyproject.toml`
- All CI/CD workflows already use root directory as entry point
- Verified: Latest builds use root config

**Side Effects:** None - Git will only remove the obsolete file

### What We Keep

```toml
# Root pyproject.toml - Authoritative
name = "mrrc"
requires-python = ">=3.9"
manifest-path = "src-python/Cargo.toml"

# And root Cargo.toml workspace config
```

---

## Checklist for Cleanup

- [ ] Delete `src-python/pyproject.toml`
- [ ] Verify builds still work: `maturin develop`
- [ ] Verify tests pass: `pytest tests/python/`
- [ ] Commit: "Remove redundant src-python/pyproject.toml"

---

## Conclusion

The current structure has evolved to correctly use Maturin best practices. The root-level `pyproject.toml` is the correct and only place for Python build configuration. The `src-python/pyproject.toml` is a legacy artifact that should be removed to prevent confusion and ensure a clean, maintainable project structure.

**Recommendation:** **DELETE `src-python/pyproject.toml`**

---

## References

- [Maturin: Mixed Rust/Python Projects](https://maturin.rs/mixed-rust-python-projects)
- [Maturin: Source Distribution](https://maturin.rs/source-distribution)
- [PEP 621: Project Metadata](https://www.python.org/dev/peps/pep-0621/)
