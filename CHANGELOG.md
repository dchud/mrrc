# Changelog

All notable changes to MRRC (MARC Rust Crate) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

#### Field Insertion Order Preservation (2026-01-14)
- **Record fields now preserve insertion order** instead of sorting by tag
  - Uses `IndexMap` for field storage in `Record`, `AuthorityRecord`, and `HoldingsRecord`
  - Enables round-trip fidelity: serialization and deserialization now preserve original field order
  - Field iteration via `fields()` and `control_fields_iter()` returns fields in insertion order
  - **Breaking change for code assuming tag-sorted order**: Use `fields_by_tag()` for explicit tag-based access
- **Unplanned changes during implementation:**
  - `fields_in_range()` now iterates and filters (was using `BTreeMap::range()`)
  - `remove_fields_by_tag()` now uses `shift_remove()` to preserve order of remaining fields
- **Performance impact**: ~17-22% regression in record parsing benchmarks (acceptable trade-off for round-trip fidelity)
  - Baseline: 9.5ms for 10k records → After: 11.3ms for 10k records
  - Still significantly faster than pymarc (7.5x+ improvement maintained)

#### Performance Optimizations (2026-01-19)
- **SmallVec for Subfield Storage** (mrrc-u33.6):
  - Replaced `Vec<Subfield>` with `SmallVec<[Subfield; 4]>` for subfield arrays
  - Targets common case: 85-90% of real-world MARC records have ≤4 subfields per field
  - Performance gain: **+4.6% roundtrip throughput** (read+write cycle)
  - Zero-copy inline storage for typical records, automatic heap spillover for large records
  - Maintains API compatibility - transparent to users
- **Parse Digits Optimization**:
  - Eliminated string allocations in `parse_digits()` parser combinator
  - Reduced numeric field parsing overhead with direct byte-based validation
  - Contributes to overall **+6.0% combined optimization gain** (measured across parse + serialize pipeline)

### Added

#### Format Research & Support Strategy (2026-01-19, mrrc-fks)
- **Comprehensive Format Evaluation Framework**:
   - Completed analysis of Arrow/Parquet columnar storage (mrrc-ks7)
   - Integrated Polars, Arrow, and DuckDB evaluation findings (mrrc-fks.10)
   - Binary format comparison matrix updated with MessagePack, CBOR, and Avro benchmarks
   - All evaluations converted to Rust-native implementations (removed Python dependencies)
- **Format Support Strategy Document** (docs/design/format-research/FORMAT_SUPPORT_STRATEGY.md):
   - Decisiveness-focused analysis: clear recommendations vs exploratory notes
   - Format support strategy: Core formats (ISO 2709, Protobuf) plus feature-gated formats (Arrow, FlatBuffers, MessagePack)
   - Integration guidance for Arrow columnar analytics tier
   - Performance targets and trade-off analysis for each tier
- **Research Documentation**:
   - Comprehensive README for format-research directory
   - Structured evaluation results with comparative benchmarks
   - Migration guides for users adopting new format support
   - Analysis of Arrow Analytics tier for bulk data operations

#### Format Support Implementation Planning (2026-01-20, mrrc-d4g)
- **Implementation Epic & Issue Planning**:
   - Created mrrc-d4g epic: "Format Support Implementation (Tiers 1-2)" with 63 total issues
   - Structured as Phase 0-4 with hierarchical sub-epics matching implementation roadmap
   - Phase 0: Foundation (4 tasks + 2 cleanup tasks)
   - Phase 1: Core Formats (1A: ISO 2709 refactoring + 1B: Protobuf implementation)
   - Phase 2: High-Value Formats (2A: Arrow + 2B: FlatBuffers + 2C: MessagePack)
   - Phase 4: Python Wrapper & Documentation (4A: PyO3 bindings + 4B: Format modules + 4C: Documentation)
- **Evaluation Artifact Consolidation Strategy** (Part 8 of FORMAT_SUPPORT_STRATEGY.md):
   - Mapped all evaluation artifacts (code, docs, data) to production use or archival
   - Identified reusable components from evaluation phase (protobuf.rs, test fixtures, benchmark patterns)
   - Documented refactoring needs (arrow_impl.rs, flatbuffers_impl.rs module structure)
   - Cleanup tasks integrated into mrrc-d4g phases to minimize blocking
   - Archive strategy: Move docs/design/format-research/ → docs/history/format-research/ in Phase 4C
