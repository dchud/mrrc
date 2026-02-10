# Hybrid GIL Release Strategy: Technical Implementation Plan

**Status:** Approved for Implementation  
**Date:** January 2, 2026  
**Related Documents:**
- [Review Assessment](GIL_RELEASE_HYBRID_PLAN_REVIEW_ASSESSMENT.md) (Basis for this plan)
- [Design Review](GIL_RELEASE_PROPOSAL_REVIEW.md)
- [Punchlist](GIL_RELEASE_PUNCHLIST.md) (Tracking)

---

## 1. Executive Summary

This plan replaces the previous "Three-Phase" implementation strategy. To resolve the performance bottleneck caused by sequential Python I/O (Phase B regression), we will implement a **Hybrid Strategy** with two distinct code paths:

1.  **Batching Path (Phase C):** For `File`-like objects (Streams, Sockets). Uses batch reading to amortize GIL acquisition costs.
2.  **Pure Rust Path (Phase H):** For file paths (`str`, `Path`). Bypasses Python I/O entirely for generic file reading, enabling near-native Rust performance via Rayon.

**Target Outcomes:**
- **Compatibility:** existing `open()` workflows speed up by **~1.8x**.
- **Performance:** `MARCReader("file.mrc")` speeds up by **>3.0x**.

---

## 2. Architecture: Dual Backend

The core architectural change is refactoring `PyMarcReader` to support swappable backends.

```rust
enum ReaderBackend {
    /// Phase C: Legacy compatibility with batching
    /// Wraps a Python file-like object
    PythonFile(BufferedMarcReader),
    
    /// Phase H: High-performance pure Rust I/O
    /// Wraps a native Rust file handle
    RustFile(std::io::BufReader<std::fs::File>),
}

struct PyMarcReader {
    backend: ReaderBackend,
    decoder: MarcRecordDecoder, 
}
```

---

## 3. Implementation Roadmap

| Phase | Name | Focus | Status | Dependency |
|---|---|---|---|---|
| **A** | Core Buffering | Infrastructure | ✅ Complete | — |
| **B** | GIL Integration | Mechanics | ✅ Code Done | A |
| **C** | **Batch Reading** | Compatibility | **TODO** | B |
| **H** | **Pure Rust I/O** | Performance | **TODO** | C (Parallel) |
| **D** | Writer Impl | Feature | ⚠️ Blocked | C |
| **E** | Validation | QA | Pending | D, H |
| **F** | Benchmarking | Proof | Pending | E |

---

## 4. Phase C: Batch Reading (Compatibility Path)

**Objective:** Repair the performance regression in the existing file-object reader by reading records in batches.

### C.1: Implement Internal Batch Buffer
*   **Task:** Modify `BufferedMarcReader` (or wrapper) to read `N` records at once.
*   **Method:**
    *   Add `read_batch(py, batch_size) -> Vec<RecordBytes>` method.
    *   Inside `__next__`:
        *   If `internal_queue` is empty, call `read_batch` (acquires GIL once).
        *   Pop from `internal_queue` and return.
*   **Constraints:**
    *   **Fixed Batch Size:** 100 records. (Hardware constant, non-configurable).
    *   **Queue Bounds:** Max 200 records capacity to prevent OOM.
*   **Acceptance:** `test_read_batch` unit test passes.

### C.2: Verify Iterator Semantics
*   **Task:** Ensure the batching is transparent to the user.
*   **Method:**
    *   Python `__next__` must return exactly one `PyObject` at a time.
    *   Handle `StopIteration` correctly when batch is exhausted and file is EOF.
*   **Acceptance:** Existing tests `tests/test_reader.py` pass without modification.

### C.3: Gate C Benchmark
*   **Task:** Verify speedup.
*   **Criteria:** ≥ 1.8x speedup vs single-thread baseline on `fixture_10k.mrc`.
*   **Action if Fail:** Stop and re-assess. Do not proceed to Phase H until understood.

---

## 5. Phase H: Pure Rust I/O (Performance Path)

**Objective:** Enable zero-GIL I/O for file paths.

