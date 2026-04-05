# Pymarc API Compatibility Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make mrrc's Python wrapper a near-drop-in replacement for pymarc by closing all verified API gaps.

**Architecture:** All changes are in the Python wrapper layer (`mrrc/__init__.py`) and test suite (`tests/python/test_pymarc_compatibility.py`). Rust internals stay unchanged except where new Rust-side methods are needed (positional subfield insert, field-level binary serialization). The type stubs (`mrrc/_mrrc.pyi`) are updated to match.

**Tech Stack:** Python 3.12+, PyO3/maturin, Rust, pytest

**Bead:** bd-sgwi | **GitHub Issue:** #71

---

## Task 1: Unify ControlField into Field

Currently `ControlField` is a standalone class. pymarc has no separate ControlField — both control and data fields are `pymarc.Field`, distinguished by `is_control_field()`. This breaks `isinstance(f, Field)` checks.

**Files:**
- Modify: `mrrc/__init__.py:106-131` (ControlField class)
- Modify: `mrrc/__init__.py:133-339` (Field class)
- Modify: `mrrc/__init__.py:666-687` (Record.__getitem__)
- Modify: `mrrc/__init__.py:688-710` (Record.get_fields)
- Modify: `mrrc/__init__.py:757-763` (Record.add_control_field, control_field)
- Modify: `mrrc/__init__.py:1575-1626` (__all__)
- Modify: `tests/python/test_pymarc_compatibility.py`
- Modify: `mrrc/_mrrc.pyi`

- [ ] **Step 1: Write failing tests for unified Field behavior**

Add a new test class in `tests/python/test_pymarc_compatibility.py`:

```python
class TestFieldUnification:
    """Test that ControlField is a subclass of Field (pymarc compatibility)."""

    def test_control_field_is_field_instance(self):
        """Control fields should be instances of Field (pymarc has no separate ControlField)."""
        field = Field('001', data='12345')
        assert isinstance(field, Field)

    def test_control_field_is_control_field(self):
        """Control fields identified by is_control_field()."""
        cf = Field('001', data='12345')
        df = Field('245', '1', '0')
        assert cf.is_control_field() is True
        assert df.is_control_field() is False

    def test_control_field_data_attribute(self):
        """Control fields store content in .data (pymarc compatibility)."""
        field = Field('001', data='12345')
        assert field.data == '12345'

    def test_control_field_tag(self):
        """Control fields have .tag attribute."""
        field = Field('008', data='some-fixed-length-data')
        assert field.tag == '008'

    def test_control_field_no_indicators(self):
        """Control fields should not have meaningful indicators."""
        field = Field('001', data='12345')
        assert field.is_control_field() is True

    def test_data_field_no_data_attribute(self):
        """Data fields should not have .data attribute."""
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Title')
        assert not hasattr(field, 'data') or field.data is None

    def test_legacy_control_field_class_still_works(self):
        """ControlField class should still be importable and work."""
        from mrrc import ControlField
        cf = ControlField('001', '12345')
        assert isinstance(cf, Field)
        assert cf.is_control_field() is True
        assert cf.data == '12345'

    def test_record_getitem_returns_field_for_control(self):
        """Record['001'] should return a Field, not a separate ControlField."""
        record = Record(Leader())
        record.add_control_field('001', '12345')
        field = record['001']
        assert isinstance(field, Field)
        assert field.is_control_field() is True
        assert field.data == '12345'

    def test_get_fields_returns_uniform_type(self):
        """get_fields() should return Field instances for both control and data fields."""
        record = Record(Leader())
        record.add_control_field('001', '12345')
        record.add_field(Field('245', '1', '0', subfields=[Subfield('a', 'Title')]))
        all_fields = record.get_fields()
        for f in all_fields:
            assert isinstance(f, Field)
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestFieldUnification -v`
Expected: Multiple failures — `Field('001', data='12345')` not supported, `isinstance` checks fail

- [ ] **Step 3: Implement unified Field class**

In `mrrc/__init__.py`, modify the `Field` class to handle both control and data fields:

```python
class Field:
    """Enhanced Field wrapper with pymarc-compatible API.

    Handles both control fields (001-009) and data fields (010+).
    Control fields are distinguished by is_control_field() and store
    content in .data. Data fields have indicators and subfields.
    """

    def __init__(self, tag: str, indicator1: str = ' ', indicator2: str = ' ',
                 *, subfields=None, indicators=None, data=None):
        """Create a new Field.

        For control fields (001-009), use the data parameter:
            Field('001', data='12345')

        For data fields, use indicators and subfields:
            Field('245', '1', '0', subfields=[Subfield('a', 'Title')])

        Args:
            tag: 3-character field tag.
            indicator1: First indicator (default ' ').
            indicator2: Second indicator (default ' ').
            subfields: Optional list of Subfield objects to add.
            indicators: Optional list/tuple of [ind1, ind2], overrides indicator1/indicator2.
            data: Content string for control fields (001-009).
        """
        self._data = data
        if data is not None:
            # Control field — create a minimal inner Field (won't use indicators/subfields)
            self._inner = _Field(tag, ' ', ' ')
        else:
            self._inner = _Field(tag, indicator1, indicator2,
                                 subfields=subfields, indicators=indicators)

    @property
    def data(self):
        """Control field content (pymarc compatibility). None for data fields."""
        return self._data

    @data.setter
    def data(self, value):
        """Set control field content."""
        self._data = value

    def is_control_field(self) -> bool:
        """Check if this is a control field (tag 001-009)."""
        return self._data is not None or self.tag < '010'

    # ... rest of existing Field methods unchanged ...
```

Update `ControlField` to be a thin subclass for backward compatibility:

```python
class ControlField(Field):
    """Backward-compatible alias. In pymarc, both control and data fields are Field."""

    def __init__(self, tag: str, value: str):
        super().__init__(tag, data=value)
```

Update `Record.__getitem__` to return unified Field for control fields:

```python
def __getitem__(self, tag: str):
    """Get first field with given tag (pymarc compatibility).

    Raises KeyError if the tag doesn't exist (matching pymarc behavior).
    """
    if tag < '010':
        value = self._inner.control_field(tag)
        if value is not None:
            return Field(tag, data=value)
        raise KeyError(tag)

    field = self._inner.get_field(tag)
    if field:
        wrapper = Field.__new__(Field)
        wrapper._inner = field
        wrapper._data = None
        return wrapper
    raise KeyError(tag)
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestFieldUnification -v`
Expected: All pass

- [ ] **Step 5: Update existing tests that assume old ControlField or None-return behavior**

Update tests that use `ControlField` directly or assert `record['xxx'] is None`:

In `test_control_field_dict_access` (line 95): change `isinstance(field_001, ControlField)` to `isinstance(field_001, Field)` and `field_001.value` to `field_001.data`.

In `test_control_field_value_property` (line 107): change `.value` to `.data`.

In `test_control_field_backward_compat` (line 115): change `.value` references to `.data`.

In `test_missing_control_field_returns_none` (line 124): change to expect `KeyError`:
```python
def test_missing_control_field_raises_keyerror(self):
    """Test that missing control fields raise KeyError via dict access."""
    record = Record(Leader())
    with pytest.raises(KeyError):
        record['001']
    with pytest.raises(KeyError):
        record['008']
```

In `test_record_getitem_missing_tag` (line 606): change to expect `KeyError`:
```python
def test_record_getitem_missing_tag(self):
    """Test Record.__getitem__ raises KeyError for missing tag (pymarc compatibility)."""
    record = Record(Leader())
    with pytest.raises(KeyError):
        record['999']
    with pytest.raises(KeyError):
        record['245']
```

- [ ] **Step 6: Run full test suite to verify no regressions**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py -v`
Expected: All pass

- [ ] **Step 7: Commit**

```bash
git add mrrc/__init__.py tests/python/test_pymarc_compatibility.py
git commit -m "Unify ControlField into Field for pymarc compatibility

