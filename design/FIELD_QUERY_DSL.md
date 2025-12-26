# Advanced Field Query DSL Patterns

## Overview

This document outlines domain-specific patterns for finding fields based on complex criteria in the MARC record. The goal is to provide Rust-idiomatic query patterns that are more powerful than simple tag-based lookups.

## Current State

The mrrc library currently supports:
- `get_fields(tag: &str)` - Get all fields with a specific tag
- `fields_by_tag(tag: &str)` - Iterator over fields with a specific tag
- `remove_fields_where(predicate)` - Generic predicate-based filtering
- Helper methods like `title()`, `author()`, `subjects()` for common bibliographic fields

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

### Phase 1: Foundation (High-value, Low-complexity)
1. Indicator-based filtering
2. Tag range queries
3. Basic subfield existence checks

### Phase 2: Advanced Patterns (Higher-complexity)
1. Subfield pattern matching with regex
2. FieldQuery builder pattern
3. Value-based filtering helpers

### Phase 3: Specialized Queries (Domain-specific)
1. Linked field navigation
2. Authority control helpers
3. Format-specific queries

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
