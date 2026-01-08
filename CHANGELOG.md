# Changelog

All notable changes to MRRC (MARC Rust Crate) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-01-08

### Added

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

### Planned for 0.4.0 (Priority 2, Documentation & Optimization)
- **API Compatibility Review** (mrrc-5mn): Identify additional pymarc API parity opportunities
- **Concurrency Documentation** (mrrc-9wc): Add comprehensive Rust concurrency guidance alongside Python patterns
- **Example Code Review** (mrrc-s8h): Expand and improve examples for all recommended usage modes
- **Performance Optimization Epic** (mrrc-u33): Comprehensive profiling and optimization across all implementations
  - Profile pure Rust single-threaded (mrrc-u33.2)
  - Profile pure Rust concurrent with rayon (mrrc-u33.3)
  - Profile Python wrapper single-threaded (mrrc-u33.4)
  - Profile Python wrapper ProducerConsumerPipeline concurrent (mrrc-u33.5)

### Planned for 0.5.0+ (Priority 3, Optional Enhancements)
- **API Standardization** (mrrc-jwb): Code review enhancements including:
  - Format conversion API naming standardization (mrrc-jwb.1)
  - Dublin Core XML convenience function (mrrc-jwb.2)
  - Query DSL and ValidationFramework documentation (mrrc-jwb.3)
  - Shared test helpers module (mrrc-jwb.4)
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