ControlField is now a subclass of Field. Control fields use .data
attribute (matching pymarc). Record.__getitem__ raises KeyError for
missing tags. is_control_field() dispatches based on tag/data."
```

---

## Task 2: Field.__str__ and __repr__

pymarc `str(field)` returns `=245  10$aThe Great Gatsby$cF. Scott Fitzgerald` for data fields, and `=001  12345` for control fields. mrrc returns the default Python repr because `__getattr__` delegation doesn't intercept dunder methods.

**Files:**
- Modify: `mrrc/__init__.py` (Field class)
- Modify: `tests/python/test_pymarc_compatibility.py`

- [ ] **Step 1: Write failing tests**

```python
class TestFieldStringRepresentation:
    """Test Field.__str__ and __repr__ match pymarc format."""

    def test_data_field_str(self):
        """str(field) returns pymarc MARC display format for data fields."""
        field = Field('245', '1', '0', subfields=[
            Subfield('a', 'The Great Gatsby'),
            Subfield('c', 'F. Scott Fitzgerald'),
        ])
        assert str(field) == '=245  10$aThe Great Gatsby$cF. Scott Fitzgerald'

    def test_control_field_str(self):
        """str(field) returns pymarc format for control fields."""
        field = Field('001', data='12345')
        assert str(field) == '=001  12345'

    def test_data_field_str_blank_indicators(self):
        """Blank indicators display as spaces."""
        field = Field('650', ' ', '0', subfields=[Subfield('a', 'Python')])
        assert str(field) == '=650  \\0$aPython'

    def test_field_repr(self):
        """repr(field) should be informative."""
        field = Field('245', '1', '0', subfields=[Subfield('a', 'Title')])
        r = repr(field)
        assert '245' in r

    def test_control_field_repr(self):
        """repr for control field."""
        field = Field('001', data='12345')
        r = repr(field)
        assert '001' in r
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestFieldStringRepresentation -v`
Expected: FAIL

- [ ] **Step 3: Implement __str__ and __repr__ on Field**

Add to the `Field` class in `mrrc/__init__.py`:

```python
def __str__(self) -> str:
    """MARC display format (pymarc compatibility).

    Data fields: =TAG  IND1IND2$aCONTENT$bCONTENT
    Control fields: =TAG  CONTENT
    """
    if self.is_control_field():
        return f'={self.tag}  {self.data}'
    ind1 = self.indicator1.replace(' ', '\\')
    ind2 = self.indicator2.replace(' ', '\\')
    subfield_str = ''.join(f'${sf.code}{sf.value}' for sf in self.subfields())
    return f'={self.tag}  {ind1}{ind2}{subfield_str}'

def __repr__(self) -> str:
    """Informative repr."""
    if self.is_control_field():
        return f"<Field {self.tag}={self.data!r}>"
    return f"<Field {self.tag} {self.indicator1}{self.indicator2} {len(self.subfields())} subfields>"
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestFieldStringRepresentation -v`
Expected: All pass

- [ ] **Step 5: Commit**

```bash
git add mrrc/__init__.py tests/python/test_pymarc_compatibility.py
git commit -m "Add pymarc-compatible Field.__str__ and __repr__

Data fields: =245  10\$aTitle\$cAuthor
Control fields: =001  12345"
```

---

## Task 3: Record accessors as @property with pymarc-compatible names

pymarc uses `@property` for all 17 record accessors (`record.title`, not `record.title()`). mrrc also needs pymarc-compatible name aliases (`physicaldescription`, `uniformtitle`) and the missing `addedentries`.

**Files:**
- Modify: `mrrc/__init__.py:774-833` (Record convenience methods)
- Modify: `tests/python/test_pymarc_compatibility.py`
- Modify: `mrrc/_mrrc.pyi`

- [ ] **Step 1: Write failing tests**

```python
class TestRecordPropertyAccessors:
    """Test that Record accessors are @property (pymarc compatibility)."""

    def test_title_is_property(self):
        """record.title should be a property, not a method."""
        record = Record(fields=[Field('245', '1', '0', subfields=[Subfield('a', 'Test Title')])])
        assert record.title == 'Test Title'

    def test_author_is_property(self):
        record = Record(fields=[Field('100', '1', ' ', subfields=[Subfield('a', 'Author, Test')])])
        assert 'Author' in record.author

    def test_isbn_is_property(self):
        record = Record(fields=[Field('020', ' ', ' ', subfields=[Subfield('a', '0201616165')])])
        assert record.isbn == '0201616165'

    def test_issn_is_property(self):
        record = Record(fields=[Field('022', ' ', ' ', subfields=[Subfield('a', '0028-0836')])])
        assert record.issn == '0028-0836'

    def test_subjects_is_property(self):
        record = Record(fields=[
            Field('650', ' ', '0', subfields=[Subfield('a', 'Subject 1')]),
            Field('650', ' ', '0', subfields=[Subfield('a', 'Subject 2')]),
        ])
        assert len(record.subjects) == 2

    def test_publisher_is_property(self):
        record = Record(fields=[Field('260', ' ', ' ', subfields=[Subfield('b', 'Publisher')])])
        assert 'Publisher' in record.publisher

    def test_pubyear_returns_string(self):
        """pubyear should return str, not int (pymarc compatibility)."""
        record = Record(fields=[Field('260', ' ', ' ', subfields=[Subfield('c', '2023')])])
        result = record.pubyear
        assert result == '2023'
        assert isinstance(result, str)

    def test_physicaldescription_alias(self):
        """physicaldescription (no underscore) should work (pymarc name)."""
        record = Record(fields=[Field('300', ' ', ' ', subfields=[Subfield('a', '256 pages')])])
        assert record.physicaldescription is not None

    def test_uniformtitle_alias(self):
        """uniformtitle (no underscore) should work (pymarc name)."""
        record = Record(fields=[Field('130', ' ', '0', subfields=[Subfield('a', 'Uniform')])])
        assert record.uniformtitle is not None

    def test_addedentries(self):
        """addedentries should return 700/710/711/730 fields."""
        record = Record(fields=[
            Field('700', '1', ' ', subfields=[Subfield('a', 'Co-Author')]),
            Field('710', '2', ' ', subfields=[Subfield('a', 'Corp')]),
        ])
        entries = record.addedentries
        assert len(entries) == 2

    def test_notes_is_property(self):
        record = Record(fields=[Field('500', ' ', ' ', subfields=[Subfield('a', 'A note')])])
        assert 'A note' in record.notes

    def test_location_is_property(self):
        record = Record(fields=[Field('852', ' ', ' ', subfields=[Subfield('a', 'Library')])])
        assert 'Library' in record.location

    def test_series_is_property(self):
        record = Record(fields=[Field('490', ' ', ' ', subfields=[Subfield('a', 'Series')])])
        assert record.series is not None

    def test_sudoc_is_property(self):
        record = Record(fields=[Field('086', ' ', ' ', subfields=[Subfield('a', 'I 19.2')])])
        assert record.sudoc == 'I 19.2'

    def test_issn_title_is_property(self):
        record = Record(fields=[Field('222', ' ', ' ', subfields=[Subfield('a', 'Key Title')])])
        assert 'Key Title' in record.issn_title

    def test_issnl_is_property(self):
        record = Record(fields=[Field('024', ' ', ' ', subfields=[Subfield('a', '1234-5678')])])
        # issnl may return None if 024 doesn't have the right indicator; just test it's a property
        _ = record.issnl
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestRecordPropertyAccessors -v`
Expected: FAIL — `record.title` returns a bound method, not the value

- [ ] **Step 3: Convert Record convenience methods to @property**

In `mrrc/__init__.py`, replace the method definitions with properties:

```python
@property
def title(self) -> Optional[str]:
    """Title from 245 field."""
    return self._inner.title()

@property
def author(self) -> Optional[str]:
    """Author from 100/110/111 field."""
    return self._inner.author()

@property
def isbn(self) -> Optional[str]:
    """ISBN from 020 field."""
    return self._inner.isbn()

@property
def issn(self) -> Optional[str]:
    """ISSN from 022 field."""
    return self._inner.issn()

@property
def subjects(self) -> List[str]:
    """Subject headings from 6XX fields."""
    return self._inner.subjects()

@property
def location(self) -> List[str]:
    """Location fields (852)."""
    return self._inner.location()

@property
def notes(self) -> List[str]:
    """Notes from 5xx fields."""
    return self._inner.notes()

@property
def publisher(self) -> Optional[str]:
    """Publisher from 260 or 264 field."""
    return self._inner.publisher()

@property
def uniform_title(self) -> Optional[str]:
    """Uniform title from 130 field."""
    return self._inner.uniform_title()

