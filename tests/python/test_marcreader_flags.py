"""
Tests for MARCReader to_unicode, permissive, and recovery_mode flags (GitHub issue #78).

Verifies pymarc-compatible constructor kwargs and mrrc-native recovery_mode.
"""

import io
import warnings
import pytest
import mrrc


# ============================================================================
# Helpers
# ============================================================================

def _build_valid_record_bytes() -> bytes:
    """Build a minimal valid MARC record and return as bytes."""
    record = mrrc.Record()
    record.add_field(mrrc.Field("001", data="test001"))
    record.add_field(mrrc.Field("008", data="230327s2023    wau    ef     000 1 eng d"))
    f245 = mrrc.Field("245", "1", "0")
    f245.add_subfield("a", "Test title")
    record.add_field(f245)
    output = io.BytesIO()
    writer = mrrc.MARCWriter(output)
    writer.write_record(record)
    writer.close()
    return output.getvalue()


def _build_malformed_record_bytes() -> bytes:
    """Build a record with valid leader length but corrupt directory/data.

    The leader claims 50 bytes total with a base address of 25 (1 directory
    entry of 12 bytes + field terminator). The directory entry has an invalid
    tag and the data area is garbage. This is enough for the batch reader to
    read the correct number of bytes but causes a parse error.
    """
    # Leader: 00050nam  2200025   4500  (50 bytes total, base address 25)
    leader = b"00050nam  2200025   4500"
    # Directory: one 12-byte entry with garbage, then field terminator
    directory = b"XXX000100000" + b"\x1e"
    # Data area: fill to reach 50 bytes total, end with record terminator
    data_needed = 50 - len(leader) - len(directory) - 1  # -1 for record terminator
    data = b"\xff" * data_needed + b"\x1d"
    return leader + directory + data


def _build_two_record_stream(inject_bad_second: bool = False) -> bytes:
    """Build a stream with two records; optionally corrupt the second."""
    good = _build_valid_record_bytes()
    if inject_bad_second:
        bad = _build_malformed_record_bytes()
        return good + bad
    second = good  # just duplicate the valid record
    return good + second


# ============================================================================
# to_unicode tests
# ============================================================================

class TestToUnicode:
    """Test to_unicode kwarg behavior."""

    def test_to_unicode_true_no_warning(self):
        """to_unicode=True (default) should not emit a warning."""
        data = _build_valid_record_bytes()
        with warnings.catch_warnings(record=True) as w:
            warnings.simplefilter("always")
            mrrc.MARCReader(io.BytesIO(data), to_unicode=True)
            assert len(w) == 0

    def test_to_unicode_default_no_warning(self):
        """Default (no to_unicode arg) should not emit a warning."""
        data = _build_valid_record_bytes()
        with warnings.catch_warnings(record=True) as w:
            warnings.simplefilter("always")
            mrrc.MARCReader(io.BytesIO(data))
            assert len(w) == 0

    def test_to_unicode_false_warns(self):
        """to_unicode=False should emit a warning."""
        data = _build_valid_record_bytes()
        with warnings.catch_warnings(record=True) as w:
            warnings.simplefilter("always")
            mrrc.MARCReader(io.BytesIO(data), to_unicode=False)
            assert len(w) == 1
            assert "to_unicode=False has no effect" in str(w[0].message)

    def test_to_unicode_false_still_reads(self):
        """to_unicode=False should still read records normally."""
        data = _build_valid_record_bytes()
        with warnings.catch_warnings(record=True):
            warnings.simplefilter("always")
            reader = mrrc.MARCReader(io.BytesIO(data), to_unicode=False)
            records = list(reader)
            assert len(records) == 1
            assert records[0].get_fields("001")[0].data == "test001"


# ============================================================================
# permissive tests
# ============================================================================

