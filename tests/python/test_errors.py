"""Tests for the typed exception hierarchy and the structured positional
attributes attached to each exception class.

Snapshot tests (via syrupy) pin the externally-visible str/repr/detailed
output for representative exception classes. Property tests cover pickle
round-trip, hierarchy assertions, and bare-constructor compatibility.
"""

from __future__ import annotations

import pickle

import pytest

import mrrc


# ---------------------------------------------------------------------------
# Hierarchy assertions
# ---------------------------------------------------------------------------


class TestExceptionHierarchy:
    """The new mrrc-specific subclasses must extend the closest pymarc
    parent so existing pymarc-style ``except`` clauses keep catching them.
    """

    def test_invalid_indicator_subclass_of_record_directory_invalid(self):
        assert issubclass(mrrc.InvalidIndicator, mrrc.RecordDirectoryInvalid)

    def test_bad_subfield_code_subclass_of_record_directory_invalid(self):
        assert issubclass(mrrc.BadSubfieldCode, mrrc.RecordDirectoryInvalid)

    def test_invalid_field_subclass_of_record_directory_invalid(self):
        assert issubclass(mrrc.InvalidField, mrrc.RecordDirectoryInvalid)

    def test_truncated_record_subclass_of_end_of_record_not_found(self):
        assert issubclass(mrrc.TruncatedRecord, mrrc.EndOfRecordNotFound)

    def test_all_mrrc_specific_classes_subclass_mrrc_exception(self):
        for cls in (
            mrrc.InvalidIndicator,
            mrrc.BadSubfieldCode,
            mrrc.InvalidField,
            mrrc.TruncatedRecord,
            mrrc.EncodingError,
            mrrc.XmlError,
            mrrc.JsonError,
            mrrc.WriterError,
        ):
            assert issubclass(cls, mrrc.MrrcException), cls

    def test_pymarc_style_catch_catches_new_subclasses(self):
        """``except RecordDirectoryInvalid`` must catch the new subclasses
        unchanged, so existing pymarc-style code keeps working.
        """
        try:
            raise mrrc.InvalidIndicator(
                record_index=1,
                field_tag="245",
                indicator_position=0,
                found=b":",
                expected="digit or space",
            )
        except mrrc.RecordDirectoryInvalid as caught:
            assert isinstance(caught, mrrc.InvalidIndicator)


# ---------------------------------------------------------------------------
# Bare-constructor + kwarg-only compatibility
# ---------------------------------------------------------------------------


class TestBareConstructor:
    """All exception classes must accept ``raise Foo()`` for pymarc
    bare-constructor compatibility, with every positional kwarg defaulting
    to ``None``.
    """

    @pytest.mark.parametrize(
        "cls",
        [
            mrrc.MrrcException,
            mrrc.RecordLengthInvalid,
            mrrc.RecordLeaderInvalid,
            mrrc.BaseAddressInvalid,
            mrrc.BaseAddressNotFound,
            mrrc.RecordDirectoryInvalid,
            mrrc.EndOfRecordNotFound,
            mrrc.FieldNotFound,
            mrrc.FatalReaderError,
            mrrc.InvalidIndicator,
            mrrc.BadSubfieldCode,
            mrrc.InvalidField,
            mrrc.TruncatedRecord,
            mrrc.EncodingError,
            mrrc.XmlError,
            mrrc.JsonError,
            mrrc.WriterError,
        ],
    )
    def test_bare_construct(self, cls):
        instance = cls()
        assert instance.record_index is None
        assert instance.byte_offset is None
        assert instance.field_tag is None

    def test_unknown_kwarg_raises_type_error(self):
        with pytest.raises(TypeError, match="unexpected keyword"):
            mrrc.InvalidIndicator(not_a_real_field="oops")


# ---------------------------------------------------------------------------
# Pickle round-trip
# ---------------------------------------------------------------------------


