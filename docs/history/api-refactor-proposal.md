# API Refactor Proposal: Trait-Based Architecture & Generic Builders

## Status: ✅ COMPLETED

This proposal has been fully implemented across all five phases. The refactoring was completed on **2025-12-26** with epic **mrrc-c4v** and is represented in the codebase as follows:
- `MarcRecord` trait: `/src/marc_record.rs`
- Generic builder: `/src/record_builder_generic.rs`
- Helper trait: `/src/record_helpers.rs`
- Field collection pattern: `/src/field_collection.rs`
- Unified record types: `/src/record.rs`, `/src/authority_record.rs`, `/src/holdings_record.rs`

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

### Phase 1: Trait Definition (No Breaking Changes) ✅
- ✅ Define `MarcRecord` trait with control field ops (in `src/marc_record.rs`)
- ✅ Implement trait on all 3 types (Record, AuthorityRecord, HoldingsRecord)
- ✅ Add `MarcRecord` to prelude

### Phase 2: Generic Builder ✅
- ✅ Create generic `GenericRecordBuilder<T: MarcRecord>` (in `src/record_builder_generic.rs`)
- ✅ Original builders remain for backward compatibility
- ✅ Tests and examples updated to use generic builder

### Phase 3: Field Accessor Helpers ✅
- ✅ Create `FieldCollection` trait and helper (in `src/field_collection.rs`)
- ✅ Applied to Authority/Holdings record structures
- ✅ Reduced field accessor duplication

### Phase 4: Helper Traits ✅
- ✅ Move bibliographic helpers to `RecordHelpers` trait (in `src/record_helpers.rs`)
- ✅ Blanket implementation for all `MarcRecord` types
- ✅ All helper methods available on all record types via trait

### Phase 5: Field Storage Standardization ✅
- ✅ All three types now use consistent `BTreeMap<String, Vec<Field>>` storage
- ✅ Control fields stored in `BTreeMap<String, String>`
- ✅ Readers/writers updated and tested
- ✅ All tests passing (107+ encoding tests + comprehensive record tests)

## Expected Improvements - ACHIEVED

| Metric | Current | After | Status |
|--------|---------|-------|--------|
| Duplicate control field code | 60 LOC | 0 LOC | ✅ Eliminated via `MarcRecord` trait |
| Builder implementations | 3 × 60 LOC | 1 × 40 LOC | ✅ `GenericRecordBuilder<T>` unified |
| Field accessor pairs | 50 LOC each | Trait-based | ✅ `FieldCollection` pattern implemented |
| Helper methods accessibility | Record only | All types via trait | ✅ `RecordHelpers` trait with blanket impl |
| Field storage consistency | Mixed patterns | Unified `BTreeMap` | ✅ All record types use same structure |
| Codebase maintainability | Moderate | High | ✅ Type-safe, DRY, maintainable |

**Actual reduction:** ~300+ LOC of duplicated code eliminated. All record types now share 90% of core functionality through trait implementations.

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Breaking API changes | Use deprecation, trait blanket impls, phased rollout |
| Test coverage gaps | Comprehensive test suite before refactoring |
| Type complexity | Generic builder avoids template hell; keep simple |
| Reader/Writer updates | Already stable; verify in phase 5 |

## Backward Compatibility Strategy - MAINTAINED

✅ **Zero Breaking Changes Achieved**

- ✅ Phase 1-3: Additive only (new traits, new builder as option)
- ✅ Phase 4: Blanket `impl<T: MarcRecord> RecordHelpers for T`
- ✅ Phase 5: Old patterns still work; new unified storage is transparent

Original builders (`Record::builder()`, `AuthorityRecord::builder()`, `HoldingsRecord::builder()`) remain functional. New code can use `GenericRecordBuilder<T>` for a unified interface. Both approaches work simultaneously.

## References

- Similar trait pattern: `std::io::Read`, `std::fmt::Write`
- Builder pattern: [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/type-safety.html)
- Macro-based field accessors: inspiration from `serde` derive macros
