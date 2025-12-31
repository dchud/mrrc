# Advanced Field Query DSL Patterns

## Status: All Phases ✅ Completed

- **Phase 1**: ✅ Completed (mrrc-9n8) - 9 unit tests, 17 integration tests
- **Phase 2**: ✅ Completed (mrrc-131) - 39 comprehensive tests for pattern matching and convenience methods
- **Phase 3**: ✅ Completed (mrrc-08k, mrrc-69n)
  - ✅ Linked field navigation (880 field linkage) - 28 integration tests, bidirectional lookups
  - ✅ Authority control helpers (AuthorityQueries trait) - 12 unit tests
  - ✅ Format-specific query traits (BibliographicQueries, AuthoritySpecificQueries, HoldingsSpecificQueries) - 11 unit tests

## Current State - All Phases Delivered

### Phase 1 & 2: Core Query DSL
- `FieldQuery` builder for complex criteria (tag, indicators, subfields)
- `TagRangeQuery` for range-based field lookups (e.g., 600-699)
- `Record::fields_by_indicator()` - Filter by first/second indicators
- `Record::fields_in_range()` - Get fields within tag range
- `Record::fields_matching()` - Apply FieldQuery to records
- `Record::fields_matching_range()` - Apply TagRangeQuery to records
- `Record::fields_with_subfield()` - Get fields containing specific subfield
- `Record::remove_fields_where()` - Generic predicate-based filtering
- `Record::fields_mut_with_subfield()` - Mutable access for batch operations
- Helper methods like `title()`, `author()`, `subjects()` for common bibliographic fields

### Phase 2: Advanced Patterns
- **Subfield pattern matching**: `SubfieldPatternQuery`, regex-based field matching
- **Value-based filtering**: `SubfieldValueQuery` with exact and partial matching
- **Convenience methods**: `subjects_with_subdivision()`, `isbns_matching()`, `names_in_range()`, `authors_with_dates()`, `subjects_with_note()`

### Phase 3: Linked Field Navigation
- `Record::get_linked_field()` - Get 880 field linked to an original field
- `Record::get_original_field()` - Get original field from an 880 field (reverse lookup)
- `Record::get_all_880_fields()` - Get all alternate graphical representation fields
- `Record::get_field_pairs()` - Get (original, Option<880>) pairs for a tag
- `Record::find_linked_by_occurrence()` - Find all fields linked by occurrence number
- `LinkageInfo` struct for parsing subfield 6 (linkage information)

### Phase 3: Authority Control Helpers
- **AuthorityQueries trait** for `AuthorityRecord`:
  - `get_see_from_headings()` - Extract 4XX fields (non-preferred forms)
  - `get_see_also_headings()` - Extract 5XX fields (related terms)
  - `get_relationship_fields()` - Extract 7XX fields (authority relationships)
  - `get_authority_references()` - Unified view of all reference fields (4XX+5XX+7XX)
  - `find_related_heading()` - Navigate to related authority headings
  - `extract_authority_label()` - Get main heading term (subfield 'a')
  - `get_subdivisions()` - Extract topical/geographic/chronological/genre subdivisions

### Phase 3: Format-Specific Query Traits
- **BibliographicQueries trait** for `Record`:
  - `get_titles()` - Get title fields (245)
  - `get_all_subjects()` - Get all subject fields (6XX range)
  - `get_topical_subjects()` - Get topical subjects (650)
  - `get_geographic_subjects()` - Get geographic subjects (651)
  - `get_all_names()` - Get all name fields (1XX, 6XX name, 7XX)
  - `get_linked_field_pairs()` - Get field pairs with 880 alternates
  - `get_all_880_fields()` - Get alternate graphical representation fields

- **AuthoritySpecificQueries trait** for `AuthorityRecord`:
  - `get_preferred_heading()` - Get the main heading (1XX)
  - `get_variant_headings()` - Get non-preferred forms (4XX)
  - `get_broader_related_headings()` - Get related headings (5XX)
  - `get_scope_note()` - Get scope note (680 subfield 'a')

- **HoldingsSpecificQueries trait** for `HoldingsRecord`:
  - `get_call_number()` - Get call number (090 or 050)
  - `get_holding_location()` - Get location (852 subfield 'b')
  - `get_holding_notes()` - Get all notes on holdings (5XX fields)

## Proposed Query Patterns

### 1. Indicator-Based Filtering

**Use Case**: Find all 245 fields where indicator2 is '0' (no non-filing characters)

```rust
record.fields_by_indicator("245", None, Some('0'))
record.fields_by_indicators("650", Some(' '), Some('0'))
```

**Implementation Strategy**:
- Add method: `fields_by_indicator(tag: &str, ind1: Option<char>, ind2: Option<char>)` -> Iterator
- Option = wildcard (matches any character)
- Build on existing field iterator infrastructure

### 2. Tag Range Queries

**Use Case**: Get all subject-related fields (6XX range)

```rust
// Get all 600-699 fields
record.fields_in_range("600", "699")

// Get all control fields (000-009)
record.control_fields_in_range("000", "009")

// Subject fields specifically
record.subject_fields()  // convenience for 600, 610, 611, 650, 651, etc.
```

**Implementation Strategy**:
- Parse tag strings as numeric ranges
- Iterate through stored BTreeMap keys within range
- Provide convenience methods for common ranges (subjects, names, titles, etc.)

### 3. Subfield Pattern Matching

**Use Case**: Find all fields containing specific subfield codes, with optional value patterns

```rust
// Fields with subfield 'a'
record.fields_with_subfield("650", 'a')

// Fields with both 'a' and 'x' subfields
record.fields_with_subfields("650", &['a', 'x'])

// Fields where subfield 'a' matches a pattern
record.fields_where_subfield_matches("650", 'a', "^Computer.*")
```