- **Implementation Workflow Documentation**:
   - Detailed Phase 0-4 cleanup workflows with task dependencies
   - Migration checklist for consolidating evaluation code into production test infrastructure
   - Archive index template for docs/history/format-research/README.md
   - Clear separation of completed evaluation project from ongoing implementation work

### Fixed

- **Example Code Quality**: Resolved clippy warnings in example code for clean builds
- **Code Formatting**: Applied rustfmt to all recent additions for consistency

### Documentation Updates

- **Benchmark Documentation Accuracy** (2026-01-19):
  - Refreshed all benchmark measurements with latest performance data
  - Clarified that reported numbers are post-warm-up (JIT stabilized)
  - Updated comparison baselines to reflect recent optimizations
- **Architecture Documentation**: Enhanced documentation structure in docs/design/

## [0.4.0] - 2026-01-09

### Added

#### Query DSL Python Bindings (2026-01-11)
- **Python Query DSL**: Exposed Rust Query DSL to Python with full feature parity
  - `FieldQuery`: Builder pattern for complex field matching (tag, indicators, subfields)
  - `TagRangeQuery`: Match fields within a tag range (e.g., 600-699 for all subjects)
  - `SubfieldPatternQuery`: Regex matching on subfield values
  - `SubfieldValueQuery`: Exact or partial string matching on subfield values
- **Record Query Methods**: New methods on Record for advanced field searching
  - `fields_by_indicator(tag, indicator1=None, indicator2=None)`: Filter by indicators
  - `fields_in_range(start_tag, end_tag)`: Find fields within a tag range
  - `fields_matching(query)`: Use FieldQuery objects for complex matching
  - `fields_matching_range(query)`: Use TagRangeQuery for range-based matching
  - `fields_matching_pattern(query)`: Regex matching via SubfieldPatternQuery
  - `fields_matching_value(query)`: String matching via SubfieldValueQuery
- **Query DSL Documentation**: Comprehensive guide at docs/QUERY_DSL.md covering:
  - Philosophy: Why multiple query types (performance, clarity)
  - All query types with examples
  - Practical cataloging scenarios (LCSH filtering, ISBN-13, subject analysis)
  - Comparison table vs pymarc's get_fields()
- **Query DSL Tests**: 42 new tests in tests/python/test_query_dsl.py

#### Developer Experience (2026-01-11)
- **Unified Testing Workflow**: Single command (`.cargo/check.sh`) for full pre-push verification (~30s)
  - Runs rustfmt, clippy, documentation, security audit, maturin build, and Python tests
  - Uses pytest marker-based test selection (`-m "not benchmark"`) to run 314 core tests in ~6s
  - Excludes 61 benchmark tests from default run (available via `pytest -m benchmark`)
  - Documented in AGENTS.md with command reference table and CI alignment
- **CI Workflow Fixes**:
  - Fixed ASAN memory-safety workflow: Exclude dev-dependencies (zerocopy) that have nightly Rust incompatibility
  - Fixed coverage workflow: Exclude PyO3 bindings package to avoid Python linker errors in tarpaulin

#### API Standardization (2026-01-12)
- **Format Conversion API Naming**: Standardized across all format modules for consistency
  - Added `record_to_csv()` for single-record CSV export (delegates to `records_to_csv()`)
  - Documented `records_to_csv()` plural pattern: semantically correct for batch-oriented tabular format
  - All format modules now follow: `record_to_format()` (single) + `records_to_format()` (batch where applicable)
- **CSV Export to Python**: Exposed Rust CSV functions to Python bindings
  - `record_to_csv(record)`: Export single record to CSV format
  - `records_to_csv(records)`: Export multiple records to CSV (batch mode)
  - `records_to_csv_filtered(records, filter_fn)`: Export with custom field filtering (filter_fn takes tag string, returns bool)
  - Handles both direct PyRecord and wrapped Record instances for flexibility
  - Completes API consistency with other format converters (JSON, XML, MARCJSON, Dublin Core, MODS)

#### Code Quality & Cleanup (2026-01-09)
- **Memory Safety CI**: Added ASAN (AddressSanitizer) workflow for nightly memory safety checks
   - Runs on schedule (daily) and manual dispatch
   - Validates no memory safety issues in core library
   - Local `.cargo/check.sh --memory-checks` option for developers with nightly toolchain
