# MARC Authority Record Data Structure Design

## Status: ✅ COMPLETED

This design has been fully implemented. Authority records (Type Z) and Holdings records are now fully supported with readers, writers, and comprehensive test coverage. See epic **mrrc-fzy** for implementation details (4 subtasks completed 2025-12-21 through 2025-12-26).

## Overview

MARC Authority records (Type Z, Leader/06='z') are used to maintain standardized forms of names, subjects, and titles. They differ fundamentally from bibliographic records:

- **Bibliographic records** describe items/resources
- **Authority records** describe headings and their variants

## Core Differences from Bibliographic Records

| Aspect | Bibliographic | Authority |
|--------|---------------|-----------|
| Leader/06 | a, c, d, e, g, i, j, k, m, o, p, r, t | **z** |
| Main entry | 245 (Title) | 1XX (Heading) |
| Variants | 4XX, 5XX are optional fields | 4XX (See), 5XX (See Also) are primary content |
| Purpose | Describe a resource | Establish normalized headings and cross-references |
| Scope | Broader range of data | Focused on names, subjects, titles |

## Authority Record Heading Types (1XX field)

The 1XX field defines the type of authority record:

- **100** - Personal names (X00 pattern)
- **110** - Corporate names (X10 pattern)
- **111** - Meeting names (X11 pattern)
- **130** - Uniform titles (X30 pattern)
- **148** - Chronological terms (X48 pattern)
- **150** - Topical terms (X50 pattern)
- **151** - Geographic names (X51 pattern)
- **155** - Genre/form terms (X55 pattern)

## Key Fields and Content

### Control Fields (00X-09X)

Same basic structure as bibliographic records:
- **001** - Control Number
- **005** - Date and Time of Latest Transaction
- **008** - Fixed-length data elements (authority-specific)

#### 008 Authority-Specific Positions

| Position | Name | Values |
|----------|------|--------|
| 00-05 | Date entered on file | yymmdd |
| 06 | Direct/indirect geographic subdivision | a, i, n, |, (fill) |
| 07 | Type of URI | a, b, c, d, (fill) |
| 08 | Undefined | (fill) |
| 09 | Kind of record | a, b, c, d, e, f, g |
| 10 | Descriptive cataloging rules | a, b, c, (fill) |
| 11 | Subject heading system/thesaurus | a, b, c, d, k, n, v, z, (fill) |
| 12 | Type of series | a, b, c, (fill) |
| 13 | Number series/dates of birth-death | a, b, (fill) |
| 14 | Heading use-main or added entry | a, b |
| 15 | Heading use-subject added entry | a, b |
| 16 | Heading use-series added entry | a, b |
| 17 | Type of subject subdivision | n, p, (fill) |
| 28 | Type of government agency | a, b, c, d, u, z, (fill) |
| 29 | Reference evaluation | a, b, c, d, n, (fill) |
| 31 | Record update in process | a, b |
| 32 | Undifferentiated personal name | a, b, n |
| 33 | Level of establishment | a, b, c, d, n |
| 34 | Modified record | (space), d |
| 35 | Source of heading or term | a, c, (fill) |
| 39 | Cataloging source | (space), c |

### Heading Fields (1XX)

Single, primary heading field containing the established form.

**Structure:** Same as bibliographic field structure (indicators + subfields)

Examples:
```
100 1_ $aSmith, John,$d1873-1944.$tAutobiography
110 2_ $aNEW YORK (STATE).$bDepartment of Environmental Conservation
150 _0 $aArtistic anatomy
```

### Tracing and Reference Fields (4XX, 5XX)

Track variant and related headings.

#### 4XX - See From Tracing Fields
Show variants NOT used as headings (point TO the 1XX):
- **400** - See From Tracing-Personal Name
- **410** - See From Tracing-Corporate Name
- **411** - See From Tracing-Meeting Name
- **430** - See From Tracing-Uniform Title
- **448** - See From Tracing-Chronological Term
- **450** - See From Tracing-Topical Term
- **451** - See From Tracing-Geographic Name
- **455** - See From Tracing-Genre/Form Term

#### 5XX - See Also From Tracing Fields
Show related headings that ARE authorized (point BETWEEN headings):
- **500** - See Also From Tracing-Personal Name
- **510** - See Also From Tracing-Corporate Name
- **511** - See Also From Tracing-Meeting Name
- **530** - See Also From Tracing-Uniform Title
- **548** - See Also From Tracing-Chronological Term
- **550** - See Also From Tracing-Topical Term
- **551** - See Also From Tracing-Geographic Name
- **555** - See Also From Tracing-Genre/Form Term

### Note Fields (6XX, 66X, 67X, 68X)

Provide context and history:
- **667** - Nonpublic General Note
- **670** - Source data found (used for verification)
- **671** - Source data not found (negative reference)
- **680** - Public General Note
- **681** - Subject heading or term note
- **685** - History note

