# pymarc API Parity Issues - Summary

**Analysis Date**: 2026-01-08  
**Task**: mrrc-5mn - Revisit pymarc API compatibility for additional parity opportunities

## Overview

Comprehensive analysis of pymarc's public API (documented in `docs/history/PYMARC_API_AUDIT.md`) identified **7 distinct gaps** between mrrc's Python wrapper and pymarc's behavior. All gaps have been filed as beads issues with detailed implementation guidance.

**Design Document**: `docs/design/PYMARC_API_PARITY_PLAN.md`

---

## Issues Created

### Priority 1: Critical for Drop-In Replacement

#### mrrc-6it: Fix Field.__getitem__ to return None for missing subfields
- **Status**: Open
- **Problem**: `field['z']` raises KeyError instead of returning None
- **Impact**: Breaks pymarc compatibility for subfield access
- **Files**: `mrrc/__init__.py`, tests
- **Implementation**: Change Field.__getitem__ to return None instead of raising KeyError

#### mrrc-qkj: Implement Leader position-based access
- **Status**: Open
- **Problem**: Leader doesn't support `leader[5]` or `leader[0:5]` notation
- **Impact**: Advanced users relying on position-based access will break
- **Files**: `mrrc/__init__.py`, type stubs
- **Implementation**: Add __getitem__ and __setitem__ to Leader class

#### mrrc-7ni: Add Record.__getitem__ None-return behavior
- **Status**: Open
- **Problem**: `record['999']` might raise KeyError instead of returning None
- **Impact**: Breaks dict-like access pattern expected in pymarc
- **Files**: `mrrc/__init__.py`
- **Implementation**: Verify and fix Record.__getitem__ to return None for missing tags

#### mrrc-vyf: Create comprehensive pymarc compatibility test suite
- **Status**: Open
- **Problem**: No systematic testing of pymarc API compatibility
- **Impact**: Can't verify all compatibility patterns work correctly
- **Files**: `tests/test_compatibility/` (new)
- **Implementation**: Create 5 test modules covering field, record, leader, roundtrip, reader/writer patterns

#### mrrc-89p: Update MIGRATION_GUIDE.md with detailed API parity notes
- **Status**: Open
- **Problem**: Migration guide doesn't document subtle API differences
- **Impact**: Users don't understand when behavior differs from pymarc
- **Files**: `docs/MIGRATION_GUIDE.md`
- **Implementation**: Add detailed sections explaining differences and showing examples

---

### Priority 2: Important for Full Compatibility

#### mrrc-l0n: Add Leader value lookup helpers and validation
- **Status**: Open
- **Problem**: No way to discover or validate leader byte values per MARC 21 spec
- **Enhancement**: Add optional helper methods to describe valid values
- **Files**: `mrrc/__init__.py`, tests
- **Implementation**: Create MARC 21 spec mappings for leader positions 5-19

#### mrrc-iyk: Support Indicators as tuple-like object
- **Status**: Open
- **Problem**: pymarc uses `field.indicators[0]` but mrrc uses `field.indicator1`
- **Impact**: Syntactic incompatibility (not functional)
- **Files**: `mrrc/__init__.py`
- **Implementation**: Add indicators @property returning tuple-like Indicators object

#### mrrc-mt4: Support control field access via record['001'].value pattern
- **Status**: Open
- **Problem**: `record['001'].value` pattern not supported
- **Impact**: Users expecting this pattern will get error
- **Files**: `mrrc/__init__.py`
- **Implementation**: Return Field-like object for control fields from record['001']

---

## Issue Summary Table

| Issue | Priority | Category | Status |
|-------|----------|----------|--------|
| mrrc-6it | 1 | Field dict access | Open |
| mrrc-qkj | 1 | Leader position access | Open |
| mrrc-7ni | 1 | Record dict access | Open |
| mrrc-vyf | 1 | Test coverage | Open |
| mrrc-89p | 1 | Documentation | Open |
| mrrc-l0n | 2 | Leader helpers | Open |
| mrrc-iyk | 2 | Indicators object | Open |
| mrrc-mt4 | 2 | Control fields | Open |

---

## Implementation Order

**Recommended sequence** for maximum compatibility with minimal risk:

### Phase 1: Dict-like Access (highest impact)
1. **mrrc-vyf** - Create test suite first (define expectations)
2. **mrrc-6it** - Fix Field.__getitem__ to return None
3. **mrrc-7ni** - Fix Record.__getitem__ to return None
4. **mrrc-89p** - Update documentation with new behavior

### Phase 2: Leader Access
5. **mrrc-qkj** - Implement Leader position/slice access

### Phase 3: Polish
6. **mrrc-l0n** - Add Leader value lookup helpers
7. **mrrc-iyk** - Add Indicators tuple-like object
8. **mrrc-mt4** - Support control field .value pattern

---

## Key Design Decisions

### 1. None vs KeyError (mrrc-6it, mrrc-7ni)
**Decision**: Return None for missing keys, matching pymarc behavior

**Rationale**: 
- Allows graceful handling without try/except blocks
- Matches pymarc user expectations
- Consistent with None-based pattern throughout API

**Trade-off**: 
- Less explicit error detection
- Requires users to check for None

### 2. Leader Position-Based Access (mrrc-qkj)
**Decision**: Support BOTH property-based and position-based access

**Rationale**:
- Property-based (leader.record_status) is primary, documented API
- Position-based (leader[5]) is legacy/advanced access
- Both should stay synchronized

**Implementation**:
- Implement __getitem__ and __setitem__
- Use property getters/setters internally for sync

### 3. Indicators as Object (mrrc-iyk)
**Decision**: Optional enhancement, not required for compatibility

**Rationale**:
- Current indicator1/indicator2 properties work fine
- Indicators object is syntactic sugar
- Low priority since properties are functional

**Scope**: Nice-to-have for Priority 2

---

## Success Criteria

After all issues are resolved:

✅ **field['a']** returns str or None (not KeyError)  
✅ **record['245']** returns Field or None (not KeyError)  
✅ **record['001'].value** works for control fields  
✅ **leader[5]** and **leader[0:5]** work for position access  
✅ **Comprehensive test suite** validates all patterns  
✅ **Migration guide** explains all remaining differences (should be minimal/none)  

---

## References

- **Design Document**: `docs/design/PYMARC_API_PARITY_PLAN.md` - Detailed gap analysis
- **API Audit**: `docs/history/PYMARC_API_AUDIT.md` - pymarc API documentation
- **Migration Guide**: `docs/MIGRATION_GUIDE.md` - User-facing documentation
- **Related Task**: mrrc-5mn - Parent task to revisit API compatibility

---

## Related Discussions

All issues reference:
- **docs/design/PYMARC_API_PARITY_PLAN.md** - For design rationale
- **mrrc-5mn** - Parent task for API parity review
- Each other for implementation dependencies

---

## Timeline Estimate

**Phase 1 (Critical)**: 2-3 hours
- Test suite setup
- Field and Record __getitem__ fixes
- Documentation updates

**Phase 2 (Important)**: 1-2 hours
- Leader position access

**Phase 3 (Polish)**: 1-2 hours
- Helper methods and optional enhancements

**Total**: 4-7 hours for full API parity

---

**Created**: 2026-01-08  
**Created By**: API compatibility audit  
**Next Steps**: Start with Phase 1 issues for highest impact