- **Phase Reference Removal**: Cleaned up implementation-plan-internal references from all user-facing code
   - Removed "Phase A-H" and gate nomenclature from source code comments
   - Removed from Python wrapper module documentation
   - Removed from test file names and test documentation
   - Removed from example code
   - Codebase now approachable to new users unfamiliar with development history
- **Test File Reorganization**:
   - `test_h_gate_benchmarking.py` → `test_parallel_benchmarking.py`
   - `test_h5_integration.py` → `test_integration.py`
   - `test_queue_state_machine_c2.py` → `test_queue_state_machine.py`
   - `test_memory_profiling_c4.py` → `test_memory_profiling.py`

### Previously Added (2026-01-08)

#### Full pymarc API Parity ✅
- **Leader Value Validation Helpers**: Complete MARC 21 leader position reference implementation
  - `Leader.get_valid_values(position)` - Returns dict of valid values for each leader position
  - `Leader.is_valid_value(position, value)` - Validates values per MARC 21 specification
  - `Leader.get_value_description(position, value)` - Gets human-readable descriptions
  - Support for positions 5 (Record Status), 6 (Type of Record), 7 (Bibliographic Level), 17 (Encoding Level), 18 (Cataloging Form)
- **Indicators as Tuple-Like Object**: Full support for pymarc-compatible indicator access
  - `field.indicators[0]` and `field.indicators[1]` indexing
  - `field.indicators` unpacking support
  - Backward compatibility with `field.indicator1` and `field.indicator2` properties
- **Control Field Access Pattern**: Support for pymarc's `record['001'].value` pattern
  - ControlField wrapper with `.value` property for control fields (001-009)
  - Both `record['001'].value` and `record.control_field('001')` patterns work identically
  - Backward compatibility maintained

#### Documentation
- **pymarc API Parity Plan**: Moved completed work to docs/history/ for archival
- **API Compatibility Summary**: Historical record of all 7 API gaps identified and resolved
- **README Enhancement**: Updated to highlight full pymarc API compatibility as primary feature
- **Migration Guide**: Comprehensive guide showing all compatible patterns with minimal migration path

### Changed

- **README**: Removed "nearly" qualifier - now describes "full" pymarc API compatibility
- **Python Test Suite**: Expanded from 88+ to comprehensive coverage of all parity features
- **Documentation Structure**: Organized completed parity work into history directory

### Performance

- No performance regressions - all existing benchmarks maintained
- Leader validation methods are zero-copy (dictionary lookups only)

### Technical Details

- **MARC 21 Reference Data**: All leader position value mappings per MARC 21 specification
- **API Stability**: Python wrapper API now guaranteed stable for pymarc compatibility
- **Backward Compatibility**: All new features fully backward compatible with existing code

## [0.3.1] - 2026-01-07

### Added

#### CI/CD Improvements
- **Python Wheel Build Workflow**: Fixed multi-platform wheel building for Ubuntu, macOS, and Windows
  - Configured maturin to use `-i pythonX.Y` flag for manylinux Python version selection (per maturin docs)
  - Fixed Windows glob expansion in test step using PowerShell conditionals
  - Removed deprecated mypy/pyright strict type checking (to be addressed in future typing effort)
  - Wheels now build and test successfully for Python 3.9, 3.10, 3.11, and 3.12 across all platforms

### Changed

#### Documentation Updates
- **Performance Reference Table** (benchmarks/RESULTS.md): Updated to use pymarc as baseline (1.0x) for clearer speedup comparison
- **Real-World Performance Scenarios**: Standardized table formats across all four scenarios for consistent speedup and time-saved metrics
- **PERFORMANCE.md Executive Summary**: Updated to clarify recommended strategies (ProducerConsumerPipeline for single-file, ThreadPoolExecutor for multi-file)
- **Documentation Audit**: Moved completed audit to docs/history/ for archival

## [0.3.0] - 2026-01-06

### Added

#### GIL Release & Concurrency (Phase A-F)
- **GIL Release during I/O**: Python wrapper now releases GIL during record parsing for true multi-thread parallelism
- **Three-Phase Pattern**: Robust pattern separates Python object access (GIL held) from CPU-intensive parsing (GIL released)
- **Measured Performance**: 2.04x speedup on 2 threads, 3.20x on 4 threads (Phase H benchmarking)
- **BatchedMarcReader**: Queue-based buffering reduces GIL contention from N to N/100 operations
- **SmallVec Optimization**: 4 KB inline buffer avoids allocations for ~85-90% of MARC records