@property
def sudoc(self) -> Optional[str]:
    """SuDoc from 086 field."""
    return self._inner.sudoc()

@property
def issn_title(self) -> Optional[str]:
    """ISSN title from 222 field."""
    return self._inner.issn_title()

@property
def issnl(self) -> Optional[str]:
    """ISSN-L from 024 field."""
    return self._inner.issnl()

@property
def pubyear(self) -> Optional[str]:
    """Publication year (returns str, matching pymarc)."""
    result = self._inner.pubyear()
    return str(result) if result is not None else None

@property
def series(self) -> Optional[str]:
    """Series from 490 field."""
    return self._inner.series()

@property
def physical_description(self) -> Optional[str]:
    """Physical description from 300 field."""
    return self._inner.physical_description()

# pymarc-compatible name aliases (no underscores)
@property
def physicaldescription(self) -> Optional[str]:
    """Physical description (pymarc-compatible name)."""
    return self.physical_description

@property
def uniformtitle(self) -> Optional[str]:
    """Uniform title (pymarc-compatible name)."""
    return self.uniform_title

@property
def addedentries(self) -> List['Field']:
    """Added entries from 700/710/711/730 fields (pymarc compatibility)."""
    return self.get_fields('700', '710', '711', '730')
```

- [ ] **Step 4: Update existing tests that call accessors as methods**

The existing `TestConvenienceMethods` class (line 257) calls these as methods like `record.title()`. Update all of them to use property syntax: `record.title`.

For example, change `assert record.title() == 'Test Title'` to `assert record.title == 'Test Title'` throughout the class. Same for `author`, `isbn`, `issn`, `publisher`, `subjects`, `location`, `notes`, `series`, `physical_description`, `uniform_title`, `sudoc`, `issn_title`, `pubyear`.

Also update `pubyear` test to expect `str`:
```python
def test_pubyear(self):
    """Test pubyear property."""
    record = Record(Leader())
    record.add_field(create_field('260', ' ', ' ', c='2023'))
    year = record.pubyear
    assert year == '2023'
```

Also update `TestConstructorKwargs` tests (line 958, 974, 998) that call `record.title()` to `record.title`.

- [ ] **Step 5: Run full test suite**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py -v`
Expected: All pass

- [ ] **Step 6: Commit**

```bash
git add mrrc/__init__.py tests/python/test_pymarc_compatibility.py
git commit -m "Convert Record accessors to @property for pymarc compatibility

All 17 accessors (title, author, isbn, etc.) are now properties.
pubyear returns str instead of int. Added physicaldescription,
uniformtitle, and addedentries aliases."
```

---

## Task 4: add_field(*fields), remove_field(*fields), remove_fields(*tags)

pymarc's `add_field` and `remove_field` accept multiple arguments. pymarc also has `remove_fields(*tags)` for bulk removal by tag. pymarc returns None from removal methods.

**Files:**
- Modify: `mrrc/__init__.py:712-755` (Record.add_field, remove_field)
- Modify: `tests/python/test_pymarc_compatibility.py`

- [ ] **Step 1: Write failing tests**

```python
class TestBulkFieldOperations:
    """Test add_field/remove_field accept multiple args (pymarc compatibility)."""

    def test_add_multiple_fields(self):
        """add_field() should accept multiple fields."""
        record = Record()
        f1 = Field('245', '1', '0', subfields=[Subfield('a', 'Title')])
        f2 = Field('100', '1', ' ', subfields=[Subfield('a', 'Author')])
        f3 = Field('650', ' ', '0', subfields=[Subfield('a', 'Subject')])
        record.add_field(f1, f2, f3)
        assert len(record.get_fields('245')) == 1
        assert len(record.get_fields('100')) == 1
        assert len(record.get_fields('650')) == 1

    def test_add_field_single_still_works(self):
        """add_field() with one arg should still work."""
        record = Record()
        record.add_field(Field('245', '1', '0', subfields=[Subfield('a', 'Title')]))
        assert record.title == 'Title'

    def test_remove_field_by_object(self):
        """remove_field() should accept Field objects (pymarc compatibility)."""
        record = Record(fields=[
            Field('245', '1', '0', subfields=[Subfield('a', 'Title')]),
            Field('650', ' ', '0', subfields=[Subfield('a', 'Subject')]),
        ])
        f245 = record['245']
        record.remove_field(f245)
        assert '245' not in record

    def test_remove_field_multiple(self):
        """remove_field() should accept multiple Field objects."""
        record = Record(fields=[
            Field('245', '1', '0', subfields=[Subfield('a', 'Title')]),
            Field('650', ' ', '0', subfields=[Subfield('a', 'Subject')]),
        ])
        f245 = record['245']
        f650 = record['650']
        record.remove_field(f245, f650)
        assert '245' not in record
        assert '650' not in record

    def test_remove_field_returns_none(self):
        """remove_field() should return None (pymarc compatibility)."""
        record = Record(fields=[
            Field('245', '1', '0', subfields=[Subfield('a', 'Title')]),
        ])
        result = record.remove_field(record['245'])
        assert result is None

    def test_remove_fields_by_tags(self):
        """remove_fields() removes all fields with given tags."""
        record = Record(fields=[
            Field('245', '1', '0', subfields=[Subfield('a', 'Title')]),
            Field('650', ' ', '0', subfields=[Subfield('a', 'Subj 1')]),
            Field('650', ' ', '0', subfields=[Subfield('a', 'Subj 2')]),
            Field('700', '1', ' ', subfields=[Subfield('a', 'Author')]),
        ])
        record.remove_fields('650', '700')
        assert '650' not in record
        assert '700' not in record
        assert '245' in record

    def test_remove_fields_returns_none(self):
        """remove_fields() returns None."""
        record = Record(fields=[
            Field('245', '1', '0', subfields=[Subfield('a', 'Title')]),
        ])
        result = record.remove_fields('245')
        assert result is None
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestBulkFieldOperations -v`
Expected: FAIL

- [ ] **Step 3: Implement bulk field operations**

In `mrrc/__init__.py`, replace `add_field` and `remove_field`:

```python
def add_field(self, *fields: 'Field') -> None:
    """Add one or more fields to the record."""
    for field in fields:
        if field.is_control_field():
            self._inner.add_control_field(field.tag, field.data)
        else:
            self._inner.add_field(field._inner)

def remove_field(self, *fields: 'Field') -> None:
    """Remove one or more fields from the record (pymarc compatibility).

    Accepts Field objects. Removes by tag.
    """
    for field in fields:
        self._inner.remove_field(field.tag)

def remove_fields(self, *tags: str) -> None:
    """Remove all fields with the given tags (pymarc compatibility)."""
    for tag in tags:
        self._inner.remove_field(tag)
```

- [ ] **Step 4: Update existing tests**

Update `test_remove_field` (line 145) to not expect a return value:
```python
def test_remove_field(self):
    """Test removing a specific field."""
    record = Record(Leader())
    field = create_field('245', '1', '0', a='Title')
    record.add_field(field)
    assert record.get_field('245') is not None
    record.remove_field(field)
    assert record.get_field('245') is None
```

Update `TestRecordAdvanced.test_record_remove_field` (line 634) similarly.

- [ ] **Step 5: Run full test suite**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py -v`
Expected: All pass

- [ ] **Step 6: Commit**

```bash
git add mrrc/__init__.py tests/python/test_pymarc_compatibility.py
git commit -m "Accept *args in add_field/remove_field, add remove_fields

