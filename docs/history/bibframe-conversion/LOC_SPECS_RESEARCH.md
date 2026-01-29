# LOC BIBFRAME Specification Research (mrrc-uab.1)

> Research completed: 2026-01-27
> Spec version: 3.0 (December 2025)

## Key Resources

### Primary Specifications
| Resource | URL | Purpose |
|----------|-----|---------|
| MARC→BIBFRAME Specs | https://www.loc.gov/bibframe/mtbf/ | 17 mapping documents by field range |
| BIBFRAME→MARC Specs | https://www.loc.gov/bibframe/bftm/ | Reverse conversion specs |
| BIBFRAME Model | https://www.loc.gov/bibframe/docs/bibframe2-model.html | Conceptual model |
| BIBFRAME Vocabulary | http://id.loc.gov/ontologies/bibframe.html | RDF classes/properties |
| BFLC Extensions | http://id.loc.gov/ontologies/bflc.html | LOC-specific extensions |

### Tools
| Tool | URL | Purpose |
|------|-----|---------|
| marc2bibframe2 | https://github.com/lcnetdev/marc2bibframe2 | Official MARC→BF converter (XSLT) |
| bibframe2marc | https://github.com/lcnetdev/bibframe2marc | Official BF→MARC converter |
| Compare Tool | https://id.loc.gov/tools/bibframe/compare-id/full-rdf | Online comparison |

## BIBFRAME 2.0 Model

### Namespace Prefixes

```turtle
@prefix bf:      <http://id.loc.gov/ontologies/bibframe/> .
@prefix bflc:    <http://id.loc.gov/ontologies/bflc/> .
@prefix madsrdf: <http://www.loc.gov/mads/rdf/v1#> .
@prefix rdfs:    <http://www.w3.org/2000/01/rdf-schema#> .
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
```

### Core Classes

```
bf:Work (intellectual content)
  ├── bf:Instance (physical/digital manifestation)
  │     └── bf:Item (specific copy)
  │
  └── bf:Hub (optional: groups expressions/translations)
        └── bf:Instance (instances of this expression)
```

**Note**: Hub is optional and used for expression-level grouping (e.g., "the French translation of Work X"). Most simple records go directly Work → Instance.

### Class Definitions

| Class | Definition | MARC Analog | When Created |
|-------|------------|-------------|--------------|
| **Work** | Conceptual essence: authors, languages, subjects | Implied by record | Always (one per record) |
| **Instance** | Material embodiment: publisher, date, format | Most descriptive fields | Always (one per record) |
| **Item** | Actual copy: location, barcode, condition | Holdings (852, 876-878) | When holdings present |
| **Hub** | Groups related expressions (translations, versions) | Uniform titles (130, 240) | Only when 240 present |

### Key Relationships

```
Work ──hasInstance──→ Instance ──hasItem──→ Item
  │                      │
  ├── contribution ──→ Agent (with Role)
  ├── subject ──→ Topic/Place/Person/etc.
  └── hasExpression ──→ Hub
```

## MARC→BIBFRAME Mapping Documents

The specs are organized into 17 Excel/Word documents:

| Document | MARC Fields | Key Mappings |
|----------|-------------|--------------|
| 001-007 | Control fields | Identifiers, physical format |
| 006, 008 | Fixed fields | Work/Instance type, dates, language |
| 010-048 | Numbers | LC control #, ISBN, ISSN, music numbers |
| 050-088 | Classification | LC/Dewey call numbers |
| 1XX, 7XX, 8XX | Names | Agents with roles (1XX=primary, 7XX=added) |
| 210-247 | Titles | Main title (245), variant titles (246), etc. |
| 240, X30 | Uniform titles | Hub creation (expression grouping) |
| 250-270 | Edition/Imprint | ProvisionActivity |
| 3XX | Physical | Extent, dimensions, media |
| 490, 510 | Series/Links | Series statements |
| 5XX | Notes | Various note types |
| 600-662 | Subjects | Topic, name, geographic subjects |
| 720, 740-758 | Added entries | Additional agents, titles |
| 760-788 | Linking | Related works/instances |
| 841-887 | Holdings | Item information |

### Mapping Conventions

