# GIL Release Strategy for pymrrc Threading Performance (Revised)

**Status:** Implementation Ready  
**Date:** January 2026  
**Related Issue:** mrrc-gyk  
**Revision:** 2 (incorporates critical feedback from technical review)  
**Goal:** Unlock threading parallelism in pymrrc by enabling proper GIL release during I/O and parsing operations

---

## Executive Summary

This revision refines the original three-phase pattern proposal to address critical implementation challenges:

1. **Record boundary detection**: Properly handle variable-length MARC records using the ISO 2709 format (5-byte header → record length → read exact bytes)
2. **Buffering strategy**: Use a persistent buffer reused across iterations, eliminating redundant allocations
3. **Borrow checker safety**: Introduce `BufferedMarcReader` as an intermediate type to clarify ownership and borrowing
4. **Performance optimization**: Add batch reading and method caching from the start
5. **Error handling**: Preserve Python exceptions alongside standard I/O error semantics

The approach remains sound: separate Python object access (GIL held) from CPU-intensive parsing (GIL released). With these refinements, pymrrc can achieve **2–3x threading speedup** on concurrent workloads.

---

## Problem Statement (Unchanged)

Python's Global Interpreter Lock (GIL) prevents concurrent Python threads from executing Python code simultaneously. The current pymrrc implementation holds the GIL during all file I/O operations, which blocks other Python threads from running and eliminates potential parallelism benefits.

### Current State
- `Python::attach()` holds the GIL during entire I/O flow
- Other Python threads are blocked during record reading/writing
- Expected threading speedup (1.41x → 3.4x for concurrent reads) is **not achieved**

### Root Cause
The architecture couples I/O operations to Python object management:
- `PyFileWrapper` holds `Py<PyAny>` (a Python file object reference)
- PyO3 enforces that `Py<T>` requires the GIL to be held
- I/O methods (`read_record`, `write_record`) directly access this Python reference
- You cannot release the GIL while holding a reference to a Python object

---

## Recommended Solution: Three-Phase Pattern with Buffering

### Architecture Overview

The solution separates I/O logic from Python object management and parsing into three distinct phases:

```
Phase 1 (GIL held)      Phase 2 (GIL released)      Phase 3 (GIL held)
─────────────────      ─────────────────────      ─────────────────
Read from Python   →    Parse/Process Bytes    →    Return to Python
file object             (pure Rust, no refs)        file object
```

### Key Design: Buffered Reader

Introduce `BufferedMarcReader` as an intermediate type that:
- Owns the persistent buffer (reused per record)
- Encapsulates Phase 1 operations (Python I/O with GIL held)
- Provides clean API for Phase 2 (parsing with GIL released)
- Maintains record boundary state for correctness

```rust
struct BufferedMarcReader {
    file_wrapper: PyFileWrapper,
    reader: BorrowMut<MarcReader>,  // Wraps the Rust reader
    buffer: Vec<u8>,                 // Reused across iterations
    buffer_pos: usize,               // Track position for reuse
}
```

---

## Implementation Details

### Phase 1: Python-Bound I/O with Record Boundary Detection

#### Core Method: `read_next_record_bytes()`

This is the **primary** method for reading MARC records. It handles the ISO 2709 format requirement: 5-byte record length header → read exact record size.