### Relationship/Linking Fields (7XX)

Link to other authorized headings:
- **700** - Established Heading Linking Entry-Personal Name
- **710** - Established Heading Linking Entry-Corporate Name
- **711** - Established Heading Linking Entry-Meeting Name
- **730** - Established Heading Linking Entry-Uniform Title
- **748** - Established Heading Linking Entry-Chronological Term
- **750** - Established Heading Linking Entry-Topical Term
- **751** - Established Heading Linking Entry-Geographic Name
- **755** - Established Heading Linking Entry-Genre/Form Term

## Proposed Rust Data Structures

### AuthorityRecord Structure

```rust
pub struct AuthorityRecord {
    pub leader: Leader,
    pub control_fields: BTreeMap<String, String>,
    pub heading: Option<Field>,          // 1XX field
    pub tracings_see_from: Vec<Field>,   // 4XX fields
    pub tracings_see_also: Vec<Field>,   // 5XX fields
    pub notes: Vec<Field>,               // 66X, 67X, 68X fields
    pub linking_entries: Vec<Field>,     // 7XX fields
    pub other_fields: BTreeMap<String, Vec<Field>>, // Other data fields
}
```

### AuthorityRecordBuilder

Similar to `RecordBuilder`, provide fluent API for constructing authority records:

```rust
pub struct AuthorityRecordBuilder {
    record: AuthorityRecord,
}

impl AuthorityRecordBuilder {
    pub fn heading(mut self, field: Field) -> Self { ... }
    pub fn add_see_from(mut self, field: Field) -> Self { ... }
    pub fn add_see_also(mut self, field: Field) -> Self { ... }
    pub fn add_note(mut self, field: Field) -> Self { ... }
    pub fn build(self) -> AuthorityRecord { ... }
}
```

### Helper Methods on AuthorityRecord

```rust
impl AuthorityRecord {
    // Get the 1XX heading
    pub fn heading(&self) -> Option<&Field> { ... }
    
    // Get heading type (PersonalName, CorporateName, etc.)
    pub fn heading_type(&self) -> Option<HeadingType> { ... }
    
    // Get all 4XX see-from tracings
    pub fn see_from_tracings(&self) -> Vec<&Field> { ... }
    
    // Get all 5XX see-also tracings
    pub fn see_also_tracings(&self) -> Vec<&Field> { ... }
    
    // Get notes
    pub fn notes(&self) -> Vec<&Field> { ... }
    
    // Get source data notes (670)
    pub fn source_data_found(&self) -> Vec<&Field> { ... }
    
    // Check if heading is fully established
    pub fn is_established(&self) -> bool { ... }
    
    // Get kind of record from 008/09
    pub fn kind_of_record(&self) -> Option<KindOfRecord> { ... }
}
```

### Enums for Authority-Specific Values

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadingType {
    PersonalName,
    CorporateName,
    MeetingName,
    UniformTitle,
    ChronologicalTerm,
    TopicalTerm,
    GeographicName,
    GenreFormTerm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KindOfRecord {
    EstablishedHeading,
    ReferenceUntracted,
    ReferenceTraced,
    Subdivision,
    EstablishedHeadingAndSubdivision,
    ReferenceAndSubdivision,
    NodeLabel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelOfEstablishment {
    FullyEstablished,
    Memorandum,
    Provisional,
    Preliminary,
    NotApplicable,
}
```

## Implementation Phases - COMPLETED

1. **Phase 1 (mrrc-fzy.1)**: Design and define data structures ✅
   - ✅ Create `AuthorityRecord` struct with unified `BTreeMap` field storage
   - ✅ Implement enums for authority-specific values (HeadingType, KindOfRecord, LevelOfEstablishment)
   - ✅ Create helper methods for common operations
   - ✅ Add unit tests for structure validation

2. **Phase 2 (mrrc-fzy.2)**: Reader/Writer ✅
   - ✅ Extended reader to detect and parse authority records
   - ✅ Implemented authority record writer
   - ✅ Roundtrip serialization tested

3. **Phase 3 (mrrc-fzy.3)**: Holdings records design ✅
   - ✅ Designed HoldingsRecord data structures
   - ✅ Implemented comprehensive unit tests

4. **Phase 4 (mrrc-fzy.4)**: Holdings reader/writer ✅
   - ✅ Implemented holdings record reader
   - ✅ Implemented holdings record writer
   - ✅ Comprehensive roundtrip testing

## Key Considerations

1. **Reuse existing structures**: Authority records use same Leader, Field, Subfield types as bibliographic records
2. **Focused structure**: AuthorityRecord doesn't need all the methods bibliographic records have (no publication info, no ISBN validation needed)
3. **Field organization**: Clear separation of heading, tracings, notes, and linking fields
4. **Validation**: Ensure 1XX is present, proper field relationships
5. **Type safety**: Use enums to encode authority-specific concepts from 008 field