#### Parallel I/O Backend (Phase H)
- **ReaderBackend Enum**: Unified reader supporting multiple input types with automatic selection
  - RustFile: Pure Rust file I/O (zero GIL overhead)
  - CursorBackend: In-memory bytes (zero GIL overhead)
  - PythonFile: Python file objects (GIL-managed)
- **File Path Support**: Direct file path input bypasses Python I/O layer entirely
- **Bytes/Bytearray Support**: In-memory MARC data via CursorBackend
- **Automatic Detection**: Input type automatically detected, optimal backend selected

#### Documentation (Phase G)
- **PERFORMANCE.md**: Comprehensive guide with threading patterns, benchmarking methods, and tuning recommendations
- **Threading Examples**: Practical concurrent_reading.py and concurrent_writing.py examples
- **API Documentation**: Updated PyMarcReader/PyMarcWriter docstrings with threading guidance
- **README Threading Section**: Concrete speedup numbers (2.04x for 2 threads, 3.20x for 4)
- **Benchmark Documentation**: Updated docs/benchmarks/ with Phase H results

### Changed

- **Python Wrapper API**: No breaking changes (fully backward compatible)
- **Performance Profile**: Single-thread throughput stable, multi-thread speedup now available
- **Reader Construction**: Accepts str/Path/bytes/bytearray in addition to file objects

### Fixed

- **GIL Release Bug (Phase B)**: SmallVec copy pattern properly avoids borrow checker violations
- **Error Handling (Phase B)**: ParseError conversion happens after GIL re-acquisition
- **GIL Crossing (Phase C)**: py.detach() correctly releases GIL during Phase 2 parsing

### Performance

- **Threading Speedup**: 2.04x (2 threads), 3.20x (4 threads) vs sequential reading
- **Memory Overhead**: SmallVec buffering <3% overhead vs single-threaded
- **GIL Contention**: Reduced from linear to O(n/100) with BatchedMarcReader
- **File I/O**: Pure Rust backend (file paths) eliminates all GIL overhead

### Technical Details

- **Phase H Integration**: Producer-consumer pipeline with Rayon parallel record scanning
- **Backpressure**: Queue-based buffering prevents runaway producer threads
- **Thread Safety**: Each thread requires own reader instance (not Send/Sync by design)
- **Efficiency**: pymrrc 92% efficient vs pure Rust Rayon baseline

## [0.2.0] - 2025-12-31

### Added

#### Python Integration (Phase 2-3)
- **PyO3 Python Wrapper**: Full Python bindings via PyO3/Maturin
- **Python Parallel Benchmarks**: Threading and multiprocessing performance analysis
- **pymarc Compliance Test Suite**: 75+ tests validating compatibility with pymarc API
- **Memory Usage Profiling**: Benchmarks comparing Python wrapper overhead vs Rust native
- **Context Manager Support**: Python `with` statement support for file I/O

#### Performance & Benchmarking (Phase 1-4)
- **Rust Parallel Benchmarks**: Performance testing with rayon for concurrent MARC processing
- **Rayon Parser Pool (H.4b)**: Parallel batch processing functions via Rayon thread pool
  - `parse_batch_parallel()` - Unlimited parallel parsing with dynamic work distribution
  - `parse_batch_parallel_limited()` - Bounded parallel parsing respecting configured thread pool
  - Record boundary scanning and parallel record assembly
  - Thread-safe batch processing for large MARC files
- **Comprehensive Benchmark Suite**: 
  - 1K/10K/100K record read performance
  - Field access overhead measurements
  - JSON/XML serialization performance
  - Roundtrip (read+write) benchmarks
  - Sequential vs parallel processing comparison
- **Benchmark Results Documentation**: Real measured performance data with pymarc comparisons
- **CI Optimization**: Caching and performance gate integration

#### Documentation & Organization
- **Design Documentation**: Architecture and design decision documentation
- **Benchmarking Documentation**: Feasibility studies and performance analysis
- **Executive Summaries**: High-level overview of parallel processing capabilities
- **Reorganized Docs**: Consolidated design/ and history/ into docs/ hierarchy
- **Examples**: Real-world usage patterns demonstrating library features
- **Migration Guide**: Comprehensive guide highlighting near-100% API compatibility with pymarc

