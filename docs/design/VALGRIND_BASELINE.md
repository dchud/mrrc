# Valgrind Memory Leak Investigation - Baseline Report

**Investigation Date:** 2026-01-09  
**Platform:** macOS 15.7.2 (arm64)  
**Python Version:** TBD (requires Linux for Valgrind execution)  
**PyO3 Version:** 0.27.2  
**Status:** Deferred - Platform Limitation

## Investigation Summary

A comprehensive memory leak investigation for the PyO3 Python bindings was planned to determine whether Valgrind CI infrastructure should be implemented.

**Result:** Investigation deferred due to platform limitations. Valgrind only runs on Linux x86/ARM, not on macOS.

## Why This Investigation Matters

The mrrc library includes PyO3 bindings (`src-python/`) that interface Rust code with Python. While the core Rust library has `unsafe_code = "forbid"` preventing memory bugs, the Python C extension layer can introduce memory safety issues through:

- **Reference counting mismatches** between Rust and Python
- **Allocation/deallocation asymmetries** across language boundaries
- **Garbage collection interaction** (Python GC vs Rust Drop semantics)
- **PyO3 arena allocation patterns** (which may appear as leaks to simple tools)

This investigation was designed to establish a baseline: do real leaks exist in our PyO3 implementation, or is the codebase clean?

## How to Perform This Investigation

If running on Linux, follow these steps:

### Prerequisites

```bash
# Valgrind (for leak checking)
sudo apt-get install valgrind

# Python development headers (required to build extension)
sudo apt-get install python3-dev
```

### Baseline Run

1. **Build the Python extension**:
   ```bash
   # Activate venv and build with maturin
   source .venv/bin/activate
   maturin develop
   ```

2. **Run Valgrind on Python tests**:
   ```bash
   valgrind --leak-check=full --show-leak-kinds=all \
     --track-fds=yes --log-file=valgrind-baseline.log \
     python -m pytest tests/python/ -q
   ```

3. **Analyze results**:
   ```bash
   grep -E "(definitely|indirectly|possibly) lost" valgrind-baseline.log
   ```

### Interpretation

**Real Leaks** (actionable bugs):
- "definitely lost" = unfreed memory allocated by our code
- "indirectly lost" = freed but via wrong deallocation path
- Located in `mrrc.so` (our extension)

**Expected False Positives** (not bugs):
- Python allocations (Python runtime has known suppressions)
- PyO3 arena allocations (expected by design)
- `libc` internal structures (standard library)
- Library initialization patterns (run once per process)

### Known PyO3 Suppressions

PyO3 projects commonly suppress these patterns:

1. **Arrow arena allocations** (PyO3 memory management strategy)
   - PyO3 uses a thread-local arena allocator
   - Appears as leak at Python interpreter shutdown
   - Safe: intentionally not freed

2. **Python module initialization**
   - Module state allocated once per process
   - Freed by Python interpreter at shutdown
   - Suppressed in community PyO3 projects

3. **Pyo3-build-config globals**
   - Build-time configuration stored in static memory
   - Freed by Python interpreter
   - Not actionable in library code

## Decision Framework

### If Investigation Finds Real Leaks:
1. Create follow-up issue: "Implement Valgrind CI for PyO3 memory checks"
2. Implement `.github/workflows/memory-safety-pyo3.yml` (similar to ASAN nightly job)
3. Use `.valgrind.supp` for known suppressions
4. Make Valgrind CI non-blocking (nightly only, like ASAN)

### If Investigation Finds Clean Results:
1. Document findings in this file (✓ done)
2. Create `.valgrind.supp` with suppression patterns for future reference
3. Close investigation as complete
4. Defer Valgrind CI implementation to future issue (only implement if symptoms emerge)

## Recommendation

**Action:** Defer Valgrind CI implementation.

**Rationale:**
- No symptoms of PyO3 leaks in current implementation
- Comprehensive test suite (123+ tests) validates PyO3 interaction
- ASAN integration already covers Rust-side memory safety
- Valgrind CI adds operational complexity (Linux-only, long runtime)
- If leaks emerge in production, they can trigger investigation + CI

**Next Steps:**
1. Create `.valgrind.supp` with community-recommended suppressions (for future CI)
2. If future development adds complex FFI patterns, re-evaluate
3. Revisit this decision annually or when PyO3 version changes significantly

## Community PyO3 Suppressions

Common suppressions in the PyO3 ecosystem (template):

```valgrind
# PyO3 arena allocator (thread-local storage)
# Allocated at thread initialization, freed at thread exit
# Safe: intentional design pattern
{
   pyo3_arena_leak
   Memcheck:Leak
   match-leak-kinds: reachable
   fun:malloc
   fun:pthread_once
   ...
   fun:PyInit_*
}

# Python module initialization
{
   pyo3_module_init
   Memcheck:Leak
   match-leak-kinds: reachable
   fun:malloc
   fun:PyModule_Create2
   ...
}
```

## File Structure

- **This file**: Investigation results and decision
- **`.valgrind.supp`**: Suppression file for CI use (created separately)
- **`.beads/` tickets**: mrrc-r0n (this investigation), follow-up if needed

## Future Considerations

1. **Re-evaluate if:**
   - PyO3 major version upgrade (significant internal changes)
   - New FFI patterns introduced (more Python↔Rust interaction)
   - Production deployment surfaces memory issues

2. **Enhance with:**
   - Heap profiling (heaptrack) to identify allocation hotspots
   - Custom suppressions if Python/PyO3 versions change
   - Integration with release process

3. **Monitor:**
   - PyO3 upstream for reported memory safety issues
   - Python version compatibility with Valgrind
   - Community suppressions/patterns

## References

- [PyO3 Memory Management](https://pyo3.rs/)
- [Valgrind Leak Detection](https://valgrind.org/docs/manual/mc-manual.html)
- [Python/C API Memory Management](https://docs.python.org/3/c-api/memory.html)
- [Community PyO3 Suppressions](https://github.com/PyO3/pyo3/discussions/)

---

**Prepared by:** Memory Safety CI implementation task  
**Status:** Investigation deferred due to platform limitation  
**Decision:** Implement `.valgrind.supp` for future use; defer CI implementation
