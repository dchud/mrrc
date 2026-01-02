# Hybrid Implementation Plan Revisions

**Date:** January 2, 2026  
**Status:** Approved with Additional Technical Depth  
**Related Documents:**
- [Review](GIL_RELEASE_HYBRID_IMPLEMENTATION_PLAN_REVIEW.md) - Technical assessment
- [Original Plan](GIL_RELEASE_HYBRID_IMPLEMENTATION_PLAN.md) - Strategic foundation
- [Parallel Benchmarking Summary](PARALLEL_BENCHMARKING_SUMMARY.md) - Discovered GIL limitation

---

## 1. Overview

This document formalizes revisions to the **Hybrid GIL Release Implementation Plan** based on:
1. Technical review of the original plan (GIL_RELEASE_HYBRID_IMPLEMENTATION_PLAN_REVIEW.md)
2. Execution readiness assessment and gap analysis
3. Integration with current codebase state (Phase B partially complete with GIL issues)
4. Lessons from parallel benchmarking feasibility study

The plan is **strategically sound** and remains **approved for execution**. These revisions add:
- Concrete implementation specifications to reduce execution risk
- Technical decisions addressing identified gaps
- Refined task breakdown aligned with current codebase state
- Dependency ordering and gate criteria

---

## 2. Critical Context: Current State Assessment

### 2.1 Phase B Status (GIL Release)
Current implementation (src-python/src/readers.rs) shows:
- **SmallVec buffer pattern** is implemented correctly for borrow checker compliance
- **Python::detach()** approach used for GIL release
- **Gap:** Batching not yet implemented; each `__next__` calls `read_batch(1)` implicitly
- **Gap:** No measurable speedup from GIL release in concurrent scenarios (1.4x observed vs 3-4x expected)

