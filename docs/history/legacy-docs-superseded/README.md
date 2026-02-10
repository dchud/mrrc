# MRRC Documentation Index

Welcome to the MRRC (MARC Rust Crate) documentation. This directory contains comprehensive guides, design documents, benchmarking results, and project history.

## Quick Navigation

### 📚 User Guides & Tutorials

**Getting Started:**
- **[Migration Guide](./MIGRATION_GUIDE.md)** - Migrate from pymarc to mrrc (drop-in replacement)
- **[Installation Guide](./INSTALLATION_GUIDE.md)** - Feature flags, optional dependencies, platform support
- **[Format Selection Guide](./FORMAT_SELECTION_GUIDE.md)** - Decision tree for choosing serialization formats

**Language Tutorials:**
- **[Python Tutorial](./PYTHON_TUTORIAL.md)** - Read, write, and convert MARC records in Python
- **[Rust Tutorial](./RUST_TUTORIAL.md)** - Reader/Writer traits, custom implementations

**Advanced Topics:**
- **[Streaming Guide](./STREAMING_GUIDE.md)** - Large file handling and memory optimization
- **[Query DSL Guide](./QUERY_DSL.md)** - Advanced field searching with indicators and patterns
- **[Performance Guide](./PERFORMANCE.md)** - GIL release, optimization, and tuning
- **[Threading Documentation](./THREADING.md)** - Thread safety and multi-threaded patterns
- **[Architecture](./ARCHITECTURE.md)** - System design, GIL management, and implementation details

### 📊 Benchmarking Results

**Performance Data**: See [`benchmarks/`](./benchmarks/)
- [Complete Results](./benchmarks/RESULTS.md) - Detailed performance metrics
  - Single-threaded vs multi-threaded performance
  - pymrrc vs pymarc vs pure Rust comparison
  - Throughput analysis and memory usage profiles
- [README](./benchmarks/README.md) - Benchmarking methodology and frameworks

### 🏗️ Design & Architecture

**Current Design Documents**: See [`design/`](./design/)
- Architectural decisions and proposals
- Active feature design documentation
- Implementation strategies

**Key Design Areas**:
- Python wrapper implementation
- Parallel processing and benchmarking
- API and module structure

### 📖 Project History

**Historical Documents**: See [`history/`](./history/)
- Code review audits and findings (December 2025)
- Completed design documents and proposals
- Implementation notes and decision rationale
- Module-specific design documentation

## Document Organization

```
docs/
├── README.md                       # This file (documentation index)
├── ARCHITECTURE.md                 # System architecture and design
├── MIGRATION_GUIDE.md              # pymarc → mrrc migration
├── INSTALLATION_GUIDE.md           # Feature flags and dependencies
├── FORMAT_SELECTION_GUIDE.md       # Choosing serialization formats
├── PYTHON_TUTORIAL.md              # Python read/write/convert examples
├── RUST_TUTORIAL.md                # Rust Reader/Writer trait usage
├── STREAMING_GUIDE.md              # Large file and memory optimization
├── PERFORMANCE.md                  # Performance analysis and optimization
├── THREADING.md                    # Thread safety and patterns
├── QUERY_DSL.md                    # Advanced field searching
│
├── design/                         # Active design documents
│   └── ...
│
├── history/                        # Archived design & project history
│   └── README.md                   # History index and grouping
│
└── benchmarks/                     # Performance benchmarks
    ├── README.md                   # Benchmarking frameworks
    └── RESULTS.md                  # Latest performance data
```

## Finding What You Need

### I want to...

**...search for fields by indicators, tag ranges, or patterns**
→ Read `QUERY_DSL.md` for the Query DSL guide with examples

**...understand how the project is structured**
→ Start with `ARCHITECTURE.md` for system design, then `history/README.md` for code review findings

**...implement a new feature**
→ See `design/README.md` for design document templates and active work areas

**...understand performance characteristics**
→ Check `benchmarks/RESULTS.md` for throughput, memory, and parallel speedup data

**...understand threading and concurrency**
→ Read `THREADING.md` for patterns, gotchas, debugging, and best practices

**...understand why a decision was made**
→ Look in `history/` for the original design proposal and decision rationale

**...optimize my code**
→ See `PERFORMANCE.md` for GIL release details, thread count tuning, and backend strategy

**...set up benchmarking**
→ See `benchmarks/README.md` for framework setup and running benchmarks

## Key Documentation Areas

### Getting Started