**Implementation Strategy**:
- Build on existing `subfields_by_code()` iterator
- Regex support using the `regex` crate (already in dependencies)
- Return filtered field iterators

### 4. Complex Predicates with Builder Pattern

**Use Case**: Multi-criteria queries combining indicators, subfields, and custom logic

```rust
let query = FieldQuery::new()
    .tag("650")
    .indicator2('0')  // LCSH
    .has_subfield('a')
    .has_subfield_value('2', "lcsh")  // Authority source
    .custom(|field| {
        field.subfields.len() >= 2
    });

for field in record.matching_fields(query) {
    println!("Subject: {}", field);
}
```

**Implementation Strategy**:
- Create `FieldQuery` builder struct
- Store predicates and combine with AND logic
- Implement iterator over matching fields
- Keep it extensible for future criteria

### 5. Value-Based Filtering

**Use Case**: Find fields where specific subfield values meet criteria

```rust
// All 020 fields with ISBNs containing specific pattern
record.fields_where(|field| {
    field.tag == "020" && 
    field.get_subfield('a')
        .map(|v| v.contains("978"))
        .unwrap_or(false)
})

// Named convenience methods
record.isbns_like("978-0")
record.subjects_with_subdivision('x', "History")
```

**Implementation Strategy**:
- Leverage existing `remove_fields_where` structure
- Add read-only variant: `find_fields_where(predicate)`
- Provide convenience methods for common searches

### 6. Linked Field Queries

**Use Case**: Work with field linkage (e.g., 880 fields linked via subfield 6)

```rust
// Get the 880 field linked to a given field
if let Some(linked) = field.get_linked_field(record) {
    println!("Romanized form: {}", linked);
}

// Get all fields linked to a specific field
record.get_all_linked_fields(&field)

// Get vernacular/Romanized pairs
record.get_field_pairs("100")
```

**Implementation Strategy**:
- Parse linkage occurrence number from subfield 6
- Create helper methods to navigate linkage
- Handle both forward and reverse lookups

## Implementation Roadmap

### Phase 1: Foundation ✅ COMPLETED
- ✅ Indicator-based filtering (`fields_by_indicator()`)
- ✅ Tag range queries (`fields_in_range()`, `TagRangeQuery`)
- ✅ Basic subfield existence checks (`fields_with_subfield()`)
- ✅ FieldQuery builder pattern (`FieldQuery` struct with fluent API)
- ✅ Comprehensive test coverage (9 unit tests, 17 integration tests)
- ✅ Mutable field iterators for batch operations (epic mrrc-4zn)

**Source**: `/src/field_query.rs`, `/src/record.rs` (query methods)

### Phase 2: Advanced Patterns ✅ COMPLETED
1. ✅ Subfield pattern matching with regex (e.g., `fields_where_subfield_matches()`)
2. ✅ Value-based filtering helpers (e.g., `SubfieldValueQuery`)
3. ✅ Convenience methods for common searches (e.g., `subjects_with_subdivision()`)

### Phase 3: Specialized Queries ✅ COMPLETED
1. ✅ Linked field navigation (880 field linkage via subfield 6)
2. ✅ Authority control helpers (AuthorityQueries trait)
3. ✅ Format-specific queries (BibliographicQueries, AuthoritySpecificQueries, HoldingsSpecificQueries)

## Design Principles

1. **Rust-Idiomatic**: Use iterators, builders, and Option/Result types
2. **Composable**: Queries should combine naturally
3. **Performant**: Minimize allocations, leverage BTreeMap ordering
4. **Backwards Compatible**: Don't break existing API
5. **Well-Documented**: Every pattern should have examples in doc comments

## Example Usage Scenarios

### Scenario 1: Extract all LCSH subject headings

```rust
record.fields_by_tag("650")
    .filter(|f| f.indicator2 == '0')
    .filter_map(|f| f.get_subfield('a'))
    .collect::<Vec<_>>()
```

**Better with DSL**:
```rust
record.fields_by_indicator("650", None, Some('0'))
    .filter_map(|f| f.get_subfield('a'))
    .collect::<Vec<_>>()
```

### Scenario 2: Find all name fields (100, 600-611, 700, 710, 711)

```rust
let mut names = Vec::new();
for tag in &["100", "600", "610", "611", "700", "710", "711"] {
    names.extend(record.fields_by_tag(tag).cloned());
}
```

**Better with DSL**:
```rust
let names: Vec<_> = record
    .fields_matching(FieldQuery::new().is_name_field())
    .collect();
```

### Scenario 3: Update all authority control subfield values

```rust
record.update_subfields_where(
    |f| f.tag.starts_with('6') && f.tag != "690",
    'd',
    "new-authority"
);
```

**Better with DSL**:
```rust
let query = FieldQuery::new()
    .in_range("600", "688")
    .exclude_tags(&["690"]);

record.update_fields_matching(query, |field| {
    for subfield in &mut field.subfields {
        if subfield.code == 'd' {
            subfield.value = "new-authority".to_string();
        }
    }
});
```

## Testing Strategy

1. **Unit tests**: Each query type in isolation
2. **Integration tests**: Realistic MARC records with complex criteria
3. **Performance tests**: Large batches with multiple query types
4. **Backwards compatibility tests**: Ensure existing API still works

## Open Questions

1. Should indicator queries support regex patterns? (Probably not initially)
2. Should tag ranges support more complex patterns (e.g., "6[0-5]0")? (Start simple)
3. Should we provide specialized queries for different record types (Bib, Auth, Holdings)?
4. Should FieldQuery support export to human-readable form (for logging)?