The specs use shorthand notation:
- **W** = Property of Work
- **I** = Property of Instance
- **It** = Property of Item
- **H** = Property of Hub

Example from specs:
```
245 $a → I bf:title/bf:Title/bf:mainTitle
245 $b → I bf:title/bf:Title/bf:subtitle
245 $c → I bf:responsibilityStatement
```

## Key Mapping Patterns

### 1. Leader/008 → Type Determination

The Leader positions 06 (type of record) and 07 (bibliographic level) determine Work and Instance types:

| Leader/06 | Description | Leader/07 | Work Type | Instance Type |
|-----------|-------------|-----------|-----------|---------------|
| a | Language material | m (mono) | bf:Text | bf:Instance |
| a | Language material | s (serial) | bf:Text | bf:Serial |
| t | Manuscript | m/s | bf:Text | bf:Manuscript |
| c | Notated music | * | bf:NotatedMusic | bf:Instance |
| d | Manuscript music | * | bf:NotatedMusic | bf:Manuscript |
| e | Cartographic | * | bf:Cartography | bf:Instance |
| f | Manuscript carto | * | bf:Cartography | bf:Manuscript |
| g | Projected medium | * | bf:MovingImage | bf:Instance |
| i | Nonmusical sound | * | bf:Audio | bf:Instance |
| j | Musical sound | * | bf:MusicAudio | bf:Instance |
| k | 2D nonprojectable | * | bf:StillImage | bf:Instance |
| m | Computer file | * | bf:Multimedia | bf:Electronic |
| o | Kit | * | bf:Kit | bf:Instance |
| p | Mixed materials | * | bf:MixedMaterial | bf:Instance |
| r | 3D artifact | * | bf:Object | bf:Instance |

**Note**: Leader/07 values: m=monograph, s=serial, a=component part, c=collection, i=integrating resource

### 2. Names → Agents with Contributions

```
100 1_ $a Smith, John $d 1950- $e author
    ↓
Work
  └── bf:contribution
        └── bf:Contribution
              ├── bf:agent → bf:Person
              │               └── rdfs:label "Smith, John, 1950-"
              └── bf:role → <http://id.loc.gov/vocabulary/relators/aut>
```

### 3. Subjects → Multiple Entity Types

| MARC Field | Subject Type | BIBFRAME Class |
|------------|--------------|----------------|
| 600 | Personal name | bf:Person (as subject) |
| 610 | Corporate name | bf:Organization (as subject) |
| 611 | Meeting | bf:Meeting (as subject) |
| 630 | Uniform title | bf:Work (as subject) |
| 650 | Topical | bf:Topic |
| 651 | Geographic | bf:Place |
| 655 | Genre/Form | bf:GenreForm |

### 4. Identifiers → Typed bf:Identifier

| MARC Field | BIBFRAME Identifier Type |
|------------|-------------------------|
| 010 | bf:Lccn |
| 020 | bf:Isbn |
| 022 | bf:Issn |
| 024 ind1=0 | bf:Isrc |
| 024 ind1=1 | bf:Upc |
| 024 ind1=2 | bf:Ismn |
| 024 ind1=3 | bf:Ean |
| 024 ind1=7 | (check $2 for type) |
| 035 | bf:Local |

### 5. Provision Activity (260/264)

The 264 field's second indicator determines the activity type:

| 264 ind2 | Activity Type | BIBFRAME Class |
|----------|---------------|----------------|
| 0 | Production | bf:Production |
| 1 | Publication | bf:Publication |
| 2 | Distribution | bf:Distribution |
| 3 | Manufacture | bf:Manufacture |
| 4 | Copyright notice | bf:copyrightDate (property, not activity) |

```
264 _1 $a New York : $b Publisher, $c 2020
    ↓
Instance
  └── bf:provisionActivity
        └── bf:Publication
              ├── bf:place → bf:Place (rdfs:label "New York")
              ├── bf:agent → bf:Agent (rdfs:label "Publisher")
              └── bf:date "2020"
```

**Note**: Field 260 (older format) maps similarly but lacks the indicator-based type distinction; defaults to bf:Publication.

## BIBFRAME→MARC Conversion Notes

### Reverse Conversion Process