add_field(*fields) and remove_field(*fields) now accept multiple
arguments. Added remove_fields(*tags) for bulk removal by tag.
Both removal methods return None, matching pymarc."
```

---

## Task 5: add_ordered_field() and add_grouped_field()

pymarc has both. `add_ordered_field` inserts maintaining tag sort order. `add_grouped_field` inserts after the last field with the same tag.

**Files:**
- Modify: `mrrc/__init__.py` (Record class)
- Modify: `tests/python/test_pymarc_compatibility.py`

- [ ] **Step 1: Write failing tests**

```python
class TestOrderedFieldInsertion:
    """Test add_ordered_field and add_grouped_field (pymarc compatibility)."""

    def test_add_ordered_field(self):
        """add_ordered_field inserts field in tag-sorted position."""
        record = Record(fields=[
            Field('100', '1', ' ', subfields=[Subfield('a', 'Author')]),
            Field('650', ' ', '0', subfields=[Subfield('a', 'Subject')]),
        ])
        f245 = Field('245', '1', '0', subfields=[Subfield('a', 'Title')])
        record.add_ordered_field(f245)
        tags = [f.tag for f in record.get_fields()]
        assert tags == ['100', '245', '650']

    def test_add_ordered_field_at_end(self):
        """add_ordered_field appends when tag is highest."""
        record = Record(fields=[
            Field('100', '1', ' ', subfields=[Subfield('a', 'Author')]),
        ])
        f650 = Field('650', ' ', '0', subfields=[Subfield('a', 'Subject')])
        record.add_ordered_field(f650)
        tags = [f.tag for f in record.get_fields()]
        assert tags == ['100', '650']

    def test_add_grouped_field(self):
        """add_grouped_field inserts after last field with same tag."""
        record = Record(fields=[
            Field('650', ' ', '0', subfields=[Subfield('a', 'Subject 1')]),
            Field('650', ' ', '0', subfields=[Subfield('a', 'Subject 2')]),
            Field('700', '1', ' ', subfields=[Subfield('a', 'Author')]),
        ])
        f650 = Field('650', ' ', '0', subfields=[Subfield('a', 'Subject 3')])
        record.add_grouped_field(f650)
        tags = [f.tag for f in record.get_fields()]
        assert tags == ['650', '650', '650', '700']

    def test_add_grouped_field_no_existing(self):
        """add_grouped_field falls back to add_ordered_field when no match."""
        record = Record(fields=[
            Field('100', '1', ' ', subfields=[Subfield('a', 'Author')]),
            Field('650', ' ', '0', subfields=[Subfield('a', 'Subject')]),
        ])
        f245 = Field('245', '1', '0', subfields=[Subfield('a', 'Title')])
        record.add_grouped_field(f245)
        tags = [f.tag for f in record.get_fields()]
        assert tags == ['100', '245', '650']
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestOrderedFieldInsertion -v`
Expected: FAIL — methods don't exist

- [ ] **Step 3: Implement add_ordered_field and add_grouped_field**

These need to work at the Python level since we can't easily do positional insertion via the Rust API. They rebuild the internal field list to achieve ordering. Add to the `Record` class:

```python
def _rebuild_fields(self, field_list):
    """Replace all data fields with the given list (internal helper).

    Removes all existing data fields from the inner record, then re-adds
    them in the order specified. Control fields are left untouched since
    they're stored separately in the Rust layer.
    """
    # Collect all existing tags and remove them
    existing_tags = set(f.tag for f in self._inner.fields())
    for tag in existing_tags:
        self._inner.remove_field(tag)
    # Re-add in order
    for f in field_list:
        self._inner.add_field(f)

def add_ordered_field(self, *fields: 'Field') -> None:
    """Add fields maintaining tag sort order (pymarc compatibility).

    Inserts each field at the position that maintains ascending tag order.
    """
    for field in fields:
        if field.is_control_field():
            self._inner.add_control_field(field.tag, field.data)
        else:
            existing = list(self._inner.fields())
            insert_idx = len(existing)
            for i, f in enumerate(existing):
                if f.tag > field.tag:
                    insert_idx = i
                    break
            existing.insert(insert_idx, field._inner)
            self._rebuild_fields(existing)

def add_grouped_field(self, *fields: 'Field') -> None:
    """Add fields after the last field with the same tag (pymarc compatibility).

    If no field with the same tag exists, falls back to add_ordered_field.
    """
    for field in fields:
        if field.is_control_field():
            self._inner.add_control_field(field.tag, field.data)
            continue
        existing = list(self._inner.fields())
        # Find last field with same tag
        last_idx = None
        for i, f in enumerate(existing):
            if f.tag == field.tag:
                last_idx = i
        if last_idx is None:
            self.add_ordered_field(field)
        else:
            existing.insert(last_idx + 1, field._inner)
            self._rebuild_fields(existing)
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestOrderedFieldInsertion -v`
Expected: All pass

- [ ] **Step 5: Commit**

```bash
git add mrrc/__init__.py tests/python/test_pymarc_compatibility.py
git commit -m "Add add_ordered_field and add_grouped_field for pymarc compat

add_ordered_field inserts maintaining tag sort order.
add_grouped_field inserts after the last field with the same tag,
falling back to add_ordered_field when no match exists."
```

---

## Task 6: Field.value() and Field.format_field()

pymarc `field.value()` returns space-joined subfield values (or `.data` for control fields). `field.format_field()` returns human-readable text without indicators or subfield codes.

**Files:**
- Modify: `mrrc/__init__.py` (Field class)
- Modify: `tests/python/test_pymarc_compatibility.py`

- [ ] **Step 1: Write failing tests**

```python
class TestFieldValueMethods:
    """Test Field.value() and Field.format_field() (pymarc compatibility)."""

    def test_value_data_field(self):
        """value() returns space-joined subfield values for data fields."""
        field = Field('245', '1', '0', subfields=[
            Subfield('a', 'The Great Gatsby'),
            Subfield('c', 'F. Scott Fitzgerald'),
        ])
        assert field.value() == 'The Great Gatsby F. Scott Fitzgerald'

    def test_value_control_field(self):
        """value() returns .data for control fields."""
        field = Field('001', data='12345')
        assert field.value() == '12345'

    def test_value_single_subfield(self):
        """value() with single subfield returns just that value."""
        field = Field('020', ' ', ' ', subfields=[Subfield('a', '0201616165')])
        assert field.value() == '0201616165'

    def test_format_field_data(self):
        """format_field() returns human-readable text for data fields."""
        field = Field('245', '1', '0', subfields=[
            Subfield('a', 'The Great Gatsby'),
            Subfield('c', 'F. Scott Fitzgerald'),
        ])
        assert field.format_field() == 'The Great Gatsby F. Scott Fitzgerald'

    def test_format_field_control(self):
        """format_field() returns data for control fields."""
        field = Field('001', data='12345')
        assert field.format_field() == '12345'
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestFieldValueMethods -v`
Expected: FAIL

- [ ] **Step 3: Implement value() and format_field()**

Add to the `Field` class in `mrrc/__init__.py`:

```python
def value(self) -> str:
    """Return the field's value (pymarc compatibility).

    For control fields, returns the data content.
    For data fields, returns space-joined subfield values.
    """
    if self.is_control_field():
        return self.data or ''
    return ' '.join(sf.value for sf in self.subfields())

def format_field(self) -> str:
    """Return human-readable text without indicators or subfield codes (pymarc compatibility).

    For control fields, returns the data content.
    For data fields, returns space-joined subfield values.
    """
    if self.is_control_field():
        return self.data or ''
    return ' '.join(sf.value for sf in self.subfields())
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestFieldValueMethods -v`
Expected: All pass

- [ ] **Step 5: Commit**

```bash
git add mrrc/__init__.py tests/python/test_pymarc_compatibility.py
git commit -m "Add Field.value() and Field.format_field() for pymarc compat

