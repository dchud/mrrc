"""pymarc-compatible field removal semantics.

``Record.remove_field(field)`` removes exactly the given field: the
precise occurrence for live handles obtained from this record, or the
first value-equal field for detached ``Field`` objects. A tag string
removes all fields with that tag (as does ``remove_fields``), control
tags included. ``fields()`` and ``get_fields()`` enumerate repeated
control tags identically.
"""

from __future__ import annotations

import pytest

import mrrc


def _build_record() -> mrrc.Record:
    """A record with a 245, three 650s, and 001/005 control fields."""
    record = mrrc.Record()
    record.add_field(mrrc.Field("001", data="ocm12345678"))
    record.add_field(mrrc.Field("005", data="20200101000000.0"))

    title = mrrc.Field("245", "1", "0")
    title.add_subfield("a", "Original title /")
    title.add_subfield("c", "by Original Author.")
    record.add_field(title)

    for heading in ("Cats.", "Dogs.", "Birds."):
        subject = mrrc.Field("650", " ", "0")
        subject.add_subfield("a", heading)
        subject.add_subfield("9", "local")
        record.add_field(subject)

    return record


# ---------------------------------------------------------------------
# remove_field with a live data-field handle
# ---------------------------------------------------------------------


def test_remove_field_handle_removes_only_that_occurrence() -> None:
    """Removing the 2nd of three 650s leaves the other two intact."""
    record = _build_record()
    second = record.get_fields("650")[1]
    record.remove_field(second)
    remaining = [f["a"] for f in record.get_fields("650")]
    assert remaining == ["Cats.", "Birds."]


def test_remove_field_handle_first_occurrence() -> None:
    record = _build_record()
    first = record.get_fields("650")[0]
    record.remove_field(first)
    remaining = [f["a"] for f in record.get_fields("650")]
    assert remaining == ["Dogs.", "Birds."]


def test_remove_field_handle_last_occurrence() -> None:
    record = _build_record()
    last = record.get_fields("650")[2]
    record.remove_field(last)
    remaining = [f["a"] for f in record.get_fields("650")]
    assert remaining == ["Cats.", "Dogs."]


def test_remove_field_sole_occurrence_removes_tag() -> None:
    record = _build_record()
    record.remove_field(record["245"])
    assert record.get_field("245") is None


def test_remove_field_invalidates_other_handles() -> None:
    """Occurrence indices shift, so outstanding handles go stale."""
    record = _build_record()
    first, _, third = record.get_fields("650")
    record.remove_field(first)
    with pytest.raises(mrrc.StaleFieldError):
        third["a"]


def test_remove_field_stale_handle_raises() -> None:
    """A handle invalidated by an earlier removal cannot be removed."""
    record = _build_record()
    first, second = record.get_fields("650")[:2]
    record.remove_field(first)
    with pytest.raises(mrrc.StaleFieldError):
        record.remove_field(second)


# ---------------------------------------------------------------------
# remove_field with a control-field handle or control tag
# ---------------------------------------------------------------------


def test_remove_field_control_handle() -> None:
    """Removing a control field via its handle actually removes it."""
    record = _build_record()
    record.remove_field(record["001"])
    assert record.control_field("001") is None
    assert record.control_field("005") == "20200101000000.0"


def test_remove_field_control_handle_occurrence() -> None:
    """Removing the 2nd of three 007s leaves the other two intact."""
    record = _build_record()
    for value in ("ta", "cr", "vz"):
        record.add_control_field("007", value)
    second = record.get_fields("007")[1]
    record.remove_field(second)
    values = [f.data for f in record.get_fields("007")]
    assert values == ["ta", "vz"]


def test_remove_field_control_tag_string() -> None:
    """A control tag string removes the control field (was a no-op)."""
    record = _build_record()
    record.remove_field("001")
    assert record.control_field("001") is None


def test_remove_fields_control_tag() -> None:
    """remove_fields removes all values of a repeated control tag."""
    record = _build_record()
    record.add_control_field("007", "ta")
    record.add_control_field("007", "cr")
    record.remove_fields("007")
    assert record.get_fields("007") == []


# ---------------------------------------------------------------------
# remove_field with a detached Field (pymarc add-then-remove idiom)
# ---------------------------------------------------------------------


def test_remove_field_detached_matches_by_value() -> None:
    """A detached field equal in value removes the first match only."""
    record = _build_record()
    detached = mrrc.Field("650", " ", "0")
    detached.add_subfield("a", "Dogs.")
    detached.add_subfield("9", "local")
    record.remove_field(detached)
    remaining = [f["a"] for f in record.get_fields("650")]
    assert remaining == ["Cats.", "Birds."]


def test_remove_field_detached_control_matches_by_value() -> None:
    record = _build_record()
    record.remove_field(mrrc.Field("005", data="20200101000000.0"))
    assert record.control_field("005") is None


def test_remove_field_not_in_record_raises() -> None:
    """pymarc raises ValueError when the field is not in the record."""
    record = _build_record()
    missing = mrrc.Field("700", "1", " ")
    missing.add_subfield("a", "Nobody, Here.")
    with pytest.raises(ValueError):
        record.remove_field(missing)


# ---------------------------------------------------------------------
# Tag-string removal still removes the whole tag
# ---------------------------------------------------------------------


def test_remove_field_tag_string_removes_all() -> None:
    record = _build_record()
    record.remove_field("650")
    assert record.get_fields("650") == []


# ---------------------------------------------------------------------
# fields() / get_fields() enumeration consistency
# ---------------------------------------------------------------------


def test_fields_includes_all_repeated_control_values() -> None:
    """fields() enumerates every 007, not just the first."""
    record = _build_record()
    for value in ("ta", "cr", "vz"):
        record.add_control_field("007", value)
    from_fields = [f.data for f in record.fields() if f.tag == "007"]
    from_get_fields = [f.data for f in record.get_fields("007")]
    assert from_fields == from_get_fields == ["ta", "cr", "vz"]


def test_fields_matches_get_fields_enumeration() -> None:
    """fields() and no-arg get_fields() return the same field sequence."""
    record = _build_record()
    record.add_control_field("006", "m        d        ")
    record.add_control_field("006", "s        h        ")
    fields_view = [(f.tag, f.value()) for f in record.fields()]
    get_fields_view = [(f.tag, f.value()) for f in record.get_fields()]
    assert fields_view == get_fields_view