The BIBFRAME→MARC conversion must:
1. Identify the Work, Instance(s), and Item(s) in the graph
2. Reconstruct Leader and 008 from bf:content, bf:media, bf:carrier types
3. Map properties back to appropriate MARC fields
4. Handle multiple Instances (may produce multiple MARC records)

### URI Preservation
From specs: "URIs in the BIBFRAME description are preserved in the MARC $0 and $1 fields"

- `$0` = URI of the entity (authority record link)
- `$1` = Real World Object URI (the thing itself, not the description)

### AdminMetadata → MARC Control Fields

bf:AdminMetadata properties map back to Leader and control fields:

| BIBFRAME Property | MARC Location |
|-------------------|---------------|
| bf:creationDate | 008/00-05 |
| bf:changeDate | 005 |
| bflc:encodingLevel | Leader/17 |
| bf:descriptionConventions | 040 $e |
| bf:descriptionLanguage | 040 $b |
| bf:source | 040 $a, $c, $d |

### Known Data Loss (BF→MARC)

| BIBFRAME Property | MARC Handling | Notes |
|-------------------|---------------|-------|
| bf:awards | 586 (best effort) | May lose structured data |
| Multiple bf:title types | First or concatenate | MARC has limited title repetition |
| bf:relation (generic) | Best-effort 76X-78X | May lose relationship specificity |
| Complex subject URIs | URI excluded | Subdivided headings lose $0 |
| bf:summary (long) | 520 truncation | MARC field length limits |
| Multiple bf:Instance | Separate records | One MARC record per Instance |

## BFLC Extensions Required

BFLC (BIBFRAME Library of Congress) extensions fill gaps where core BIBFRAME lacks concepts needed for MARC round-trip fidelity:

| Extension | Purpose | MARC Equivalent |
|-----------|---------|-----------------|
| bflc:aap | Authorized access point (text form) | 1XX/7XX concatenated |
| bflc:PrimaryContribution | Distinguishes 1XX from 7XX | 1XX = primary, 7XX = added |
| bflc:encodingLevel | Cataloging completeness level | Leader/17 |
| bflc:simplePlace | Transcribed place (not parsed) | 260/264 $a as-is |
| bflc:simpleDate | Transcribed date (not parsed) | 260/264 $c as-is |
| bflc:simpleAgent | Transcribed agent (not parsed) | 260/264 $b as-is |
| bflc:marcKey | Raw MARC for round-trip | Preserves original encoding |
| bflc:SeriesTreatment | Series tracing decisions | 490/8XX relationship |
| bflc:applicableInstitution | Institution-specific data | Holdings context |

**Why bflc:simple* properties?** Core BIBFRAME parses provision statements into structured bf:Place/bf:Agent entities. The bflc:simple* properties preserve the original transcribed text for display and round-trip fidelity.

## Edge Cases and Ambiguities

### 1. Hub vs Work Decision
When to create a Hub (expression-level grouping) vs link directly to Work:
- **Create Hub**: When 240 (uniform title) differs from 245
- **Direct to Work**: When no expression-level distinction needed

### 2. 880 Linked Fields (Alternate Scripts)
The specs handle 880 by creating parallel literals:
```
245 $6 880-01 $a Title in Latin script
880 $6 245-01 $a タイトル
    ↓
bf:Title
  ├── bf:mainTitle "Title in Latin script"
  └── bf:mainTitle "タイトル"@ja
```

### 3. Relator Codes vs Terms
- `$4` (code) → Link to id.loc.gov/vocabulary/relators/{code}
- `$e` (term) → Try to match to known relator, else use text

### 4. Series Treatment
Series (490/8XX) mapping depends on traced vs untraced:
- **Traced (490 1_)**: Create bf:hasSeries relationship to Hub/Work
- **Untraced (490 0_)**: Use bf:seriesStatement (text only)

### 5. Multiple 1XX Fields
Invalid MARC but exists in wild data:
- Spec doesn't explicitly address
- **Recommendation**: Use first 1XX, log warning for others

## Decisions for Implementation

Based on this research, recommended decisions for mrrc:

### Entity Creation

| Question | Decision | Rationale |
|----------|----------|-----------|
| Hub creation | Only when 240 present | Matches LOC converter behavior |
| Item creation | Only when 852/876-878 present | Don't create empty Items |
| Multiple Instances | One per MARC record | MARC doesn't distinguish editions well |

