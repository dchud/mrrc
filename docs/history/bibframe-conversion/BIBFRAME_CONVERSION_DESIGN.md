# BIBFRAME Conversion Design (mrrc-uab)

> **Documentation Directory**: All design documents, research notes, and specifications
> for this epic live in `docs/design/bibframe-conversion/`. Upon completion, this folder
> will be migrated to `docs/history/bibframe-conversion/`.

## Overview

This document outlines the design for bidirectional MARC ↔ BIBFRAME conversion in mrrc, enabling data interchange with BIBFRAME-native systems.

### What is BIBFRAME?

BIBFRAME (Bibliographic Framework) is the Library of Congress's linked data model for bibliographic description, designed as a modern replacement for MARC. Key differences:

- **MARC**: Flat records with tagged fields and subfields (1970s design)
- **BIBFRAME**: RDF graph with distinct Work, Instance, and Item entities (linked data)

A single MARC record typically becomes multiple BIBFRAME entities linked by relationships.

## Scope Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Use case** | Data interchange | Target systems that speak BIBFRAME natively |
| **Direction** | Bidirectional | MARC → BIBFRAME and BIBFRAME → MARC |
| **Fidelity** | Full LOC spec compliance | Industry standard, well-documented |
| **Record types** | Bibliographic first | Most common use case; authority/holdings follow-on |
| **BFLC extensions** | Include | BFLC (BIBFRAME LOC) extensions required for practical LOC compatibility |

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
mrrc::Record ←────────────────────────→ BibframeGraph
     ↓                                        ↓
  .mrc/.xml                          RDF/XML, JSON-LD, Turtle
(ISO 2709, MARCXML)                  (Serialized RDF formats)
```

**Conversion functions:**
```rust
// MARC to BIBFRAME
fn marc_to_bibframe(record: &Record, config: &BibframeConfig) -> Result<BibframeGraph>;

// BIBFRAME to MARC
fn bibframe_to_marc(graph: &BibframeGraph) -> Result<Record>;
```

**BibframeGraph** is an RDF graph containing:
- One `bf:Work` entity (the intellectual content)
- One or more `bf:Instance` entities (physical/digital manifestations)
- Zero or more `bf:Item` entities (specific copies)
- Related entities: Agents, Subjects, Identifiers, etc.

### Conversion Example

**MARC input (simplified):**
```
001    123456
100 1_ $a Smith, John, $d 1950-
245 10 $a Introduction to cataloging / $c John Smith.
260 __ $a New York : $b Publisher, $c 2020.
```

**BIBFRAME output (simplified JSON-LD):**
```json
{
  "@graph": [
    {
      "@id": "_:work1",
      "@type": "bf:Text",
      "bf:contribution": { "@id": "_:agent1" },
      "bf:hasInstance": { "@id": "_:instance1" }
    },
    {
      "@id": "_:instance1",
      "@type": "bf:Instance",
      "bf:title": { "bf:mainTitle": "Introduction to cataloging" },
      "bf:provisionActivity": {
        "bf:place": "New York",
        "bf:agent": "Publisher",
        "bf:date": "2020"
      }
    },
    {
      "@id": "_:agent1",
      "@type": "bf:Person",
      "rdfs:label": "Smith, John, 1950-"
    }
  ]
}
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

BIBFRAME entities require URIs (or blank nodes). The strategy depends on configuration:

**When `base_uri` is set** (e.g., `http://example.org/`):

| Entity Type | URI Pattern | Example |
|-------------|-------------|---------|
| Work | `{base}/work/{control-number}` | `http://example.org/work/123456` |
| Instance | `{base}/instance/{control-number}` | `http://example.org/instance/123456` |
| Item | `{base}/item/{control-number}-{seq}` | `http://example.org/item/123456-1` |
| Agent | `{base}/agent/{hash}` | `http://example.org/agent/a1b2c3` |

**When `base_uri` is None** (default):

All entities use blank nodes (`_:work1`, `_:instance1`, etc.). This is valid RDF and simplifies interchange without requiring URI minting infrastructure.

**When `link_authorities` is true**:

Agents and subjects with identifiable authority control numbers link to external URIs:
- `http://id.loc.gov/authorities/names/n12345678` (LC Names)
- `http://id.loc.gov/authorities/subjects/sh12345678` (LCSH)

Otherwise, use blank nodes or minted URIs based on `base_uri` setting.

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

**Decision: oxrdfio** (v0.2.2, January 2026)

| Criteria | oxrdfio | sophia |
|----------|---------|--------|
| Version | 0.2.2 (Jan 2026) | 0.9.0 (Nov 2024) |
| RDF/XML | ✓ | ✓ (optional) |
| JSON-LD | ✓ (1.0) | ✓ (optional) |
| Turtle/N3/TriG | ✓ | ✓ |
| License | MIT/Apache 2.0 | CECILL-B |
| Scope | Focused parser/serializer | Full toolkit with SPARQL |

**Rationale:**
- More recent release with active maintenance (Oxigraph project)
- Focused scope (parsing/serialization only) matches our needs
- MIT/Apache 2.0 license aligns with mrrc licensing
- Part of Oxigraph ecosystem (oxrdf, oxttl, oxrdfxml, oxjsonld)
- Rio (deprecated) replaced by these Oxigraph crates

