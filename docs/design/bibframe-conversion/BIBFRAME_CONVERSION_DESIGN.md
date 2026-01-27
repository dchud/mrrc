# BIBFRAME Conversion Design (mrrc-uab)

> **Documentation Directory**: All design documents, research notes, and specifications
> for this epic live in `docs/design/bibframe-conversion/`. Upon completion, this folder
> will be migrated to `docs/history/bibframe-conversion/`.

## Overview

This document outlines the design for bidirectional MARC ↔ BIBFRAME conversion in mrrc, enabling data interchange with BIBFRAME-native systems.

## Scope Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Use case** | Data interchange | Target systems that speak BIBFRAME natively |
| **Direction** | Bidirectional | MARC → BIBFRAME and BIBFRAME → MARC |
| **Fidelity** | Full LOC spec compliance | Industry standard, well-documented |

## Architecture

### Data Flow

```
MARC Record ←→ Internal Representation ←→ BIBFRAME RDF Graph
     ↓                                           ↓
  .mrc/.xml                              RDF/XML, JSON-LD, Turtle
```

### Key Components

1. **RDF Serialization Layer** (uab.4.1)
   - Parse/serialize RDF/XML, JSON-LD, Turtle formats
   - Handle BIBFRAME namespaces (bf:, bflc:, madsrdf:)

2. **MARC→BIBFRAME Converter** (uab.4.2)
   - Transform MARC fields to BIBFRAME entities
   - Create Work/Instance/Item graph structure
   - Map controlled vocabularies

3. **BIBFRAME→MARC Converter** (uab.4.3)
   - Flatten RDF graph back to MARC fields
   - Handle inherent data loss gracefully
   - Preserve round-trip fidelity where possible

4. **Edge Case Handling** (uab.4.4)
   - Complex linked data relationships
   - Authority record linking
   - Non-Latin scripts

## Implementation Phases

### Phase 1: Research (parallel)
- **mrrc-uab.1**: Obtain and study LOC conversion specifications
- **mrrc-uab.2**: Set up marc2bibframe2 for baseline generation

### Phase 2: Baseline
- **mrrc-uab.3**: Generate reference conversions for testing

### Phase 3: Implementation (sequential)
- **mrrc-uab.4.1**: RDF serialization support
- **mrrc-uab.4.2**: MARC→BIBFRAME core mapping
- **mrrc-uab.4.3**: BIBFRAME→MARC core mapping
- **mrrc-uab.4.4**: Edge cases and complex fields

### Phase 4: Testing
- **mrrc-uab.5**: Comprehensive comparison tests

### Phase 5: Documentation & Completion
- **mrrc-uab.6**: Update documentation and examples (Rust and Python)
- **mrrc-uab.7**: Migrate design docs to history (final step)

## Dependency Graph

```
uab.1 (specs)  ──┐
                 ├──→ uab.3 (baseline) ──→ uab.4.1 (RDF) ──→ uab.4.2 (M→B)
uab.2 (tools)  ──┘                                              ↓
                                                           uab.4.3 (B→M)
                                                              ↓
                                                           uab.4.4 (edge)
                                                              ↓
                                                           uab.5 (tests)
                                                              ↓
                                                           uab.6 (docs)
                                                              ↓
                                                           uab.7 (migrate)
```

## Key Resources

### Library of Congress
- [BIBFRAME Home](https://www.loc.gov/bibframe/)
- [MARC to BIBFRAME Specs](https://www.loc.gov/bibframe/mtbf/)
- [marc2bibframe2 Tool](https://github.com/lcnetdev/marc2bibframe2)

### BIBFRAME Ontology
- [BIBFRAME 2.0 Vocabulary](http://id.loc.gov/ontologies/bibframe/)
- [BFLC Extensions](http://id.loc.gov/ontologies/bflc/)

### Related Standards
- [MARC21 Format](https://www.loc.gov/marc/)
- [RDF 1.1 Concepts](https://www.w3.org/TR/rdf11-concepts/)

## Design Considerations

### Data Model Differences

MARC is a flat, field-based format while BIBFRAME is a linked data model with entities and relationships. Key mapping challenges:

| MARC Concept | BIBFRAME Concept |
|--------------|------------------|
| Single record | Work + Instance + Item graph |
| Field/subfield | Properties on entities |
| Indicators | Often implicit in property choice |
| Linked headings | Separate Agent/Subject entities |

### Round-Trip Fidelity

Perfect round-trip is not always possible because:
1. BIBFRAME is semantically richer than MARC
2. Some MARC conventions have no BIBFRAME equivalent
3. BIBFRAME normalizes what MARC keeps denormalized

Strategy: Preserve essential bibliographic data; document known lossy conversions.

### RDF Library Selection

Options to evaluate:
- **rio**: Fast, minimal, good for parsing/serializing
- **sophia**: Full-featured RDF library with SPARQL
- **oxrdf**: Part of oxigraph, well-maintained

Decision criteria: Performance, API ergonomics, format support, maintenance status.

## Testing Strategy

1. **Unit tests**: Individual field mappings
2. **Integration tests**: Complete record conversion
3. **Round-trip tests**: MARC → BIBFRAME → MARC
4. **Baseline comparison**: Against marc2bibframe2 output
5. **Performance tests**: Batch conversion benchmarks

## Open Questions

1. Which RDF library to use?
2. How to handle BFLC extensions vs core BIBFRAME only?
3. Support for BIBFRAME 2.1 (if/when released)?
4. Streaming conversion for large files?

---

*Last updated: 2026-01-27*
*Epic: mrrc-uab*
