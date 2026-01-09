# Memory Safety Validation

This guide helps you validate memory safety of MRRC changes using Address Sanitizer (ASAN).

## Quick Start

### Run Memory Safety Checks Locally

```bash
# Standard pre-push checks (does NOT include memory checks)
.cargo/check.sh

# Optional: Add memory safety validation
.cargo/check.sh --memory-checks
```

The `--memory-checks` flag runs ASAN (Address Sanitizer) on library tests, detecting:
- **Use-after-free**: Accessing freed memory
- **Memory leaks**: Unreleased allocations
- **Heap buffer overflows**: Writing outside allocated regions
- **Data races**: Concurrent memory access issues

## When to Run Memory Checks

Run `./cargo/check.sh --memory-checks` when:

- Making significant changes to memory-critical code
- Updating Rust dependencies (potential memory issues in dependencies)
- Modifying the PyO3 Python bindings (`src-python/`)
- Before submitting PRs with complex allocation patterns
- As part of pre-release validation

For routine changes (adding features, fixing bugs that don't touch allocation): The regular `.cargo/check.sh` is sufficient.

## Understanding ASAN Output

When ASAN detects an issue, it prints a detailed error report:

```
==12345==ERROR: AddressSanitizer: heap-use-after-free on unknown address 0x612000a6a000
==12345==READ of size 8 at 0x612000a6a000 thread T0
    #0 0x486b3e in your_function /path/to/file.rs:42:15
    #1 0x486c45 in main /path/to/file.rs:100:5
...
Address 0x612000a6a000 is 1 bytes inside a 100-byte region [0x612000a69b80,0x612000a69c44)
freed by thread T0 here:
    #0 0x486b3e in allocator /path/to/allocator.rs:10:3
...
```

**Key sections:**
- **Error type**: (e.g., `heap-use-after-free`, `memory-leak`)
- **Location**: File, line, column of problematic code
- **Stack trace**: Call chain leading to the issue
- **Memory details**: Address, region, and allocation history

## Common False Positives

ASAN integration uses `.cargo/asan_suppressions.txt` to filter known safe patterns:

- **PyO3 thread-local storage**: Allocated at thread init, freed at thread exit
- **Python interned strings**: Intentional string pool optimization
- **Dependency initialization**: Third-party libraries allocating static state

These suppressions are documented with technical rationale in `.cargo/asan_suppressions.txt`.

## Interpreting Results

**If `./cargo/check.sh --memory-checks` passes:**
- ✓ No memory issues detected in your changes
- ✓ Safe to proceed with normal CI

**If ASAN reports errors:**
1. **Read the error message** - Identify the issue type and location
2. **Check the backtrace** - Find where the problem occurs
3. **Is it in mrrc code?** - If yes, fix the bug
4. **Is it in a dependency?** - File an issue with the dependency or add a suppression (if known pattern)
5. **Is it a false positive?** - Document and add to suppressions if appropriate

## Troubleshooting

### ASAN Requires Nightly Rust

Memory safety checks require Rust nightly (for `-Z sanitizer=address`):

```bash
# Install nightly
rustup install nightly

# Or upgrade existing
rustup update nightly
```

If you don't have nightly, install it:
```bash
rustup install nightly
```

### Tests Fail with "Unsupported Sanitizer" Error

This usually means:
- Nightly toolchain not installed
- Nightly toolchain misconfigured

Try:
```bash
rustup update nightly
.cargo/check.sh --memory-checks
```

### Timeout or Slowness

ASAN adds overhead (~2x runtime). If tests timeout:
- Run on a less-busy machine
- Increase timeout settings
- Run only the affected test: `cargo test --lib --package mrrc -- --test-threads=1`

## CI Memory Safety Checks

MRRC includes a nightly CI job that runs ASAN on every commit:

- **Trigger**: Automatic nightly schedule (2 AM UTC daily)
- **Can also run**: Manually via "Run workflow" in GitHub Actions
- **Blocking**: No - reports issues but doesn't block merges
- **Purpose**: Detect regressions in memory safety

To check results:
1. Go to GitHub Actions
2. Select "Memory Safety Checks (ASAN)" workflow
3. Review recent runs

## For Library Maintainers

See `docs/design/MEMORY_SAFETY_CI.md` for comprehensive infrastructure details:
- How ASAN is configured
- CI strategy (nightly, non-blocking)
- Troubleshooting false positives
- Upgrading sanitizer versions

For suppression management, see `docs/design/MEMORY_SAFETY_RUNBOOK.md`:
- When to add/update suppressions
- Quarterly review procedure
- Escalation guidelines

## References

- **ASAN Documentation**: https://github.com/google/sanitizers/wiki/AddressSanitizer
- **Clang Sanitizer Flags**: https://clang.llvm.org/docs/SanitizerCoverage.html
- **Rust Sanitizer Support**: https://rustc-dev-guide.rust-lang.org/sanitizers.html
- **CONTRIBUTING.md**: Contributing guide (includes memory safety section)

## Questions?

- Check existing issues in the repository
- Review `.cargo/asan_suppressions.txt` for documented patterns
- Ask in GitHub Discussions

---

**Last updated:** 2026-01-09  
**Maintained by:** MRRC development team