**Note**: Rio was considered but is deprecated in favor of oxttl/oxrdfxml which oxrdfio unifies.

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
    // === URI Generation ===
    /// Base URI for generated resources (None = use blank nodes)
    pub base_uri: Option<String>,
    /// Use MARC 001 control number in generated URIs
    pub use_control_number: bool,  // default: true
    /// Link to external authority URIs (id.loc.gov, etc.) when identifiable
    pub link_authorities: bool,    // default: false

    // === Output Control ===
    /// Output format for RDF serialization
    pub output_format: RdfFormat,  // RdfXml, JsonLd, Turtle
    /// Include BFLC (BIBFRAME Library of Congress) extensions
    pub include_bflc: bool,        // default: true
    /// Include source MARC in bf:AdminMetadata (for debugging/provenance)
    pub include_source: bool,      // default: false

    // === Error Handling ===
    /// Stop on first conversion error vs. continue and collect errors
    pub fail_fast: bool,           // default: false
    /// Strict validation (reject questionable data) vs. lenient (best-effort)
    pub strict: bool,              // default: false
}

pub enum RdfFormat {
    RdfXml,   // application/rdf+xml - Most compatible
    JsonLd,   // application/ld+json - Modern, readable
    Turtle,   // text/turtle - Compact, human-friendly
}
```

**Default configuration** uses blank nodes, includes BFLC extensions, outputs JSON-LD, and continues on errors with lenient validation.

### API Usage Examples

**Rust:**
```rust
use mrrc::{Reader, bibframe::{marc_to_bibframe, BibframeConfig, RdfFormat}};

// Basic conversion with defaults
let reader = Reader::from_path("records.mrc")?;
for record in reader {
    let graph = marc_to_bibframe(&record?, &BibframeConfig::default())?;
    println!("{}", graph.to_jsonld()?);
}

// Custom configuration
let config = BibframeConfig {
    base_uri: Some("http://example.org/".into()),
    output_format: RdfFormat::Turtle,
    ..Default::default()
};
let graph = marc_to_bibframe(&record, &config)?;
```

**Python (pymrrc):**
```python
import pymrrc

# Basic conversion
for record in pymrrc.Reader("records.mrc"):
    graph = pymrrc.marc_to_bibframe(record)
    print(graph.to_jsonld())

# With configuration
config = pymrrc.BibframeConfig(base_uri="http://example.org/")
graph = pymrrc.marc_to_bibframe(record, config)
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

### To Decide During Implementation

| Question | Options | Decision |
|----------|---------|----------|
| RDF library | rio, sophia, oxrdfio | **oxrdfio** (decided uab.4.1) |
| Default URI strategy | Blank nodes vs minted | Blank nodes (per design doc) |
| 880 field handling | Parallel properties vs notes | Parallel literals with @lang (per LOC spec) |

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
- Combining characters: Normalize to NFC (Unicode Normalization Form C)

### Identifier Complexity

| MARC Field | Complexity | Handling |
|------------|------------|----------|
| 020 (ISBN) | Qualifiers in $q | Separate bf:qualifier property |
| 022 (ISSN) | Linking ISSN in $l | bf:issn with note |
| 024 (Other) | Multiple types via ind1 | Map indicator to identifier type |
| 035 (System) | Prefixes in parens | Parse prefix, create bf:Local |

---

## Implementation Status

### Completed Tasks

| Task | Status | Notes |
|------|--------|-------|
| mrrc-uab.1 | ✓ Complete | LOC specification research |
| mrrc-uab.2 | ✓ Complete | LOC tool setup (marc2bibframe2, bibframe2marc) |
| mrrc-uab.3 | ✓ Complete | Baseline generation with 12 test categories |
| mrrc-uab.4.1 | ✓ Complete | RDF library selection (oxrdfio) |
| mrrc-uab.4.2 | ✓ Complete | Core MARC→BIBFRAME conversion |
| mrrc-uab.4.3 | ✓ Complete | Core BIBFRAME→MARC conversion |
| mrrc-uab.4.4 | ✓ Complete | Edge case and complex field handling |

### Edge Case Coverage (mrrc-uab.4.4)

**Implemented edge cases:**

1. **880 Linked Fields** - Alternate script representations
   - Extracts linked tag from $6 subfield
   - Detects script from Unicode ranges (Japanese, Korean, Chinese, Cyrillic, Hebrew, Arabic, Greek)
   - Creates parallel literals with appropriate `@lang` tags

2. **Linking Fields (76X-78X)** - Related work references
   - All 15 linking entry types supported (760-787)
   - Maps to appropriate BIBFRAME relationships (precededBy, succeededBy, partOf, etc.)
   - Extracts identifiers ($x=ISSN, $z=ISBN, $w=control numbers)

3. **Series Fields (490/8XX)** - Series treatment
   - 490 untraced → bf:seriesStatement
   - 490 traced → links to 8XX entries
   - 800/810/811/830 → bf:hasSeries with agent contributions

4. **Identifier Enhancements**
   - ISBN qualifiers ($q) → bf:qualifier
   - ISBN invalid ($z) → bf:status "invalid"
   - ISSN linking ($l) → bflc:IssnL
   - ISSN incorrect/canceled ($y/$z) → bf:status
   - 024 source ($2) → bf:source
   - 035 prefix parsing → bf:source

5. **Format-Specific Fields**
   - Music: 382 (medium of performance), 384 (key), 348 (notation format)
   - Cartographic: 255 (scale, projection, coordinates), 342 (geospatial)
   - Serials: 310/321 (frequency), 362 (dates of publication)

6. **Round-Trip Support**
   - Series fields (490/830) preserved through MARC→BIBFRAME→MARC
   - Linking entries (780/785) preserved through round-trip
   - Enhanced identifiers preserve qualifiers and status

### Test Coverage

- 48 BIBFRAME-specific unit tests
- All edge cases have dedicated tests
- Round-trip tests verify bidirectional conversion
- 474 total library tests passing

---

*Last updated: 2026-01-28*
*Epic: mrrc-uab*
