# Python 3.13/3.14 Support Plan

**Status**: Completed
**Created**: 2026-02-03
**Issue**: mrrc-83m

## Executive Summary

This document outlines the plan to update mrrc's Python version support:

- **Add**: Python 3.13 (stable, released October 2024)
- **Add**: Python 3.14 (stable, released October 2025)
- **Drop**: Python 3.9 (reached EOL October 2025)

The changes primarily involve CI/CD configuration updates. No code changes are expected—PyO3 0.27 already supports both versions.

## Current State

### Supported Versions

- Python 3.9, 3.10, 3.11, 3.12
- Platforms: Linux (x86_64, aarch64), macOS (x86_64, arm64), Windows (x86_64)

### Key Dependencies

| Dependency | Version | Python Support |
|------------|---------|----------------|
| PyO3 | 0.27.x | 3.8–3.14, including free-threaded 3.13t/3.14t |
| maturin | >=1.0,<2.0 | 3.8+ |

### Configuration Files

- `pyproject.toml`: `requires-python = ">=3.9"`, classifiers list 3.9–3.12
- `.github/workflows/python-build.yml`: matrix includes 3.9–3.12
- `.github/workflows/benchmark-python.yml`: matrix includes 3.9–3.12
- `src-python/Cargo.toml`: PyO3 0.27 with `extension-module` feature

## Python 3.9 Deprecation

### EOL Status

Python 3.9 reached End-of-Life on **October 31, 2025**. As of February 2026:

- No security patches from PSF
- Major platforms have dropped support:
  - AWS Lambda: December 15, 2025
  - Heroku: January 7, 2026
  - Palantir Foundry: February 1, 2026

### Recommendation: Drop Python 3.9

**Rationale**:

1. **Security**: No upstream security patches available
2. **Maintenance burden**: Testing 6 Python versions increases CI time
3. **User impact**: Minimal—most production environments have migrated
4. **Industry alignment**: NumPy, pandas, and other major libraries dropped 3.9 in 2025

**RHEL 9 Users**: RHEL 9 ships Python 3.9 as system default and will maintain it through RHEL 9's lifecycle. Users on RHEL 9 should install Python 3.10+ via `dnf module` or use containers.

## Python 3.13 Support

### Status

Python 3.13 was released **October 2024** and is now at 3.13.x. Fully stable.

### PyO3 Compatibility

PyO3 0.27.x fully supports Python 3.13:

- Standard builds work out of the box
- Free-threaded builds (3.13t) supported with `Sync` requirement on `#[pyclass]` types
- ABI3 stable ABI support via `abi3-py313` feature

### mrrc Compatibility Assessment

| Component | Status | Notes |
|-----------|--------|-------|
| Core parsing | Ready | No Python-specific code |
| PyO3 bindings | Ready | All types already `Send + Sync` |
| GIL release | Ready | Three-phase pattern unchanged |
| Format converters | Ready | No version-specific code |

### Free-Threaded Python (3.13t/3.14t)

Python 3.13 introduced experimental free-threaded mode (no GIL). mrrc is well-positioned:

- Rust types are inherently thread-safe
- GIL release pattern already in use for parallel I/O
- `#[pyclass]` types implement `Sync` (required for free-threaded builds)

**Recommendation**: Do not officially support free-threaded builds yet. The feature remains experimental and requires opt-in at build time. Revisit when Python 3.15 stabilizes the feature.

**Future potential**: When free-threading stabilizes, mrrc could simplify its Python wrapper by removing GIL management code (three-phase pattern, `BatchedMarcReader` queue). The current implementation will continue working on both GIL and free-threaded builds.

## Python 3.14 Support

### Status

