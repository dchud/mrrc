# Phase 3: Linked Field Navigation and Specialized Queries

## Overview
Phase 3 focuses on three key areas:
1. Linked field navigation (880 fields via subfield 6)
2. Authority control helpers
3. Format-specific query traits

---

### ⚠️ Current Status: GIL Release Investigation Ongoing

**Important Note:** Active investigation into GIL release performance issues is underway. See **@docs/history/GIL_RELEASE_INVESTIGATION_ADDENDUM.md** for:
- Root cause analysis (error handling in Phase 2)
- Critical issues identified in implementation
- Five new beads created to fix issues (mrrc-18s, mrrc-69r, mrrc-kzw, mrrc-5ph, mrrc-4rm)
- Critical path for fixes

Phase 3 planning may be affected by GIL release timeline. Consult the addendum for current status and dependencies.

---

## 1. Linked Field Navigation (880 Fields)

### Background: MARC 880 Fields
- **880**: Alternate Graphical Representation
- Used to link romanized/vernacular text with their original script versions
- Linkage via **subfield 6** (Linkage) in both the original field and the 880 field

### Subfield 6 Structure
Format: `NNN-XX` or `NNN-XX/r`
- `NNN`: Occurrence number (001-999) - links related fields
- `XX`: Script identification code
- `/r`: Reverse script flag (optional)

Example:
- Field 100: `$6 880-01` (occurrence 01)
- Field 880: `$6 100-01` (links back to 100)

### Implementation Plan

#### 1.1 Parse Linkage Occurrence
Create a utility to parse subfield 6:

```rust
pub struct LinkageInfo {
    pub occurrence: String,  // "01", "02", etc.
    pub script_id: String,   // "01" = Hebrew, "02" = Arabic, etc.
    pub is_reverse: bool,
}

impl LinkageInfo {
    pub fn from_subfield_6(value: &str) -> Option<Self> { ... }
}
```

#### 1.2 Field Linking Methods

Add to `Record`:

```rust
pub fn get_linked_field(&self, field: &Field) -> Option<&Field> {
    // Find the 880 field linked to this field
    // 1. Parse subfield 6 from field
    // 2. Find matching 880 field with same occurrence
}

pub fn get_all_linked_fields(&self, field: &Field) -> Vec<&Field> {
    // Get both original and 880 versions
    // Returns vector of 0-2 fields
}

pub fn get_field_pairs(&self, tag: &str) -> Vec<(&Field, Option<&Field>)> {
    // Get original field paired with its 880 counterpart (if exists)
}

pub fn get_880_fields(&self) -> impl Iterator<Item = &Field> {
    // Get all 880 fields
}

pub fn find_linked_by_occurrence(&self, occurrence: &str) -> Vec<&Field> {
    // Find all fields linked by occurrence number
}
```

#### 1.3 Bidirectional Lookup
- Original field (e.g., 100) has `$6 880-01`
- 880 field has `$6 100-01`
- Methods should work from either direction

### Implementation Strategy
1. Create `LinkageInfo` struct to parse subfield 6
2. Add `get_linked_field()` to Record
3. Add helper method to extract occurrence from subfield 6
4. Write comprehensive tests with realistic data

## 2. Authority Control Helpers

### Background: Authority Record Fields
- **4XX fields**: See From tracings (alternate forms)
- **5XX fields**: See Also tracings (related headings)
- **7XX fields**: Relationship information
- Field structure: same as regular fields (tag, indicators, subfields)

### Implementation Plan

#### 2.1 Authority Navigation Methods

Add to `AuthorityRecord`:

```rust
pub fn get_see_from_headings(&self) -> Vec<&Field> {
    // 4XX fields - non-preferred forms
}

pub fn get_see_also_headings(&self) -> Vec<&Field> {
    // 5XX fields - related terms
}

pub fn get_relationship_fields(&self) -> Vec<&Field> {
    // 7XX fields - authority relationships
}

pub fn get_authority_references(&self) -> Vec<&Field> {
    // All reference/tracing fields (4XX, 5XX, 7XX)
}
```

#### 2.2 Query Helpers

```rust
pub fn find_related_heading(&self, heading: &Field) -> Option<&Field> {
    // Find a related authority heading from 5XX
}

pub fn extract_authority_label(&field: &Field) -> Option<&str> {
    // Get $a subfield (preferred label)
}
```

## 3. Format-Specific Query Traits

### Goals
- Separate concerns for different record types
- Provide domain-specific helper methods
- Keep existing API unchanged

### Implementation Plan

#### 3.1 Trait Design

```rust
// For bibliographic records
pub trait BibliographicQueries {
    fn get_linked_field_pairs(&self) -> Vec<(&Field, Option<&Field>)>;
    fn get_all_880_fields(&self) -> Vec<&Field>;
}

// For authority records
pub trait AuthorityQueries {
    fn get_see_from_headings(&self) -> Vec<&Field>;
    fn get_see_also_headings(&self) -> Vec<&Field>;
    fn get_authority_references(&self) -> Vec<&Field>;
}

// For holdings records
pub trait HoldingsQueries {
    // Holdings-specific queries
}
```

#### 3.2 Implementation
- Implement `BibliographicQueries` for `Record`
- Implement `AuthorityQueries` for `AuthorityRecord`
- Implement `HoldingsQueries` for `HoldingsRecord`

## 4. Testing Strategy

### Test Data
Create sample MARC records with:
- Romanized/vernacular text linked via 880
- Authority records with 4XX, 5XX, 7XX fields
- Complex linking with multiple occurrence numbers
- Edge cases (missing linkage, malformed subfield 6)

### Test Coverage
1. **Linkage parsing**:
   - Parse valid `NNN-XX` format
   - Handle reverse script flag `/r`
   - Parse script ID codes
   - Reject invalid formats

2. **Field linking**:
   - Find 880 field from original
   - Find original from 880
   - Get field pairs
   - Handle non-existent links
   - Multiple occurrences

3. **Authority navigation**:
   - Extract see-from headings
   - Extract see-also headings
   - Extract relationship fields
   - Find related headings

4. **Format-specific traits**:
   - Bibliographic queries work on Record
   - Authority queries work on AuthorityRecord
   - Holdings queries work on HoldingsRecord

## Implementation Order

1. **Week 1**: Linked field navigation
   - LinkageInfo struct and parsing
   - get_linked_field() implementation
   - Basic tests

2. **Week 2**: Enhanced linking + Authority control
   - get_field_pairs() implementation
   - Authority helper methods
   - Authority tests

3. **Week 3**: Format-specific traits + Testing
   - Trait implementations
   - Comprehensive integration tests
   - Documentation

## Open Questions

1. Should we cache linkage information for performance?
2. How to handle malformed subfield 6 values?
3. Should we support additional linkage types beyond 880?
4. How deep should authority reference navigation go (just direct 4XX/5XX or also 7XX)?

## Success Criteria

- ✓ All linked field queries work bidirectionally
- ✓ Authority control methods handle all standard reference types
- ✓ Format-specific traits provide useful domain-specific queries
- ✓ 100+ comprehensive tests passing
- ✓ All quality gates passing (rustfmt, clippy, doc)
- ✓ No breaking changes to existing API