value() returns space-joined subfield values for data fields, .data
for control fields. format_field() returns human-readable text."
```

---

## Task 7: Field.add_subfield with positional insertion

pymarc `add_subfield(code, value, pos=None)` supports inserting at a specific position.

**Files:**
- Modify: `mrrc/__init__.py` (Field.add_subfield)
- Modify: `tests/python/test_pymarc_compatibility.py`

- [ ] **Step 1: Write failing tests**

```python
class TestSubfieldPositionalInsert:
    """Test Field.add_subfield(code, value, pos=N) (pymarc compatibility)."""

    def test_add_subfield_default_appends(self):
        """add_subfield without pos appends to end."""
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Title')
        field.add_subfield('c', 'Author')
        subs = field.subfields()
        assert subs[0].code == 'a'
        assert subs[1].code == 'c'

    def test_add_subfield_at_position(self):
        """add_subfield with pos inserts at that position."""
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Title')
        field.add_subfield('c', 'Author')
        field.add_subfield('b', 'Subtitle', pos=1)
        subs = field.subfields()
        assert subs[0].code == 'a'
        assert subs[1].code == 'b'
        assert subs[2].code == 'c'

    def test_add_subfield_at_zero(self):
        """add_subfield with pos=0 inserts at beginning."""
        field = Field('245', '1', '0')
        field.add_subfield('c', 'Author')
        field.add_subfield('a', 'Title', pos=0)
        subs = field.subfields()
        assert subs[0].code == 'a'
        assert subs[1].code == 'c'
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestSubfieldPositionalInsert -v`
Expected: FAIL on pos= parameter tests

- [ ] **Step 3: Implement positional insertion**

The Rust `add_subfield` only appends. We need to handle positional insertion at the Python level by rebuilding the subfield list. Modify `Field.add_subfield` in `mrrc/__init__.py`:

```python
def add_subfield(self, code: str, value: str, pos: Optional[int] = None) -> None:
    """Add a subfield, optionally at a specific position (pymarc compatibility).

    Args:
        code: Subfield code (single character).
        value: Subfield value.
        pos: Optional position to insert at. If None, appends to end.
    """
    if pos is None:
        self._inner.add_subfield(code, value)
    else:
        # Get current subfields, insert at position, rebuild
        current = list(self._inner.subfields())
        new_sf = Subfield(code, value)

        # Clear existing subfields by rebuilding the field
        tag = self._inner.tag
        ind1 = self._inner.indicator1
        ind2 = self._inner.indicator2
        self._inner = _Field(tag, ind1, ind2)

        # Re-add with insertion
        current.insert(pos, new_sf)
        for sf in current:
            self._inner.add_subfield(sf.code, sf.value)
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestSubfieldPositionalInsert -v`
Expected: All pass

- [ ] **Step 5: Commit**

```bash
git add mrrc/__init__.py tests/python/test_pymarc_compatibility.py
git commit -m "Add positional insertion to Field.add_subfield

add_subfield(code, value, pos=N) inserts at position N.
Without pos, appends to end (unchanged behavior)."
```

---

## Task 8: Field.linkage_occurrence_num()

pymarc extracts the occurrence number from subfield $6 linkage (e.g., `880-03` → `'03'`).

**Files:**
- Modify: `mrrc/__init__.py` (Field class)
- Modify: `tests/python/test_pymarc_compatibility.py`

- [ ] **Step 1: Write failing tests**

```python
class TestFieldLinkage:
    """Test Field.linkage_occurrence_num() (pymarc compatibility)."""

    def test_linkage_occurrence_num(self):
        """Extract occurrence number from $6 linkage."""
        field = Field('245', '1', '0', subfields=[
            Subfield('6', '880-03'),
            Subfield('a', 'Title'),
        ])
        assert field.linkage_occurrence_num() == '03'

    def test_linkage_occurrence_num_no_subfield_6(self):
        """Return None when no $6 subfield."""
        field = Field('245', '1', '0', subfields=[Subfield('a', 'Title')])
        assert field.linkage_occurrence_num() is None

    def test_linkage_occurrence_num_no_dash(self):
        """Return None when $6 has no dash."""
        field = Field('245', '1', '0', subfields=[
            Subfield('6', '880'),
            Subfield('a', 'Title'),
        ])
        assert field.linkage_occurrence_num() is None
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestFieldLinkage -v`
Expected: FAIL

- [ ] **Step 3: Implement linkage_occurrence_num()**

Add to the `Field` class:

```python
def linkage_occurrence_num(self) -> Optional[str]:
    """Extract the occurrence number from subfield $6 linkage (pymarc compatibility).

    Returns the occurrence number portion after the dash in $6
    (e.g., '880-03' → '03'), or None if not present.
    """
    if self.is_control_field():
        return None
    sub6 = self['6']
    if sub6 is None:
        return None
    if '-' not in sub6:
        return None
    parts = sub6.split('-', 1)
    occ = parts[1]
    # Strip any additional data after the occurrence number (e.g., script codes)
    if '/' in occ:
        occ = occ.split('/')[0]
    return occ if occ else None
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestFieldLinkage -v`
Expected: All pass

- [ ] **Step 5: Commit**

```bash
git add mrrc/__init__.py tests/python/test_pymarc_compatibility.py
git commit -m "Add Field.linkage_occurrence_num() for pymarc compat

Extracts occurrence number from $6 linkage subfield."
```

---

## Task 9: Record.as_marc() / as_marc21()

pymarc `record.as_marc()` returns ISO 2709 bytes. Rust already has `to_marc21()` returning `Vec<u8>`. Just needs pymarc-compatible method names.

**Files:**
- Modify: `mrrc/__init__.py` (Record class)
- Modify: `tests/python/test_pymarc_compatibility.py`

- [ ] **Step 1: Write failing tests**

```python
class TestRecordBinarySerialization:
    """Test Record.as_marc() / as_marc21() (pymarc compatibility)."""

    def test_as_marc_returns_bytes(self):
        """as_marc() returns bytes (ISO 2709)."""
        record = Record(fields=[
            Field('245', '1', '0', subfields=[Subfield('a', 'Test Title')]),
        ])
        record.add_control_field('001', 'test-id')
        result = record.as_marc()
        assert isinstance(result, bytes)
        assert len(result) > 0

    def test_as_marc21_alias(self):
        """as_marc21() is an alias for as_marc()."""
        record = Record(fields=[
            Field('245', '1', '0', subfields=[Subfield('a', 'Test')]),
        ])
        record.add_control_field('001', 'test-id')
        assert record.as_marc() == record.as_marc21()

    def test_as_marc_roundtrip(self):
        """as_marc() output can be read back by MARCReader."""
        import io
        record = Record(fields=[
            Field('245', '1', '0', subfields=[Subfield('a', 'Roundtrip Test')]),
        ])
        record.add_control_field('001', 'rt-001')
        marc_bytes = record.as_marc()
        reader = MARCReader(io.BytesIO(marc_bytes))
        recovered = next(reader)
        assert recovered.title == 'Roundtrip Test'
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestRecordBinarySerialization -v`
Expected: FAIL — `as_marc()` not defined

- [ ] **Step 3: Implement as_marc() and as_marc21()**

Add to the `Record` class in `mrrc/__init__.py`:

```python
def as_marc(self) -> bytes:
    """Serialize record to ISO 2709 binary MARC (pymarc compatibility).

    Returns:
        bytes: The record in ISO 2709 format.
    """
    self._sync_leader()
    return bytes(self._inner.to_marc21())

def as_marc21(self) -> bytes:
    """Alias for as_marc() (pymarc compatibility)."""
    return self.as_marc()
```

Also update `test_encoding_to_marc` (line 876) to use `as_marc()`:
```python
def test_encoding_to_marc(self):
    """Test encoding record to MARC."""
    record = Record(Leader())
    field = Field('245', '1', '0')
    field.add_subfield('a', 'Test')
    record.add_field(field)
    encoded = record.as_marc()
    assert encoded is not None
    assert isinstance(encoded, bytes)
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestRecordBinarySerialization -v`
Expected: All pass

- [ ] **Step 5: Commit**

```bash
git add mrrc/__init__.py tests/python/test_pymarc_compatibility.py
git commit -m "Add Record.as_marc() and as_marc21() for pymarc compat