### Data Handling

| Question | Decision | Rationale |
|----------|----------|-----------|
| Relator codes ($4) | Map to id.loc.gov URIs | Standard LOC practice |
| Relator terms ($e) | Try match, else preserve text | Don't lose data |
| Local fields (9XX) | Skip with info log | Not in spec, institution-specific |
| Invalid MARC | Best-effort, log warnings | Real-world data is messy |
| Missing 245 | Create Instance anyway, log error | Record may still be useful |

### Round-Trip Fidelity

| Question | Decision | Rationale |
|----------|----------|-----------|
| Use bflc:marcKey | Optional (config flag) | Enables perfect round-trip but verbose |
| Use bflc:simple* | Yes | Preserves transcribed text for display |
| AdminMetadata | Always include | Needed for Leader/008 reconstruction |

## Tool Setup (mrrc-uab.2)

### Prerequisites

- `xsltproc` (libxslt) - typically pre-installed on macOS/Linux
- Git for cloning repositories

### Installation

Both LOC converters are installed in `tools/` (gitignored):

```bash
mkdir -p tools && cd tools
git clone --depth 1 https://github.com/lcnetdev/marc2bibframe2.git
git clone --depth 1 https://github.com/lcnetdev/bibframe2marc.git

# Build bibframe2marc (compiles rules into XSLT)
cd bibframe2marc && make
```

### Usage

**MARC → BIBFRAME:**
```bash
xsltproc --stringparam baseuri http://example.org/ \
  tools/marc2bibframe2/xsl/marc2bibframe2.xsl \
  input.xml > output-bibframe.xml
```

**BIBFRAME → MARC:**
```bash
xsltproc tools/bibframe2marc/bibframe2marc.xsl \
  input-bibframe.xml > output-marc.xml
```

### Key Parameters

| Tool | Parameter | Default | Purpose |
|------|-----------|---------|---------|
| marc2bibframe2 | `baseuri` | `http://example.org/` | URI stem for entities |
| marc2bibframe2 | `idfield` | `001` | Field for record ID |
| marc2bibframe2 | `idsource` | (none) | URI for source institution |
| marc2bibframe2 | `serialization` | `rdfxml` | Output format (only rdfxml supported) |
| bibframe2marc | `pRecordId` | (auto) | Override record ID |
| bibframe2marc | `pCatScript` | `Latn` | Default cataloging script |

### Verified Working

- **marc2bibframe2 v3.0.0** (December 2025)
- **bibframe2marc v3.0.0** (December 2025)
- **Round-trip tested**: MARC → BIBFRAME → MARC preserves core data

### Known Tool Limitations

1. **Output format**: marc2bibframe2 only outputs RDF/XML (no JSON-LD/Turtle)
2. **Duplicate 264**: Round-trip may create duplicate provision activity fields
3. **880 linking**: Alternate script fields don't link back to source tags
4. **Item deduplication**: Multiple Items may represent same copy, need manual collapsing

### Baseline Generation Issues (mrrc-uab.3)

Issues discovered while generating baseline conversions:

1. **Incomplete 008 fields cause failures**: Records with minimal 008 fields (e.g., only country code) cause XPath errors in the converter. Real-world data should have complete 008 fields.

2. **Missing `idsource` warning**: The converter emits warnings when `idsource` parameter is not provided. This is informational only and doesn't affect output.

3. **Test data location**: Baseline conversions stored in `tests/data/bibframe-baselines/` with 12 test cases covering all major MARC field categories.

### Test Data Location

LOC provides comprehensive test data:
```
tools/marc2bibframe2/test/data/           # Sample MARC XML files
tools/marc2bibframe2/test/data/ConvSpec-*/  # Field-specific test cases
tools/marc2bibframe2/test/data/ConvSpec-880/  # Alternate script tests
```

## Next Steps

1. ~~**uab.1**: Research LOC specifications~~ ✓
2. ~~**uab.2**: Set up LOC conversion tools~~ ✓
3. **uab.3**: Create test corpus and baseline conversions
4. **uab.4.1**: Select RDF library based on format support needs

---

*Research task: mrrc-uab.1 - Complete*
*Tool setup task: mrrc-uab.2 - Complete*
