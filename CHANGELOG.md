# Changelog

All notable changes to MRRC (MARC Rust Crate) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **MODS XML read support**: `mods_to_record(xml_str)` and `mods_collection_to_records(xml_str)` parse MODS XML (single record or `<modsCollection>`) into MARC `Record` objects. Covers `titleInfo`, `name`, `subject`, `originInfo`, `physicalDescription`, `abstract`, `note`, `genre`, `classification`, `location`, `relatedItem`, `recordInfo`, `identifier`, `accessCondition`, `tableOfContents`, `targetAudience`, and `language` elements with conformance tests using LOC-derived fixtures.
- **Cross-compiled Linux wheels** ([mrrc-experiments#16](https://github.com/dchud/mrrc-experiments/issues/16)): Release workflow now builds `aarch64` and `i686` Linux wheels via maturin-action Docker cross-compilation, increasing wheel count from 15 to 25 per release.
- **Field constructor `subfields=` and `indicators=` kwargs** ([mrrc-experiments#15](https://github.com/dchud/mrrc-experiments/issues/15)): `Field` now accepts `subfields=` (list of `Subfield`) and `indicators=` (list/tuple of two strings) as keyword arguments, enabling inline construction matching pymarc style: `Field('245', indicators=['0', '1'], subfields=[Subfield('a', 'Title')])`. The Rust binding already supported this signature; the Python wrapper now passes through both kwargs.
- **Record constructor `fields=` kwarg** ([mrrc-experiments#15](https://github.com/dchud/mrrc-experiments/issues/15)): `Record` now accepts an optional `fields=` keyword argument (list of `Field`), enabling inline record construction: `Record(fields=[Field(...), ...])`. This goes beyond pymarc parity (pymarc's `Record.__init__` does not accept `fields=`) as a UX improvement. `Record()` with no arguments also now works, defaulting to `Leader()`.
- **Constructor kwargs tests**: 8 new tests in `TestConstructorKwargs` covering `indicators=`, `subfields=`, `fields=`, combined usage, full inline construction, and backward compatibility.
- **PEP 561 `py.typed` marker**: Added to root `mrrc/` package so type checkers recognize shipped type stubs.
- **Project layout documentation**: New `docs/contributing/project-layout.md` describes the three-layer architecture (core Rust → PyO3 bindings → Python package), how maturin and the Cargo workspace connect them, directory structure, build commands, and common development workflows.
- **Pre-push git hook**: `.githooks/pre-push` runs `.cargo/check.sh` before every push, preventing code from reaching CI that would fail local checks. Opt-in via `git config core.hooksPath .githooks`.
- **`check.sh --quick` flag**: Skips docs, audit, and maturin build for faster inner-loop iteration (keeps fmt, clippy, tests, ruff).

### Changed

- **GitHub releases now include changelog notes**: `python-release.yml` auto-extracts the relevant CHANGELOG.md section and includes it in the GitHub Release body. Backfilled v0.7.1 release notes.
- **Pymarc compatibility tests fully enabled**: Removed 10 `pytest.skip` guards from `test_pymarc_compatibility.py` — all features (to_json, to_xml, to_marcjson, to_dublin_core, to_marc21, writer, roundtrip, test data) were already implemented. 88/88 tests now pass with 0 skipped.
- **Documentation updated for inline construction**: Quickstart, API reference, migration guide, writing tutorial, and examples now show `Field(subfields=..., indicators=...)` and `Record(fields=...)` as the primary construction pattern, with `add_subfield()`/`add_field()` as the incremental alternative. Migration guide updated to reflect `Record()` no longer requires an explicit `Leader` argument and field creation is now closer to pymarc.
- **Type stubs updated**: `Field.__new__` and `Record.__new__` in `.pyi` now include the new keyword-only parameters.
- **Type stubs enriched**: Merged rich docstrings into `mrrc/_mrrc.pyi` from the old `src-python` stubs — Leader properties with MARC position descriptions, MARCReader GIL/concurrency/thread-safety docs, MARCWriter context manager protocol, BibframeConfig setter methods and property getters, RdfGraph `parse()`/`triples()`/`__len__()`. Added `mods_to_record` and `mods_collection_to_records` stubs.
- **Removed stale `src-python/python/` directory**: Deleted redundant Python package directory (1,235 lines) left over from the Phase 5 layout migration. Root `mrrc/` is now the sole Python package location.
- **Test suite audit**: Deleted 5 redundant/inert files from `src-python/tests/`, migrated 4 test files to `tests/python/` (backend parity, type detection, rayon parser pool, record boundary scanner), extracted 1 pipeline regression test. Deleted `src-python/tests/` entirely. Python test count: 364 → 433.
- **Ruff lint compliance**: Fixed 76 ruff errors across `mrrc/` and `tests/python/` (bare except, unused imports/variables, walrus operator patterns, f-string placeholders, true/false comparisons). Added ruff check step to `check.sh`.
- **Rust integration tests in check.sh**: Changed `cargo test --lib` to `cargo test --lib --tests`, adding 17 integration test files (bibframe, mods, field query, concurrent GIL, etc.) to the local CI script.
- **CI workflow alignment with check.sh**: Closed gaps where CI was missing checks that check.sh runs locally. Added clippy on mrrc-python to `lint.yml` (was only linting mrrc core). Added ruff Python lint job to `lint.yml` (via `uvx`). Fixed rustdoc scope from `--package mrrc` to `--all` to include PyO3 bindings. Changed `test.yml` from `--lib --bins` to `--lib --tests --package mrrc`, adding 15+ integration test files that were skipped in CI. Removed unused mypy/pyright install from `python-build.yml` test-wheels job. Upgraded `actions/cache` from v3 to v4 across all workflows.
- **CI efficiency improvements**: Switched all `pip install` calls to `uv pip install --system` across `python-build.yml`, `benchmark-python.yml`, and `python-release.yml` for faster dependency resolution. Consolidated three separate cargo cache actions (registry, index, build) into single multi-path caches across five workflows. Switched cargo-audit and cargo-tarpaulin from `cargo install` (compiles from source) to pre-built binaries via `taiki-e/install-action`. Combined two tarpaulin runs into one (`--out Xml Html`). Upgraded `codecov/codecov-action` from v3 to v4, `actions/setup-python` from v4 to v5. Removed redundant `pip install maturin` and single-element Rust version matrix from `build.yml`.

### Fixed

- **Rust formatting**: Applied `cargo fmt` to `src/mods.rs` and `src-python/src/formats.rs` (pre-existing match arm brace style).
- **Clippy doc lint**: Escaped `OCoLC` in `tests/mods_conformance_tests.rs` doc comment to satisfy `clippy::doc_markdown`.
- **Flaky CI benchmark**: `test_bytesio_vs_file_isolation` switched from mean to median for I/O overhead measurement, preventing single-iteration CI runner spikes from failing the test. Removed hard assertion (test value is diagnostic, not a correctness gate).

## [0.7.1] - 2026-02-10

### Changed

- **BREAKING: Removed 7 experimental serialization formats** ([mrrc-experiments#19](https://github.com/dchud/mrrc-experiments/pull/19)): Deleted protobuf, arrow, parquet, flatbuffers, messagepack, cbor, and avro support (~15,100 lines across 78 files). These added significant build complexity and dependency weight without proven adoption. Only ISO 2709 and BIBFRAME remain.
- **BIBFRAME promoted to core**: BIBFRAME conversion is now always-compiled rather than feature-gated (`format-bibframe`), reflecting its importance as the LOC linked data format. Dependencies `oxrdfio` and `oxrdf` are now non-optional.
- **Python bindings simplified**: Removed format-specific classes (ProtobufReader, ArrowWriter, etc.), `mrrc/analytics.py`, and format submodules. Simplified `read()`/`write()` helpers and `__init__.py` exports.
- **Clean repo history**: Recreated repository as `dchud/mrrc` with a clean two-commit history, replacing `dchud/mrrc-experiments`. Build artifacts, large test fixtures, and accumulated cruft from the experimental phase are no longer in git history.
- **Test fixtures**: Removed 25MB `100k_records.mrc` from repo; gitignored to prevent re-commit. Regenerable via `scripts/generate_benchmark_fixtures.py`. Benchmark tests depending on 100k fixture removed.
- **Build simplification**: Removed `build.rs` (was only for protobuf/flatbuffers code generation), `src/generated/`, `proto/`, and ~20 dependencies from `Cargo.toml`.
- **Local CI script**: Updated `.cargo/check.sh` to use `uv run` for maturin and pytest steps instead of manual venv activation.
- **100k fixture scrub**: Removed remaining references to `100k_records.mrc` from scripts, benches, CI workflow names, and public docs (benchmarks, contributing, tests README). Fixture generation script no longer produces the 100k file.
- **Developer docs standardized on uv**: Replaced manual venv activation and bare `maturin develop`/`pytest` commands with `uv sync`/`uv run` equivalents across installation, development setup, testing, and migration guides.
- **Release procedure updated**: Refreshed for OIDC trusted publishing (replaces `PYPI_API_TOKEN`), added `installation.md` and `quickstart-rust.md` to version-bump checklist.
- **Rust dependency version in docs**: Updated hardcoded `mrrc = "0.6"` to `"0.7"` in installation and quickstart guides.

### Fixed

- **Flaky benchmark test**: `test_backend_comparison_1k` now uses 5 iterations with median instead of 3 iterations with mean, making it resilient to single-iteration CI runner I/O spikes.
- **CI: Python release workflow**: Added `actions/setup-python` before `maturin-action` to properly set Python version; `python-version` is not a valid input for maturin-action
- **CI: Python release OIDC publishing**: Removed `password:` secret from `pypa/gh-action-pypi-publish` step; action auto-detects OIDC token from `id-token: write` permission
- **CI: Re-enabled ASAN memory-safety workflow**: Upstream Rust issue (rust-lang/rust#144168) that caused zerocopy nightly incompatibility has been resolved
- **CI: ASAN workflow zerocopy_derive fix**: Removed `-Zavoid-dev-deps` flag which caused zerocopy_derive proc-macro resolution failures on nightly
- **CI: CodSpeed benchmark integration**: Re-linked repository with CodSpeed after clean repo creation
- **Beads: Gitignore rotated daemon logs**: Added `daemon-*.log.gz` pattern to `.beads/.gitignore` and removed accidentally committed 2.7MB rotated log file

## [0.7.0] - 2026-02-05

### Added

#### Testing (2026-02-04)
- **pathlib.Path GIL Release Tests**: Added tests verifying that `pathlib.Path` objects achieve the same GIL release behavior as string paths
  - `test_pathlib_path_sequential_baseline`: Baseline sequential reading with Path objects
  - `test_pathlib_path_threading_equivalent_to_str`: Verifies Path and str have equivalent threading speedup (within 30%)
  - `test_pathlib_vs_str_equivalent_performance`: Confirms both use the same RustFile backend
  - Location: `tests/python/test_pathlib_gil_release.py`

### Changed

#### Python Version Support (2026-02-03)
- **Dropped Python 3.9 support**: Python 3.9 reached end-of-life in October 2025
- **Added Python 3.13 support**: Stable release from October 2024
- **Added Python 3.14 support**: Stable release from October 2025
- Updated minimum Python version from 3.9 to 3.10 in pyproject.toml, CI workflows, and documentation
- Note: RHEL 9 users should use `dnf module enable python3.11` or containers for Python 3.10+

### Fixed

#### Security (2026-02-03)
- **bytes crate upgrade**: Upgraded `bytes` from 1.9.0 to 1.11.1 to address RUSTSEC-2026-0007

#### CI/CD Improvements (2026-02-03)
- **CodSpeed Rust Benchmark Integration** (PR #8): Added continuous performance tracking for Rust benchmarks
  - Integrated `codspeed-criterion-compat` v4.3.0 as drop-in replacement for criterion
  - New `benchmark-rust.yml` workflow runs Rust benchmarks via `cargo codspeed`
  - Added CodSpeed badge to README.md
- **Benchmark Workflow Naming Consistency**: Renamed workflows for clarity
  - `python-benchmark.yml` → `benchmark-python.yml` ("Benchmarks: Python")
  - `codspeed.yml` → `benchmark-rust.yml` ("Benchmarks: Rust")
  - Groups benchmark workflows alphabetically, makes target language clear at a glance

#### CI/CD Fixes (2026-02-02)
- **CodSpeed Action v4 Migration**: Fixed benchmark workflow after upgrading from deprecated v1
  - Added required `mode: walltime` parameter for standard CI runners
  - CodSpeed v4 now requires explicit mode selection ('simulation' or 'walltime')

### Documentation

#### Documentation Updates (2026-02-04)
- **Fixed benchmark documentation links**: Updated internal links in `index.md`, `query-dsl.md`, and `architecture.md` to use lowercase filenames
- **Testbed Design Document** (`docs/design/ideas-for-test-projects.md`):
  - Added comprehensive state management design (YAML + SQLite hybrid)
  - Added interaction models section (centralized vs local/private testing)
  - Added repository growth timeline and pruning strategies
  - Added MkDocs documentation structure with Divio-style organization
  - Updated project phases (7 phases including documentation and initial data)
  - Consolidated duplicate sections for cleaner document structure

#### Documentation Improvements (2026-02-02)
- **File Renames and Reorganization**:
  - Renamed `streaming-large-files.md` → `working-with-large-files.md` to better reflect content scope
  - Renamed benchmark docs to lowercase convention: `RESULTS.md` → `results.md`, `FAQ.md` → `faq.md`, `README.md` → `index.md`
  - Updated all cross-references and mkdocs.yml navigation
- **Fixed Broken Links**:
  - Fixed README.md tutorial links that were returning 404s
  - Updated to point to actual tutorial paths (`tutorials/python/reading-records/`, `tutorials/rust/reading-records/`)
- **Formatting Fixes**:
  - Fixed bullet list formatting in `threading-python.md`, `performance-tuning.md`, `results.md`, `migration-from-pymarc.md`
  - Added blank lines after colons before bullet lists (required by some markdown parsers)
- **Content Improvements**:
  - Added uv example to `installation.md` for building from source
  - Added hardware info (2025 MacBook Air M4) to `performance-tuning.md`
  - Added Rust/Python format name columns to bibframe RDF serialization table
  - Added "Same?" column to `migration-from-pymarc.md` API comparison tables for clarity
  - Removed emojis from `migration-from-pymarc.md` section headers
  - Fixed `encoding.md`: removed non-existent `detect_encoding` import, noted `EncodingValidator` is Rust-only
  - Simplified `python-api.md` format note (all formats bundled in wheel, no extra steps)
  - Added feature-gated format installation link to `formats.md`

#### Documentation Reorganization (2026-01-29)
- **Complete restructure using Material for MkDocs**:
  - New tabbed navigation with Getting Started, Tutorials, Guides, Reference, Examples sections
  - Light/dark theme toggle with persistent preference
  - Built-in search with instant results
  - Mobile-responsive layout
- **Getting Started**:
  - `quickstart-python.md`: 5-minute Python quickstart with installation, reading, writing examples
  - `quickstart-rust.md`: 5-minute Rust quickstart with Cargo setup and basic operations
- **Tutorials** (split from monolithic guides):
  - Python tutorials: reading-records, writing-records, querying-fields, format-conversion, concurrency (5 files)
  - Rust tutorials: reading-records, writing-records, querying-fields, format-conversion, concurrency (5 files)
- **Reference Documentation**:
  - `marc-primer.md`: MARC record structure explained for non-librarians (leader, directory, fields, indicators, subfields)
  - `encoding.md`: Character encoding reference (MARC-8, UTF-8, escape sequences, detection)
  - `python-api.md`: Complete Python API reference (Record, Field, MARCReader, formats, BIBFRAME)
  - `rust-api.md`: Complete Rust API reference (types, builders, traits, feature flags)
- **Examples Index**: Organized links to 29 example files by category (Python/Rust)
- **README Reduction**: Streamlined from 897 to 115 lines per documentation best practices
- **Archived Legacy Docs**: Moved superseded files to `docs/history/legacy-docs-superseded/`

## [0.6.0] - 2026-01-29

### Added

#### BIBFRAME Conversion (2026-01-28, updated 2026-01-29)
- **Bidirectional MARC ↔ BIBFRAME conversion** (`format-bibframe` feature):
  - `marc_to_bibframe()`: Convert MARC bibliographic records to BIBFRAME 2.0 RDF graphs
  - `bibframe_to_marc()`: Reverse conversion from BIBFRAME RDF back to MARC records
  - Full implementation of LOC MARC21 to BIBFRAME 2.0 Conversion Specifications
  - Support for all MARC content types: books, serials, music, maps, visual materials
- **Python BIBFRAME Bindings** (2026-01-29):
  - `mrrc.marc_to_bibframe(record, config)`: Python wrapper for MARC→BIBFRAME conversion
  - `mrrc.bibframe_to_marc(graph)`: Python wrapper for BIBFRAME→MARC conversion
  - `mrrc.BibframeConfig`: Configuration class with all options exposed
  - `mrrc.RdfGraph`: RDF graph class with `serialize()` method for all formats
  - `mrrc.RdfFormat`: Enum for output format selection (RdfXml, NTriples, Turtle, JsonLd)
- **Hub/Expression Support** (2026-01-29):
  - MARC 240 (Uniform Title) creates bf:Hub for expression-level grouping
  - Work → hasExpression → Hub → hasInstance → Instance linking pattern
  - Hub → expressionOf → Work reverse relationship
  - Supports translations, versions, and other expression-level variants
- **Item Creation from Holdings** (2026-01-29):
  - MARC 852 (Location) creates bf:Item entities linked to Instance
  - Call number from $h/$i, sublocation from $b, barcode from $p
  - MARC 876-878 (Item Information) support for detailed holdings
  - Multiple Items per Instance with sequential URI generation
- **Classification Support** (2026-01-29):
  - MARC 050 → bf:ClassificationLcc (Library of Congress Classification)
  - MARC 060 → bf:ClassificationNlm (National Library of Medicine)
  - MARC 080 → bf:ClassificationUdc (Universal Decimal Classification)
  - MARC 082 → bf:ClassificationDdc (Dewey Decimal Classification)
  - MARC 084 → bf:Classification (Other schemes with source in $2)
  - classificationPortion ($a) and itemPortion ($b) properties
- **RDF Serialization Formats**:
  - RDF/XML (W3C standard, maximum interoperability)
  - N-Triples (simple line-based triple format)
  - Turtle (human-readable, compact RDF syntax)
  - JSON-LD (modern web standard, JSON representation)
- **Configuration Options** (`BibframeConfig`):
  - Custom base URI for generated resources
  - Output format selection
  - Authority linking control
  - BFLC extension support
- **Comprehensive Test Suite** (111 tests):
  - 48 unit tests for individual MARC→BIBFRAME mappings (including Hub, Item, Classification)
  - 26 validation tests for BIBFRAME 2.0 ontology compliance
  - 16 integration tests with real-world record types
  - 16 round-trip tests documenting acceptable data loss
  - 5 baseline comparison tests against official LOC tool
- **Examples**:
  - Rust: `marc_to_bibframe.rs`, `bibframe_to_marc.rs`, `bibframe_batch.rs`
  - Python: `marc_to_bibframe.py`, `bibframe_roundtrip.py`, `bibframe_config.py`
- **Documentation**:
  - README section with conversion examples
  - API documentation with docstrings
  - Known limitations and data loss documentation

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
  - Still significantly faster than pymarc (~4x improvement maintained)

#### Performance Optimizations (2026-01-19)
- **SmallVec for Subfield Storage**:
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

#### New Binary Format Support (2026-01-22)
- **Apache Arrow Columnar Format** (`format-arrow` feature):
  - `ArrowWriter`: Converts MARC records to Arrow RecordBatch for analytics workflows
  - `ArrowReader`: Reads Arrow IPC files back to MARC records with full fidelity
  - Preserves field sequence and control field data through round-trip conversion
  - Performance: 96% compression ratio, ~865k records/second throughput
  - Enables SQL queries over MARC data via DuckDB and Polars integration
- **FlatBuffers Zero-Copy Format** (`format-flatbuffers` feature):
  - `FlatbuffersWriter`: Serializes MARC records to FlatBuffers binary format
  - `FlatbuffersReader`: Zero-copy deserialization without parsing overhead
  - Ideal for memory-constrained environments and streaming applications
  - Performance: 64% memory savings vs JSON, 1M+ records/second zero-copy access
- **MessagePack Compact Binary** (`format-messagepack` feature):
  - `MessagePackReader`/`MessagePackWriter` for compact binary serialization
  - 25% smaller than JSON with equivalent structure preservation
  - Cross-platform interoperability with 50+ language implementations
  - Performance: ~750k records/second throughput
- **FormatReader/FormatWriter Traits**: Unified interface enabling consistent API across all formats

#### Python Format Bindings (2026-01-22)
- **All Formats Available in Python**: Arrow, FlatBuffers, MessagePack, and Protobuf readers/writers
- **Format-Agnostic Helpers**:
  - `mrrc.read(path_or_data, format=None)`: Auto-detect format from file extension or content
  - `mrrc.write(records, path, format=None)`: Write records to any supported format
- **Type Stubs** (`.pyi` files): Full IDE autocompletion and type checking support
- **`mrrc/formats/` Package**: Organized modules for format-specific operations
  - `mrrc.formats.marc`: ISO 2709 reading/writing
  - `mrrc.formats.protobuf`: Protobuf serialization with schema evolution
  - `mrrc.formats.arrow`: Arrow/Parquet operations with analytics integration
  - `mrrc.formats.flatbuffers`: FlatBuffers zero-copy access
  - `mrrc.formats.messagepack`: MessagePack compact binary for APIs

#### Analytics Integration (2026-01-22)
- **`mrrc.analytics` Module**: SQL and DataFrame operations over MARC data
  - `to_duckdb(records)`: Create DuckDB relation for SQL queries
  - `to_polars(records)`: Create Polars DataFrame for data analysis
  - Enables filtering, aggregation, and joins across millions of records
- **`export_to_parquet()` Helper**: Convert Arrow tables to Parquet files for data lakes
- **Integration Examples**: Working examples demonstrating analytics workflows

#### Documentation Expansion (2026-01-22)
- **New User Guides**:
  - `INSTALLATION_GUIDE.md`: Complete installation instructions for Python and Rust, feature flags, platform-specific notes, troubleshooting
  - `FORMAT_SELECTION_GUIDE.md`: Decision tree for choosing the right format, comparison matrix, use case recommendations
  - `PYTHON_TUTORIAL.md`: Comprehensive Python tutorial covering reading, writing, field access, format conversion, Query DSL
  - `RUST_TUTORIAL.md`: Complete Rust tutorial with builder pattern, traits, parallel processing with Rayon
  - `STREAMING_GUIDE.md`: Large file handling patterns, O(1) memory streaming, parallel processing strategies
- **Format Support Matrix**: Added comprehensive format comparison table to README.md
- **Enhanced Python Docstrings**: All format modules include use cases, performance notes, and working examples

#### Format Evaluation & Strategy (2026-01-19)
- **Binary Format Comparison**: Evaluated MessagePack, CBOR, Avro, Arrow, FlatBuffers against library requirements
- **Strategy Documentation**: Decision rationale and benchmarks archived in `docs/history/format-research/`

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

## Project Information

- **Repository**: https://github.com/dchud/mrrc
- **Documentation**: https://docs.rs/mrrc
- **Crates.io**: https://crates.io/crates/mrrc
- **Issue Tracking**: GitHub Issues and Beads issue tracking system
- **Status**: Active development, experimental (APIs may change)