#### API Enhancements
- **Record.to_marc21()**: Convert records back to ISO 2709 binary format
- **Enhanced Python API**: Full feature parity with Rust API in Python wrapper
- **MARCWriter Fixes**: Fixed write() method and improved record serialization

### Fixed
- Fixed deprecated PyO3 type alias warning
- Fixed 20+ clippy linting violations in benchmark files (missing semicolons in closure statements)
- Suppressed benchmark-specific documentation warnings at file level
- Fixed MARCWriter record serialization issues
- Cleaned up CI pipeline to pass all quality gates
- **H.4a Bug Fix**: RecordBoundaryScanner now correctly scans for 0x1D (record terminator) instead of 0x1E (field terminator) per ISO 2709 specification

### Changed
- Improved documentation structure and organization
- Enhanced Python test coverage with comprehensive pymarc compatibility suite

## [0.1.0] - 2025-12-28

### Added

#### Core Features
- **ISO 2709 Binary Format**: Full read/write support for MARC records in the standard binary interchange format
- **Record Types**: Support for three MARC record types:
  - Bibliographic records (standard MARC records)
  - Authority records (Type Z) for standardized headings and cross-references
  - Holdings records (Types x/y/v/u) for item location and enumeration data
- **Builder Pattern API**: Fluent, idiomatic Rust interface for record construction
- **Field Access API**: Comprehensive methods for reading, filtering, and iterating over fields
  - `fields_by_tag()` - Get fields by tag
  - `fields_by_indicator()` - Filter by indicators
  - `fields_in_range()` - Get fields within a tag range
  - `fields_with_subfield()` - Get fields containing specific subfields

#### Advanced Query DSL (Phase 1-3)
- **FieldQuery Builder**: Complex criteria-based field matching (tags, indicators, subfields)
- **TagRangeQuery**: Range-based field lookups (e.g., 600-699)
- **Subfield Pattern Matching**: Regex-based field filtering
- **Linked Field Navigation**: Support for MARC 880 (Alternate Graphical Representation) fields
  - Parse linkage information from subfield 6
  - Bidirectional lookups between original and 880 fields
- **Authority Control Helpers**: Query traits for authority-specific operations
- **Format-Specific Traits**: 
  - `BibliographicQueries` for bibliographic records
  - `AuthorityQueries` for authority records
  - `HoldingsQueries` for holdings records

#### Serialization Formats
- **JSON**: Generic JSON representation with fields as keys
- **MARCJSON**: Standard JSON-LD format for MARC records
- **XML**: XML representation with proper field/subfield structure
- **CSV**: Tabular export format for spreadsheet applications
- **Dublin Core**: Simplified 15-element metadata schema
- **MODS**: Metadata Object Description Schema for detailed descriptions

