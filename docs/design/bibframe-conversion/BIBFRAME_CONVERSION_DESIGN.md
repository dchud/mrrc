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
| **Record types** | Bibliographic first | Most common use case; authority/holdings follow-on |
| **BFLC extensions** | Include | Required for practical LOC compatibility |

### Record Type Scope

**Phase 1 (this epic):** Bibliographic records only
- `Record` → BIBFRAME Work/Instance/Item

**Future work (separate epic):** Authority and Holdings records
- `AuthorityRecord` → BIBFRAME authority entities (requires different mapping)
- `HoldingsRecord` → BIBFRAME Item with holdings details

### Explicit Non-Goals

- BIBFRAME 1.0 support (deprecated, use 2.0 only)
- SPARQL query interface (use external tools)
- Full ontology validation (basic structure validation only)
- Linked data resolution (no network fetches during conversion)

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

### URI Generation Strategy

BIBFRAME entities require URIs. Strategy:

| Entity Type | URI Pattern | Example |
|-------------|-------------|---------|
| Work | `{base}/work/{control-number}` | `http://example.org/work/123456` |
| Instance | `{base}/instance/{control-number}` | `http://example.org/instance/123456` |
| Item | `{base}/item/{control-number}-{seq}` | `http://example.org/item/123456-1` |
| Agent | Blank node or `{base}/agent/{hash}` | `_:agent1` or linked URI |
| Subject | Blank node or linked URI | Link to id.loc.gov when available |

**Configuration options:**
- `base_uri`: Base URI for generated resources (default: blank nodes)
- `use_control_number`: Use MARC 001 in URIs (default: true)
- `link_authorities`: Link to external authority URIs when identifiable (default: false)

**Blank node fallback:** When no suitable identifier exists, use blank nodes.
This is valid RDF and simplifies interchange without requiring URI minting infrastructure.

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

## Error Handling Strategy

### MARC → BIBFRAME Errors

| Error Type | Handling | Example |
|------------|----------|---------|
| Missing required field | Skip entity, log warning | No 245 → no Instance title |
| Invalid indicator | Use default mapping, log | Indicator '9' → treat as blank |
| Malformed subfield | Include as-is, log warning | Bad UTF-8 → preserve bytes |
| Unknown field | Pass through as note or skip | Local 9XX fields |

### BIBFRAME → MARC Errors

| Error Type | Handling | Example |
|------------|----------|---------|
| No MARC equivalent | Skip property, log info | bf:awards → no standard field |
| Multiple values | First value or concatenate | Multiple bf:title |
| Missing required | Return error | No Work/Instance found |
| Type mismatch | Best-effort conversion | Date as string |

### Batch Processing

- **Fail-fast mode**: Stop on first error (default: false)
- **Continue mode**: Log errors, continue processing, return summary
- **Error callback**: Optional user-provided error handler

## Configuration Options

```rust
pub struct BibframeConfig {
    /// Base URI for generated resources (None = blank nodes)
    pub base_uri: Option<String>,

    /// Include BFLC extensions (default: true)
    pub include_bflc: bool,

    /// Output format for RDF serialization
    pub output_format: RdfFormat, // RdfXml, JsonLd, Turtle

    /// Link to external authority URIs when identifiable
    pub link_authorities: bool,

    /// Strictness level for validation
    pub strict: bool,

    /// Include source MARC in AdminMetadata (for debugging)
    pub include_source: bool,
}
```

## Performance Considerations

### Memory Usage

- RDF graphs can be large (3-10x MARC record size)
- Work/Instance/Item split creates multiple entities per record
- Consider streaming output for large batches

### Optimization Strategies

- **Lazy URI generation**: Only mint URIs when serializing
- **Shared vocabulary terms**: Intern common predicates
- **Streaming serialization**: Write RDF as generated, don't buffer
- **Parallel conversion**: Records are independent, parallelize batch jobs

### Expected Performance

Target: Within 2x of raw MARC parsing speed for simple records.
Complex records with many linked fields may be slower.

## Testing Strategy

### Test Categories

1. **Unit tests**: Individual field mappings
   - Each MARC tag → BIBFRAME property mapping
   - Indicator handling variations
   - Subfield combinations

2. **Integration tests**: Complete record conversion
   - Simple bibliographic records
   - Records with linked 880 fields
   - Records with authority-controlled headings

3. **Round-trip tests**: MARC → BIBFRAME → MARC
   - Verify essential data preserved
   - Document expected losses
   - Property-based testing with arbitrary records

4. **Baseline comparison**: Against marc2bibframe2 output
   - Structural equivalence (same entities created)
   - Property coverage (same fields mapped)
   - Acceptable divergence documented

5. **Validation tests**
   - Output validates against BIBFRAME 2.0 ontology
   - Required properties present
   - Correct entity types used

6. **Performance tests**: Batch conversion benchmarks
   - Records per second throughput
   - Memory usage per record
   - Scaling with record complexity

### Test Corpus

Build from:
- LOC sample records
- mrrc existing test fixtures
- Edge case records (complex serials, multi-script, etc.)
- Intentionally malformed records (error handling)

## Open Questions

### Decided

| Question | Decision | Rationale |
|----------|----------|-----------|
| BFLC extensions? | Include | Required for LOC compatibility |
| BIBFRAME 2.1? | Not initially | Spec not finalized; design for extensibility |
| Streaming? | Yes, for output | Memory efficiency for large batches |

### To Decide During Research (uab.1, uab.2)

| Question | Options | Decision Point |
|----------|---------|----------------|
| RDF library | rio, sophia, oxrdf | Evaluate during uab.4.1 based on benchmarks |
| Default URI strategy | Blank nodes vs minted | Based on common use cases discovered |
| 880 field handling | Parallel properties vs notes | Per LOC spec interpretation |

## Edge Cases Reference (uab.4.4)

### MARC Structural Edge Cases

| Case | Challenge | Strategy |
|------|-----------|----------|
| **880 linked fields** | Alternate script representations | Parallel literals with language tags |
| **Relator codes** | $4 vs $e, code vs term | Map to bf:role, prefer URI when known |
| **Linking fields (76X-78X)** | Related work references | bf:relatedTo with appropriate subtype |
| **Series (4XX/8XX)** | Series vs series-like | Distinguish bf:hasSeries vs bf:partOf |
| **Multiple 1XX** | Invalid but exists in wild | Use first, log warning |

### Content Type Variations

| MARC Format | BIBFRAME Handling |
|-------------|-------------------|
| Books (BK) | Standard Work/Instance |
| Serials (SE) | bf:Serial subclass, frequency properties |
| Music (MU) | bf:MusicAudio, notation, instrumentation |
| Maps (MP) | bf:Cartographic, scale, projection |
| Visual (VM) | bf:MovingImage or bf:StillImage |
| Mixed (MX) | bf:MixedMaterial |
| Computer (CF) | bf:Electronic, file characteristics |

### Character Encoding

- MARC-8 → UTF-8 conversion before BIBFRAME (already handled by mrrc)
- Non-Latin scripts: Preserve in RDF literals with `@lang` tags
- Combining characters: Normalize to NFC

### Identifier Complexity

| MARC Field | Complexity | Handling |
|------------|------------|----------|
| 020 (ISBN) | Qualifiers in $q | Separate bf:qualifier property |
| 022 (ISSN) | Linking ISSN in $l | bf:issn with note |
| 024 (Other) | Multiple types via ind1 | Map indicator to identifier type |
| 035 (System) | Prefixes in parens | Parse prefix, create bf:Local |

---

*Last updated: 2026-01-27*
*Epic: mrrc-uab*
