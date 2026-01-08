# pymarc API Parity Plan

**Status**: In Progress  
**Date**: 2026-01-08  
**Objective**: Identify and implement gaps between mrrc's Python wrapper API and pymarc's public API to achieve drop-in replacement compatibility.

## Executive Summary

Analysis of pymarc's public API (documented in `docs/history/PYMARC_API_AUDIT.md`) against mrrc's current implementation reveals **several small but important gaps** that prevent true drop-in replacement compatibility. These gaps fall into four categories:

1. **Dict-like access patterns** - Record/Field don't support index notation for non-existent keys (return None vs KeyError)
2. **Leader value lookups** - Missing dictionary-based lookup patterns for leader byte meanings
3. **Indicator handling** - pymarc uses an `Indicators` class, mrrc uses separate `indicator1`/`indicator2` properties
4. **Field property aliases** - Missing some convenience property names that pymarc supports

## Gap Analysis

### Gap 1: Dict-like Field Access Returns None (Not KeyError)

**pymarc behavior**:
```python
field['a']           # Returns str or None (no KeyError if missing)
field['z']           # Returns None if doesn't exist
```

**Current mrrc behavior**:
```python
field['a']           # Raises KeyError if missing
```

**Impact**: Users migrating from pymarc expect `None` for missing subfields, not exceptions.

**Current Implementation** (in `mrrc/__init__.py:52-64`):
```python
def __getitem__(self, code: str) -> str:
    try:
        values = self._inner.subfields_by_code(code)
        if not values:
            raise KeyError(code)  # ← WRONG: Should return None
        return values[0]
    except Exception as e:
        raise KeyError(code) from e
```

**Fix**: Return `None` instead of raising `KeyError` when subfield not found.

### Gap 2: Dict-like Record Access Returns None (Not KeyError)

**pymarc behavior**:
```python
record['245']        # Returns Field or None
record['999']        # Returns None if tag doesn't exist
record['245']['a']   # Returns str or None if subfield missing
```

**Current mrrc behavior**:
```python
record['245']        # Implemented via Record.__getitem__ (should check implementation)
record['999']        # Should return None
```

**Status**: Need to verify current implementation handles all cases properly.

### Gap 3: Leader Position-Based Access Missing

**pymarc behavior**:
```python
leader[0:5]          # Record length (string slice)
leader[5]            # Record status (single character)
leader[18]           # Cataloging form (single character)
```

**Current mrrc behavior**:
```python
leader.record_length              # Property (integer)
leader.record_status              # Property (string)
leader.cataloging_form            # Property (string)
# No support for leader[0:5] or leader[5]
```

**Impact**: Advanced users relying on position-based access will break.

**Fix**: Implement `__getitem__` and `__setitem__` on Leader to support both:
- Single character access: `leader[5]` returns string
- Slice access: `leader[0:5]` returns string
- Maintain property-based access as primary API

### Gap 4: Leader Value Lookups (Dictionary Maps)

**pymarc provides** (implicit, not documented but supported):
- Maps for leader byte interpretations (e.g., record_status -> 'a'/'c'/'d'/'n'/'p')
- Descriptions of valid values per position

**Current mrrc behavior**:
- Properties accept/return raw values
- No helper methods for validating or describing values

**Impact**: Users can't easily discover valid values without consulting MARC 21 spec.

**Fix**: Add optional helper methods:
```python
Leader.RECORD_STATUS_VALUES  # {'a': 'increase in encoding level', ...}
Leader.get_valid_values(position)  # Returns dict of valid values
```

### Gap 5: Indicators as Object vs Separate Properties

**pymarc behavior**:
```python
Field('245', ['1', '0'], [...])        # indicators as list
field.indicators                        # Returns Indicators(ind1, ind2) object
field.indicators[0]                    # Access individual indicators
```

**Current mrrc behavior**:
```python
Field('245', '1', '0')                 # indicators as separate params
field.indicator1                       # Property
field.indicator2                       # Property
# No Indicators object
```

**Impact**: Code like `field.indicators[0]` will fail. Mostly cosmetic since properties work.

**Fix**: Consider adding optional `indicators` property that returns tuple-like object.

### Gap 6: Missing Convenience Property Names

**pymarc has** (from PYMARC_API_AUDIT.md):
- `record.issn_title()` - from 222 field
- `record.issnl()` - ISSN-L from 024 field
- `record.sudoc()` - SuDoc from 086 field
- `record.uniform_title()` - from 130 field
- `record.series()` - from 490 field
- `record.physical_description()` - from 300 field

**Current mrrc status**: ✓ All implemented

**Status**: No gap here - mrrc exceeds pymarc API.

### Gap 7: Control Field Access Pattern

**pymarc behavior**:
```python
record['001'].value       # For control fields
```

**Current mrrc behavior**:
```python
record.control_field('001')  # Returns string directly
```

**Impact**: Users expecting `record['001'].value` will get different interface.

**Fix**: Support both patterns:
- Keep `record.control_field('001')` -> string
- Also support `record['001']` -> Field-like object with `.value` property

## Implementation Plan

### Priority 1: Critical for Drop-In Replacement
1. **Field.__getitem__ returns None instead of KeyError** (Gap 1)
2. **Record.__getitem__ returns None for missing tags** (Gap 2)
3. **Leader position-based access (__getitem__, __setitem__)** (Gap 3)

### Priority 2: Important for Full Compatibility
4. **Leader value validation helpers** (Gap 4)
5. **Indicators as object property** (Gap 5)
6. **Control field access patterns** (Gap 7)

### Priority 3: Polish
7. **Documentation updates to highlight differences**
8. **Test coverage for all pymarc-compatible patterns**
9. **Migration guide improvements**

## Testing Strategy

For each gap, add tests comparing mrrc to pymarc behavior:

```python
# test_pymarc_compatibility.py
def test_field_getitem_returns_none_for_missing():
    """Field['x'] returns None, not KeyError"""
    field = mrrc.Field('245', '1', '0')
    field.add_subfield('a', 'Title')
    assert field['a'] == 'Title'
    assert field['z'] is None  # ← Key difference from current behavior

def test_record_getitem_returns_none_for_missing():
    """record['999'] returns None, not KeyError"""
    record = mrrc.Record(mrrc.Leader())
    assert record['999'] is None
    
def test_leader_position_access():
    """leader[5] returns record_status character"""
    leader = mrrc.Leader()
    leader.record_status = 'a'
    assert leader[5] == 'a'
    assert leader[5:6] == 'a'
```

## Files to Modify

- `mrrc/__init__.py` - Field, Record, Leader wrapper classes
- `src-python/python/mrrc/__init__.pyi` - Type stubs
- `docs/MIGRATION_GUIDE.md` - Document remaining differences
- `tests/` - Add comprehensive pymarc compatibility tests

## Success Criteria

✓ Field['missing'] returns None (not KeyError)  
✓ Record['missing'] returns None (not KeyError)  
✓ Leader position-based access works: leader[5], leader[0:5]  
✓ Control field access via record['001'].value works  
✓ All existing tests pass  
✓ New pymarc compatibility test suite passes  
✓ Migration guide lists any remaining differences (should be minimal)  

## References

- `docs/history/PYMARC_API_AUDIT.md` - Detailed pymarc API documentation
- `docs/MIGRATION_GUIDE.md` - Current migration guide
- `mrrc/__init__.py` - Current Python wrapper implementation