Returns ISO 2709 bytes. Wraps existing Rust to_marc21()."
```

---

## Task 10: Field.as_marc() / as_marc21()

pymarc Field has field-level binary serialization. This needs a Rust-side implementation since we need to serialize a single field to MARC binary format.

**Files:**
- Modify: `src-python/src/wrappers.rs` (PyField)
- Modify: `mrrc/__init__.py` (Field class)
- Modify: `tests/python/test_pymarc_compatibility.py`
- Modify: `mrrc/_mrrc.pyi`

- [ ] **Step 1: Write failing tests**

```python
class TestFieldBinarySerialization:
    """Test Field.as_marc() / as_marc21() (pymarc compatibility)."""

    def test_field_as_marc_returns_bytes(self):
        """Field.as_marc() returns bytes."""
        field = Field('245', '1', '0', subfields=[Subfield('a', 'Title')])
        result = field.as_marc()
        assert isinstance(result, bytes)
        assert len(result) > 0

    def test_control_field_as_marc(self):
        """Control field as_marc() returns data as bytes."""
        field = Field('001', data='12345')
        result = field.as_marc()
        assert isinstance(result, bytes)
        # Control field MARC: data + field terminator
        assert b'12345' in result

    def test_field_as_marc21_alias(self):
        """as_marc21() is alias for as_marc()."""
        field = Field('245', '1', '0', subfields=[Subfield('a', 'Test')])
        assert field.as_marc() == field.as_marc21()

    def test_field_as_marc_contains_subfield_data(self):
        """Field binary should contain subfield codes and values."""
        field = Field('245', '1', '0', subfields=[
            Subfield('a', 'Title'),
            Subfield('c', 'Author'),
        ])
        result = field.as_marc()
        assert b'Title' in result
        assert b'Author' in result
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestFieldBinarySerialization -v`
Expected: FAIL

- [ ] **Step 3: Add to_marc21() to PyField in Rust**

In `src-python/src/wrappers.rs`, add to the `PyField` impl block (after `__eq__` around line 577):

```rust
/// Serialize field to ISO 2709 binary format.
///
/// For data fields: indicators + subfield data + field terminator.
/// For control fields: data + field terminator.
pub fn to_marc21(&self) -> Vec<u8> {
    let mut buf = Vec::new();
    // Data fields: indicators + subfields
    buf.push(self.inner.indicator1 as u8);
    buf.push(self.inner.indicator2 as u8);
    for sf in &self.inner.subfields {
        buf.push(0x1F); // subfield delimiter
        buf.push(sf.code as u8);
        buf.extend_from_slice(sf.value.as_bytes());
    }
    buf.push(0x1E); // field terminator
    buf
}
```

- [ ] **Step 4: Rebuild Rust extension**

Run: `uv run maturin develop --release`

- [ ] **Step 5: Implement Python wrapper methods**

Add to the `Field` class in `mrrc/__init__.py`:

```python
def as_marc(self) -> bytes:
    """Serialize field to ISO 2709 binary format (pymarc compatibility).

    Returns:
        bytes: The field in binary MARC format.
    """
    if self.is_control_field():
        # Control field: data + field terminator
        return (self.data or '').encode('utf-8') + b'\x1e'
    return bytes(self._inner.to_marc21())

def as_marc21(self) -> bytes:
    """Alias for as_marc() (pymarc compatibility)."""
    return self.as_marc()
```

- [ ] **Step 6: Update type stubs**

Add to `mrrc/_mrrc.pyi` in the Field class:

```python
def to_marc21(self) -> bytes: ...
```

- [ ] **Step 7: Run tests to verify they pass**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestFieldBinarySerialization -v`
Expected: All pass

- [ ] **Step 8: Commit**

```bash
git add src-python/src/wrappers.rs mrrc/__init__.py mrrc/_mrrc.pyi tests/python/test_pymarc_compatibility.py
git commit -m "Add Field.as_marc() and as_marc21() for pymarc compat

Field-level binary serialization. Rust-side to_marc21() added to
PyField. Python wrapper adds as_marc()/as_marc21() aliases."
```

---

## Task 11: Record.as_json() / as_dict() with pymarc-compatible JSON schema

pymarc's JSON format follows MARC-in-JSON (code4lib): `{"leader": "...", "fields": [{"245": {"ind1": "1", "ind2": "0", "subfields": [{"a": "Title"}]}}]}`. mrrc's `to_json()` uses a different structure. We implement `as_dict()` and `as_json()` producing pymarc-compatible output while keeping `to_json()` as mrrc's native format.

**Files:**
- Modify: `mrrc/__init__.py` (Record class)
- Modify: `tests/python/test_pymarc_compatibility.py`

- [ ] **Step 1: Write failing tests**

```python
class TestPymarcJsonSchema:
    """Test Record.as_json() / as_dict() produce pymarc-compatible JSON."""

    def test_as_dict_structure(self):
        """as_dict() returns pymarc-compatible MARC-in-JSON structure."""
        record = Record(Leader())
        record.add_control_field('001', 'test-id')
        record.add_field(Field('245', '1', '0', subfields=[
            Subfield('a', 'Title'),
            Subfield('c', 'Author'),
        ]))
        d = record.as_dict()
        assert 'leader' in d
        assert 'fields' in d
        assert isinstance(d['fields'], list)

    def test_as_dict_control_field_format(self):
        """Control fields are {tag: value} in the fields list."""
        record = Record(Leader())
        record.add_control_field('001', 'test-id')
        d = record.as_dict()
        cf = d['fields'][0]
        assert cf == {'001': 'test-id'}

    def test_as_dict_data_field_format(self):
        """Data fields have ind1, ind2, and subfields as list of single-key dicts."""
        record = Record(fields=[
            Field('245', '1', '0', subfields=[
                Subfield('a', 'Title'),
                Subfield('c', 'Author'),
            ]),
        ])
        d = record.as_dict()
        df = d['fields'][0]
        assert '245' in df
        inner = df['245']
        assert inner['ind1'] == '1'
        assert inner['ind2'] == '0'
        assert isinstance(inner['subfields'], list)
        assert inner['subfields'][0] == {'a': 'Title'}
        assert inner['subfields'][1] == {'c': 'Author'}

    def test_as_dict_duplicate_subfield_codes_preserved(self):
        """Duplicate subfield codes are preserved (array of single-key dicts)."""
        record = Record(fields=[
            Field('650', ' ', '0', subfields=[
                Subfield('a', 'Topic 1'),
                Subfield('a', 'Topic 2'),
            ]),
        ])
        d = record.as_dict()
        sfs = d['fields'][0]['650']['subfields']
        assert len(sfs) == 2
        assert sfs[0] == {'a': 'Topic 1'}
        assert sfs[1] == {'a': 'Topic 2'}

    def test_as_json_returns_string(self):
        """as_json() returns a JSON string."""
        record = Record(Leader())
        record.add_control_field('001', 'test-id')
        result = record.as_json()
        assert isinstance(result, str)
        parsed = json.loads(result)
        assert 'leader' in parsed

    def test_as_json_kwargs_forwarded(self):
        """as_json(**kwargs) forwards to json.dumps."""
        record = Record(Leader())
        record.add_control_field('001', 'test-id')
        result = record.as_json(indent=2)
        assert '\n' in result  # indented output

    def test_to_json_unchanged(self):
        """to_json() should still return mrrc's native format (not pymarc)."""
        record = Record(Leader())
        record.add_control_field('001', 'test-id')
        native = record.to_json()
        pymarc_compat = record.as_json()
        # They should be different formats
        assert json.loads(native) != json.loads(pymarc_compat)
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestPymarcJsonSchema -v`
Expected: FAIL — `as_dict()` and `as_json()` don't exist

- [ ] **Step 3: Implement as_dict() and as_json()**

Add to the `Record` class in `mrrc/__init__.py`:

```python
def as_dict(self) -> dict:
    """Return pymarc-compatible MARC-in-JSON dict (code4lib schema).

    Structure: {"leader": "...", "fields": [{tag: ...}, ...]}
    Control fields: {tag: value}
    Data fields: {tag: {"ind1": "X", "ind2": "Y", "subfields": [{"code": "val"}, ...]}}
    """
    import json as _json

    fields_list = []

    # Add control fields
    for tag, value in self._inner.control_fields():
        fields_list.append({tag: value})

    # Add data fields
    for field in self._inner.fields():
        subfields_list = [{sf.code: sf.value} for sf in field.subfields()]
        fields_list.append({
            field.tag: {
                'ind1': field.indicator1,
                'ind2': field.indicator2,
                'subfields': subfields_list,
            }
        })

    return {
        'leader': self.leader()._get_leader_as_string(),
        'fields': fields_list,
    }

def as_json(self, **kwargs) -> str:
    """Serialize to pymarc-compatible MARC-in-JSON string.

    Accepts any keyword arguments supported by json.dumps (e.g., indent=2).
    """
    import json as _json
    return _json.dumps(self.as_dict(), **kwargs)
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestPymarcJsonSchema -v`
Expected: All pass

- [ ] **Step 5: Commit**

```bash
git add mrrc/__init__.py tests/python/test_pymarc_compatibility.py
git commit -m "Add Record.as_dict() and as_json() with pymarc JSON schema

Produces MARC-in-JSON (code4lib) format with subfields as array of
single-key dicts (preserving duplicate codes). to_json() unchanged
as mrrc's native format."
```

---

## Task 12: parse_xml_to_array()

