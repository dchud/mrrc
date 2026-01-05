# Changelog

All notable changes to MRRC (MARC Rust Crate) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

### Planned for 0.3.0 (Priority 1-2)
- **Parallel Processing Enhancements** (mrrc-086): Complete parallel benchmarking suite with rayon and threading optimizations
- **GIL Release in Python I/O** (mrrc-gyk): Enable PyO3's allow_threads for concurrent Python operations without GIL blocking

### Planned for 0.4.0+ (Priority 3, Optional Enhancements)
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