### H.1: Refactor `PyMarcReader` Construction
*   **Task:** Update `__init__` to detect input type.
*   **Logic:**
    1.  Check if `source` is `String` (or `Path` converted to string) → **Init `RustFile`**.
    2.  Check if `source` is `Bytes` (raw bytes) → **Init `Cursor` (Rust)** (Optional optimization).
    3.  Check if `source` has `.read()` → **Init `PythonFile`** (Phase C).
    4.  Else → `TypeError`.
*   **Error Handling:** Map Rust `io::Error` to Python `FileNotFoundError` / `PermissionError`.

### H.2: Implement Pure Rust Read Loop
*   **Task:** Implement `Iterator` for `ReaderBackend::RustFile`.
*   **Method:**
    *   Read bytes using `std::io::BufReader`.
    *   Parse record using shared `MarcRecordDecoder`.
    *   **Crucial:** Wrap the specific read+parse block in `py.allow_threads()`? 
        *   *Correction:* For `RustFile`, we don't need `py.allow_threads` for I/O because we shouldn't be holding the GIL at all if possible? 
        *   Actually, `__next__` is called *with* the GIL. We must release it:
        ```rust
        // H.2 Implementation Sketch
        let py = unsafe { Python::assume_gil_acquired() };
        let result = py.allow_threads(|| {
             // Pure Rust: Read I/O + Parse
             self.backend.read_record() 
        });
        // Re-acquire GIL only to convert Result<Record> to PyObject
        ```
*   **Acceptance:** Proves zero GIL usage during I/O.

### H.3: Implement Rayon Parallelism
*   **Task:** Optimize `RustFile` backend with a background thread pool.
*   **Method:**
    *   Use `rayon` (default thread count).
    *   Producer-Consumer pattern:
        *   Producer: Rayon parallel iterator reads chunk, finds records, parses them.
        *   Consumer: `__next__` pops parsed records from a bounded channel (`crossbeam::channel` size ~1000).
*   **Constraint:** Hidden from user API. No `num_threads` arg.
*   **Acceptance:**
    *   Gate H Benchmark: ≥ 2.5x speedup on 4 threads.

---

## 6. Phase D: Writer Implementation Update

**Objective:** Ensure writer also benefits where possible, though Writer is less critical for the "Read-Analyze" use case.

### D.1: Writer Backend Refactoring (Deferred)
*   **Note:** We can stick to the Phase D plan (Python I/O wrappers) for now because writing to a new file often involves a file handle opened by the user (e.g., `with open(...) as f`).
*   **Decision:** Completing existing Phase D tasks (Python Write Wrapper) is sufficient for v1.

---

## 7. Phase E & F: Validation & Benchmarking

### E.1: Thread Safety Verification
*   **Task:** Run race condition torture tests (from `mrrc-kzw`).
*   **Scope:** Test both `PythonFile` (Batching) and `RustFile` backends.

### F.1: Comparative Benchmark Suite
*   **Task:** Create a final report comparing:
    1.  `pymarc` (Pure Python)
    2.  `pymrrc` (Legacy/Current)
    3.  `pymrrc` (Batching)
    4.  `pymrrc` (Pure Rust)
*   **Metrics:** Records/sec, Memory High Watermark.

---

## 8. Development Checklist

### Phase C (Batching)
- [ ] C.1: Implement `BufferedMarcReader::read_batch` (100 size)
- [ ] C.2: Update `PyMarcReader::__next__` to use batch queue
- [ ] C.3: Verify `StopIteration` and EOF behavior
- [ ] **GATE C:** Benchmark 2-thread speedup (Target 1.8x)

### Phase H (Pure Rust)
- [ ] H.1: Create `ReaderBackend` enum & Refactor `PyMarcReader` struct
- [ ] H.2: Implement `__init__` type detection (Path vs Object)
- [ ] H.3: Implement `RustFile` read loop (Sequential I/O + `allow_threads`)
- [ ] H.4: Add Rayon parallelism via `par_bridge` / channel
- [ ] **GATE H:** Benchmark 4-thread speedup (Target 2.5x)

### Phase D (Writer)
- [ ] D.1: Finalize Writer three-phase implementation (Existing plan)
- [ ] D.2: Add round-trip verification tests

### Finalization
- [ ] E.1: Stress tests for thread safety
- [ ] F.1: Benchmarking Report
- [ ] G.1: Update documentation (API docs, "Performance" section)
