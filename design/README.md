# Design Documents

This directory contains architectural and design documentation for the MRRC (MARC Rust Crate) project.

## Active Design Documents

### [FIELD_QUERY_DSL.md](./FIELD_QUERY_DSL.md)
**Status**: Phase 1 ✅ Complete, Phases 2-3 Planned  
**Overview**: Domain-specific query patterns for finding MARC fields based on complex criteria.

**Phase 1 Delivered**:
- ✅ FieldQuery builder pattern for combining criteria (tag, indicators, subfields)
- ✅ TagRangeQuery for finding fields within tag ranges (e.g., 600-699)
- ✅ Record API methods for indicator-based filtering (`fields_by_indicator()`)
- ✅ Record API methods for tag range queries (`fields_in_range()`)
- ✅ Record API methods for subfield-based filtering (`fields_with_subfield()`)
- ✅ Record API methods for complex queries (`fields_matching()`, `fields_matching_range()`)
- ✅ Mutable field iterators for batch operations

**Completed**: Epic mrrc-9n8 (Phase 1 field query DSL). Epic mrrc-4zn (mutable field iterators).

**What's Planned**:
- Phase 2: Subfield pattern matching with regex, value-based filtering helpers, convenience methods
- Phase 3: Linked field navigation (880), authority control helpers, format-specific queries

### [AUTHORITY_RECORD_DESIGN.md](./AUTHORITY_RECORD_DESIGN.md)
**Status**: ✅ Completed  
**Overview**: Specialized support for MARC Authority and Holdings records (Types Z, x, y, v, u).

Covers:
- Authority record structure and field organization
- Heading types (1XX) and tracing fields (4XX/5XX)
- Implementation details for AuthorityRecord and HoldingsRecord types
- 008 fixed field interpretation for authority records

**Completed**: Epic mrrc-fzy with all 4 phases (phases 1-2 for authority, phases 3-4 for holdings). Readers, writers, and comprehensive tests implemented.

## Historical & Proposal Documents

### [api-refactor-proposal.md](./api-refactor-proposal.md)
**Status**: ✅ Completed  
**Overview**: Comprehensive refactoring to reduce code duplication across Record, AuthorityRecord, and HoldingsRecord types.

**Delivered**:
- `MarcRecord` trait for common operations (`src/marc_record.rs`)
- `GenericRecordBuilder<T>` for unified interface (`src/record_builder_generic.rs`)
- `RecordHelpers` trait with 20+ helper methods (`src/record_helpers.rs`)
- `FieldCollection` pattern for field accessors (`src/field_collection.rs`)
- Unified field storage across all record types

**Impact**: ~300+ LOC of duplication eliminated. Zero breaking changes. Full backward compatibility maintained. See epic mrrc-c4v.

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
