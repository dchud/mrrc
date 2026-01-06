# MRRC Architecture

This document describes the architecture of the MRRC library and key design decisions.

## Overview

MRRC is a Rust library for reading, writing, and manipulating MARC bibliographic records with Python bindings via PyO3. The library is organized into three main components:

1. **Core Rust Library** - Pure Rust MARC record parsing and manipulation
2. **Python Wrapper** - PyO3 bindings providing Python access with GIL release for concurrency
3. **Benchmarking Infrastructure** - Comprehensive performance testing and profiling

## Core Architecture

### Record Types

MRRC supports three MARC record types:

- **Bibliographic Records** - Standard library catalog records (type 'a', 'c', 'm', etc.)
- **Authority Records** - Subject headings and name authority data (type 'z')
- **Holdings Records** - Physical item location and enumeration data (types 'x', 'y', 'v', 'u')

All record types share common infrastructure through the `MarcRecord` trait.

### Data Structure

A MARC record consists of:

1. **Leader** - 24-byte header with metadata
2. **Control Fields** (000-009) - Fixed-length fields
3. **Data Fields** (010+) - Variable-length fields with indicators and subfields

### Parser Architecture

The core parser uses a state machine approach:

```
ISO 2709 Binary Format
    ↓
Record Boundary Scanner (finds 0x1D terminators)
    ↓
Leader Parser (24 bytes)
    ↓
Directory Parser (field offsets and lengths)
    ↓
Field Parser (control fields vs data fields)
    ↓
Subfield Parser (for data fields with indicators)
    ↓
Character Decoder (MARC-8 or UTF-8)
    ↓
Record Object
```

## Python Wrapper Architecture

### GIL Release Strategy

The Python wrapper implements a three-phase pattern for GIL management:

```
Phase 1 (GIL held):
  Read record bytes from source
  - Python file object: via Python read() method
  - File path: via Rust std::fs::File (no GIL!)
  - Bytes: already in memory (no I/O)

Phase 2 (GIL released):
  Parse record bytes to MARC structure
  - cpu_isolate() macro releases GIL
  - py.detach() in PyO3 terms
  - SmallVec<[u8; 4096]> buffer avoids borrow violations
  - ParseError created without GIL

Phase 3 (GIL re-acquired):
  Convert result to Python object
  - ParseError → PyErr (GIL required)
  - MarcRecord → PyRecord (GIL required)
  - Return to caller
```

### Why GIL Release?

The Python GIL (Global Interpreter Lock) serializes all Python bytecode execution. When multiple threads try to process MARC data:

**Without GIL Release (Old)**:
```
Thread 1: Read bytes → Parse (blocked by GIL) → Convert
Thread 2: Waiting for GIL...
Result: No parallelism, threads serialize around GIL
```

**With GIL Release (New)**:
```
Thread 1: Read bytes (hold GIL) → Parse (release GIL) → Convert (re-acquire GIL)
Thread 2: Read bytes (hold GIL) → Parse (release GIL) → Convert (re-acquire GIL)
Result: Threads can parse in parallel, GIL only held briefly
```

The parsing phase is CPU-intensive and doesn't need Python objects, so releasing the GIL here enables true parallelism.

### ReaderBackend Enum (Phase H)

The unified reader supports multiple input types via a backend enum:

```rust
enum ReaderBackend {
    RustFile(std::fs::File),           // Pure Rust I/O, zero GIL
    Cursor(io::Cursor<Vec<u8>>),       // In-memory, zero GIL
    PythonFile(PyObject),               // Python file object, GIL managed
}
```

**Advantages**:
- **Automatic Detection**: Input type determined at construction time
- **Optimal Performance**: Each backend uses the fastest available method
- **Backward Compatible**: Python file objects still work via GIL management
- **Zero-GIL Paths**: File paths and bytes bypass Python entirely

### BufferedMarcReader (Phase C)

The batch reader reduces GIL contention:

