"""Write-through semantics for fields obtained from a ``Record``.

Fields returned by record accessors (``record[tag]``, ``get_field``,
``get_fields``, ``fields_by_tag``) are live handles: every read and
write goes through to the record. Handles invalidated by field
removal raise :class:`mrrc.StaleFieldError` instead of silently
targeting the wrong field.

Detached fields (constructed directly or via builders) are unaffected:
they own their data and are added to a record with ``add_field``.
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
# Mutation persistence
# ---------------------------------------------------------------------


def test_getitem_indicator_assignment_persists() -> None:
    """``record[tag].indicator1 = ...`` writes through to the record."""
    record = _build_record()
    record["245"].indicator1 = "0"
    assert record["245"].indicator1 == "0"


def test_getitem_add_subfield_persists() -> None:
    """``record[tag].add_subfield(...)`` writes through to the record."""
    record = _build_record()
    record["245"].add_subfield("b", "a subtitle /")
    assert record["245"]["b"] == "a subtitle /"


def test_getitem_setitem_subfield_persists() -> None:
    """``record[tag][code] = value`` writes through to the record."""
    record = _build_record()
    record["245"]["a"] = "Revised title /"
    assert record["245"]["a"] == "Revised title /"


def test_get_field_delete_subfield_persists() -> None:
    """``get_field`` returns a handle; ``delete_subfield`` persists."""
    record = _build_record()
    field = record.get_field("245")
    assert field is not None
    field.delete_subfield("c")
    assert record["245"]["c"] is None


def test_get_fields_iteration_edits_persist() -> None:
    """Editing each field from ``get_fields`` persists (pymarc idiom)."""
    record = _build_record()
    for field in record.get_fields("650"):
        field.delete_subfield("9")
    assert all(f["9"] is None for f in record.get_fields("650"))


def test_fields_by_tag_edit_persists() -> None:
    """``fields_by_tag`` also returns live handles."""
    record = _build_record()
    record.fields_by_tag("650")[0].add_subfield("x", "History.")
    assert record.get_fields("650")[0]["x"] == "History."


# ---------------------------------------------------------------------
# Alias visibility
# ---------------------------------------------------------------------


def test_alias_handles_see_each_others_edits() -> None:
    """Two handles to the same field read the same underlying state."""
    record = _build_record()
    first = record["245"]
    second = record["245"]
    first.add_subfield("n", "Part 1.")
    assert second["n"] == "Part 1."


# ---------------------------------------------------------------------
# Control fields
# ---------------------------------------------------------------------


def test_control_field_data_assignment_persists() -> None:
    """``record[tag].data = ...`` on a control field writes through."""
    record = _build_record()
    record["005"].data = "20260603120000.0"
    assert record["005"].data == "20260603120000.0"


def test_control_field_handles_alias() -> None:
    """Control-field handles also read through to the record."""
    record = _build_record()
    handle = record["001"]
    record["001"].data = "ocn987654321"
    assert handle.data == "ocn987654321"


# ---------------------------------------------------------------------
# Staleness: removal invalidates outstanding handles
# ---------------------------------------------------------------------


def test_stale_handle_write_raises_after_removal() -> None:
    """Writing through a handle after a removal raises StaleFieldError."""
    record = _build_record()
    second_subject = record.get_fields("650")[1]
    record.remove_field("650")  # removes all 650 fields
    with pytest.raises(mrrc.StaleFieldError):
        second_subject.add_subfield("x", "History.")


def test_stale_handle_read_raises_after_removal() -> None:
    """Reading through a handle after a removal raises StaleFieldError."""
    record = _build_record()
    second_subject = record.get_fields("650")[1]
    record.remove_field("650")
    with pytest.raises(mrrc.StaleFieldError):
        second_subject["a"]


def test_handle_survives_field_addition() -> None:
    """Appending fields does not invalidate outstanding handles."""
    record = _build_record()
    handle = record.get_fields("650")[2]
    extra = mrrc.Field("650", " ", "0")
    extra.add_subfield("a", "Fish.")
    record.add_field(extra)
    handle.add_subfield("x", "Behavior.")
    assert record.get_fields("650")[2]["x"] == "Behavior."


# ---------------------------------------------------------------------
# Detached fields: unchanged contract
# ---------------------------------------------------------------------


def test_detached_field_mutate_then_add() -> None:
    """Fields constructed directly still mutate locally and add cleanly."""
    record = _build_record()
    note = mrrc.Field("500", " ", " ")
    note.add_subfield("a", "A general note.")
    record.add_field(note)
    assert record["500"]["a"] == "A general note."


# ---------------------------------------------------------------------
# End to end: edits survive serialization
# ---------------------------------------------------------------------


def test_edits_survive_marc_roundtrip() -> None:
    """An in-place edit is present after serializing and re-reading."""
    record = _build_record()
    record["245"]["a"] = "Persisted title /"
    record["005"].data = "20260603120000.0"

    data = record.as_marc()
    reread = next(mrrc.MARCReader(data))
    assert reread is not None
    assert reread["245"]["a"] == "Persisted title /"
    assert reread["005"].data == "20260603120000.0"