pymarc exports `parse_xml_to_array(xml_file)` accepting file paths, open file handles, and StringIO. mrrc has `xml_to_records(str)` only.

**Files:**
- Modify: `mrrc/__init__.py` (module-level function)
- Modify: `mrrc/__init__.py:1575-1626` (__all__)
- Modify: `tests/python/test_pymarc_compatibility.py`

- [ ] **Step 1: Write failing tests**

```python
class TestParseXmlToArray:
    """Test parse_xml_to_array() (pymarc compatibility)."""

    def test_parse_xml_from_string(self):
        """parse_xml_to_array accepts XML string."""
        xml = '''<?xml version="1.0" encoding="UTF-8"?>
        <collection xmlns="http://www.loc.gov/MARC21/slim">
          <record>
            <leader>00000nam a2200000 a 4500</leader>
            <controlfield tag="001">test-id</controlfield>
            <datafield tag="245" ind1="1" ind2="0">
              <subfield code="a">Test Title</subfield>
            </datafield>
          </record>
        </collection>'''
        records = parse_xml_to_array(xml)
        assert len(records) >= 1
        assert isinstance(records[0], Record)

    def test_parse_xml_from_file_path(self, fixture_dir):
        """parse_xml_to_array accepts file path."""
        xml_file = fixture_dir / 'simple_book.xml'
        if xml_file.exists():
            records = parse_xml_to_array(str(xml_file))
            assert len(records) >= 1

    def test_parse_xml_from_file_object(self):
        """parse_xml_to_array accepts file-like object."""
        import io
        xml = '''<?xml version="1.0" encoding="UTF-8"?>
        <collection xmlns="http://www.loc.gov/MARC21/slim">
          <record>
            <leader>00000nam a2200000 a 4500</leader>
            <controlfield tag="001">test-id</controlfield>
          </record>
        </collection>'''
        records = parse_xml_to_array(io.StringIO(xml))
        assert len(records) >= 1

    def test_parse_xml_returns_list(self):
        """parse_xml_to_array always returns a list."""
        xml = '''<?xml version="1.0" encoding="UTF-8"?>
        <collection xmlns="http://www.loc.gov/MARC21/slim">
        </collection>'''
        records = parse_xml_to_array(xml)
        assert isinstance(records, list)
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestParseXmlToArray -v`
Expected: FAIL — `parse_xml_to_array` not defined

- [ ] **Step 3: Implement parse_xml_to_array()**

Add to `mrrc/__init__.py` as a module-level function:

```python
def parse_xml_to_array(xml_file) -> List[Record]:
    """Parse MARCXML to a list of Records (pymarc compatibility).

    Accepts file paths (str/Path), open file handles, or XML strings.

    Args:
        xml_file: A file path, file-like object, or XML string.

    Returns:
        List of Record objects parsed from the XML.
    """
    import os

    # File path
    if isinstance(xml_file, (str, os.PathLike)):
        path = str(xml_file)
        if os.path.isfile(path):
            with open(path, 'r', encoding='utf-8') as f:
                xml_str = f.read()
        else:
            # Assume it's an XML string
            xml_str = path
    elif hasattr(xml_file, 'read'):
        # File-like object
        xml_str = xml_file.read()
    else:
        xml_str = str(xml_file)

    return xml_to_records(xml_str)
```

Add `"parse_xml_to_array"` to `__all__`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestParseXmlToArray -v`
Expected: All pass

- [ ] **Step 5: Commit**

```bash
git add mrrc/__init__.py tests/python/test_pymarc_compatibility.py
git commit -m "Add parse_xml_to_array() for pymarc compat

Accepts file paths, file objects, or XML strings. Wraps
xml_to_records() with input type dispatch."
```

---

## Task 13: Field.convert_legacy_subfields() classmethod

Converts old pymarc `['a', 'value', 'b', 'value']` format to Subfield objects.

**Files:**
- Modify: `mrrc/__init__.py` (Field class)
- Modify: `tests/python/test_pymarc_compatibility.py`

- [ ] **Step 1: Write failing tests**

```python
class TestLegacySubfields:
    """Test Field.convert_legacy_subfields() (pymarc compatibility)."""

    def test_convert_legacy_subfields(self):
        """Convert ['a', 'val1', 'b', 'val2'] to Subfield objects."""
        result = Field.convert_legacy_subfields(['a', 'Title', 'b', 'Subtitle'])
        assert len(result) == 2
        assert result[0].code == 'a'
        assert result[0].value == 'Title'
        assert result[1].code == 'b'
        assert result[1].value == 'Subtitle'

    def test_convert_empty_list(self):
        result = Field.convert_legacy_subfields([])
        assert result == []
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestLegacySubfields -v`
Expected: FAIL

- [ ] **Step 3: Implement convert_legacy_subfields()**

Add to the `Field` class:

```python
@classmethod
def convert_legacy_subfields(cls, subfields: list) -> List[Subfield]:
    """Convert legacy pymarc subfield list to Subfield objects.

    Converts the old format ['code', 'value', 'code', 'value', ...]
    to a list of Subfield namedtuple-like objects.
    """
    result = []
    for i in range(0, len(subfields), 2):
        if i + 1 < len(subfields):
            result.append(Subfield(subfields[i], subfields[i + 1]))
    return result
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestLegacySubfields -v`
Expected: All pass

- [ ] **Step 5: Commit**

```bash
git add mrrc/__init__.py tests/python/test_pymarc_compatibility.py
git commit -m "Add Field.convert_legacy_subfields() classmethod

Converts old pymarc ['code', 'value', ...] format to Subfield objects."
```

---

## Task 14: Exception hierarchy

pymarc has ~10 specific exception classes. mrrc exports none, making it hard for users to catch specific errors.

**Files:**
- Modify: `mrrc/__init__.py` (add exception classes)
- Modify: `mrrc/__init__.py:1575-1626` (__all__)
- Modify: `tests/python/test_pymarc_compatibility.py`

- [ ] **Step 1: Write failing tests**

```python
class TestExceptionHierarchy:
    """Test pymarc-compatible exception classes."""

    def test_exception_classes_importable(self):
        """All pymarc exception classes should be importable."""
        from mrrc import (
            MrrcException,
            RecordLengthInvalid,
            RecordLeaderInvalid,
            BaseAddressInvalid,
            BaseAddressNotFound,
            RecordDirectoryInvalid,
            EndOfRecordNotFound,
            FieldNotFound,
            FatalReaderError,
        )
        # All should be subclasses of MrrcException
        assert issubclass(RecordLengthInvalid, MrrcException)
        assert issubclass(RecordLeaderInvalid, MrrcException)
        assert issubclass(FatalReaderError, MrrcException)

    def test_exception_hierarchy(self):
        """Exceptions should be catchable via base class."""
        from mrrc import MrrcException, RecordLengthInvalid
        try:
            raise RecordLengthInvalid("bad length")
        except MrrcException as e:
            assert "bad length" in str(e)

    def test_exceptions_are_exceptions(self):
        """All exception classes should be subclasses of Exception."""
        from mrrc import MrrcException
        assert issubclass(MrrcException, Exception)
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestExceptionHierarchy -v`
Expected: FAIL — import errors

- [ ] **Step 3: Define exception classes**

Add to `mrrc/__init__.py` before the `Indicators` class:

```python
# Exception hierarchy (pymarc compatibility)
class MrrcException(Exception):
    """Base exception for mrrc errors."""
    pass

class RecordLengthInvalid(MrrcException):
    """Record length in leader is invalid."""
    pass

class RecordLeaderInvalid(MrrcException):
    """Record leader is malformed."""
    pass

class BaseAddressInvalid(MrrcException):
    """Base address of data in leader is invalid."""
    pass

class BaseAddressNotFound(MrrcException):
    """Base address of data not found in leader."""
    pass

class RecordDirectoryInvalid(MrrcException):
    """Record directory entries are malformed."""
    pass

class EndOfRecordNotFound(MrrcException):
    """End-of-record marker not found."""
    pass

class FieldNotFound(MrrcException):
    """Expected field not found in record."""
    pass

class FatalReaderError(MrrcException):
    """Unrecoverable error during record reading."""
    pass

class BadSubfieldCodeWarning(UserWarning):
    """Warning for invalid subfield codes."""
    pass
```

Add all exception class names to `__all__`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestExceptionHierarchy -v`
Expected: All pass