#### Character Encoding Support
- **MARC-8 (Legacy)**: Full support for legacy MARC-8 encoding with:
  - Basic Latin (ASCII)
  - ANSEL Extended Latin with diacritical marks
  - Hebrew (ESC ) 2)
  - Arabic (ESC ) 3, 4)
  - Cyrillic (ESC ( N)
  - Greek (ESC ( S)
  - Subscripts/Superscripts/Special character sets
  - East Asian (Chinese, Japanese, Korean via EACC)
- **UTF-8 (Modern)**: Full Unicode support for modern MARC records
- **Automatic Detection**: Encoding detection from MARC leader position 9

#### Helper Methods
- **RecordHelpers Trait**: Available on all record types via blanket implementation
  - `title()` - Extract main title
  - `author()` / `authors()` - Extract author names
  - `subjects()` - Extract subject headings
  - `isbns()` - Extract ISBN values
  - `issns()` - Extract ISSN values
  - `publication_info()` - Extract publication details
  - Record type helpers: `is_book()`, `is_music()`, `is_map()`, `is_serial()`, `is_audiovisual()`, `is_electronic_resource()`

#### API Refactoring
- **MarcRecord Trait**: Common interface for all record types (control field operations)
- **GenericRecordBuilder<T>**: Unified builder for all record types
- **FieldCollection Trait**: Standardized field collection management
- Unified field storage pattern across Record, AuthorityRecord, and HoldingsRecord

#### Error Handling & Recovery
- **MarcError Type**: Comprehensive error handling for MARC operations
- **Recovery Mode**: Graceful handling of truncated/malformed MARC records
- **Result Type**: Convenient `Result<T>` alias for library operations

#### Testing
- 282+ comprehensive unit and integration tests
- Test data with:
  - Simple bibliographic records
  - Music scores
  - Records with control fields (008)
  - Multiple records in one file
  - Authority records
  - Holdings records
  - Multilingual records
  - MARC-8 encoded records

#### Documentation
- Comprehensive API documentation with doc comments
- Module-level documentation with examples
- Examples directory with real-world usage patterns:
  - Creating records with builders
  - Reading and querying fields
  - Converting between formats
  - Working with authority and holdings records
  - MARC-8 encoding demonstration
  - Multilingual record handling
  - CSV export

### Design Principles

1. **Rust-Idiomatic**: Leverages iterators, Result types, and ownership patterns naturally
2. **Zero-Copy Where Possible**: Efficient memory usage for large record sets
3. **Format Flexibility**: Support for multiple serialization formats out of the box
4. **Compatibility**: Maintains data fidelity with pymarc and standard MARC tools
5. **Extensible**: Trait-based architecture allows easy addition of new query types and formats

### Known Limitations

None known at this time. The following have been resolved:
- ✓ Field indicator validation with MARC21 semantics (implemented in 0.1.0)
- ✓ MARC-8 combining character handling with Unicode NFC normalization (implemented in 0.1.0)

### Technical Details

- **Minimum Rust Version**: 1.70
- **Dependencies**: serde, serde_json, regex, unicode-normalization
- **License**: MIT

## Future Roadmap

### Completed in [Unreleased] (Priority 2, Performance & Format Research)
- ✅ **Performance Optimization Epic** (mrrc-u33): Comprehensive profiling and optimization across all implementations
  - ✅ Profile pure Rust single-threaded (mrrc-u33.2)
  - ✅ Profile pure Rust concurrent with rayon (mrrc-u33.3)
  - ✅ Profile Python wrapper single-threaded (mrrc-u33.4)
  - ✅ Profile Python wrapper ProducerConsumerPipeline concurrent (mrrc-u33.5)
  - ✅ SmallVec optimization for subfield storage (mrrc-u33.6) - **+4.6% throughput gain**
- ✅ **Format Research & Evaluation** (mrrc-fks epic):
  - ✅ Binary format comparison matrix with MessagePack, CBOR, Avro (mrrc-fks.8)
  - ✅ Format support strategy & recommendations (mrrc-fks.9)
  - ✅ Arrow/Parquet columnar analysis (mrrc-fks.10)

### Planned for 0.5.0 (Priority 2+, Features & Polish)
- **Calendar Versioning Decision** (mrrc-vmk): Evaluate and implement calver scheme (proposal complete, awaiting decision)
- **API Compatibility Review** (mrrc-5mn): Identify additional pymarc API parity opportunities
- **Concurrency Documentation** (mrrc-9wc): Add comprehensive Rust concurrency guidance alongside Python patterns
- **Example Code Review** (mrrc-s8h): Expand and improve examples for all recommended usage modes

### Planned for 0.5.0+ (Priority 3, Optional Enhancements)
- **API Standardization** (mrrc-jwb): Code review enhancements including:
  - Format conversion API naming standardization (mrrc-jwb.1)
  - Dublin Core XML convenience function (mrrc-jwb.2)
  - ~~Query DSL and ValidationFramework documentation (mrrc-jwb.3)~~ ✓ Completed in 0.4.0
  - Shared test helpers module (mrrc-jwb.4)
  - ~~Expose Query DSL to Python wrapper (mrrc-jwb.5)~~ ✓ Completed in 0.4.0
- **Performance Tracking** (mrrc-3od): Codspeed integration for CI performance regression detection

### Long-term Vision (0.5.0+)
- Field-level metadata (subfield constraints, cardinality)
- Streaming reader for large file sets
- Integration with library discovery systems
- Web API support
- Database backends for record storage
- Advanced cataloging workflows
- Machine learning-friendly formats

---

## Project Information

- **Repository**: https://github.com/dchud/mrrc
- **Documentation**: https://docs.rs/mrrc
- **Crates.io**: https://crates.io/crates/mrrc
- **Issue Tracking**: GitHub Issues and Beads issue tracking system
- **Status**: Active development, experimental (APIs may change)