```
Without batching (N records):
  FOR i = 1 to N:
    Acquire GIL → Read 1 record → Release GIL → Parse
    Result: N GIL acquisitions

With batching (N records, batch size = 100):
  FOR batch = 1 to N/100:
    Acquire GIL → Read 100 records → Release GIL
    FOR record in batch:
      Parse record → Serve from queue
  Result: N/100 GIL acquisitions
```

**Implementation**:
- Queue-based state machine: CHECK_QUEUE → CHECK_EOF → READ_BATCH
- VecDeque buffer holds up to 100 pre-read records
- Reduces GIL overhead by ~99% for typical workloads

### SmallVec Optimization

MARC records vary in size (typically 500 - 4000 bytes). The SmallVec buffer:

```rust
SmallVec<[u8; 4096]>
```

**Benefits**:
- Inline storage for ~85-90% of records (no allocation)
- Dynamic heap allocation for oversized records
- <3% memory overhead vs single-threaded
- Eliminates borrow checker issues (owned data for Phase 2 closure)

## Error Handling

### ParseError Enum

Custom error type allows error creation without GIL:

```rust
pub enum ParseError {
    InvalidRecord(String),
    InvalidLeader(String),
    InvalidDirectory(String),
    EncodingError(String),
}

impl From<ParseError> for PyErr {
    // Conversion happens after GIL re-acquisition in Phase 3
}
```

**Why Custom Error Type?**
- PyErr requires GIL to create
- ParseError can be created during Phase 2 (GIL released)
- Defers PyErr creation to Phase 3 (after GIL re-acquired)

## Thread Safety

### Not Send/Sync by Design

The readers are intentionally **not** Send or Sync:

```rust
// Readers hold Python::GIL reference (not Send/Sync)
pub struct PyMARCReader {
    reader: Option<ReaderType>,
    // ReaderType contains PythonFile(PyObject) which is !Send
}
```

**Why?**
- Each thread needs its own GIL-aware reader
- Sharing readers across threads causes undefined behavior
- Forces correct usage pattern: one reader per thread

## Concurrency Model

### Recommended Pattern

```python
from concurrent.futures import ThreadPoolExecutor
from mrrc import MARCReader

def process_file(filename):
    with open(filename, 'rb') as f:
        reader = MARCReader(f)  # New reader per thread
        for record in reader:
            process(record)

files = ['file1.mrc', 'file2.mrc', 'file3.mrc', 'file4.mrc']
with ThreadPoolExecutor(max_workers=4) as executor:
    futures = [executor.submit(process_file, f) for f in files]
    results = [f.result() for f in futures]
```

**Performance**:
- 2 threads: 2.04x speedup
- 4 threads: 3.20x speedup
- 8 threads: ~4.5x speedup (diminishing returns)

### Why ThreadPoolExecutor?

- **Correct**: Creates separate reader per thread
- **Simple**: No manual thread management
- **Efficient**: Thread reuse reduces startup overhead
- **Production-Ready**: Handles exceptions and cleanup

## Performance Characteristics

### Throughput (Records/Second)

| Mode | Throughput | Notes |
|------|-----------|-------|
| Sequential (1 thread) | 113,100 rec/s | Baseline |
| Parallel (2 threads) | ~230,000 rec/s | ~2.04x speedup |
| Parallel (4 threads) | ~360,000 rec/s | ~3.20x speedup |

### Memory Usage

| Scenario | Memory | Notes |
|----------|--------|-------|
| Per reader | ~4 KB | SmallVec buffer |
| Per record (memory) | ~4 KB | Typical MARC record |
| Overhead (4 readers) | ~16 KB | Negligible impact |

### GIL Contention

| Phase | GIL Status | Duration | Notes |
|-------|-----------|----------|-------|
| Phase 1 | Held | Short | Read bytes only |
| Phase 2 | Released | Long | Parsing (CPU-bound) |
| Phase 3 | Held | Short | Convert to Python |

