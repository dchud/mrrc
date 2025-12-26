# Design Documents

This directory contains architectural and design documentation for the MRRC (MARC Rust Crate) project.

## Active Design Documents

### [FIELD_QUERY_DSL.md](./FIELD_QUERY_DSL.md)
**Status**: Phase 1 Complete, Phases 2-3 Planned  
**Overview**: Domain-specific query patterns for finding MARC fields based on complex criteria.

**What's Implemented**:
- FieldQuery builder pattern for combining criteria (tag, indicators, subfields)
- TagRangeQuery for finding fields within tag ranges (e.g., 600-699)
- Record API methods for indicator-based filtering
- Record API methods for tag range queries
- Record API methods for subfield-based filtering

**What's Planned**:
- Phase 2: Subfield pattern matching with regex
- Phase 3: Value-based filtering helpers, linked field navigation, authority control helpers

### [AUTHORITY_RECORD_DESIGN.md](./AUTHORITY_RECORD_DESIGN.md)
**Status**: Implemented  
**Overview**: Specialized support for MARC Authority records (Type Z).

Covers:
- Authority record structure and field organization
- Heading types (1XX) and tracing fields (4XX/5XX)
- Implementation details for AuthorityRecord type

## Historical & Proposal Documents

### [api-refactor-proposal.md](./api-refactor-proposal.md)
**Status**: Proposal (Not Yet Started)  
**Overview**: Comprehensive proposal for reducing code duplication across Record, AuthorityRecord, and HoldingsRecord types.

**Key Points**:
- Eliminates ~350 LOC of redundant code
- Proposes trait-based common operations (`MarcRecord` trait)
- Suggests generic builder pattern
- 5-phase implementation plan with backward compatibility strategy

**Recommendation**: Review before implementing mrrc-y37 (documentation and examples) to avoid documenting patterns that may change.

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
