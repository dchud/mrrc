# GIL Release Strategy for pymrrc Threading Performance

**Status:** Design Proposal  
**Date:** January 2026  
**Related Issue:** mrrc-gyk  
**Goal:** Unlock threading parallelism in pymrrc by enabling proper GIL release during I/O operations

---

## Problem Statement

Python's Global Interpreter Lock (GIL) prevents concurrent Python threads from executing Python code simultaneously. The current pymrrc implementation holds the GIL during all file I/O operations, which blocks other Python threads from running and eliminates potential parallelism benefits.

### Current State
- `Python::attach()` holds the GIL during entire I/O flow
- Other Python threads are blocked during record reading/writing
- Expected threading speedup (1.41x → 3.4x for concurrent reads) is **not achieved**

### Root Cause
The architecture couples I/O operations to Python object management:
- `PyFileWrapper` holds `Py<PyAny>` (a Python file object reference)
- PyO3 enforces that `Py<T>` requires the GIL to be held
- I/O methods (`read_record`, `write_record`) directly access this Python reference within their logic
- You cannot release the GIL while holding a reference to a Python object

---

## Approaches Investigated

### Approach 1: Naive `#[pyo3(allow_threads)]` Decorator
```rust
#[pyo3(allow_threads)]
fn read_record(&mut self) -> ... { ... }
```

**Why it failed:**
- Compilation errors due to incorrect syntax for PyO3 0.27
- Decorator alone doesn't solve the fundamental problem of holding Python references

### Approach 2: `py.allow_threads()` Wrapper
```rust
let result = py.allow_threads(|| {
    // I/O logic here, accessing slf
});
```

**Why it failed:**
- The closure captures `slf` (a `PyRefMut`), which holds a reference to the Python wrapper object
- `allow_threads` requires the closure to be `Send` (thread-safe)
- `PyRefMut` cannot cross thread boundaries while maintaining the GIL guarantee
- **Fundamental blocker:** You cannot access Python objects without the GIL held

### Approach 3: `Python::detach()` Pattern
**Why it failed:**
- Same underlying issue as Approach 2
- The closure still captures `slf`, remaining tied to the GIL
- Detach cannot make Python references Send

---

## Recommended Solution: Intermediate Buffer Pattern

### Architecture Overview

The solution separates I/O logic from Python object management into three distinct phases:

```
Phase 1 (GIL held)      Phase 2 (GIL released)      Phase 3 (GIL held)
─────────────────      ─────────────────────      ─────────────────
Read from Python   →    Parse/Process Bytes    →    Return to Python
file object            (pure Rust, no refs)        file object
```

### Implementation Details

#### Phase 1: Python-Bound I/O (GIL held)
Create a thin wrapper method that reads from the Python file object and returns raw bytes:

```rust
// In PyFileWrapper
fn read_bytes(&self, py: Python, bytes_to_read: usize) -> PyResult<Vec<u8>> {
    // This method MUST hold the GIL because it accesses self.file (Py<PyAny>)
    let file_obj = self.file.bind(py);
    let read_method = file_obj.getattr("read")?;
    let bytes_obj: PyBytes = read_method.call1((bytes_to_read,))?.extract()?;
    Ok(bytes_obj.as_bytes().to_vec())
}
```

#### Phase 2: Pure Rust Processing (GIL released)
The core parsing logic operates on bytes only—no Python references:

```rust
// In src-python/src/readers.rs
#[pymethods]
impl PyMarcReader {
    fn __next__(&mut self, py: Python) -> PyResult<PyObject> {
        // Phase 1: Read bytes (GIL held, fast)
        let bytes = self.file_wrapper.read_bytes(py, 65536)?;
        
        // Phase 2: Parse bytes (GIL released, allows other threads)
        let record = py.allow_threads(|| {
            self.reader.read_record(&bytes)
        })?;
        
        // Phase 3: Convert to Python (GIL held)
        Ok(record.into_pyobject(py)?)
    }
}
```

#### Phase 3: Result Conversion (GIL held)
Convert processed Rust data back to Python objects while holding the GIL.

### Why This Works

1. **Separation of concerns:** Python object access is isolated to a thin wrapper layer
2. **No dangling references:** The closure in `allow_threads` captures only `&mut self.reader`, which holds no Python references
3. **GIL released during expensive work:** CPU-intensive parsing runs without the GIL, allowing other Python threads to execute
4. **API compatibility:** End users don't see internal changes; the API remains pymarc-compatible
5. **Performance:** Threading speedup becomes achievable because I/O doesn't monopolize the GIL

### Alternative: Thread Pool Pattern

A more complex but potentially higher-throughput approach:
- Batch multiple I/O operations
- Release GIL once per batch instead of per-record
- Better for bulk processing workloads
- More complex API and state management
- Deferred as secondary optimization

---

## Expected Outcomes

With the Intermediate Buffer Pattern:
- **Threading speedup achieved:** Expected 1.41x → 3.4x for concurrent reads (previously blocked)
- **Rust performance parity:** pymrrc threading efficiency matches pure Rust parallelism
- **Minimal API changes:** Transparent to end users
- **Backward compatible:** Existing code continues to work

---

## Implementation Steps

1. Create `PyFileWrapper::read_bytes()` method (Phase 1)
2. Create `PyFileWrapper::peek_bytes()` for record boundary detection (Phase 1)
3. Refactor `PyMarcReader::__next__()` to use three-phase pattern (Phase 2/3)
4. Refactor `PyMarcReader::read_record()` to use three-phase pattern (Phase 2/3)
5. Refactor `PyMarcWriter::write_record()` similarly
6. Add benchmarking to verify threading speedup
7. Verify pymrrc matches Rust parallelism efficiency

---

## Risk Analysis

| Risk | Mitigation |
|------|-----------|
| Increased memory copies (Phase 1 reads into Vec) | Minor: I/O buffer sizes are already large; CPU savings from GIL release far outweigh memory cost |
| Complexity of three-phase pattern | Manageable: Pattern is localized to reader/writer methods |
| Edge cases in byte boundary handling | Covered: Existing record parsing logic already handles byte sequences |
| Binary compatibility | None: This is internal refactoring; API unchanged |

---

## Success Criteria

1. Threading benchmarks show 2x+ speedup for concurrent operations (currently 1.41x)
2. pymrrc threading performance within 90% of pure Rust performance
3. All existing tests pass without modification
4. No data loss or corruption in record processing
5. Backward compatibility maintained for all public APIs

---

## Related Review

See **GIL_RELEASE_REVIEW.md** for detailed technical feedback on this proposal, including:
- Critical implementation issues (record boundary detection, borrow checker interactions)
- Design improvements and optimization opportunities
- Testing recommendations for edge cases