class TestPermissive:
    """Test permissive kwarg behavior (pymarc compatibility)."""

    def test_permissive_false_raises_on_bad_record(self):
        """Default (permissive=False) should raise on malformed records."""
        data = _build_two_record_stream(inject_bad_second=True)
        reader = mrrc.MARCReader(io.BytesIO(data), permissive=False)
        # First record should be fine
        record = next(reader)
        assert record is not None
        # Second record is malformed — should raise
        with pytest.raises(Exception):
            next(reader)

    def test_permissive_true_yields_none_for_bad_record(self):
        """permissive=True should yield None for malformed records."""
        data = _build_two_record_stream(inject_bad_second=True)
        reader = mrrc.MARCReader(io.BytesIO(data), permissive=True)
        records = list(reader)
        # Should get the good record and None for the bad one
        assert any(r is not None for r in records), "Should have at least one valid record"
        assert any(r is None for r in records), "Should have None for malformed record"

    def test_permissive_true_valid_records_normal(self):
        """permissive=True should return valid records normally."""
        data = _build_two_record_stream(inject_bad_second=False)
        reader = mrrc.MARCReader(io.BytesIO(data), permissive=True)
        records = list(reader)
        assert len(records) == 2
        assert all(r is not None for r in records)

    def test_permissive_pymarc_pattern(self):
        """The standard pymarc permissive pattern should work."""
        data = _build_two_record_stream(inject_bad_second=True)
        reader = mrrc.MARCReader(io.BytesIO(data), permissive=True)
        valid_count = 0
        error_count = 0
        for record in reader:
            if record is None:
                error_count += 1
                continue
            valid_count += 1
        assert valid_count >= 1
        assert error_count >= 1


# ============================================================================
# recovery_mode tests
# ============================================================================

class TestRecoveryMode:
    """Test recovery_mode kwarg behavior."""

    def test_recovery_mode_default_strict(self):
        """Default recovery_mode should be strict."""
        data = _build_valid_record_bytes()
        reader = mrrc.MARCReader(io.BytesIO(data))
        records = list(reader)
        assert len(records) == 1

    def test_recovery_mode_lenient(self):
        """recovery_mode='lenient' should be accepted."""
        data = _build_valid_record_bytes()
        reader = mrrc.MARCReader(io.BytesIO(data), recovery_mode="lenient")
        records = list(reader)
        assert len(records) == 1

    def test_recovery_mode_permissive(self):
        """recovery_mode='permissive' should be accepted."""
        data = _build_valid_record_bytes()
        reader = mrrc.MARCReader(io.BytesIO(data), recovery_mode="permissive")
        records = list(reader)
        assert len(records) == 1

    def test_recovery_mode_invalid_raises(self):
        """Invalid recovery_mode should raise ValueError."""
        data = _build_valid_record_bytes()
        with pytest.raises(ValueError, match="Invalid recovery_mode"):
            mrrc.MARCReader(io.BytesIO(data), recovery_mode="invalid")


# ============================================================================
# Conflict validation tests
# ============================================================================

class TestConflictValidation:
    """Test that conflicting options are rejected."""

    def test_permissive_with_lenient_raises(self):
        """permissive=True + recovery_mode='lenient' should raise ValueError."""
        data = _build_valid_record_bytes()
        with pytest.raises(ValueError, match="Cannot combine"):
            mrrc.MARCReader(io.BytesIO(data), permissive=True, recovery_mode="lenient")

    def test_permissive_with_permissive_recovery_raises(self):
        """permissive=True + recovery_mode='permissive' should raise ValueError."""
        data = _build_valid_record_bytes()
        with pytest.raises(ValueError, match="Cannot combine"):
            mrrc.MARCReader(io.BytesIO(data), permissive=True, recovery_mode="permissive")

    def test_permissive_with_strict_ok(self):
        """permissive=True + recovery_mode='strict' (default) should be fine."""
        data = _build_valid_record_bytes()
        reader = mrrc.MARCReader(io.BytesIO(data), permissive=True, recovery_mode="strict")
        records = list(reader)
        assert len(records) == 1