**Root Cause:** Phase B only releases GIL *during parsing*. When file I/O is Python-based, the actual bottleneck (Python file object's `.read()` method) still requires GIL. GIL is re-acquired on every record read.

**Decision:** Phase C (Batching) is **prerequisite** to realizing Phase B's GIL release benefit. Batching reduces GIL acquire/release frequency from N (records) to N/100 (batches).

### 2.2 Phase H Dependency Implications
- **Sequential baseline (Phase H.3) must run after Phase C** to establish correct performance foundation
- **Rayon parallelism (Phase H.4) cannot be tuned accurately** without sequential baseline
- Estimated timeline impact: +1 week due to sequential dependency

---

## 3. Phase C Revisions (Batch Reading - Compatibility Path)

### 3.1 Internal Data Structure Specification

**Struct Definition:**
```rust
pub struct PyMarcReaderState {
    /// Buffered reader maintaining reference to Python file object
    buffered_reader: BufferedMarcReader,
    
    /// Queue of raw record bytes, ready for parsing
    /// Using VecDeque for O(1) pop_front performance
    record_queue: VecDeque<SmallVec<[u8; 4096]>>,
    
    /// Capacity tracking for memory bounds
    queue_capacity_bytes: usize,
    
    /// EOF tracking for idempotent behavior
    eof_reached: bool,
    
    /// Batch size for read_batch() calls
    /// Fixed at 100 for initial implementation
    batch_size: usize,
}
```

**Design Rationale:**
- `VecDeque<SmallVec>` provides:
  - O(1) front-pop for `__next__` calls
  - Stack-allocated buffer (4KB) for typical MARC records (~1.5KB avg)
  - Fallback to heap allocation for records >4KB
- Capacity tracking prevents OOM in edge cases (malformed records with inflated length)
- `eof_reached` boolean ensures idempotent behavior: calling `__next__` after EOF repeatedly returns `None` without I/O attempts

### 3.2 Batch Size & Memory Model

**Decision: Fixed Batch Size = 100 (with validation task)**

**Memory Impact:**
- Average MARC record: 1.5 KB (empirically verified from test fixtures)
- Batch of 100: ~150 KB resident
- Queue capacity hard limit: 200 records (300 KB max)
- Safety margin: Prevents burst allocation during parsing slowdown

**Benchmark Task (C.Gate Sub-task):**
- Test batch sizes: 10, 25, 50, 100, 200, 500
- Measure:
  - GIL acquire/release frequency (via perf tracing)
  - Memory high watermark
  - Per-record processing time variance
  - Speedup curve shape (linear, knee point, plateau)

**Decision Tree: Batch Size Acceptance**

```
1. Run benchmarks at all 6 sizes (10, 25, 50, 100, 200, 500)
   ↓
2. Does speedup curve flatten/plateau (diminishing returns)?
   ├─→ YES (peak at 50, 100, or 200): Accept that size; move to criterion
   │   (e.g., if peak at 100, accept batch_size=100)
   │
   ├─→ NO (still climbing at 500): Continue benchmarking at 1000, 2000
   │   └─→ Peak found? Use that size
   │   └─→ No peak by 2000? Accept 500; investigate CPU vs I/O (see §3.2 diagnostic)
   │
   └─→ FLAT (all sizes similar speedup): Batch size not bottleneck
       └─→ Accept batch_size=100 (baseline); proceed to criterion

3. Achievement Check: Did best batch size achieve ≥1.8x speedup?
   ├─→ YES: ✓ Gate passes; proceed to Phase H
   │
   └─→ NO (max speedup <1.8x): Diagnose bottleneck
       1. Is GIL being released? (Run threaded concurrency test; measure GIL hold time)
       2. Is batch size the issue? (Increase hard capacity limit from 300KB to 600KB; retest)
       3. Is Python file I/O the bottleneck? (Profile `.read()` call overhead; may require Phase H)
       Decision: Document findings; proceed with caveats or defer to Phase H for RustFile
```

**Acceptance Criterion:** Target ≥1.8x speedup must be achieved with optimal batch size.

**If <1.8x Achieved:** Document root cause; proceed with batch_size=100 if:
- GIL is verifiably releasing (confirmed via threading test)
- No obvious implementation errors (code review passes)
- Note: Phase H RustFile may be required for further speedup

### 3.3 EOF State Machine & Iterator Semantics

**Formal State Specification:**

```
┌─────────────────────────────────────┐
│ INITIAL                             │
│ eof_reached=false, queue=empty      │
└──────────────┬──────────────────────┘
               │
               │ __next__() called
               ▼
┌──────────────────────────────────────┐
│ CHECK_QUEUE_NON_EMPTY                │
│ if !queue.is_empty()? YES → RETURN   │
│             NO ↓                     │
└──────────────┬───────────────────────┘
               │
               ▼
┌──────────────────────────────────────┐
│ CHECK_EOF_STATE                      │
│ if eof_reached? YES → RETURN NONE    │
│             NO ↓                     │
└──────────────┬───────────────────────┘
               │
               ▼
┌──────────────────────────────────────┐
│ READ_BATCH                           │
│ Call read_batch(batch_size=100)      │
│ Returns: Vec<RecordBytes> or error   │
└──────────────┬───────────────────────┘
               │
               ├─→ Error? Return Err → (state unchanged)
               │
               ├─→ Empty (0 records)? Set eof_reached=true → RETURN NONE
               │
               └─→ Non-empty (N records)? 
                   ├─ queue.extend(records)
                   ├─ pop_front() → RETURN
                   └─ Re-enter at CHECK_QUEUE_NON_EMPTY
```

**Idempotence Guarantee:**
```rust
// After EOF is reached, this is safe:
loop {
    match reader.__next__() {
        Ok(Some(record)) => { /* never reached */ },
        Ok(None) => { /* always here */ },
        Err(e) => { /* never here on 2nd+ call */ },
    }
}
// Behavior: Always returns Ok(None) without I/O
```

**Special Case: Partial Batch at EOF**
- File contains 217 records
- read_batch(100) at offset 200 returns: [record_200...217] (17 records)
- These 17 records are queued normally
- Next read_batch() returns 0 records → set eof_reached → return None
- **Result:** All 217 records delivered correctly via queue pops

### 3.4 Implementation Details: read_batch() Method

**GIL Contract Specification:**

The method acquires the GIL **once at entry** and holds it for the entire duration. Caller must NOT hold the GIL before calling. Upon return, GIL is released (via `py.allow_threads()` pattern or PyO3 RAII guard).

```rust
fn read_batch(&mut self, py: Python) -> Result<Vec<SmallVec<[u8; 4096]>>> {
    // Entry: Caller does NOT hold GIL
    // This block acquires GIL implicitly (py parameter guarantees it)
    
    let mut batch = Vec::with_capacity(self.batch_size);
    let mut bytes_read = 0usize;
    
    while batch.len() < self.batch_size {
        // GIL held throughout this loop
        match self.buffered_reader.read_next_record_bytes(py) {
            Ok(Some(record_bytes)) => {
                bytes_read = bytes_read.saturating_add(record_bytes.len());
                // Capacity check: hard limit at 200 records or 300KB
                if batch.len() >= 200 || bytes_read > 300_000 {
                    break;
                }
                batch.push(SmallVec::from_slice(&record_bytes));
            },
            Ok(None) => break,  // EOF reached
            Err(e) => return Err(e),
        }
    }
    
    // Exit: GIL automatically released (PyO3 scope ends)
    Ok(batch)
}
```

**Capacity Limit Behavior:**

When hard limits (200 records OR 300KB) are reached:
- **Action:** Immediately `break` from the loop and return the batch accumulated so far
- **Result:** Batch size becomes variable (e.g., 87 records if limit hit early)
- **Guarantee:** All returned records are complete; no partial/truncated records
- **Idempotence:** Caller will invoke `read_batch()` again on next `__next__()` call to continue reading

**Key Properties:**
- **Single GIL acquire/release cycle** for entire batch (amortizes acquisition overhead from N calls to 1)
- **Capacity safety:** Hard limits prevent OOM; batch size is upper bound, not guarantee
- **Boundary-safe:** Respects MARC record boundaries (0x1E record terminators), no split records
- **Overflow handling:** SmallVec stacks 4KB; records >4KB transparently heap-allocate without panic

---

## 4. Phase H Revisions (Pure Rust I/O - Performance Path)

### 4.1 Type Detection Mapping (Detailed)

**Input Type → Backend Mapping & Fallback Strategy:**

**Supported Input Types (in detection order):**

| Input Type | Python Signature | Rust Backend | GIL Requirement | Notes |
|---|---|---|---|---|
| `str` (path) | `"file.mrc"` | `RustFile` | Phase 1 only (init) | Uses `fs::File::open(path)` |
| `pathlib.Path` | `Path("file.mrc")` | `RustFile` | Phase 1 only (init) | Convert via `os.fspath()` in Python wrapper |
| `bytes` | `b"..."` | `CursorBackend` | Phase 1 only (init) | Wrap in `std::io::Cursor` |
| `bytearray` | `bytearray(b"...")` | `CursorBackend` | Phase 1 only (init) | Convert to bytes first |
| File object | `open(...)` | `PythonFile` | Every record (current Phase C) | Has `.read()` method |
| `io.BytesIO` | `io.BytesIO(...)` | `PythonFile` | Every record (current Phase C) | Falls through to `.read()` check |
| Socket, stream | Any with `.read()` | `PythonFile` | Every record (current Phase C) | Generic file-like protocol |

**Detection Algorithm (pseudo-code):**
```python
def __init__(self, source):
    # 1. Check string or Path (highest priority)
    if isinstance(source, str):
        return RustFile(source)
    if isinstance(source, pathlib.Path):
        return RustFile(os.fspath(source))
    
    # 2. Check bytes-like (optional, Phase H.2b)
    if isinstance(source, (bytes, bytearray)):
        return CursorBackend(source)
    
    # 3. Check file-like (fallback to Phase C)
    if hasattr(source, 'read') and callable(source.read):
        return PythonFile(source)
    
    # 4. Reject unknown types
    raise TypeError(
        f"Expected str, Path, bytes, or file-like object with .read() method, "
        f"got {type(source).__name__}"
    )
```

**Error Handling Mapping:**
- `RustFile`: `std::io::Error::NotFound` → `FileNotFoundError`
- `RustFile`: `std::io::Error::PermissionDenied` → `PermissionError`
- `RustFile`: Other `std::io::Error` → `IOError`
- `PythonFile`: Any error from `.read()` → Propagate from Python exception

**Unknown/Unsupported Input Type Fallback:**

If an input type does not match any of the above, the implementation follows this strategy:

1. **Detection Failure Behavior:**
   - Raise `TypeError` with a descriptive message listing supported types
   - Example: `"Unsupported input type: <class 'custom_reader'>. Supported types: str, Path, bytes, bytearray, file object (with .read()), BytesIO, socket.socket. Please pass a file path (str/Path) or file-like object with .read() method."`

2. **Rationale:**
   - **Fail-fast principle:** Do not silently attempt duck-typing; that causes unclear errors later
   - **API stability:** Explicit error messages prevent accidental reliance on undefined behavior
   - **Migration path:** If user needs custom type, they wrap it: `BytesIO(custom_reader.to_bytes())`

3. **No fallback to PythonFile:** Even if the object has a `.read()` method, if it's not explicitly recognized, it's rejected. This prevents downstream issues (e.g., custom object's `.read()` may not be thread-safe).

4. **Testing requirement (H.1):**
   - Test: `test_unknown_type_raises_type_error` - Verify descriptive error on unknown input
   - Test: `test_supported_types_complete` - Verify all 8 supported types are documented

### 4.2 CursorBackend Scope & Implementation

**Decision: Include CursorBackend in Phase H.2 (primary task scope)**

**Rationale for Including:**
1. **Enables in-memory batch processing** - Essential for testing parallelism without I/O bottleneck. Allows validation that Rayon speedup is from parsing, not I/O luck.

2. **Zero Python dependency** - Cursor-backed reader never calls Python, so it validates that `ReaderBackend` enum dispatch works correctly. Isolates threading issues from GIL issues.

3. **Minimal implementation cost** - `std::io::Cursor` is standard library; ~50 LOC wrapper. Adds negligible complexity vs benefit.

4. **Unblocks Phase H.3 baseline testing** - Can benchmark RustFile vs PythonFile using same test data without waiting for socket/stream support.

5. **High-value use case:** In-memory MARC batches (e.g., loaded from database)

6. **Send/Sync guarantees:** Unlike `PythonFile`, `CursorBackend` is `Send + Sync`, enabling safe use in Rayon thread pools

**API Layer Location (Python):**

Type detection and dispatch happens in `PyMARCReader.__init__` (PyO3 #[new] wrapper). The detection algorithm shown in §4.1 is implemented here:
```python
# In src-python/src/lib.rs, PyMARCReader::__new__
#[new]
fn new(source: PyObject, py: Python) -> PyResult<Self> {
    let backend = match source {
        // Try string path
        if PyUnicode_Check(&source) => {
            ReaderBackend::RustFile(File::open(PyUnicode_AsUTF8(py, &source))?)
        },
        // Try pathlib.Path
        if has_method(&source, "__fspath__") => {
            let path = source.call_method0(py, "__fspath__")?;
            ReaderBackend::RustFile(File::open(PyUnicode_AsUTF8(py, path)?)?)
        },
        // Try bytes/bytearray
        if PyBytes_Check(&source) | PyByteArray_Check(&source) => {
            let bytes = PyBytes_AsStringAndSize(&source);
            ReaderBackend::CursorBackend(Cursor::new(bytes.to_vec()))
        },
        // Try file-like object
        if has_method(&source, "read") => {
            ReaderBackend::PythonFile(source)
        },
        // Reject unknown
        _ => Err(TypeError::new_err("..."))?
    };
    Ok(PyMARCReader { backend, ... })
}
```

**Implementation:**
```rust
enum ReaderBackend {
    PythonFile(BufferedMarcReader),
    RustFile(BufReader<std::fs::File>),
    CursorBackend(Cursor<Vec<u8>>),
}

impl ReaderBackend {
    fn read_record(&mut self) -> Result<Option<Record>> {
        match self {
            Self::PythonFile(reader) => reader.read_record(),
            Self::RustFile(reader) => { /* sequential */ },
            Self::CursorBackend(cursor) => { /* sequential */ },
        }
    }
}
```

**Testing Strategy (H.5 integration tests):**
- Load 1000-record fixture into memory
- Create reader via `CursorBackend(bytes)`
- Verify identical output to `RustFile` backend
- Verify Cursor-backed reader can be used in parallel pool (thread safety check)

### 4.3 GIL Release Architecture (Clarified)

**Phase C (PythonFile backend):**
```rust
// GIL is held for entire __next__() call
// This is necessary because Python file object access requires GIL
let py = Python::acquire_gil();  // Implicit in PyO3 methods
let result = {
    // Phase 1: Read bytes (GIL held - required by Python file object)
    let record_bytes = buffered_reader.read_batch(py)?;  // Calls Python .read()
    
    // Phase 2: Parse batch (GIL released)
    py.detach(|| {
        let mut parsed = Vec::new();
        for bytes in record_bytes {
            let cursor = std::io::Cursor::new(bytes.to_vec());
            let record = MarcReader::new(cursor).read_record()?;
            parsed.push(record);
        }
        Ok(parsed)
    })
};
// Phase 3: Convert to PyRecord (GIL re-acquired)
```

**Phase H (RustFile backend):**
```rust
// GIL is NOT held during I/O and parsing
let py = Python::acquire_gil();  // Implicit in PyO3 method
let result = {
    // Phase 1: Release GIL entirely for pure Rust code
    py.detach(|| {
        // Both I/O and parsing happen without GIL
        // No Python object access allowed here
        let mut reader = BufReader::new(self.file_handle);
        reader.read_record()
    })?
};
// Phase 2: Convert to PyRecord (GIL re-acquired)
```

**Key Difference:** `PythonFile` holds GIL during Phase 1 (Python `.read()` call). `RustFile` releases GIL for both I/O and parsing.

### 4.4 Rayon Architecture (Refined)

**Challenge:** File I/O is inherently sequential (can't know where record boundaries are without reading). Parallelization must handle logical record extraction.

**Record Boundary Detection:**

MARC records are terminated by the **field terminator (0x1E)**, which also serves as the record terminator when it appears after the final field and before the record length indicator. When scanning file bytes sequentially, detect record boundaries by finding the `0x1E` byte that completes a MARC record structure.

**Refined Two-Thread Strategy:**

```
File Reader Thread (Single, Sequential):
  ┌─────────────────────────────────┐
  │ Read file in 64KB chunks        │
  │ Identify MARC record boundaries │
  │ (scan for 0x1E terminators)     │
  │ Extract raw record byte ranges  │
  └────────────────────┬────────────┘
                       │
                       │ (record_boundaries: Vec<(offset, len)>)
                       ▼
        ┌──────────────────────────┐
        │ Bounded Channel (1000)   │
        │ (acts as work queue)     │
        └──────────────┬───────────┘
                       │
   Rayon Thread Pool (Parallel):
  ┌────────────────────┴────────────────────┐
  │ for each record_boundary in queue:      │
  │   - Fetch bytes from mmap/buffer        │
  │   - Parse into MarcRecord               │
  │   - Push result to output channel       │
  └────────────────────┬────────────────────┘
                       │
                       ▼
        ┌──────────────────────────┐
        │ Output Channel (1000)    │
        │ (parsed records)         │
        └──────────────┬───────────┘
                       │
                       ▼
               Consumer: __next__()
               (pops parsed records)
               ```

               **Backpressure & Memory Safety:**

               When input queue reaches 1000 boundaries:
               - File reader thread blocks on `channel.send()`
               - Rayon workers continue processing pending work
               - Consumer calls `__next__()` → drains output queue
               - Reader unblocks once output queue shrinks

               **Boundary Queue Memory:**
               - Each boundary tuple: (offset: usize, len: usize) = ~16 bytes
               - 1000 items: ~16 KB
               - Safe for all MARC files

               **Error Propagation:**
               - Parse error in Rayon task → captured in `Result<Record>`
               - Propagates through output channel to consumer
               - Consumer sees error on next `__next__()` call
               - Reader continues reading (errors don't halt pipeline)

               **Send/Sync Guarantee:**

               Only `RustFile` and `CursorBackend` can use Rayon pipeline (both `Send + Sync`). `PythonFile` remains sequential in Phase C (cannot be shared across threads safely due to GIL).

               **Detailed Implementation Plan:**

**H.4a: Delimiter Scanner (I/O Thread)**
```rust
// Optimization: Use `memchr` crate for SIMD-accelerated scanning
fn scan_record_boundaries(file: &File, buffer: &[u8], limit: usize) 
    -> Result<Vec<(usize, usize)>> {
    // Scan buffer for MARC record terminators (0x1E)
    // Each complete MARC record ends with 0x1E
    let mut boundaries = Vec::new();
    let mut offset = 0;
    
    // Use memchr::memchr_iter for significant speedup over standard iter
    for i in memchr::memchr_iter(0x1E, buffer) {
        boundaries.push((offset, i - offset + 1));
        offset = i + 1;
    }
    
    // Return boundaries up to limit
    Ok(boundaries.into_iter().take(limit).collect())
}
```

**H.4b: Rayon Parser Pool**
```rust
fn parse_batch_parallel(
    record_boundaries: Vec<(usize, usize)>,
    buffer: &[u8],
) -> Result<Vec<Record>> {
    record_boundaries
        .par_iter()  // Rayon parallel iterator
        .map(|&(offset, len)| {
            let record_bytes = &buffer[offset..offset + len];
            let cursor = std::io::Cursor::new(record_bytes);
            MarcReader::new(cursor).read_record()
        })
        .collect::<Result<Vec<_>>>()
}
```

**H.4c: Backpressure & Channel Management**
```rust
// Producer task (running in background)
fn producer_task(file: File, sender: Sender<Record>) {
    let mut buffer = vec![0u8; 512 * 1024];  // 512 KB buffer
    
    loop {
        let n = file.read(&mut buffer).expect("read failed");
        if n == 0 {
            break;  // EOF
        }
        
        let boundaries = scan_record_boundaries(&file, &buffer[..n], 100)
            .expect("scan failed");
        
        let records = parse_batch_parallel(boundaries, &buffer[..n])
            .expect("parse failed");
        
        // send() blocks if channel is full (bounded to 1000)
        // This provides backpressure when consumer is slow
        for record in records {
            sender.send(record).expect("send failed");
        }
    }
}

// Consumer: __next__()
fn __next__(&mut self) -> PyResult<Option<PyRecord>> {
    match self.receiver.try_recv() {
        Ok(record) => Ok(Some(PyRecord { inner: record })),
        Err(TryRecvError::Empty) => {
            // Wait for producer to deliver
            match self.receiver.recv() {
                Ok(record) => Ok(Some(PyRecord { inner: record })),
                Err(_) => Ok(None),  // Channel closed = producer done
            }
        },
        Err(TryRecvError::Disconnected) => Ok(None),  // Producer panicked
    }
}
```

**Backpressure Behavior:**
- Channel size = 1000 records
- Producer blocks when channel full (prevents OOM)
- Consumer unblocks producer when it drains queue
- **Side effect:** May reduce parallelism benefit if consumer is very slow (but still safe)

### 4.5 Thread Count Configuration

**Decision: Hidden from Python API, Respects RAYON_NUM_THREADS env var**

**Rationale:**
1. **API Simplicity:** No `num_threads` parameter clutters Python interface
2. **Standard convention:** Rayon's default uses all CPU cores (reasonable default)
3. **Advanced users:** Can set `RAYON_NUM_THREADS=4` before importing mrrc
4. **Testability:** Environment variable easier to control in tests

**Implementation:**
```rust
// Rayon automatically reads RAYON_NUM_THREADS
// No explicit thread_pool configuration needed
let num_threads = rayon::current_num_threads();  // Respects env var
// Use rayon as-is; parallelism automatically scales
```

**Testing & Documentation:**
- Document `RAYON_NUM_THREADS` in Python API docstring
- Add example: "To use 4 threads, set `RAYON_NUM_THREADS=4` before importing"
- Test suite must run with `RAYON_NUM_THREADS=2` (CI constraint)

---

## 5. Risk Mitigation & Prerequisites

### 5.1 Prerequisite: Rayon PoC (H.0)

**Scope:** Before implementing full H.4 (Rayon pipeline), validate thread-safety approach.

**PoC Task:**
```rust
// Create minimal example:
// 1. Spawn Rayon task pool
// 2. Populate crossbeam::channel from Rayon
// 3. Consume from channel in main thread
// 4. Verify no panic propagation issues
// 5. Measure overhead vs single-thread
```

**Success Criteria:**
- No panic leaks; channel closes cleanly
- Overhead <5% for small batches
- Speedup >1.5x for CPU-bound workload
- No memory leaks (check RSS with valgrind)

**Time Estimate:** 1 day

### 5.2 Memory Safety Checks

**Phase C:**
- Bounds checking on queue capacity (hard limit 200 records)
- Validate SmallVec ownership across GIL boundaries

**Memory safety validation (Platform-dependent):**

**Linux (preferred for ASAN):**
- **ASAN (AddressSanitizer):** Required before C.Gate approval
  - Command: `RUSTFLAGS=-Zsanitizer=address cargo test --target x86_64-unknown-linux-gnu --release`
  - Validates: No heap-buffer-overflow, use-after-free, double-free
  - Note: PyO3 tests under ASAN may have false positives from Python C API internals; document suppressed warnings in test output
  
- **Valgrind (memcheck):** Secondary validation
  - Command: `valgrind --leak-check=full --track-origins=yes --suppressions=.valgrind_suppressions target/release/test_suite`
  - Run on 10k-record fixture; goal: Zero definite leaks
  - May report "possibly lost" memory from Python allocations (acceptable; document in report)

**macOS (alternative):**
- **Leaks tool (native macOS):** Use if ASAN unavailable
  - Command: `cargo test --release && leaks -atExit -- target/release/test_suite`
  - Validates: No memory leaks in resident set
  
- **Instruments (Xcode):** For detailed profiling (optional)
  - Use Allocations instrument; compare high watermark vs baseline

**Gate requirement:** Must show zero definite leaks on chosen platform. Document platform used and any suppressed warnings.

**Phase H:**
- Verify `CursorBackend` doesn't retain Python references
  - Test: `test_cursor_no_python_refs` - no PyObject members in Cursor state
- Test: Cursor-backed reader in Rayon pool (multi-thread safety)
  - Test: `test_cursor_rayon_send_sync` - verify Send/Sync bounds at compile time
- Test: Drop reader mid-stream; verify no resource leaks
  - Test: `test_reader_drop_mid_stream` - resource cleanup on panic

### 5.3 Testing Integration Points & Coverage

**Phase C tests (must pass before Phase H):**
- `test_batch_reading_idempotence`: EOF returns None repeatedly (happy path + error recovery)
- `test_batch_partial_at_eof`: Last batch <100 records
- `test_thread_safety_with_batching`: 4 threads reading same fixture concurrently
- `test_memory_bounds`: Monitor RSS during batch queue fill
- **Malformed record handling:**
  - `test_batch_with_truncated_record`: Record missing 0x1E terminator
  - `test_batch_with_wrong_length_encoding`: Length field doesn't match actual bytes
  - `test_batch_with_oversized_record`: Record >4KB (heap allocation)

**Phase H.3 tests (sequential baseline):**
- `test_rust_file_parity`: `RustFile` output identical to `PythonFile` (bit-for-bit)
- `test_cursor_backend_parity`: `CursorBackend` output identical to `RustFile`
- `test_type_detection`: Verify all 8 input types (str, Path, bytes, bytearray, file, BytesIO, socket, unknown) routed correctly
- **Coverage target:** ≥95% line coverage for H.1, H.2, H.2b modules

**Phase H.4 tests (parallelism):**
- `test_rayon_pool_safety`: No panics, clean shutdown (verify panic hooks)
- `test_rayon_channel_backpressure`: Producer blocks when channel full; unblocks when drained
- `test_concurrent_readers`: Multiple MARCReaders in thread pool without interference
- `test_rayon_panic_propagation`: Error in Rayon task bubbles up to consumer correctly
- `test_boundary_scanner_correctness`: All record boundaries identified (0x1E scan)
- **Coverage target:** ≥95% line coverage for H.4a, H.4b, H.4c modules

---

## 6. Revised Task Breakdown

### Phase C: Batch Reading (Compatibility Path)

**C.0: Data Structure, State Machine & GIL Verification Test**
- Define `PyMarcReaderState` struct with `VecDeque<SmallVec>` buffer
- Implement EOF state machine (3-state logic as per §3.3)
- Implement idempotence guarantee for `__next__` calls
- **Create diagnostic tests (§7.1):**
  - `tests/concurrent_gil_tests.rs` - `test_gil_release_verification` test
  - `scripts/benchmark_batch_sizes.py` - Batch size sweep utility
  - `scripts/test_python_file_overhead.py` - I/O overhead profiler
- Acceptance: State diagram verified in code review; diagnostic tests pass

**C.1: Implement read_batch() Method**
- Add `read_batch(batch_size=100)` to `BufferedMarcReader`
- Single GIL acquire/release cycle for entire batch
- Capacity safety: Hard limits (200 records, 300KB)
- Acceptance: Unit test `test_read_batch_returns_100_records`

**C.2: Update __next__() to Use Queue**
- Refactor `PyMARCReader::__next__()` to use queue-based FSM
- Pop from queue; if empty, call read_batch() once
- Handle all state transitions correctly
- Acceptance: Existing test suite passes unchanged

**C.3: Iterator Semantics & Idempotence**
- Verify `StopIteration` behavior matches Python expectations
- Verify repeated calls after EOF return `None` without I/O
- Test partial batches at EOF
- Acceptance: `test_iterator_idempotence` passes

**C.4: Memory Profiling Sub-task**
- Measure memory high watermark with various batch sizes
- Verify queue capacity limits are enforced
- Check for memory leaks (valgrind)
- Acceptance: Memory usage stable over 100k-record read

**C.Gate: Benchmark Batch Sizes**
- Benchmark batch sizes: 10, 25, 50, 100, 200, 500
- Measure: GIL frequency, memory, per-record latency
- Decision: Accept batch_size=100 if speedup curve flattens (diminishing returns)
- **Gate Criteria: Must achieve ≥1.8x speedup on 2-thread concurrent read**

### Phase H: Pure Rust I/O (Performance Path)

**H.0: Rayon PoC (Parallel with C if resources allow)**
- Build minimal thread pool → channel pipeline
- Validate thread-safety and panic handling
- Measure overhead vs single-thread
- Acceptance: PoC achieves >1.5x speedup, no panics

**H.1: ReaderBackend Enum & Type Detection**
- Create `ReaderBackend` enum (Variants: PythonFile, RustFile, CursorBackend)
- Implement type detection algorithm (§4.1)
- Implement error handling for unknown/unsupported types (§4.1 fallback)
- Acceptance: Type detection unit tests pass (8 supported types + unknown type error tests)
  - `test_str_path_detection` - Verify str input → RustFile
  - `test_pathlib_path_detection` - Verify pathlib.Path input → RustFile
  - `test_bytes_detection` - Verify bytes input → CursorBackend
  - `test_bytearray_detection` - Verify bytearray input → CursorBackend
  - `test_file_object_detection` - Verify file object → PythonFile
  - `test_bytesio_detection` - Verify BytesIO input → CursorBackend
  - `test_socket_detection` - Verify socket.socket input → PythonFile
  - `test_unknown_type_raises_type_error` - Verify unknown type raises TypeError (§4.1 fallback)
  - `test_supported_types_complete` - Verify documentation lists all 8 types

**H.2: Sequential RustFile Implementation**
- Implement `ReaderBackend::RustFile` using `BufReader<File>`
- Implement `__init__` path dispatch logic
- Verify GIL release during pure Rust I/O
- Acceptance: `test_rust_file_parity` - output identical to PythonFile

**H.2b: Sequential CursorBackend (Parallel with H.2)**
- Wrap `Cursor<Vec<u8>>` in ReaderBackend
- Verify in-memory processing works identically to File
- Acceptance: `test_cursor_parity` - output identical to RustFile

**H.3: Baseline Benchmarking (Before Parallelism)**
- Benchmark sequential RustFile vs PythonFile
- Measure GIL release overhead
- Establish baseline for parallelism speedup calculation
- Acceptance: RustFile ≥1.0x faster than PythonFile (sequential)

**H.4a: Delimiter Scanner (I/O Thread)**
- Implement record boundary detection (0x1E scan - record terminator)
- Read file in chunks; identify record starts/ends
- Queue raw byte boundaries (not actual bytes)
- **Acceptance:** 
  - Unit test: `test_delimiter_scanner_all_boundaries` - identifies all 0x1E terminators in fixture
  - Output matches pre-computed boundary map (via `generate_boundary_map.py`)
  - Handles truncated file (last record without terminator)
  - Performance: Scan 1MB file <10ms

**H.4b: Rayon Parser Pool**
- Build Rayon task for parsing record byte ranges
- Use `par_iter()` for parallel processing
- Handle errors within parallel context
- Acceptance: Parsing produces identical output to sequential

**H.4c: Producer-Consumer Pipeline Integration**
- Connect boundary scanner → Rayon pool → output channel
- Implement backpressure (bounded channel at 1000)
- Handle EOF and error conditions
- Acceptance: `test_rayon_pipeline_correctness` passes

**H.5: Integration Test Suite**
- `test_backend_interchangeability`: All backends produce identical output (record-by-record comparison)
- `test_type_detection_coverage`: All 8 input types route correctly to backend
- `test_concurrent_rayon_safety`: No panics in thread pool; clean channel shutdown
- `test_rayon_error_propagation`: Parse error in task propagates to consumer correctly
- `test_memory_under_parallelism`: Memory peak <2x single-thread (backpressure effective)
- **Acceptance:** All 5 tests pass; ≥95% line coverage for H.1-H.5 modules
  - Generate coverage report: `cargo tarpaulin -o Html --output-dir target/coverage`
  - Review uncovered lines; document if intentional (panic paths, error cases)

**H.Gate: Parallel Benchmarking**
- Benchmark with 2, 4, 8 threads
- Measure: Throughput, memory, lock contention
- **Gate Criteria: Must achieve ≥2.5x speedup on 4 threads vs single-thread**
- If <2.5x: Profile bottleneck before proceeding to production

---

## 7. Acceptance Criteria & Gate Decisions

### Gate C: Batch Reading Validation
**Status:** Blocks Phase H  
**Criteria:**
- [ ] C.0-C.4 tasks complete
- [ ] `test_iterator_semantics` passes (100% coverage)
- [ ] `test_batch_memory_bounds` passes
- [ ] **Concurrent speedup ≥1.8x on 2 threads with batch_size=100**

**Decision Points:**
- If speedup ≥1.8x: ✓ Gate passes; proceed to Phase H
- If speedup 1.5x-1.8x: Investigate thoroughly before deciding. Accept only if:
  - GIL is verifiably releasing (confirmed via threading test + perf trace)
  - Code review finds no obvious inefficiencies
  - Document as "acceptable interim result; Phase H may improve further"
- If speedup <1.5x: ✗ Gate fails; diagnose root cause (GIL release, batch size, Python file overhead) before proceeding

### Gate H.3: Sequential Baseline
**Status:** Prerequisite for H.4  
**Criteria:**
- [ ] RustFile output identical to PythonFile
- [ ] CursorBackend output identical to RustFile
- [ ] GIL release verified (no GIL overhead in Rust sections)
- [ ] Memory usage stable (no leaks)

**Decision:** If any parity test fails, pause H.4 and debug

### Gate H (Final): Parallel Speedup
**Status:** Blocks production release  
**Criteria:**
- [ ] All H.0-H.5 tasks complete
- [ ] Integration test suite passes (≥95% line coverage)
- [ ] **Parallel speedup ≥2.5x on 4 threads vs single-thread baseline**
- [ ] Memory peak <2x single-thread (backpressure effective)

**Decision Tree: Handling Speedup Shortfalls**

If speedup <2.5x, execute profiling strategy (§7.1) to diagnose:
1. **I/O bottleneck?** (File read is sequential; Rayon can't parallelize)
2. **Parser limited?** (Each record takes <1ms; Rayon overhead >benefit)
3. **Lock contention?** (Channel, buffer access serializes threads)

**Outcomes:**
- If <2.5x speedup: Execute diagnostics; document findings
- If 2.0-2.5x: Document limitation (acceptable given architecture); proceed with caveat in Release Notes
- If <2.0x: Either accept architectural limitation OR explore:
  - Alternative: mmap file + parallel delimiter scanning (requires more complexity)
  - Fallback: Recommend Phase C batching as primary optimization; mark H as advanced feature

---

## 6.5 Rollback & Revert Strategy

**Scenario: Phase C implementation breaks existing functionality**

If during Phase C development we discover critical regressions (data corruption, panics, data loss), follow this revert procedure:

1. **Revert commits (git):**
   ```bash
   git revert <C.0-hash>..<C.Gate-hash> --no-edit
   ```

2. **Restore Phase B state:**
   - Revert strategy keeps Phase B GIL release intact (since C.0-C.4 only modify `__next__()` and add `read_batch()`)
   - Phase B changes remain: `Python::detach()`, SmallVec buffer, existing parsing optimizations
   - Only removes: batching queue, new state machine logic

3. **Data preservation check:**
   - After revert, run regression test: `cargo test test_phase_b_data_parity`
   - Confirms: All 75+ existing test pass; no data loss in revert

4. **Decision point:**
   - If data parity confirmed after revert: Safe to proceed with Phase H or alternative batch implementation
   - If regression persists: Investigate Phase B itself (unlikely, but safeguard)

5. **Communication:**
   - Document findings in issue tracker (bd)
   - Note: "Reverted Phase C due to [specific reason]; Phase B remains stable"
   - Do not attempt Phase H until Phase C is resolved or alternative approach found

**This preserves the 1.4x baseline speedup from Phase B while allowing safe experimentation with Phase C.**

---

## 7. Profiling & Diagnostics Strategy

### 7.1 When Gate Criteria Aren't Met: Diagnostic Tools

**Scenario 1: Phase C speedup <1.8x (Batching not working)**

Diagnostic workflow:
```bash
# 1. Verify GIL is releasing (required test in C.Gate)
cargo test test_gil_release_verification --release -- --nocapture
# This test spawns 2 threads; if GIL is truly released during parsing,
# threads should run concurrently (wall clock time ≈ half of sequential)
# If GIL is held, wall clock time ≈ sequential time

# 2. Profile GIL acquisition frequency with perf
perf record -g -c 10000 target/release/mrrc_perf_test_batch_100k
perf report
# Look for Python_Release_Lock and Python_Acquire_Lock in hot path

# 3. Check batch size efficacy
python scripts/benchmark_batch_sizes.py  # Located in scripts/; tests 10, 25, 50, 100, 200, 500
# Produces: speedup_curve.csv with batch_size vs speedup columns
# Look for knee point in curve (diminishing returns)

# 4. Profile Python file I/O overhead
py-spy record -o profile.svg -- python scripts/test_python_file_overhead.py
# Identifies what % of time is spent in Python .read() vs Rust parsing
```

**Test Specifications (to be created in C.0 task):**

- `test_gil_release_verification` (Rust, in tests/): 
  - Creates 2-threaded reader; measures wall clock time vs sequential baseline
  - Success: Concurrent time < 1.5x sequential (indicates GIL released during parsing)
  - Location: `tests/concurrent_gil_tests.rs`

- `benchmark_batch_sizes.py` (Python, in scripts/):
  - Sweeps batch sizes 10, 25, 50, 100, 200, 500
  - Measures: Time per 10k records, GIL hold duration (via sys.tracing if available)
  - Outputs: CSV with columns [batch_size, time_sec, speedup_factor]
  - Location: `scripts/benchmark_batch_sizes.py`

- `test_python_file_overhead.py` (Python, in scripts/):
  - Profiles time spent in Python `.read()` calls vs time in Rust parsing
  - Uses `cProfile` or `py-spy` to measure
  - Location: `scripts/test_python_file_overhead.py`

**Decision points:**
- If GIL releasing but speedup still <1.8x: Python `.read()` is bottleneck; Phase H required
- If batch size curve flat: Increase from 100 to 200; retest
- If `.read()` takes >50% of time: I/O-bound; only Phase H RustFile helps

**Scenario 2: Phase H speedup <2.5x (Parallelism not effective)**

Diagnostic workflow:
```bash
# 1. Profile CPU utilization
cargo test test_rayon_4threads --release -- --nocapture
# Monitor: `top`, `htop`, or Activity Monitor
# Expected: 4 cores at ~80%+ utilization
# If <50%: Rayon overhead too high or work-stealing inefficient

# 2. Identify serialization bottleneck
# Option A: Check channel contention
cargo build --release --features="profiling"
./target/release/test_rayon_profile 4 threads
# Look for wait times on channel.send/recv

# Option B: Check I/O bottleneck
# Run boundary scanner separately; measure time
perf stat target/release/test_delimiter_scanner --file test.mrc
# If scan takes >50% of total time: I/O is limit

# Option C: Check parser performance
# Benchmark par_iter() with pre-computed boundaries
cargo bench --bench rayon_parse_only
# Compare to sequential; if <2x speedup: Parser isn't CPU-bound

# 3. Memory pressure?
valgrind --tool=massif target/release/test_rayon_memory_profile
# Check heap profile; look for >2x peak single-thread
```

**Decision points:**
- If CPU <50%: Rayon overhead dominant; try coarser granularity (100 records per task)
- If channel contention >30%: Reduce bounded channel size from 1000 to 500
- If I/O >60% of time: Bottleneck is file reading; parallelism can't help
- If parser <CPU-bound: MARC parsing is too fast; only large files benefit

### 7.2 Benchmarking Infrastructure

**Baseline Progression Strategy:**

The plan establishes a **cascade of baselines**, each measuring improvement against the previous phase:

1. **Phase B Baseline** (current state): Existing implementation with GIL release during parsing, no batching
   - Run once before Phase C work begins
   - Establish: 1.4x speedup on 2-thread concurrent read (known from prior testing)
   - Command: `cargo bench --bench baseline -- --save-baseline=phase_b`

2. **Phase C vs Phase B**: Measure batching improvement against Phase B
   - Gate criterion: ≥1.8x vs Phase B baseline (not absolute—relative improvement)
   - Command: `cargo bench --bench batch_reading -- --baseline=phase_b`
   - Expected: At least 1.8x speedup (factor)

3. **Phase H vs Phase B**: Measure full parallelism against original Phase B
   - Gate criterion: ≥2.5x vs Phase B baseline (measured on 4-thread sequential + Rayon)
   - Command: `cargo bench --bench rayon_parallel -- --baseline=phase_b`
   - Expected: At least 2.5x speedup (factor)

**Required tools (Phase C & H):**
```bash
# Install benchmarking suite
cargo install criterion
cargo add memchr  # Required for optimized delimiter scanner
cargo install perf

# Establish Phase B baseline (run once, before Phase C work)
cargo bench --bench baseline --release -- --save-baseline=phase_b

# After Phase C (batching) - measures vs Phase B
cargo bench --bench batch_reading --release -- --baseline=phase_b
# Expected: ≥1.8x speedup vs phase_b

# After Phase H (Rayon) - measures vs Phase B
cargo bench --bench rayon_parallel --release -- --baseline=phase_b
# Expected: ≥2.5x speedup vs phase_b
```

**Benchmark fixtures:**
- 1k-record fixture (~1.5 MB): Quick iteration
- 10k-record fixture (~15 MB): Stress test memory
- 100k-record fixture (~150 MB): Real-world scale
- Pathological fixture: 1000 very large records (10KB each)

---

## 8. Execution Roadmap & Timeline

### Week 1: Phase C (Batch Reading)
- **Day 1-2:** C.0-C.1 (Data structure + read_batch)
- **Day 3-4:** C.2-C.3 (Queue FSM + Iterator semantics)
- **Day 5:** C.4-Gate (Benchmarking + batch size tuning)
- **Outcome:** ≥1.8x speedup on batching

### Week 2: Phase H.0 PoC + H.1-H.2 (Parallel with C Day 5, if resources allow)
- **Day 1:** H.0 (Rayon PoC - standalone)
- **Day 2-3:** H.1 (ReaderBackend enum + type detection)
- **Day 4-5:** H.2-H.2b (RustFile + CursorBackend sequential)

### Week 3: Phase H.3-H.4 (Baseline + Parallelism)
- **Day 1:** H.3 (Baseline benchmarking, parity tests)
- **Day 2-4:** H.4a-H.4c (Pipeline implementation)
- **Day 5:** H.5 (Integration tests)

### Week 4: Phase H.Gate + Release Prep
- **Day 1-2:** H.Gate (Parallel benchmarking)
- **Day 3-5:** Documentation, edge case testing, release prep

**Total Effort:** 4 weeks (20 days)  
**Critical Path:** C.Gate → H.3.Gate → H.Gate

---

## 9. Risk Register & Mitigation

| Risk | Probability | Impact | Mitigation |
|------|---|---|---|
| GIL still not releasing in Phase C | Medium | High | H.0 PoC de-risks early; use diagnostic tools (§7.1); accept fallback to Phase H |
| Rayon overhead exceeds speedup benefit | Low | Medium | H.0 PoC validates; profile bottleneck (§7.1); consider mmap alternative if <2.0x |
| Memory pressure with large batches | Low | Medium | Hard limits (200 records, 300KB) prevent OOM; validate with C.4 test |
| Record boundary detection misses records | Low | High | Unit test with boundary-map verification; test truncated files; ASAN validation |
| Python object references escaping Rayon threads | Low | High | Type system enforces: only RustFile/CursorBackend in Rayon; PythonFile stays sequential |
| Backpressure causes unexpected blocking | Low | Medium | Bounded channel tuning (§7.1); monitor channel contention; document in API |
| Panic in Rayon task kills whole reader | Low | Medium | Panic hook catches in H.4b; error propagates to consumer via Result channel |
| SmallVec <4KB assumption invalid for edge cases | Low | Low | Test with 10KB records; heap allocation is fallback (no panic guaranteed) |

---

## 10. Success Metrics

**Phase C Success:**
- Speedup ≥1.8x (2-thread concurrent read)
- GIL acquisition frequency reduced by 100x
- Zero memory leaks or bounds violations

**Phase H Success:**
- Speedup ≥2.5x (4-thread concurrent read, RustFile)
- CursorBackend enables in-memory batch processing
- Type detection handles all input types correctly

**Overall Success:**
- pymrrc approaches Rust parallelism efficiency
- API remains stable and backwards-compatible
- Documentation covers threading model and performance tuning

---

## 11. Next Steps: Issue Creation in bd (Beads)

This document is **ready for decomposition into actionable work items**. Use the following to create epics and tasks:

### Epic: Phase C - Batch Reading & GIL Optimization
```bash
bd create "Phase C: Batch Reading Implementation" \
  -t epic -p 1 \
  --json
# Use returned epic ID for subtasks below

# Subtasks (numbered C.0 through C.Gate per §6):
bd create "C.0: Data Structure & State Machine" \
  --parent <epic-id> -t task -p 1 --json

bd create "C.1: Implement read_batch() Method" \
  --parent <epic-id> -t task -p 1 --deps discovered-from:C.0 --json

# ... (continue for C.2, C.3, C.4, C.Gate)
```

### Epic: Phase H - Pure Rust I/O & Parallelism
```bash
bd create "Phase H: Pure Rust I/O & Rayon Parallelism" \
  -t epic -p 1 \
  --json
# Subtasks: H.0, H.1, H.2, H.2b, H.3, H.4a, H.4b, H.4c, H.5, H.Gate
```

### Supporting Issues
- Issue for diagnostic tooling setup (§7)
- Issue for benchmark fixture generation
- Issue for ASAN/Valgrind CI integration
- Issue for documentation updates (API, threading model)

**See AGENTS.md for bd command reference and best practices.**

---

**Document Status:** Ready for epic decomposition and implementation
**Date:** January 2, 2026 (Final Revision - Implementation Ready)

**Key Improvements from V1:**
- Explicit GIL contracts and capacity limit behavior
- Decision tree for batch size acceptance with 3-tier outcomes (≥1.8x pass / 1.5-1.8x investigate / <1.5x fail)
- Rayon concurrency model with backpressure diagram
- Comprehensive profiling & diagnostic strategy (§7) with platform-specific tools
- Malformed record test cases & formal coverage targets
- Panic safety and Send/Sync validation details
- Risk register with specific mitigations
- Clear next steps for bd issue creation

**Final Round Improvements (V2 → Final):**
- **Benchmark Baselines Clarified:** Cascade strategy showing Phase B baseline → Phase C vs B → Phase H vs B measurement strategy
- **Acceptance Criteria Fixed:** Corrected logical error in §7 decision points (impossible range removed); now 3-tier: Pass (≥1.8x), Investigate (1.5-1.8x), Fail (<1.5x)
- **Unknown Type Handling:** Specified fail-fast strategy with descriptive TypeError; no silent fallback (§4.1 & H.1)
- **Memory Safety Platform Coverage:** Added macOS Leaks/Instruments as ASAN alternative; documented PyO3 false positives; clarified suppressions strategy
- **GIL Release Verification:** Assigned diagnostic tests to C.0 task; specified locations (tests/, scripts/); detailed test specifications for 3 diagnostic utilities
- **Rollback/Revert Strategy:** Added §6.5 with git revert procedure, Phase B preservation guarantee, data parity check, decision tree for post-revert actions
- **H.1 Type Detection Tests:** Expanded to 10 test cases (8 types + unknown type error + documentation completeness)