| Document | Type | Purpose |
|----------|------|---------|
| [MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md) | Guide | Migrate from pymarc to mrrc |
| [INSTALLATION_GUIDE.md](./INSTALLATION_GUIDE.md) | Guide | Feature flags, optional dependencies |
| [FORMAT_SELECTION_GUIDE.md](./FORMAT_SELECTION_GUIDE.md) | Guide | Decision tree for format selection |

### Language Tutorials

| Document | Type | Purpose |
|----------|------|---------|
| [PYTHON_TUTORIAL.md](./PYTHON_TUTORIAL.md) | Tutorial | Python read/write/convert examples |
| [RUST_TUTORIAL.md](./RUST_TUTORIAL.md) | Tutorial | Rust Reader/Writer trait usage |

### Architecture & Performance

| Document | Type | Purpose |
|----------|------|---------|
| [ARCHITECTURE.md](./ARCHITECTURE.md) | Reference | System design, components, GIL management |
| [STREAMING_GUIDE.md](./STREAMING_GUIDE.md) | Guide | Large file handling, memory optimization |
| [QUERY_DSL.md](./QUERY_DSL.md) | Guide | Advanced field searching with Query DSL |
| [PERFORMANCE.md](./PERFORMANCE.md) | Guide | Performance analysis, tuning, backend strategy |
| [THREADING.md](./THREADING.md) | Guide | Thread safety, patterns, and debugging |

### Benchmarking & Results

| Document | Type | Purpose |
|----------|------|---------|
| [benchmarks/RESULTS.md](./benchmarks/RESULTS.md) | Results | Performance metrics, 3-tier comparison |
| [benchmarks/README.md](./benchmarks/README.md) | Guide | Benchmark framework and running tests |

### Design Documents

- See [`design/README.md`](./design/README.md) for active design work
- See [`history/README.md`](./history/README.md) for completed designs and decisions

## What's in Design vs History?

**`design/`** contains:
- Active proposals and RFCs
- In-progress design documents
- Documents for features being planned or implemented
- Proposals awaiting decision

**`history/`** contains:
- Completed design documents
- Code review and audit reports
- Archived proposals and their outcomes
- Implementation notes and decisions made
- Project historical context

## Key Features by Document

### ARCHITECTURE.md

Covers:
- Core Rust library structure
- Python wrapper GIL management (3-phase pattern)
- Reader backend strategy (RustFile, PythonFile, Cursor)
- Thread safety and concurrency model
- Character encoding support
- Format conversions
- Testing infrastructure

### PERFORMANCE.md

Covers:
- Single-thread baseline (549k rec/s)
- Multi-thread speedup (2.0x with 2 threads, 3.74x with 4 threads)
- How GIL release works (3-phase pattern)
- Backend strategy (file paths vs file objects)
- Usage patterns and optimization
- Troubleshooting and profiling

### THREADING.md

Covers:
- GIL behavior and MRRC's GIL release policy
- ProducerConsumerPipeline (single-file high-throughput multi-threading)
- ThreadPoolExecutor (multi-file concurrent processing)
- Multiprocessing (CPU-bound work)
- Thread safety guarantees and gotchas
- Memory usage with threading
- Common patterns and debugging
- Performance characteristics

## Benchmarking Results Summary

From `benchmarks/RESULTS.md`:

**Single-Threaded Performance**:
- Rust (mrrc): ~1,000,000 rec/s
- Python (pymrrc): ~300,000 rec/s (~4x faster than pymarc)
- Pure Python (pymarc): ~70,000 rec/s

**Multi-Threaded Performance**:
- 2 threads: ~2x speedup
- 4 threads: ~3-4x speedup
- Linear scaling with CPU core count

**vs pymarc**: ~4x faster for single-threaded operations

## Reference Documentation

### Main Project Files

- [../README.md](../README.md) - Project overview and quick start
- [../CONTRIBUTING.md](../CONTRIBUTING.md) - Contribution guidelines
- [../AGENTS.md](../AGENTS.md) - Development workflow and CI

### External References

- [MARC 21 Standard](https://www.loc.gov/marc/) - Official MARC specification
- [ISO 2709](https://en.wikipedia.org/wiki/MARC_standards) - Binary format spec
- [pymarc Documentation](https://pymarc.readthedocs.io/) - Reference implementation
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) - Idiomatic patterns
- [PyO3 Documentation](https://pyo3.rs/) - Python wrapper framework

## Contributing Documentation

When adding new documentation:

1. **For new designs/proposals**: Add to `design/` with status indicator
2. **For completed features**: Move design doc to `history/` with completion notes
3. **For audits/reviews**: Add to `history/` with timestamp and findings
4. **Update this index** when adding major documents

---

**Last Updated**: 2026-01-22
**Next Review**: When major features complete or architectural changes occur