class TestPickleRoundTrip:
    def test_round_trip_preserves_all_positional_attrs(self):
        original = mrrc.InvalidIndicator(
            record_index=847,
            record_control_number="ocm01234567",
            field_tag="245",
            indicator_position=1,
            found=b":",
            expected="digit or space",
            byte_offset=7217,
            record_byte_offset=42,
            source="harvest.mrc",
        )
        restored = pickle.loads(pickle.dumps(original))
        assert restored.record_index == 847
        assert restored.record_control_number == "ocm01234567"
        assert restored.field_tag == "245"
        assert restored.indicator_position == 1
        assert restored.found == b":"
        assert restored.expected == "digit or space"
        assert restored.byte_offset == 7217
        assert restored.record_byte_offset == 42
        assert restored.source == "harvest.mrc"

    def test_round_trip_preserves_subclass_extras(self):
        original = mrrc.TruncatedRecord(
            record_index=12,
            expected_length=1024,
            actual_length=640,
        )
        restored = pickle.loads(pickle.dumps(original))
        assert restored.expected_length == 1024
        assert restored.actual_length == 640
        assert restored.record_index == 12

    def test_round_trip_preserves_invalid_field_message(self):
        original = mrrc.InvalidField(
            record_index=5,
            field_tag="245",
            message="exceeds data area",
        )
        restored = pickle.loads(pickle.dumps(original))
        assert restored.message == "exceeds data area"

    def test_round_trip_bare_instance(self):
        original = mrrc.RecordLeaderInvalid()
        restored = pickle.loads(pickle.dumps(original))
        assert restored.record_index is None
        assert restored.byte_offset is None

    def test_setstate_rejects_unexpected_keys(self):
        """Defense-in-depth: restoring a state dict with attribute names
        outside the per-class allowed set must raise rather than blindly
        setattr (which could shadow methods on the instance).
        """
        instance = mrrc.RecordLeaderInvalid()
        with pytest.raises(TypeError, match="unexpected"):
            instance.__setstate__({"_format": "evil_lambda_replacement"})


# ---------------------------------------------------------------------------
# Snapshot tests for str / repr / detailed
# ---------------------------------------------------------------------------


@pytest.fixture
def invalid_indicator_full():
    """Fully-populated InvalidIndicator instance used in several snapshot
    tests; mirrors the Rust-side test fixture so the two sides can be
    cross-compared.
    """
    return mrrc.InvalidIndicator(
        record_index=847,
        record_control_number="ocm01234567",
        field_tag="245",
        indicator_position=1,
        found=b":",
        expected="digit or space",
        byte_offset=7217,
        record_byte_offset=42,
        source="harvest.mrc",
    )


class TestSnapshotFormats:
    def test_str_invalid_indicator_full_context(
        self, invalid_indicator_full, snapshot
    ):
        assert str(invalid_indicator_full) == snapshot

    def test_repr_invalid_indicator_full_context(
        self, invalid_indicator_full, snapshot
    ):
        assert repr(invalid_indicator_full) == snapshot

    def test_detailed_invalid_indicator_full_context(
        self, invalid_indicator_full, snapshot
    ):
        assert invalid_indicator_full.detailed() == snapshot

    def test_str_no_context_falls_back_to_class_name(self, snapshot):
        err = mrrc.BaseAddressNotFound()
        assert str(err) == snapshot

    def test_str_truncated_record(self, snapshot):
        err = mrrc.TruncatedRecord(
            record_index=12,
            record_control_number="oc00000012",
            byte_offset=16384,
            record_byte_offset=128,
            source="partial.mrc",
            expected_length=1024,
            actual_length=640,
        )
        assert str(err) == snapshot

    def test_detailed_truncated_record(self, snapshot):
        err = mrrc.TruncatedRecord(
            record_index=12,
            record_control_number="oc00000012",
            byte_offset=16384,
            record_byte_offset=128,
            source="partial.mrc",
            expected_length=1024,
            actual_length=640,
        )
        assert err.detailed() == snapshot

    def test_str_writer_error(self, snapshot):
        err = mrrc.WriterError(
            record_index=99,
            record_control_number="oc00000099",
            message="Record length exceeds 4GB limit (5000000000 bytes)",
        )
        assert str(err) == snapshot
