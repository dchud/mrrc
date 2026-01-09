# Memory Safety CI Infrastructure

Technical reference for the ASAN (Address Sanitizer) memory safety checking system in MRRC.

## Overview

MRRC integrates ASAN to detect memory safety issues in both Rust code and PyO3 Python bindings:

- **Local validation**: `./cargo/check.sh --memory-checks` (developer opt-in)
- **Nightly CI**: Automatic ASAN runs (non-blocking, regression detection)
- **Suppressions**: Community-recommended patterns in `.cargo/asan_suppressions.txt`

## Design Principles

1. **Non-blocking**: Memory safety checks don't block regular CI or merges
   - Regular pushes complete in <5 minutes (ASAN nightly-only)
   - Developers use ASAN voluntarily before complex changes
   - Nightly CI provides regression detection without friction

2. **Fail-fast on real issues**: True memory bugs surface immediately
   - Local checks catch issues before push
   - ASAN exits with non-zero on errors (CI would report failure)
   - False positives suppressed explicitly (with rationale)

3. **Documented suppressions**: Each suppression has technical rationale
   - Explains what is suppressed and why
   - References upstream issues or community knowledge
   - Prevents suppression drift over time

4. **Rust best practices**: Uses nightly compiler features properly
   - RUSTFLAGS="-Z sanitizer=address" (official Rust integration)
   - LSAN_OPTIONS for leak detection tuning
   - Suppression file managed in git (version-controlled)

## Local Validation: .cargo/check.sh --memory-checks

### How It Works

```bash
./cargo/check.sh --memory-checks
```

This:
1. Parses `--memory-checks` flag
2. Sets `RUSTFLAGS="-Z sanitizer=address"` (requires nightly)
3. Sets `LSAN_OPTIONS="suppressions=./.cargo/asan_suppressions.txt"`
4. Runs: `cargo test --lib --package mrrc --all-targets -q`
5. ASAN instruments the binary at compile time
6. Tests run under ASAN supervision
7. Any memory issues reported with full backtrace

### Runtime Requirements

- **Rust nightly**: `rustup install nightly` (or `rustup update nightly`)
- **Target support**: Nightly must support `-Z sanitizer=address`
  - Available on: Linux (x86_64, aarch64), macOS (Apple Silicon partially)
  - Note: macOS support is limited; primarily for Linux

### Performance Impact

- **Compilation**: +10-20% (ASAN instrumentation)
- **Runtime**: +100-200% (memory tracking overhead)
- **Typical test suite**: ~30s ASAN run vs ~3s normal run

### Sample Output

**Successful run:**
```
=== ASAN memory safety checks ===
running 9 tests
.........
test result: ok. 9 passed; 0 failed
```

**Failed run** (example: use-after-free):
```
=== ASAN memory safety checks ===
running 9 tests
.F.......

thread 'test_name' panicked at 'ASAN detected memory error'
test result: FAILED
```

## Nightly CI: .github/workflows/memory-safety.yml

### Job Configuration

- **Trigger**: Scheduled nightly (2 AM UTC) + manual workflow_dispatch
- **Runner**: ubuntu-latest (ASAN fully supported on Linux)
- **Toolchain**: Rust nightly with rust-src component
- **Caching**: Cargo registry, index, and build cache

### Why Non-Blocking

1. **Nightly only**: Developers don't wait for ASAN on every push
2. **Regression detection**: Catches memory issues that sneak into main
3. **Report-only**: GitHub status shows results without blocking merge
4. **Scheduled**: Runs off-peak (2 AM UTC), low CI queue impact

### How to View Results

1. GitHub → Actions → "Memory Safety Checks (ASAN)"
2. Click recent workflow run
3. Expand "Run ASAN on library tests" step
4. Scroll through output for any ASAN reports

## Suppression File: .cargo/asan_suppressions.txt

### Format

Standard LSAN (Leak Sanitizer) suppression format:

```
# Description
# Explanation of why this is safe
# References (if applicable)

leak:function_name
leak:pattern_in_stacktrace
...
```

### Categories

1. **PyO3 patterns**: Thread-local storage, module initialization
2. **Python runtime**: Interned strings, interpreter globals
3. **libc initialization**: Locale, charset data
4. **Dependency patterns**: Third-party library static state

### Adding Suppressions

Only suppress patterns that are:
- **Confirmed safe**: Not a bug in mrrc code
- **External**: In Python, PyO3, libc, or dependencies
- **Known pattern**: Documented in community resources
- **Not growing**: Doesn't increase with test iterations

