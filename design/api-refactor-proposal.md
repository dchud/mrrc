# API Refactor Proposal: Trait-Based Architecture & Generic Builders

## Problem Statement

The three MARC record types (`Record`, `AuthorityRecord`, `HoldingsRecord`) exhibit significant code duplication and inconsistent patterns:

1. **Control field operations** duplicated identically in all 3 types (~20 LOC each)
2. **Three nearly identical builders** using copy-paste pattern (~60 LOC each, 180 total)
3. **Field-specific accessors** in Authority/Holdings use repetitive add/get pairs (~50 LOC each)
4. **Helper methods only in Record** - bibliographic-specific logic that could be trait-based
5. **Inconsistent field storage** - Record uses generic `BTreeMap<String, Vec<Field>>` while Authority/Holdings use typed `Vec` fields

**Result:** ~350 LOC of redundant code, harder to maintain, inconsistent API shape.

## Proposed Solution

### 1. Trait-Based Common Operations

Create a `MarcRecord` trait defining common operations:

```rust
pub trait MarcRecord {
    fn leader(&self) -> &Leader;
    fn leader_mut(&mut self) -> &mut Leader;
    fn add_control_field(&mut self, tag: impl Into<String>, value: impl Into<String>);
    fn get_control_field(&self, tag: &str) -> Option<&str>;
    fn control_fields_iter(&self) -> impl Iterator<Item = (&str, &str)>;
}
```

**Benefit:** 1 implementation, used by all 3 types. Eliminates ~60 LOC.

### 2. Generic Builder Pattern

Replace 3 builders with single generic:

```rust
pub struct RecordBuilder<T: MarcRecord> {
    record: T,
}

impl<T: MarcRecord> RecordBuilder<T> {
    pub fn control_field(mut self, tag: impl Into<String>, value: impl Into<String>) -> Self {
        self.record.add_control_field(tag, value);
        self
    }
    pub fn build(self) -> T { self.record }
}
```

**Benefit:** Eliminates 180 LOC of identical builder code. All three types use same builder interface.

### 3. Field-Specific Accessor Trait

For typed field collections in Authority/Holdings, create a macro or trait:

```rust
pub trait FieldCollection {
    fn add_field_to_collection(&mut self, field: Field, collection_name: &str);
    fn get_collection(&self, collection_name: &str) -> Option<&[Field]>;
}

// Or macro-based:
define_field_collection!(add_location, locations, locations_vec);
```

**Benefit:** Reduces 50 LOC per type, standardizes the pattern.

### 4. Helper Method Traits

Move bibliographic helpers to extension trait `RecordHelpers`:

```rust
pub trait RecordHelpers: MarcRecord {
    fn title(&self) -> Option<&str> { ... }
    fn author(&self) -> Option<&str> { ... }
    fn isbn(&self) -> Option<&str> { ... }
    // etc.
}

impl<T: MarcRecord> RecordHelpers for T { }
```

**Benefit:** Makes helpers available on all record types (Authority/Holdings can reuse for their needs). Move 20 LOC from impl block to shared trait.

### 5. Consistent Field Storage Pattern

Standardize all three record types to use a pattern like:

```rust
pub struct Record {
    pub leader: Leader,
    pub control_fields: BTreeMap<String, String>,
    pub fields: BTreeMap<String, Vec<Field>>,  // Generic field storage
}

pub struct AuthorityRecord {
    pub leader: Leader,
    pub control_fields: BTreeMap<String, String>,
    pub fields: BTreeMap<String, Vec<Field>>,  // Same pattern
    // Plus convenience accessors for 1XX, 4XX, etc. that delegate to fields
}
```

**Benefit:** Simpler mental model, easier to reason about field access, reduces special-case handling.

## Implementation Plan

### Phase 1: Trait Definition (No Breaking Changes)
- Define `MarcRecord` trait with control field ops
- Implement trait on all 3 types
- Add `MarcRecord` to prelude

### Phase 2: Generic Builder
- Create generic `RecordBuilder<T: MarcRecord>`
- Deprecate old builders, redirect to generic
- Update tests and examples

### Phase 3: Field Accessor Helpers
- Create macro or trait for field collection pattern
- Apply to Authority/Holdings
- Remove ~100 LOC of duplication

### Phase 4: Helper Traits (Breaking)
- Move bibliographic helpers to `RecordHelpers` trait
- Update all callers to use trait
- Keep backward compat via blanket impl

### Phase 5: Field Storage Standardization (Major)
- Refactor all three types to use consistent storage
- Update readers/writers
- Comprehensive test suite

## Expected Improvements

| Metric | Current | After |
|--------|---------|-------|
| Duplicate control field code | 60 LOC | 0 LOC |
| Builder implementations | 3 × 60 LOC | 1 × 40 LOC |
| Field accessor pairs | 50 LOC each | Macro-generated |
| Helper methods accessibility | Record only | All types via trait |
| Codebase maintainability | Moderate | High |

**Total reduction:** ~250-300 LOC while improving consistency and API surface.

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Breaking API changes | Use deprecation, trait blanket impls, phased rollout |
| Test coverage gaps | Comprehensive test suite before refactoring |
| Type complexity | Generic builder avoids template hell; keep simple |
| Reader/Writer updates | Already stable; verify in phase 5 |

## Backward Compatibility Strategy

- Phase 1-3: Additive only (new traits, deprecations)
- Phase 4: Provide blanket impl on `MarcRecord` for old callers
- Phase 5: Deprecate old patterns, provide migration guide

No breaking changes until phase 5, and only after 1-2 releases of deprecation warnings.

## References

- Similar trait pattern: `std::io::Read`, `std::fmt::Write`
- Builder pattern: [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/type-safety.html)
- Macro-based field accessors: inspiration from `serde` derive macros
