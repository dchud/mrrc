"""Write-through semantics for fields obtained from a ``Record``.

Fields returned by record accessors (``record[tag]``, ``get_field``,
``get_fields``) are live handles: every read and write goes through to
the record. Handles invalidated by field
removal raise :class:`mrrc.StaleFieldError` instead of silently
targeting the wrong field.

Detached fields (constructed directly or via builders) are unaffected:
they own their data and are added to a record with ``add_field``.
"""

from __future__ import annotations

import copy

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


def test_get_fields_index_edit_persists() -> None:
    """Editing a field by index from ``get_fields`` persists."""
    record = _build_record()
    record.get_fields("650")[0].add_subfield("x", "History.")
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


# ---------------------------------------------------------------------
# Mutation during get_fields iteration
# ---------------------------------------------------------------------


def test_remove_all_during_iteration_stales_pending_handles() -> None:
    """Removing a tag mid-iteration stales the not-yet-visited handles.

    ``get_fields`` returns a snapshot list, so the loop itself completes;
    every handle yielded after the removal raises instead of reading a
    wrong field.
    """
    record = _build_record()
    seen: list[str] = []
    for index, field in enumerate(record.get_fields("650")):
        if index == 0:
            record.remove_fields("650")
        try:
            seen.append(field["a"])
        except mrrc.StaleFieldError:
            seen.append("stale")
    assert seen == ["stale", "stale", "stale"]


def test_remove_one_occurrence_during_iteration_stales_all_handles() -> None:
    """Removing a single occurrence invalidates every outstanding handle.

    Staleness is record-wide: handles to the surviving occurrences raise
    StaleFieldError rather than shifting to a neighboring field.
    """
    record = _build_record()
    handles = record.get_fields("650")
    record.remove_field_at("650", 1)
    for handle in handles:
        with pytest.raises(mrrc.StaleFieldError):
            handle["a"]
    # Re-fetched handles see the post-removal state in order.
    assert [f["a"] for f in record.get_fields("650")] == ["Cats.", "Birds."]


def test_removal_stales_handles_of_other_tags() -> None:
    """Removing one tag invalidates handles to unrelated tags as well."""
    record = _build_record()
    title = record["245"]
    record.remove_field_at("650", 1)
    with pytest.raises(mrrc.StaleFieldError):
        title["a"]


def test_addition_during_iteration_keeps_snapshot_and_handles() -> None:
    """Adding a field mid-iteration neither grows the loop nor stales handles."""
    record = _build_record()
    visited: list[str] = []
    for index, field in enumerate(record.get_fields("650")):
        if index == 0:
            extra = mrrc.Field("650", " ", "0")
            extra.add_subfield("a", "Fish.")
            record.add_field(extra)
        visited.append(field["a"])
    assert visited == ["Cats.", "Dogs.", "Birds."]
    assert len(record.get_fields("650")) == 4


# ---------------------------------------------------------------------
# Handles across copy / deepcopy
# ---------------------------------------------------------------------


def test_copy_of_handle_aliases_same_field() -> None:
    """``copy.copy`` of a handle yields another live handle to the field."""
    record = _build_record()
    duplicate = copy.copy(record["245"])
    duplicate.add_subfield("n", "Part 1.")
    assert record["245"]["n"] == "Part 1."


def test_copy_of_handle_goes_stale_with_the_original() -> None:
    """A shallow-copied handle is invalidated by the same removals."""
    record = _build_record()
    original = record["245"]
    duplicate = copy.copy(original)
    record.remove_fields("245")
    with pytest.raises(mrrc.StaleFieldError):
        original["a"]
    with pytest.raises(mrrc.StaleFieldError):
        duplicate["a"]


def test_deepcopy_of_handle_returns_detached_snapshot() -> None:
    """``copy.deepcopy`` of a handle yields a detached snapshot, not a handle."""
    record = _build_record()
    snapshot = copy.deepcopy(record["245"])
    # A live handle goes stale when the field is removed; the snapshot,
    # being detached, keeps the data it captured.
    record.remove_fields("245")
    assert snapshot["a"] == "Original title /"
    snapshot.add_subfield("b", "copy only")
    assert record.get_fields("245") == []


def test_copy_of_detached_field_shares_state() -> None:
    """``copy.copy`` of a detached field shares the underlying data."""
    field = mrrc.Field("500", " ", " ")
    field.add_subfield("a", "A general note.")
    duplicate = copy.copy(field)
    duplicate.add_subfield("b", "Shared.")
    assert field.subfields_by_code("b") == ["Shared."]


def test_deepcopy_of_detached_field_is_independent() -> None:
    """``copy.deepcopy`` of a detached field yields an independent field."""
    field = mrrc.Field("500", " ", " ")
    field.add_subfield("a", "A general note.")
    duplicate = copy.deepcopy(field)
    duplicate.add_subfield("b", "Only on the copy.")
    assert field.subfields_by_code("b") == []
    assert duplicate.subfields_by_code("b") == ["Only on the copy."]
    assert duplicate._parent is None


def test_copy_of_record_shares_inner() -> None:
    """``copy.copy`` of a record is shallow: it shares the underlying record."""
    record = _build_record()
    duplicate = copy.copy(record)
    note = mrrc.Field("500", " ", " ")
    note.add_subfield("a", "Shared note.")
    duplicate.add_field(note)
    assert record["500"]["a"] == "Shared note."