Do NOT suppress:
- "definitely lost" leaks in mrrc code
- Unknown or unexplained patterns
- Anything without clear technical rationale

## Troubleshooting

### ASAN Runtime Errors

**Error: "AddressSanitizer is not compatible with this libc"**
- On Linux: Usually a glibc/musl mismatch or old kernel
- Solution: Use standard glibc (Ubuntu 20.04+, Debian 11+)

**Error: "Unsupported Sanitizer"**
- Rust nightly missing or misconfigured
- Solution: `rustup install nightly --force` and retry

**Error: "SEGV on unknown address" (timeout)**
- Too much ASAN overhead or kernel limit
- Solution: Use smaller test subset, increase timeout, run locally first

### False Positives

**Symptom**: ASAN reports leak, but code looks correct

**Diagnosis steps**:
1. Check if leak grows with iterations (real) vs stays same size (false positive)
2. Check if location is in mrrc code or external
3. Look up suppression in community resources
4. Run with `--verbose` to see more details

**Examples**:
- Leak in `PyInit_*`: Python module init (already suppressed)
- Leak in `pthread_once`: Thread-local init (expected pattern)
- Leak in `malloc` with no clear path: Might be libc internal

### Inconsistent Results

**Symptom**: ASAN sometimes reports issue, sometimes doesn't

**Common causes**:
- Threading issues (non-deterministic race conditions)
- Timing-dependent allocations
- Randomized ASLR (address space layout randomization)

**Mitigation**:
- Set `ASAN_OPTIONS=halt_on_error=1:abort_on_error=1` (stricter)
- Run multiple times to catch intermittent issues
- Use thread sanitizer if data races suspected

## Integration with CI/CD

### Pre-Push (Developer Workflow)

```bash
# Regular checks (always)
.cargo/check.sh

# Memory checks (before complex FFI changes)
.cargo/check.sh --memory-checks

# Then push
git push
```

### Pull Request

- Regular CI runs (rust tests, lint, doc checks)
- ASAN runs separately on nightly
- Developers can trigger manual ASAN: GitHub Actions → "Run workflow"

### Release Process

Before release:
```bash
# Run all checks including memory safety
.cargo/check.sh
.cargo/check.sh --memory-checks
cargo test --all
```

## Performance Optimization

### If ASAN Tests Are Too Slow

**Option 1: Test subset**
```bash
export RUSTFLAGS="-Z sanitizer=address"
cargo test --lib --lib  # Library only, skip integration tests
```

**Option 2: Single-threaded (more deterministic)**
```bash
RUSTFLAGS="-Z sanitizer=address" cargo test --lib -- --test-threads=1
```

**Option 3: Skip in CI, run locally**
- Don't enable ASAN in blocking CI
- Developers run locally before complex changes
- Nightly job provides regression detection

### Memory Overhead

ASAN has configurable overhead:

```bash
# Strict checking (default)
ASAN_OPTIONS=verbosity=1 ./cargo/check.sh --memory-checks

# Less strict (faster, might miss issues)
ASAN_OPTIONS=quarantine_size=0 ./cargo/check.sh --memory-checks

# With leak detection
ASAN_OPTIONS=detect_leaks=1 ./cargo/check.sh --memory-checks
```

## Maintenance

### Annual Review

Check:
- New ASAN options/improvements
- Rust nightly stability
- Community PyO3/Python suppressions changes
- False positive patterns

### When Updating Dependencies

After `cargo update`:
1. Run local ASAN: `.cargo/check.sh --memory-checks`
2. Check nightly CI for regressions
3. Update suppressions if new patterns emerge

### Version Compatibility

Track versions:
- Rust nightly (monthly updates)
- Python version (if changed)
- PyO3 version (in Cargo.toml)

## References

- **ASAN Docs**: https://github.com/google/sanitizers/wiki/AddressSanitizer
- **LSAN Suppressions**: https://github.com/google/sanitizers/wiki/AddressSanitizerLeakDetection
- **Rust Sanitizers**: https://rustc-dev-guide.rust-lang.org/sanitizers.html
- **GitHub Actions**: https://docs.github.com/actions/using-workflows/

## Related Files

- `.cargo/check.sh`: Local validation script
- `.cargo/asan_suppressions.txt`: Suppression patterns
- `.github/workflows/memory-safety.yml`: CI workflow
- `tests/memory_safety_asan.rs`: Memory safety test suite
- `docs/MEMORY_SAFETY.md`: User-facing guide
- `docs/design/MEMORY_SAFETY_RUNBOOK.md`: Maintenance procedures

---

**Last updated:** 2026-01-09  
**Audience:** MRRC developers and maintainers