- [ ] **Step 5: Commit**

```bash
git add mrrc/__init__.py tests/python/test_pymarc_compatibility.py
git commit -m "Add pymarc-compatible exception hierarchy

MrrcException base class with specific subclasses for record
parsing errors: RecordLengthInvalid, RecordLeaderInvalid, etc."
```

---

## Task 15: MARC constants

pymarc exports several MARC format constants. These are useful for users working with raw MARC data.

**Files:**
- Modify: `mrrc/__init__.py`
- Modify: `mrrc/__init__.py:1575-1626` (__all__)
- Modify: `tests/python/test_pymarc_compatibility.py`

- [ ] **Step 1: Write failing tests**

```python
class TestMarcConstants:
    """Test MARC format constants (pymarc compatibility)."""

    def test_constants_importable(self):
        """MARC constants should be importable."""
        from mrrc import (
            LEADER_LEN,
            DIRECTORY_ENTRY_LEN,
            END_OF_FIELD,
            END_OF_RECORD,
            SUBFIELD_INDICATOR,
        )
        assert LEADER_LEN == 24
        assert DIRECTORY_ENTRY_LEN == 12
        assert END_OF_FIELD == '\x1e'
        assert END_OF_RECORD == '\x1d'
        assert SUBFIELD_INDICATOR == '\x1f'

    def test_xml_constants(self):
        """MARC XML constants should be importable."""
        from mrrc import MARC_XML_NS, MARC_XML_SCHEMA
        assert 'loc.gov' in MARC_XML_NS
        assert 'loc.gov' in MARC_XML_SCHEMA
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestMarcConstants -v`
Expected: FAIL

- [ ] **Step 3: Define constants**

Add to `mrrc/__init__.py` (after the exception classes, before `Indicators`):

```python
# MARC format constants (pymarc compatibility)
LEADER_LEN = 24
DIRECTORY_ENTRY_LEN = 12
END_OF_FIELD = '\x1e'
END_OF_RECORD = '\x1d'
SUBFIELD_INDICATOR = '\x1f'
MARC_XML_NS = 'http://www.loc.gov/MARC21/slim'
MARC_XML_SCHEMA = 'http://www.loc.gov/standards/marcxml/schema/MARC21slim.xsd'
```

Add all constant names to `__all__`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestMarcConstants -v`
Expected: All pass

- [ ] **Step 5: Commit**

```bash
git add mrrc/__init__.py tests/python/test_pymarc_compatibility.py
git commit -m "Add MARC format constants for pymarc compat

LEADER_LEN, DIRECTORY_ENTRY_LEN, END_OF_FIELD, END_OF_RECORD,
SUBFIELD_INDICATOR, MARC_XML_NS, MARC_XML_SCHEMA."
```

---

## Task 16: Convenience functions (map_records, parse_json_to_array)

pymarc exports `map_records()` and `parse_json_to_array()`. These are commonly used.

**Files:**
- Modify: `mrrc/__init__.py`
- Modify: `mrrc/__init__.py:1575-1626` (__all__)
- Modify: `tests/python/test_pymarc_compatibility.py`

- [ ] **Step 1: Write failing tests**

```python
class TestConvenienceFunctions:
    """Test pymarc convenience functions."""

    def test_map_records(self, fixture_dir):
        """map_records applies a function to each record in a file."""
        test_file = fixture_dir / 'simple_book.mrc'
        titles = []
        map_records(lambda r: titles.append(r.title), str(test_file))
        assert len(titles) > 0

    def test_parse_json_to_array(self):
        """parse_json_to_array parses pymarc-format JSON."""
        record = Record(fields=[
            Field('245', '1', '0', subfields=[Subfield('a', 'Title')]),
        ])
        record.add_control_field('001', 'test-id')
        json_str = record.as_json()
        # Wrap in array
        json_array = '[' + json_str + ']'
        records = parse_json_to_array(json_array)
        assert len(records) == 1
        assert isinstance(records[0], Record)
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestConvenienceFunctions -v`
Expected: FAIL

- [ ] **Step 3: Implement convenience functions**

Add to `mrrc/__init__.py`:

```python
def map_records(func, *files: str) -> None:
    """Apply a function to each record in one or more MARC files (pymarc compatibility).

    Args:
        func: Callable that takes a Record.
        *files: One or more file paths to MARC files.
    """
    for path in files:
        reader = MARCReader(open(path, 'rb'))
        for record in reader:
            func(record)


def parse_json_to_array(json_str: str) -> List[Record]:
    """Parse a JSON array of pymarc-format records (pymarc compatibility).

    Args:
        json_str: JSON string containing an array of MARC-in-JSON records.

    Returns:
        List of Record objects.
    """
    import json as _json

    data = _json.loads(json_str)
    if not isinstance(data, list):
        data = [data]

    records = []
    for item in data:
        record = Record()
        if 'leader' in item:
            record.leader()._update_leader_from_string(str(item['leader']))
        if 'fields' in item:
            for field_dict in item['fields']:
                for tag, value in field_dict.items():
                    if isinstance(value, str):
                        # Control field
                        record.add_control_field(tag, value)
                    elif isinstance(value, dict):
                        # Data field
                        ind1 = value.get('ind1', ' ')
                        ind2 = value.get('ind2', ' ')
                        subfields = []
                        for sf_dict in value.get('subfields', []):
                            for code, val in sf_dict.items():
                                subfields.append(Subfield(code, val))
                        f = Field(tag, ind1, ind2, subfields=subfields)
                        record.add_field(f)
        records.append(record)
    return records
```

Add `"map_records"` and `"parse_json_to_array"` to `__all__`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `uv run python -m pytest tests/python/test_pymarc_compatibility.py::TestConvenienceFunctions -v`
Expected: All pass

- [ ] **Step 5: Commit**

```bash
git add mrrc/__init__.py tests/python/test_pymarc_compatibility.py
git commit -m "Add map_records and parse_json_to_array for pymarc compat

map_records applies a function to each record in MARC files.
parse_json_to_array parses pymarc-format JSON arrays."
```

---

## Task 17: Update type stubs

After all changes, update `mrrc/_mrrc.pyi` to reflect the new API surface.

**Files:**
- Modify: `mrrc/_mrrc.pyi`

- [ ] **Step 1: Update type stubs to match new API**

Update `mrrc/_mrrc.pyi` to add all new methods and properties. Key additions:

- `Field.data` property
- `Field.value()` method
- `Field.format_field()` method
- `Field.as_marc()` / `as_marc21()` methods
- `Field.linkage_occurrence_num()` method
- `Field.convert_legacy_subfields()` classmethod
- `Field.add_subfield(code, value, pos=None)` updated signature
- `Record` properties (title, author, isbn, etc. as properties not methods)
- `Record.as_marc()` / `as_marc21()` methods
- `Record.as_dict()` / `as_json()` methods
- `Record.add_field(*fields)` updated signature
- `Record.remove_field(*fields)` updated signature
- `Record.remove_fields(*tags)` method
- `Record.add_ordered_field(*fields)` method
- `Record.add_grouped_field(*fields)` method
- `Record.addedentries`, `physicaldescription`, `uniformtitle` properties
- Exception classes
- Constants
- Module-level functions: `parse_xml_to_array`, `map_records`, `parse_json_to_array`

- [ ] **Step 2: Run full test suite to verify everything still works**

Run: `uv run python -m pytest tests/python/ -m "not benchmark" -q`
Expected: All pass

- [ ] **Step 3: Commit**

```bash
git add mrrc/_mrrc.pyi
git commit -m "Update type stubs for pymarc API compatibility

Reflects all new methods, properties, constants, and exception
classes added for pymarc compatibility."
```

---

## Task 18: Full CI verification

Run the full pre-push check to ensure everything passes.

**Files:** None (verification only)

- [ ] **Step 1: Rebuild Rust extension**

Run: `uv run maturin develop --release`
Expected: Build succeeds

- [ ] **Step 2: Run full check script**

Run: `.cargo/check.sh`
Expected: All checks pass (rustfmt, clippy, docs, tests, python tests, lint)

- [ ] **Step 3: Fix any issues found**

If any checks fail, fix and re-run.

- [ ] **Step 4: Final commit if needed**

```bash
git add -A
git commit -m "Fix issues found during full CI verification"
```
