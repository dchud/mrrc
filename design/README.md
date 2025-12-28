# Design Documents

This directory contains architectural and design documentation for the MRRC (MARC Rust Crate) project.

## Completed Design Documents (Archived to /history)

### [FIELD_QUERY_DSL.md](../history/FIELD_QUERY_DSL.md)
**Status**: ✅ Phase 1, 2 & 3 All Completed  
**Overview**: Domain-specific query patterns for finding MARC fields based on complex criteria.

**All Phases Delivered**:
- ✅ **Phase 1** (mrrc-9n8): FieldQuery builder, TagRangeQuery, indicator/subfield filtering, mutable iterators
- ✅ **Phase 2** (mrrc-131): Subfield pattern matching with regex, value-based filtering, convenience methods
- ✅ **Phase 3** (mrrc-08k, mrrc-69n): Linked field navigation (880), authority helpers, format-specific traits

**Test Coverage**: 97+ specialized tests + 282 library tests passing. See `/history/FIELD_QUERY_DSL_COMPLETED.md`.

### [AUTHORITY_RECORD_DESIGN.md](../history/AUTHORITY_RECORD_DESIGN.md)
**Status**: ✅ Completed  
**Overview**: Specialized support for MARC Authority and Holdings records (Types Z, x, y, v, u).

**Delivered**:
- Authority record structure with heading types (1XX) and tracing fields (4XX/5XX)
- Holdings record implementation with location and call number support
- Readers, writers, and comprehensive test coverage
- Epic mrrc-fzy (all 4 phases completed)

### [api-refactor-proposal.md](../history/api-refactor-proposal.md)
**Status**: ✅ All 5 Phases Completed  
**Overview**: Comprehensive refactoring to reduce code duplication across Record, AuthorityRecord, and HoldingsRecord.

**All Phases Delivered**:
- ✅ **Phase 1**: `MarcRecord` trait for common operations
- ✅ **Phase 2**: `GenericRecordBuilder<T>` unified builder interface
- ✅ **Phase 3**: `FieldCollection` trait for field accessors
- ✅ **Phase 4**: `RecordHelpers` trait with blanket implementation
- ✅ **Phase 5**: Unified field storage across all record types

**Impact**: ~300+ LOC of duplication eliminated. Zero breaking changes. Full backward compatibility. Epic mrrc-c4v. See `/history/API_REFACTOR_COMPLETED.md`.

## Active Design Documents

### [PYTHON_WRAPPER_PROPOSAL.md](./PYTHON_WRAPPER_PROPOSAL.md)
**Status**: 📝 Draft  
**Overview**: Strategy for creating a PyO3-based Python extension module offering near 100% API compatibility with `pymarc`.

**Scope**: Workspace restructuring, PyO3/Maturin integration, stateful wrapper pattern, performance optimization.

## Project History

### [../history/PYMARC_RUST_PORT_PLAN.md](../history/PYMARC_RUST_PORT_PLAN.md)
Original project plan and porting strategy from the beginning of the project. Documents the overall vision and technical decisions made.

## Creating New Design Documents

When proposing significant architectural changes or new features:

1. Create a new `.md` file in this directory
2. Start with **Status** (Proposed, In Progress, Complete, Implemented)
3. Include **Overview** section
4. Document problem statement and proposed solution
5. Include implementation phases or roadmap
6. Add this document to the index above
7. Link from relevant issues using `--deps discovered-from:` in beads

## Related Resources

- [../Cargo.toml](../Cargo.toml) - Project dependencies and configuration
- [../AGENTS.md](../AGENTS.md) - Development workflow and CI checks
- [../README.md](../README.md) - Public API documentation and usage examples