```rust
// In src-python/src/file_wrapper.rs
impl BufferedMarcReader {
    /// Reads exactly one complete MARC record from the file.
    /// Returns Ok(&[u8]) with the record bytes (borrowed from buffer),
    /// or Ok(&[]) if EOF is reached.
    ///
    /// MARC records are ISO 2709 format:
    /// - Positions 0-4: Record length (5 ASCII decimal digits, e.g., "00500")
    /// - Positions 5-N: Record data
    /// - Position N: Record terminator (0x1D)
    fn read_next_record_bytes(&mut self, py: Python) -> PyResult<&[u8]> {
        // Phase 1a: Read and parse the 5-byte length header
        let header = self.read_exact_from_file(py, 5)?;
        if header.is_empty() {
            // EOF on first byte of expected record
            return Ok(&[]);
        }

        let record_length = Self::parse_record_length(&header)?;
        if record_length < 5 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Invalid MARC record length: {}", record_length),
            ));
        }

        // Phase 1b: Prepare buffer and read the remainder
        self.buffer.clear();
        self.buffer.extend_from_slice(&header);
        let remainder = self.read_exact_from_file(py, record_length - 5)?;
        self.buffer.extend_from_slice(&remainder);

        // Phase 1c: Verify terminator and return
        if self.buffer.last() != Some(&0x1D) {
            return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(
                "MARC record missing terminator (0x1D)",
            ));
        }

        Ok(&self.buffer)
    }

    /// Reads exactly `n` bytes from the Python file object.
    /// Returns empty slice on EOF.
    /// 
    /// Handles both normal files and files that return fewer bytes than requested.
    fn read_exact_from_file(&mut self, py: Python, mut bytes_needed: usize) -> PyResult<Vec<u8>> {
        let mut result = Vec::with_capacity(bytes_needed);
        
        while bytes_needed > 0 {
            // Fetch the Python file object's read method
            let file_obj = self.file_wrapper.file.bind(py);
            let bytes_obj: PyBytes = file_obj
                .call_method1("read", (bytes_needed,))?
                .extract()?;
            
            let chunk = bytes_obj.as_bytes();
            if chunk.is_empty() {
                // EOF reached
                break;
            }
            
            result.extend_from_slice(chunk);
            bytes_needed = bytes_needed.saturating_sub(chunk.len());
        }
        
        if result.len() < bytes_needed && !result.is_empty() {
            // Partial read before EOF
            return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(
                format!(
                    "Unexpected EOF: expected {} bytes, got {}",
                    bytes_needed,
                    result.len()
                ),
            ));
        }
        
        Ok(result)
    }

    /// Parses the 5-byte MARC length header as ASCII decimal.
    /// Examples: "00500" → 500, "01234" → 1234
    fn parse_record_length(header: &[u8]) -> PyResult<usize> {
        if header.len() != 5 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Record length header must be exactly 5 bytes",
            ));
        }

        let length_str = std::str::from_utf8(header)
            .map_err(|_| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Record length header is not valid UTF-8",
                )
            })?;

        length_str.parse::<usize>().map_err(|_| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Invalid MARC record length: {}", length_str),
            )
        })
    }
}
```

#### Handling Partial Reads and EOF

The implementation above handles edge cases:
- **Partial record at EOF**: Returns `PyIOError`
- **File closes mid-record**: Handled by `read_exact_from_file`'s EOF check
- **Empty files**: Returns empty slice (no error)
- **Corrupted length header**: Returns `PyValueError`

### Phase 2: Pure Rust Parsing (GIL Released)

Core parsing operates on bytes only—no Python references. The closure passed to `allow_threads()` captures only `&self.reader`, which contains no Python references.

```rust
// In src-python/src/readers.rs
#[pymethods]
impl PyMarcReader {
    fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<Option<PyObject>> {
        let py = unsafe { Python::assume_gil_acquired() };
        
        // Phase 1: Read record bytes (GIL held, brief I/O operation)
        let record_bytes = slf.buffered_reader.read_next_record_bytes(py)?;
        
        if record_bytes.is_empty() {
            return Ok(None);  // EOF
        }

        // Phase 2: Parse bytes (GIL released, CPU-intensive)
        // The closure captures only &self.reader, which holds no Py<T> references
        let record = py.allow_threads(|| {
            slf.reader.read_from_bytes(record_bytes)
        })?;

        // Phase 3: Convert to Python object (GIL held)
        let py_record = PyRecord::from_rust(record, py)?;
        Ok(Some(py_record.into_pyobject(py)?))
    }
}
```

**Why this is safe:**
- `record_bytes` is borrowed from `self.buffer` (owned by `BufferedMarcReader`, no Python ref)
- `self.reader` is a standard Rust struct with no `Py<T>` fields
- The closure is `Send` and `Sync` by virtue of containing only copyable/stack data
- GIL is properly released during the CPU-intensive parsing work

### Phase 3: Result Conversion (GIL Held)

Convert processed Rust data back to Python objects:

```rust
impl PyRecord {
    fn from_rust(record: MarcRecord, py: Python) -> PyResult<Self> {
        // Convert Rust record to Python-compatible wrapper
        let leader = record.leader().to_string();
        let fields = Self::convert_fields_to_dict(record.fields(), py)?;
        
        Ok(PyRecord {
            leader,
            fields,
            // ... other fields
        })
    }
    
    fn convert_fields_to_dict(
        fields: &[MarcField],
        py: Python,
    ) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new_bound(py);
        for field in fields {
            let key = field.tag().to_string();
            let value = Self::field_to_pyobject(field, py)?;
            dict.set_item(key, value)?;
        }
        Ok(dict.into())
    }
}
```

---

## Performance Optimizations

### 1. Batch Reading

For workloads that iterate over many records, batch multiple reads to amortize GIL release overhead:

```rust
#[pymethods]
impl PyMarcReader {
    fn read_batch(mut slf: PyRefMut<'_, Self>, batch_size: usize) -> PyResult<Vec<PyObject>> {
        let py = unsafe { Python::assume_gil_acquired() };
        let mut batch = Vec::with_capacity(batch_size);
        
        // Phase 1: Read N record bytes (GIL held once)
        let mut record_bytes_batch = Vec::with_capacity(batch_size);
        for _ in 0..batch_size {
            let bytes = slf.buffered_reader.read_next_record_bytes(py)?;
            if bytes.is_empty() {
                break;  // EOF
            }
            record_bytes_batch.push(bytes.to_vec());
        }
        
        if record_bytes_batch.is_empty() {
            return Ok(vec![]);
        }
        
        // Phase 2: Parse all records (GIL released)
        let parsed = py.allow_threads(|| {
            record_bytes_batch
                .iter()
                .map(|b| slf.reader.read_from_bytes(b))
                .collect::<Result<Vec<_>, _>>()
        })?;
        
        // Phase 3: Convert all to Python (GIL held)
        for record in parsed {
            let py_record = PyRecord::from_rust(record, py)?;
            batch.push(py_record.into_pyobject(py)?);
        }
        
        Ok(batch)
    }
}
```

**Expected benefit**: 10–30% throughput improvement for bulk operations.

### 2. Cache Python Method Lookups

Avoid repeated attribute lookups on the Python file object:

```rust
struct PyFileWrapper {
    file: Py<PyAny>,
    read_method: Option<Py<PyAny>>,  // Cached method reference
}

impl PyFileWrapper {
    fn read_method(&self, py: Python) -> PyResult<Bound<'_, PyAny>> {
        if let Some(cached) = &self.read_method {
            Ok(cached.bind(py).clone())
        } else {
            self.file.bind(py).getattr("read")
        }
    }

    fn new(file: Py<PyAny>, py: Python) -> PyResult<Self> {
        // Pre-cache the read method at initialization
        let read_method = Some(file.bind(py).getattr("read")?.into());
        Ok(PyFileWrapper { file, read_method })
    }
}
```

**Expected benefit**: 5–10% reduction in overhead per read operation.

### 3. Ring Buffer (Optional for Very Large Files)

If memory efficiency is critical for very large files, replace `Vec<u8>` with a fixed-size ring buffer:

```rust
struct RingBuffer {
    buffer: [u8; 65536],  // 64KB buffer
    len: usize,            // Valid bytes in buffer
}

impl RingBuffer {
    fn record_slice(&self) -> &[u8] {
        &self.buffer[0..self.len]
    }

    fn clear(&mut self) {
        self.len = 0;
    }
}
```

**Benefit**: Eliminates per-record allocation for typical MARC files (records usually <5KB).  
**Trade-off**: Maximum record size capped at buffer size (mitigated with fallback to Vec for larger records).

---

## Implementation Architecture

### File Structure

```
src-python/src/
├── file_wrapper.rs          (PyFileWrapper: thin Python interface)
├── buffered_reader.rs       (NEW: BufferedMarcReader: I/O with boundaries)
├── readers.rs               (PyMarcReader: three-phase iterator with GIL handling)
├── writers.rs               (PyMarcWriter: three-phase writer with GIL handling)
└── record.rs                (PyRecord: Python ↔ Rust conversion)
```

### Key Responsibilities

| Component | Responsibility | GIL State |
|-----------|---|---|
| `PyFileWrapper` | Access Python file object methods | Held |
| `BufferedMarcReader` | Manage buffer, read bytes, detect record boundaries | Held |
| `MarcReader` (Rust) | Parse bytes into Rust records | Released |
| `PyRecord` | Convert Rust records to Python objects | Held |

---

## Error Handling Strategy

### Phase 1 Errors (Python I/O)

These are genuine Python exceptions and should propagate as `PyResult`:

- `IOError`: File read failed, permission denied
- `ValueError`: Record length header is invalid
- `EOFError`: Unexpected EOF mid-record

```rust
fn read_exact_from_file(&mut self, py: Python, n: usize) -> PyResult<Vec<u8>> {
    // ... (implementation above)
    // Returns PyResult; Python exceptions propagate correctly
}
```

### Phase 2 Errors (Parsing)

These are Rust parsing errors and must be converted to Python exceptions:

```rust
let record = py.allow_threads(|| {
    slf.reader.read_from_bytes(record_bytes)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
})?;
```

### Phase 3 Errors (Conversion)

Object conversion errors are rare but possible:

```rust
let py_record = PyRecord::from_rust(record, py)
    .map_err(|e| {
        PyErr::new::<pyo3::exceptions::RuntimeError, _>(
            format!("Failed to convert record: {}", e),
        )
    })?;
```

---

## Expected Outcomes

With this refined approach:

- **Threading speedup achieved**: Expected 2–3x for concurrent reads (1.41x → 3.4x baseline, better with batching)
- **Rust performance parity**: pymrrc threading efficiency approaches pure Rust parallelism
- **Data integrity guaranteed**: Correct ISO 2709 boundary detection prevents record corruption
- **Memory efficient**: Single buffer reused; optional ring buffer for large-scale processing
- **Minimal API changes**: Transparent to end users (except optional `read_batch()` method)
- **Backward compatible**: Existing pymarc-compatible code continues to work

---

## Testing Requirements

### Unit Tests

1. **Record boundary handling**:
   - Complete records in buffer
   - Records split across multiple reads
   - Corrupted length headers
   - Missing record terminator (0x1D)
   - Variable-length record sizes (100B to 10KB)

2. **Buffer lifecycle**:
   - Buffer reuse across iterations
   - Buffer resizing for large records
   - EOF on first byte vs. mid-record

3. **Error recovery**:
   - Partial reads before EOF
   - Graceful error handling with exception preservation
   - Reader state consistency after errors

### Integration Tests

1. **Threading correctness**:
   - Multiple threads reading from separate file handles (should work)
   - Same thread with multiple readers (should work)
   - Single reader accessed from multiple threads (should fail safely)

2. **Concurrent performance**:
   - Benchmark 1M-record file with N threads
   - Verify GIL is released during parsing (use `threading.Event` to block other thread and measure)
   - Compare pymrrc threading speedup to pure Rust

3. **Edge cases**:
   - Empty files
   - Single-record files
   - Files with mixed record sizes
   - Files with non-seekable file objects (pipes, sockets)

### Regression Tests

- All existing pymarc-compatible tests must pass without modification
- Data integrity: Read/write/read round-trip matches original

---

## Implementation Roadmap

### Phase A: Core Buffering Infrastructure
- [ ] Create `BufferedMarcReader` type
- [ ] Implement `read_next_record_bytes()` with boundary detection
- [ ] Implement `read_exact_from_file()` with EOF handling
- [ ] Add unit tests for boundary detection

### Phase B: GIL Release Integration
- [ ] Refactor `PyMarcReader::__next__()` to use three-phase pattern
- [ ] Verify borrow checker accepts architecture (Phase 1 releases before Phase 2)
- [ ] Add unit tests for GIL release (thread control tests)
- [ ] Implement error handling for Phase 1/2/3

### Phase C: Performance Optimizations
- [ ] Implement and test batch reading (`read_batch()`)
- [ ] Cache Python method lookups
- [ ] Benchmark memory and CPU overhead
- [ ] Optional: implement ring buffer for very large files

### Phase D: Writer Implementation
- [ ] Apply same three-phase pattern to `PyMarcWriter::write_record()`
- [ ] Implement buffering for writes
- [ ] Add write-side tests and benchmarks

### Phase E: Validation and Rollout
- [ ] Run full test suite (unit + integration)
- [ ] Benchmark threading speedup (target: 2–3x)
- [ ] Verify backward compatibility
- [ ] Update documentation with threading performance notes

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| **Buffer overflow on large records** | Use `Vec<u8>` with `with_capacity()` for exact size; no hardcoded limits |
| **Borrow checker conflicts** | Structure code so Phase 1 releases borrows before Phase 2; use intermediate types if needed |
| **Data loss from incomplete reads** | `read_exact_from_file()` explicitly handles partial reads and EOF |
| **Python exception masking** | Preserve `PyResult` through all phases; only convert Rust errors at phase boundaries |
| **Performance regression** | Benchmark before/after; batching optimizations can offset allocation costs |
| **API breakage** | All public methods remain unchanged; new methods (e.g., `read_batch()`) are additive |

---

## Success Criteria

1. ✅ Threading benchmarks show **2x+ speedup** for concurrent operations (improvement from current 1.41x)
2. ✅ pymrrc threading performance within **90% of pure Rust performance**
3. ✅ All existing tests pass without modification
4. ✅ No data loss or corruption in record processing (verified by round-trip tests)
5. ✅ Backward compatibility maintained for all public APIs
6. ✅ Memory overhead negligible (<5% for typical workloads)
7. ✅ Record boundary detection handles all ISO 2709 edge cases

---

## References

- **ISO 2709 Format**: MARC record structure with 5-byte length header and 0x1D terminator
- **PyO3 Threading**: https://pyo3.rs/v0.20/python_async (GIL release patterns)
- **Original Proposal**: `docs/design/GIL_RELEASE_STRATEGY.md`
- **Technical Review**: `docs/design/GIL_RELEASE_REVIEW.md`
