# MRRC Documentation Index

Welcome to the MRRC (MARC Rust Crate) documentation. This directory contains comprehensive guides, design documents, benchmarking results, and project history.

## Quick Navigation

### 📚 User Guides & Tutorials

- **[Parallel Processing Guide](./parallel_processing.md)** - How to use concurrent processing with MRRC for batch MARC file processing
- **[Benchmarking Guide](./BENCHMARKING.md)** - Performance measurement and testing methodology
- **[Threading Documentation](./threading.md)** - Multi-threaded usage patterns and GIL behavior

### 🏗️ Design & Architecture

**Current Design Documents**: See [`design/`](./design/)
- Architectural decisions and proposals
- Feature design documentation
- Implementation strategies
- For active/proposed features

**Key Design Areas**:
- Python wrapper implementation strategy
- Parallel benchmarking suite design
- API and module structure

### 📖 Project History

**Historical Documents**: See [`history/`](./history/)
- Code review audits and findings
- Completed design documents and proposals
- Implementation notes and decision logs
- Module-specific design rationales

**Key Historical Content**:
- Complete code review audit suite (December 2025)
- API refactoring history and decisions
- Original MARC port planning
- Field query DSL design and implementation
- Authority record support implementation

### 📊 Benchmarking Results

**Latest Performance Data**: See [`benchmarks/`](./benchmarks/)
- [Complete Results](./benchmarks/RESULTS.md) - Detailed performance metrics
  - Sequential vs parallel processing
  - pymrrc vs pymarc vs pure Rust
  - Throughput analysis
  - Memory usage profiles
- [README](./benchmarks/README.md) - Benchmarking methodology and frameworks

## Document Organization

```
docs/
├── README.md                       # This file
├── parallel_processing.md          # Parallel/concurrent processing guide
├── threading.md                    # Threading patterns and GIL behavior
├── BENCHMARKING.md                 # Benchmark methodology
│
├── design/                         # Active design documents
│   ├── README.md                   # Design document index
│   ├── PYTHON_WRAPPER_PROPOSAL.md  # PyO3 wrapper strategy
│   └── [other active designs]
│
├── history/                        # Archived design & project history
│   ├── README.md                   # History index and code review summary
│   ├── PYMARC_RUST_PORT_PLAN.md   # Original project plan
│   ├── FIELD_QUERY_DSL.md         # Field query language design
│   ├── AUTHORITY_RECORD_DESIGN.md # Authority records implementation
│   └── [audits, reviews, completed proposals]
│
└── benchmarks/                     # Performance benchmarks
    ├── README.md                   # Benchmarking frameworks
    └── RESULTS.md                  # Latest performance data
```

## Finding What You Need

### I want to...

**...understand how the project is structured**
→ Start with `history/README.md` for code review findings and project quality metrics

**...implement a new feature**
→ See `design/README.md` for design document templates and active work areas

**...understand performance characteristics**
→ Check `benchmarks/RESULTS.md` for throughput, memory, and parallel speedup data

**...contribute to the project**
→ Read `CONTRIBUTING.md` (at root) which references `design/` and `history/` as needed

**...understand why a decision was made**
→ Look in `history/` for the original design proposal and decision rationale

**...optimize concurrent workloads**
→ See `parallel_processing.md` for current state and expected improvements

**...set up benchmarking**
→ See `benchmarks/README.md` for framework setup and running benchmarks

## Key Documentation Areas

### Architecture & Design Decisions

| Document | Type | Status |
|----------|------|--------|
| [PYTHON_WRAPPER_PROPOSAL.md](./design/PYTHON_WRAPPER_PROPOSAL.md) | Design | 📝 In Progress |
| [PYMARC_RUST_PORT_PLAN.md](./history/PYMARC_RUST_PORT_PLAN.md) | History | ✅ Completed |
| [FIELD_QUERY_DSL.md](./history/FIELD_QUERY_DSL.md) | History | ✅ Completed |

### Code Quality & Review

All completed December 28, 2025 (Epic mrrc-aw5):

- [Code Review Summary](./history/CODE_REVIEW_SUMMARY.md) - 10 comprehensive audits, EXCELLENT rating
- [Rust Idiomaticity](./history/RUST_IDIOMATICITY_AUDIT.md) - Style and patterns
- [API Consistency](./history/API_CONSISTENCY_AUDIT.md) - Public API review
- [Core Implementations](./history/CORE_DUPLICATION_AUDIT.md) - Record/Field analysis
- [Format Conversion](./history/FORMAT_CONVERSION_AUDIT.md) - JSON/XML/MARCJSON/CSV
- [I/O Modules](./history/IO_MODULES_AUDIT.md) - Reader/Writer robustness

### Performance & Benchmarking

- [Parallel Processing Guide](./parallel_processing.md) - Threading/multiprocessing usage
- [Benchmark Results](./benchmarks/RESULTS.md) - Complete performance data
- [Benchmark Framework](./benchmarks/README.md) - How benchmarks are structured

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

**Move documents from `design/` to `history/`** when:
- Design is finalized and implementation begins
- Feature is completed
- Decision has been made and work is underway
- Document becomes reference material rather than active planning

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

## Contributing Documentation

When adding new documentation:

1. **For new designs/proposals**: Add to `design/` with status indicator
2. **For completed features**: Move design doc to `history/` with completion notes
3. **For audits/reviews**: Add to `history/` with timestamp and findings
4. **Update this index** when adding major documents

## Latest Updates

**December 31, 2025**:
- Added `parallel_processing.md` - Phase 2 & 3 parallel benchmarking documentation
- Updated `benchmarks/RESULTS.md` with parallel performance findings
- Moved `design/` and `history/` into `docs/` for cleaner root structure
- Created this comprehensive documentation index

**December 28, 2025**:
- Completed comprehensive code review audit suite (Epic mrrc-aw5)
- 10 audit documents in `history/`
- Overall assessment: EXCELLENT, 0 critical issues

---

**Last Updated**: 2025-12-31  
**Next Review**: When major features complete or architectural changes occur