Python 3.14 was released **October 7, 2025** ([PEP 745](https://peps.python.org/pep-0745/)) and is now at 3.14.2. Fully stable.

### PyO3 Compatibility

PyO3 0.27.x fully supports Python 3.14:

- Tested against release candidates and stable releases
- Free-threaded 3.14t support included
- Edge cases on 32-bit systems addressed

### Considerations

1. **C API changes**: Some deprecated functions removed, but PyO3 handles this
2. **Build warnings**: May see deprecation warnings during compilation (informational only)
3. **maturin support**: Fully supported in maturin 1.x

## Implementation Plan

### Phase 1: Update Version Support

1. Update `pyproject.toml`:
   ```toml
   requires-python = ">=3.10"
   classifiers = [
       "Programming Language :: Python :: 3.10",
       "Programming Language :: Python :: 3.11",
       "Programming Language :: Python :: 3.12",
       "Programming Language :: Python :: 3.13",
       "Programming Language :: Python :: 3.14",
   ]
   ```

2. Update mypy/pyright target version:
   ```toml
   [tool.mypy]
   python_version = "3.10"

   [tool.pyright]
   pythonVersion = "3.10"
   ```

3. Update CI workflows:
   - Remove "3.9" from all matrices
   - Add "3.13" and "3.14" to all matrices

4. Update documentation (README, installation guide)

### Phase 2: Local Verification

Test locally before pushing:

```bash
# For each version 3.10, 3.11, 3.12, 3.13, 3.14
uv python install 3.14
uv venv --python 3.14
uv run maturin develop --release
pytest tests/python/ -m "not benchmark" -v
```

### Phase 3: CI Verification

Push changes and verify:

- All wheel builds succeed on Linux, macOS, Windows
- All test matrices pass
- Benchmark results are comparable across versions

## Files to Modify

| File | Change |
|------|--------|
| `pyproject.toml` | Update `requires-python`, classifiers, mypy/pyright versions |
| `.github/workflows/python-build.yml` | Update matrix: `["3.10", "3.11", "3.12", "3.13", "3.14"]` |
| `.github/workflows/python-release.yml` | Update matrix: `["3.10", "3.11", "3.12", "3.13", "3.14"]` |
| `.github/workflows/benchmark-python.yml` | Update matrix: `["3.10", "3.11", "3.12", "3.13", "3.14"]` |
| `README.md` | Update supported Python versions |
| `docs/getting-started/installation.md` | Update minimum version to 3.10 |
| `CHANGELOG.md` | Document version changes in release notes |

## CI Impact

| Change | Build Jobs | Test Jobs | Benchmark Jobs |
|--------|------------|-----------|----------------|
| Remove 3.9 | -3 | -3 | -1 |
| Add 3.13 | +3 | +3 | +1 |
| Add 3.14 | +3 | +3 | +1 |
| **Net change** | +3 | +3 | +1 |

Total additional CI time: ~15–20 minutes per PR (parallelized).

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| 3.13/3.14 build failures | Low | Medium | PyO3 0.27 fully supports both |
| User complaints (3.9 drop) | Low | Low | 4 months post-EOL; document RHEL 9 workaround |
| Performance regression on new versions | Low | Low | Benchmark comparison in CI |
| manylinux compatibility | Low | Medium | Continue using manylinux2014 (glibc 2.17) |

## Edge Cases

### manylinux Compatibility

Current configuration uses `manylinux: off` in CI, building on the runner's native environment. For PyPI distribution, manylinux2014 (glibc 2.17) provides broad compatibility including:

- RHEL/CentOS 7+
- Ubuntu 18.04+
- Debian 10+

No changes needed for 3.13/3.14 support.

### musl/Alpine Linux

Not currently supported. PyO3 supports musl but would require separate wheel builds. Out of scope for this change.

### 32-bit Platforms

PyO3 0.27 fixed 3.14-specific issues on 32-bit systems. mrrc does not officially support 32-bit platforms (not in CI matrix), but should work.

### Python 3.10 EOL Planning

Python 3.10 reaches EOL **October 2026**. Consider dropping 3.10 support in a future release (v0.8 or v1.0) to maintain a 4-version support window.

## Testing Checklist

Before merging:

- [ ] Build wheels locally for 3.10, 3.11, 3.12, 3.13, 3.14
- [ ] Run full test suite on each version
- [ ] Run benchmarks on 3.12, 3.13, 3.14 (compare performance)
- [ ] Verify CI passes on all platforms
- [ ] Test wheel installation from built artifacts
- [ ] Update CHANGELOG with version changes

## Timeline

| Task | Target |
|------|--------|
| Implement changes | 2026-02 |
| Merge to main | 2026-02 |
| Release v0.7.0 with new version support | 2026-02 |

## References

- [PEP 745 – Python 3.14 Release Schedule](https://peps.python.org/pep-0745/)
- [PyO3 Releases](https://github.com/pyo3/pyo3/releases)
- [PyO3 Multiple Python Versions Guide](https://pyo3.rs/main/building-and-distribution/multiple-python-versions.html)
- [Python Version Status](https://devguide.python.org/versions/)
- [Python EOL Dates](https://endoflife.date/python)
- [Maturin Changelog](https://www.maturin.rs/changelog.html)
