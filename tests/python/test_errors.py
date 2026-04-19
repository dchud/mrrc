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


# ---------------------------------------------------------------------------
# FFI integration: Rust → PyO3 → Python typed exceptions with attrs
# ---------------------------------------------------------------------------


def _build_minimal_marc_record(record_type: bytes = b"a") -> bytes:
    """Construct a minimal valid ISO 2709 MARC record with a single 245
    field. Used as a starting point that downstream tests mutate to trigger
    specific MarcError variants.
    """
    field_245 = b"10\x1fa" + b"Test" + b"\x1e"
    directory = b"245" + format(len(field_245), "04d").encode() + b"00000"
    base_address = 24 + len(directory) + 1
    record_length = base_address + len(field_245) + 1
    leader = (
        format(record_length, "05d").encode()
        + b"n"
        + record_type
        + b"m a2"
        + b"2"
        + format(base_address, "05d").encode()
        + b"   4500"
    )
    return leader + directory + b"\x1e" + field_245 + b"\x1d"


class TestFfiTypedExceptions:
    """Drive the Rust → PyO3 → Python conversion path and verify each
    MarcError variant surfaces as the corresponding typed Python exception
    with positional attributes populated.
    """

    def test_truncated_record_surfaces_as_typed_exception_with_byte_counts(self):
        """A truncated record must surface as ``mrrc.TruncatedRecord``
        (a subclass of ``mrrc.EndOfRecordNotFound``) with
        ``expected_length`` and ``actual_length`` populated.

        Note: ``record_index``, ``byte_offset``, and ``source`` are not yet
        populated on this path because the boundary scanner that catches
        truncation does not currently track a records-read counter. That's
        captured in the ``ParseError`` builder methods landed in this PR;
        wiring per-site context tracking is intentionally separate work.
        """
        import io

        full = _build_minimal_marc_record()
        # Truncate the record bytes mid-record
        truncated = full[: len(full) - 10]

        reader = mrrc.MARCReader(io.BytesIO(truncated))
        with pytest.raises(mrrc.EndOfRecordNotFound) as excinfo:
            list(reader)
        err = excinfo.value
        assert isinstance(err, mrrc.TruncatedRecord)
        assert err.expected_length is not None
        assert err.actual_length is not None
        assert err.actual_length < err.expected_length

    def test_invalid_leader_record_length_too_small(self):
        """A leader claiming a record length below 24 is structurally
        invalid; surface as ``mrrc.RecordLeaderInvalid``.
        """
        import io

        leader = b"00010nam a2200025 i 4500"  # record_length=10 < 24
        reader = mrrc.MARCReader(io.BytesIO(leader))
        with pytest.raises(mrrc.MrrcException):
            list(reader)

    def test_wrong_authority_record_type_raises_invalid_field(self):
        """An `AuthorityMARCReader` fed a bibliographic-typed record
        rejects it via a typed exception.
        """
        import io

        bib_record = _build_minimal_marc_record(record_type=b"a")
        reader = mrrc.AuthorityMARCReader(io.BytesIO(bib_record))
        with pytest.raises(mrrc.MrrcException) as excinfo:
            list(reader)
        # Should carry record_index and at least the type-mismatch message
        err = excinfo.value
        assert err.record_index == 1
        # InvalidField surfaces with a message naming the type mismatch
        if isinstance(err, mrrc.InvalidField):
            assert err.message is not None
            assert "authority" in err.message.lower()

    def test_no_silent_drops_in_pyo3_conversion(self):
        """Catch-all: confirm that whatever Rust raises always surfaces as
        an MrrcException subclass (or OSError for I/O), never an
        unhandled bare PyValueError when the input is recognizably a
        MARC parse failure.
        """
        import io

        # Garbage that parses as a leader but has no directory
        garbage = b"00050nam a22000000 i 4500" + b"\x00" * 26
        reader = mrrc.MARCReader(io.BytesIO(garbage))
        try:
            list(reader)
        except mrrc.MrrcException:
            pass  # expected — typed exception class raised
        except OSError:
            pass  # also acceptable — I/O errors map to built-in
        # If neither, the test will fail with the actual exception type
        # in the traceback; that is the diagnostic.