**Batching Benefit**: With BatchedMarcReader, Phase 1 is amortized over 100 records:
- 1 Phase 1 GIL acquisition → Read 100 records
- 100 Phase 2 GIL releases (concurrent)
- 100 Phase 3 GIL re-acquisitions (brief)

## Character Encoding

### MARC-8 Support

MARC-8 is a legacy encoding with:
- Basic Latin (ASCII)
- ANSEL Extended Latin with diacritical marks
- Greek, Cyrillic, Arabic, Hebrew scripts
- East Asian support (Chinese, Japanese, Korean)
- Combining characters with Unicode NFC normalization

### UTF-8 Support

Modern MARC records use UTF-8 (detected from leader position 9).

### Automatic Detection

Character set detected from MARC leader:
- Position 9: ' ' = MARC-8, 'a' = UTF-8
- Decoder selected automatically
- Invalid bytes produce errors with context

## Format Conversions

### Supported Formats

- **JSON**: Generic field-based representation
- **MARCJSON**: Standard JSON-LD format (LOC spec)
- **XML**: Field/subfield XML structure
- **CSV**: Tabular export for spreadsheets
- **Dublin Core**: Simplified 15-element metadata
- **MODS**: Metadata Object Description Schema

### Conversion Approach

Each format has:
1. **Serializer**: Record → Format bytes
2. **Deserializer**: Format bytes → Record
3. **Round-trip tests**: Ensure lossless conversion

## Testing

### Test Categories

1. **Unit Tests**: Individual components (parsers, builders, queries)
2. **Integration Tests**: End-to-end workflows (read → process → write)
3. **Compatibility Tests**: pymarc compatibility validation (75+ tests)
4. **Performance Tests**: Benchmarking with Criterion.rs and pytest-benchmark
5. **Encoding Tests**: MARC-8, UTF-8, multilingual records

### Test Fixtures

Located in `tests/data/`:
- `simple_book.mrc` - Basic bibliographic record
- `multi_records.mrc` - Multiple records in one file
- `simple_authority.mrc` - Authority record
- `simple_holdings.mrc` - Holdings record
- `with_control_fields.mrc` - Record with 008 field

### Benchmark Fixtures

Located in `tests/data/fixtures/`:
- `1k_records.mrc` (257 KB) - Quick benchmarks
- `10k_records.mrc` (2.5 MB) - Standard benchmarks
- `100k_records.mrc` (25 MB) - Comprehensive (local-only)

## Key Design Principles

1. **Rust-Idiomatic**: Uses iterators, Result types, ownership patterns
2. **Zero-Copy Where Possible**: Efficient memory usage for large workloads
3. **Format Flexibility**: Multiple serialization formats out of box
4. **Compatibility**: Maintains data fidelity with pymarc
5. **Performance**: Concurrent I/O with intelligent batching
6. **Safety**: GIL release without unsafe code (except PyO3 glue)

## Future Enhancements

### Phase I: Authority/Holdings Reader Integration
- Refactor Authority/Holdings readers to use ReaderBackend enum
- Enable parallelism benefits for specialized record types
- Timeline: ~1-2 days

### Phase J: Advanced Optimizations
- Adaptive buffer sizing based on record patterns
- Zero-copy field access in Python
- Memory-mapped file support for very large datasets

### Phase K: Web Integration
- REST API for MARC processing
- WebAssembly bindings for browsers
- Streaming JSON output for large result sets

## References

- **GIL Release Plan**: `docs/design/GIL_RELEASE_IMPLEMENTATION_PLAN.md`
- **Benchmarking Guide**: `docs/BENCHMARKING.md`
- **Performance Guide**: `docs/PERFORMANCE.md`
- **Threading Guide**: `docs/threading.md`
- **MARC Standard**: https://www.loc.gov/marc/
- **ISO 2709**: https://en.wikipedia.org/wiki/MARC_standards
