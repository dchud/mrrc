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

### GIL Release Strategy: Three-Phase Model

The Python wrapper implements a three-phase pattern for GIL management during every `read_record()` call:

```
Phase 1: Read bytes (GIL held)
   ↓
Phase 2: Parse bytes (GIL RELEASED) ← Concurrent work happens here
   ↓
Phase 3: Convert to Python object (GIL re-acquired)
```

**Phase 1 (GIL held):**
- Acquire raw record bytes from source
- Python file object: via Python `read()` method
- File path: via Rust `std::fs::File` (no GIL overhead)
- Bytes: already in memory (no I/O)
- Duration: Very short (I/O cached in kernel)

**Phase 2 (GIL released):** 
- Parse record bytes to MARC structure (CPU-intensive work)
- Uses `py.detach()` (PyO3 0.23+) to explicitly release GIL
- Creates Rust `ParseError` without Python objects
- SmallVec buffer handles most records inline
- Duration: ~90% of total parse time
- **Result: Multiple threads can parse concurrently**

**Phase 3 (GIL re-acquired):**
- Convert Rust `ParseError` to Python exception (if needed)
- Convert Rust `Record` to Python `PyRecord`
- Return to caller
- Duration: Negligible (quick object construction)

### Why GIL Release Matters

The Python GIL (Global Interpreter Lock) serializes all Python bytecode execution. Without GIL release during parsing:

**Without GIL Release (current state of pure pymarc)**:
```
Thread 1: Read bytes (GIL) → Parse (GIL) → Convert (GIL)
Thread 2: Waiting... → Waiting... → Waiting...
Result: Threading provides no speedup (1.0x)
```

**With GIL Release (pymrrc)**:
```
Thread 1: Read (GIL) → Parse (GIL RELEASED) → Convert
Thread 2:                Read (GIL) → Parse (GIL RELEASED) → Convert
Result: Threads parse in parallel (3.74x on 4 cores)
```

The key insight: parsing is CPU-intensive but doesn't need Python objects, so releasing the GIL enables true parallelism.

**Single-threaded benefit:** Even without multiple threads, Rust parsing is simply faster (~4x vs pymarc).

**Multi-threaded benefit:** With explicit `ThreadPoolExecutor`, the GIL release enables concurrent parsing across threads (additional 3.74x speedup on 4 cores).

### ReaderBackend Enum

The unified reader supports multiple input types via a backend enum:

```rust
enum ReaderBackend {
    RustFile(std::fs::File),        // Pure Rust I/O, zero GIL
    Cursor(io::Cursor<Vec<u8>>),    // In-memory, zero GIL
    PythonFile(PyObject),            // Python file object, GIL managed
}
```

**Advantages**:
- **Automatic Detection**: Input type determined at construction
- **Optimal Performance**: Each backend uses fastest available method
- **Backward Compatible**: Python file objects still work via GIL management
- **Zero-GIL Paths**: File paths and bytes bypass Python entirely

**Performance Impact**:
- File path: Pure Rust I/O, Phase 1 has minimal GIL hold
- Bytes: Zero I/O, Phase 1 is trivial
- File object: Requires GIL for `.read()`, but Phase 2 still releases it

### Batched Reader (Optimization)

For Python file objects, batching reduces GIL contention:

```
Without batching (N records):
  FOR i = 1 to N:
    Acquire GIL → Read 1 record → Release GIL → Parse

With batching (N records, batch size = 100):
  FOR batch = 1 to N/100:
    Acquire GIL → Read 100 records → Release GIL
    FOR record in batch:
      Parse record (GIL released)
```

Result: N/100 GIL acquisitions instead of N.

### SmallVec Optimization

MARC records vary in size (typically 500-4000 bytes). The SmallVec buffer:

```rust
SmallVec<[u8; 4096]>
```

**Benefits**:
- Inline storage for ~85-90% of records (no allocation)
- Dynamic heap allocation for oversized records
- <3% memory overhead
- Eliminates borrow checker issues in Phase 2

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
- Defers PyErr conversion to Phase 3 (after GIL re-acquired)

## Thread Safety

### Not Send/Sync by Design

The readers are intentionally **not** Send or Sync:

```rust
// Readers hold Python references (not Send/Sync)
pub struct PyMARCReader {
    reader: Option<ReaderType>,
    // ReaderType may contain PythonFile(PyObject) which is !Send
}
```

**Why?**
- Each thread needs its own GIL-aware reader
- Sharing readers across threads causes undefined behavior
- Forces correct usage pattern: one reader per thread

## Concurrency Model

### Two APIs for Different Use Cases

#### Standard MARCReader (Sequential - No Multi-Threading Benefit)

```python
from mrrc import MARCReader

# Simple sequential reading
reader = MARCReader("records.mrc")
for record in reader:
    process(record)
```

**Performance:**
- ✅ Single-threaded: **~4x faster than pymarc**
- ❌ Multi-threaded: **0.85x slowdown** (GIL contention)
- **Use when:** Sequential processing or single-file reads

#### ProducerConsumerPipeline (High-Performance Single-File Multi-Threading)

```python
from mrrc import ProducerConsumerPipeline

# Background producer thread reads file and parses with Rayon
pipeline = ProducerConsumerPipeline.from_file('large_file.mrc')

for record in pipeline:
    process(record)
```

**Verified Performance:**
- 2 threads: 2.0x speedup
- 4 threads: 3.74x speedup
- Scales with CPU core count

**How it works:**
- Background producer thread reads file in 512 KB chunks
- Bounded channel provides backpressure (1000 records)
- Rayon parses batches in parallel on all CPU cores
- Producer runs without GIL, eliminating contention

**Use when:** Processing a single large MARC file with maximum throughput from available cores

## Performance Characteristics

### Throughput (Records/Second)

| Mode | Throughput | Notes |
|------|-----------|-------|
| Sequential (1 thread) | 549,500 rec/s | Baseline |
| Parallel (2 threads) | ~1.1M rec/s | ~2.0x speedup |
| Parallel (4 threads) | ~2.0M rec/s | ~3.74x speedup |

### Memory Usage

| Scenario | Memory | Notes |
|----------|--------|-------|
| Per reader | ~4 KB | SmallVec buffer |
| Per record (memory) | ~4 KB | Typical MARC record |
| Overhead (4 readers) | ~16 KB | Negligible |

### GIL Contention

| Phase | GIL Status | Duration | Notes |
|-------|-----------|----------|-------|
| Phase 1 | Held | Short | Read bytes only |
| Phase 2 | Released | Long | Parsing (CPU-bound) |
| Phase 3 | Held | Short | Convert to Python |

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
5. **Performance**: Concurrent I/O with intelligent GIL management
6. **Safety**: GIL release without unsafe code (except PyO3 glue)

## References

**Performance & Benchmarking:**
- **Performance Guide**: `docs/PERFORMANCE.md` - Usage patterns and tuning
- **Benchmarking Results**: `docs/benchmarks/RESULTS.md` - Detailed performance data with four-way comparisons (mrrc, pymrrc single-threaded, pymrrc multi-threaded, pymarc)
- **Performance FAQ**: `docs/benchmarks/FAQ.md` - Quick Q&A about speedups

**Guides:**
- **Threading Guide**: `docs/THREADING.md`

**External References:**
- **MARC Standard**: https://www.loc.gov/marc/
- **ISO 2709**: https://en.wikipedia.org/wiki/MARC_standards
- **PyO3 Documentation**: https://pyo3.rs/
