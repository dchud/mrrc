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

### Core Classes (Hierarchy)

```
bf:Work (intellectual content)
  └─ bf:Hub (expression/version grouping)
     └─ bf:Instance (physical/digital manifestation)
        └─ bf:Item (specific copy)
```

### Class Definitions

| Class | Definition | MARC Analog |
|-------|------------|-------------|
| **Work** | Conceptual essence: authors, languages, subjects | Implied by record |
| **Hub** | Groups related expressions (translations, versions) | Uniform titles (130, 240) |
| **Instance** | Material embodiment: publisher, date, format | Most descriptive fields |
| **Item** | Actual copy: location, barcode, condition | Holdings (852, 876-878) |

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
| 1XX, 7XX, 8XX | Names | Agents with roles |
| 200-247 | Titles | Main/variant titles |
| 240, X30 | Uniform titles | Hub creation |
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

The Leader and 008 determine Work and Instance types:

| Leader/06 | Leader/07 | Work Type | Instance Type |
|-----------|-----------|-----------|---------------|
| a (text) | m (mono) | bf:Text | bf:Instance |
| a (text) | s (serial) | bf:Text | bf:Serial |
| c/d (music) | * | bf:NotatedMusic | bf:Instance |
| e/f (carto) | * | bf:Cartography | bf:Instance |
| g (proj) | * | bf:MovingImage | bf:Instance |
| i (sound) | * | bf:Audio | bf:Instance |
| j (music rec) | * | bf:MusicAudio | bf:Instance |
| k (2D) | * | bf:StillImage | bf:Instance |
| m (computer) | * | bf:Multimedia | bf:Electronic |
| o/p (kit) | * | bf:MixedMaterial | bf:Instance |

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

## BIBFRAME→MARC Conversion Notes

### URI Preservation
From specs: "URIs in the BIBFRAME description are preserved in the MARC $0 and $1 fields"

- `$0` = URI of the entity
- `$1` = Real World Object URI

### Known Data Loss (BF→MARC)

| BIBFRAME Property | MARC Handling |
|-------------------|---------------|
| bf:awards | No standard field (possibly 586) |
| Multiple bf:title | First only, or concatenate |
| bf:relation (generic) | Best-effort to specific 76X-78X |
| Complex subject URIs | URI excluded from subdivided headings |

## BFLC Extensions Required

These extensions are necessary for full LOC compatibility:

| Extension | Purpose |
|-----------|---------|
| bflc:aap | Authorized access point (text form) |
| bflc:PrimaryContribution | Distinguishes 1XX from 7XX |
| bflc:encodingLevel | MARC Leader/17 equivalent |
| bflc:simplePlace/Date/Agent | Transcribed provision data |
| bflc:marcKey | Round-trip MARC data preservation |
| bflc:SeriesTreatment | Series encoding specifications |
| bflc:applicableInstitution | Institution-specific data |

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

Based on this research, recommend these decisions for mrrc:

| Question | Decision | Rationale |
|----------|----------|-----------|
| Hub creation | Only when 240 present | Matches LOC behavior |
| Default relator | id.loc.gov URIs when code known | Standard practice |
| Unknown relators | Preserve $e text in bf:role | Don't lose data |
| Local fields (9XX) | Skip with warning | Not in spec |
| Invalid MARC | Best-effort, log warnings | Robustness |

## Next Steps

1. **uab.2**: Set up marc2bibframe2 to generate baseline conversions
2. **uab.3**: Create test corpus covering all mapping document areas
3. **uab.4.1**: Select RDF library based on format support needs

---

*Research task: mrrc-uab.1*
*Status: Complete*