def test_deepcopy_of_record_is_independent() -> None:
    """``copy.deepcopy`` of a record yields a fully independent record."""
    record = _build_record()
    duplicate = copy.deepcopy(record)
    note = mrrc.Field("500", " ", " ")
    note.add_subfield("a", "Only on the copy.")
    duplicate.add_field(note)
    assert len(record.get_fields("500")) == 0
    assert len(duplicate.get_fields("500")) == 1


def test_deepcopy_of_leader_is_independent() -> None:
    """``copy.deepcopy`` of a leader yields an independent leader."""
    leader = mrrc.Leader("00136nam a2200061   4500")
    duplicate = copy.deepcopy(leader)
    duplicate.record_status = "c"
    assert leader.record_status == "n"
    assert duplicate.record_status == "c"


def test_remove_field_at_returns_wrapper() -> None:
    """``remove_field_at`` returns the mrrc.Field wrapper, not the raw type."""
    record = _build_record()
    removed = record.remove_field_at("245", 0)
    assert isinstance(removed, mrrc.Field)
    # The wrapper conveniences work; the raw _mrrc.Field lacks __getitem__.
    assert removed["a"] == "Original title /"


# ---------------------------------------------------------------------
# Same-tag remove plus re-add: no occurrence aliasing
# ---------------------------------------------------------------------


def test_handle_stale_read_after_remove_and_readd_same_tag() -> None:
    """A pre-removal handle never reads a re-added same-tag field."""
    record = _build_record()
    old = record.get_fields("650")[0]
    record.remove_fields("650")
    replacement = mrrc.Field("650", " ", "0")
    replacement.add_subfield("a", "Fish.")
    record.add_field(replacement)
    with pytest.raises(mrrc.StaleFieldError):
        old["a"]


def test_handle_stale_write_after_remove_and_readd_same_tag() -> None:
    """A pre-removal handle never writes to a re-added same-tag field."""
    record = _build_record()
    old = record.get_fields("650")[0]
    record.remove_fields("650")
    replacement = mrrc.Field("650", " ", "0")
    replacement.add_subfield("a", "Fish.")
    record.add_field(replacement)
    with pytest.raises(mrrc.StaleFieldError):
        old.add_subfield("x", "History.")
    assert record.get_fields("650")[0].subfields_by_code("x") == []


def test_remove_occurrence_then_readd_requires_refetch() -> None:
    """After occurrence removal plus re-add, only re-fetched handles work."""
    record = _build_record()
    first = record.get_fields("650")[0]
    record.remove_field_at("650", 0)
    replacement = mrrc.Field("650", " ", "0")
    replacement.add_subfield("a", "Fish.")
    record.add_field(replacement)
    with pytest.raises(mrrc.StaleFieldError):
        first["a"]
    assert [f["a"] for f in record.get_fields("650")] == [
        "Dogs.",
        "Birds.",
        "Fish.",
    ]


def test_remove_field_by_handle_stales_other_handles() -> None:
    """``remove_field(handle)`` removes that occurrence and stales the rest."""
    record = _build_record()
    handles = record.get_fields("650")
    record.remove_field(handles[1])
    for handle in handles:
        with pytest.raises(mrrc.StaleFieldError):
            handle["a"]
    assert [f["a"] for f in record.get_fields("650")] == ["Cats.", "Birds."]


# ---------------------------------------------------------------------
# Reader-produced records behave like constructed records
# ---------------------------------------------------------------------


@pytest.fixture()
def reader_record() -> mrrc.Record:
    """A record parsed from disk rather than built field by field."""
    with open("tests/data/simple_book.mrc", "rb") as fh:
        record = next(mrrc.MARCReader(fh))
    assert record is not None
    return record


def test_reader_record_handle_write_through(reader_record: mrrc.Record) -> None:
    reader_record["245"]["a"] = "Revised title /"
    assert reader_record["245"]["a"] == "Revised title /"


def test_reader_record_alias_visibility(reader_record: mrrc.Record) -> None:
    first = reader_record["245"]
    second = reader_record["245"]
    first.add_subfield("n", "Part 1.")
    assert second["n"] == "Part 1."


def test_reader_record_get_fields_edit_persists(
    reader_record: mrrc.Record,
) -> None:
    for field in reader_record.get_fields("650"):
        field.add_subfield("x", "History.")
    assert all(f["x"] == "History." for f in reader_record.get_fields("650"))


def test_reader_record_stale_after_removal(reader_record: mrrc.Record) -> None:
    handle = reader_record["650"]
    reader_record.remove_fields("650")
    with pytest.raises(mrrc.StaleFieldError):
        handle["a"]


def test_reader_record_stale_after_remove_and_readd(
    reader_record: mrrc.Record,
) -> None:
    handle = reader_record["650"]
    reader_record.remove_fields("650")
    replacement = mrrc.Field("650", " ", "0")
    replacement.add_subfield("a", "Jazz age fiction.")
    reader_record.add_field(replacement)
    with pytest.raises(mrrc.StaleFieldError):
        handle["a"]
    assert reader_record["650"]["a"] == "Jazz age fiction."


def test_reader_record_edits_survive_roundtrip(
    reader_record: mrrc.Record,
) -> None:
    reader_record["245"]["a"] = "Persisted title /"
    data = reader_record.as_marc()
    reread = next(mrrc.MARCReader(data))
    assert reread is not None
    assert reread["245"]["a"] == "Persisted title /"
