"""
Tests for repeated control fields (GitHub issue #77).

MARC records can have repeated control fields, particularly 006 and 007.
This test suite verifies that mrrc correctly preserves all instances of
repeated control fields through parsing, access, serialization, and
round-tripping.

The bug: control fields were stored in IndexMap<String, String>, causing
later values to overwrite earlier ones for the same tag.

Reference record: NLS catalog BRC01945
https://search.nlscatalog.loc.gov/instances/f851ab51-0ca0-516a-8871-002de5d575b3
"""

import io
import pytest
import mrrc

try:
    import pymarc
    HAS_PYMARC = True
except ImportError:
    HAS_PYMARC = False

requires_pymarc = pytest.mark.skipif(
    not HAS_PYMARC, reason="pymarc not installed"
)


# ============================================================================
# Fixture builders — mrrc-only (no pymarc dependency)
# ============================================================================

def _build_record_with_two_007s_mrrc() -> bytes:
    """Build a MARC record with two 007 fields using mrrc, return as bytes.

    Same content as the NLS catalog record (BRC01945) which has both a
    computer resource 007 and a microform 007.
    """
    record = mrrc.Record()
    record.add_field(mrrc.Field("001", data="1118690"))
    record.add_field(mrrc.Field("007", data="cr|nn ||||||aa"))
    record.add_field(mrrc.Field("007", data="fb|a bnnnn"))
    record.add_field(mrrc.Field("008", data="230327s2023    wau    ef     000 1 eng d"))
    f245 = mrrc.Field("245", "1", "0")
    f245.add_subfield("a", "Test title")
    record.add_field(f245)
    output = io.BytesIO()
    writer = mrrc.MARCWriter(output)
    writer.write_record(record)
    writer.close()
    return output.getvalue()


def _build_record_with_repeated_006_007_mrrc() -> bytes:
    """Build a MARC record with repeated 006 AND 007 fields using mrrc."""
    record = mrrc.Record()
    record.add_field(mrrc.Field("001", data="9999999"))
    record.add_field(mrrc.Field("006", data="m     o  d        "))
    record.add_field(mrrc.Field("006", data="j  s          n   "))
    record.add_field(mrrc.Field("007", data="cr |||||||||||"))
    record.add_field(mrrc.Field("007", data="sd fsngnnmmned"))
    record.add_field(mrrc.Field("007", data="vf cbahos"))
    record.add_field(mrrc.Field("008", data="230327s2023    wau    ef     000 1 eng d"))
    f245 = mrrc.Field("245", "1", "0")
    f245.add_subfield("a", "Multi-format item")
    record.add_field(f245)
    output = io.BytesIO()
    writer = mrrc.MARCWriter(output)
    writer.write_record(record)
    writer.close()
    return output.getvalue()


# ============================================================================
# Fixture builders — pymarc (for parity tests)
# ============================================================================

def _build_record_with_two_007s_pymarc() -> bytes:
    """Build a MARC record with two 007 fields using pymarc, return as bytes."""
    record = pymarc.Record()
    record.add_field(pymarc.Field(tag="001", data="1118690"))
    record.add_field(pymarc.Field(tag="007", data="cr|nn ||||||aa"))
    record.add_field(pymarc.Field(tag="007", data="fb|a bnnnn"))
    record.add_field(
        pymarc.Field(
            tag="008",
            data="230327s2023    wau    ef     000 1 eng d",
        )
    )
    record.add_field(
        pymarc.Field(
            tag="245",
            indicators=["1", "0"],
            subfields=[pymarc.Subfield(code="a", value="Test title")],
        )
    )
    return record.as_marc()


def _build_record_with_repeated_006_007_pymarc() -> bytes:
    """Build a MARC record with repeated 006 AND 007 fields using pymarc."""
    record = pymarc.Record()
    record.add_field(pymarc.Field(tag="001", data="9999999"))
    record.add_field(pymarc.Field(tag="006", data="m     o  d        "))
    record.add_field(pymarc.Field(tag="006", data="j  s          n   "))
    record.add_field(pymarc.Field(tag="007", data="cr |||||||||||"))
    record.add_field(pymarc.Field(tag="007", data="sd fsngnnmmned"))
    record.add_field(pymarc.Field(tag="007", data="vf cbahos"))
    record.add_field(
        pymarc.Field(
            tag="008",
            data="230327s2023    wau    ef     000 1 eng d",
        )
    )
    record.add_field(
        pymarc.Field(
            tag="245",
            indicators=["1", "0"],
            subfields=[pymarc.Subfield(code="a", value="Multi-format item")],
        )
    )
    return record.as_marc()


# ============================================================================
# mrrc-only tests (always run, no pymarc dependency)
# ============================================================================

class TestMrrcRepeatedControlFields:
    """Test that mrrc preserves repeated control fields through parsing."""

    def test_mrrc_returns_two_007s(self):
        """mrrc must return both 007 fields from a parsed record."""
        marc_bytes = _build_record_with_two_007s_mrrc()
        reader = mrrc.MARCReader(io.BytesIO(marc_bytes))
        record = reader.read_record()

        fields_007 = record.get_fields("007")
        assert len(fields_007) == 2, (
            f"Expected 2 007 fields, got {len(fields_007)}: "
            f"{[f.data for f in fields_007]}"
        )
        assert fields_007[0].data == "cr|nn ||||||aa"
        assert fields_007[1].data == "fb|a bnnnn"

    def test_mrrc_returns_repeated_006_and_007(self):
        """mrrc must return all repeated 006 and 007 fields."""
        marc_bytes = _build_record_with_repeated_006_007_mrrc()
        reader = mrrc.MARCReader(io.BytesIO(marc_bytes))
        record = reader.read_record()

        fields_006 = record.get_fields("006")
        assert len(fields_006) == 2, (
            f"Expected 2 006 fields, got {len(fields_006)}"
        )

        fields_007 = record.get_fields("007")
        assert len(fields_007) == 3, (
            f"Expected 3 007 fields, got {len(fields_007)}"
        )

    def test_mrrc_add_field_preserves_repeated_control_fields(self):
        """Adding multiple control fields with the same tag must preserve all."""
        record = mrrc.Record()
        record.add_field(mrrc.Field("007", data="cr|nn ||||||aa"))
        record.add_field(mrrc.Field("007", data="fb|a bnnnn"))

        fields_007 = record.get_fields("007")
        assert len(fields_007) == 2
        assert fields_007[0].data == "cr|nn ||||||aa"
        assert fields_007[1].data == "fb|a bnnnn"


class TestMrrcRepeatedControlFieldRoundTrip:
    """Test round-tripping records with repeated control fields."""

    def test_roundtrip_preserves_two_007s(self):
        """Serialize then parse must preserve repeated 007 fields."""
        marc_bytes = _build_record_with_two_007s_mrrc()
        reader = mrrc.MARCReader(io.BytesIO(marc_bytes))
        record = reader.read_record()

        # Re-serialize
        output = io.BytesIO()
        writer = mrrc.MARCWriter(output)
        writer.write_record(record)
        writer.close()

        # Re-parse
        reader2 = mrrc.MARCReader(io.BytesIO(output.getvalue()))
        record2 = reader2.read_record()

        fields_007 = record2.get_fields("007")
        assert len(fields_007) == 2
        assert fields_007[0].data == "cr|nn ||||||aa"
        assert fields_007[1].data == "fb|a bnnnn"

    def test_roundtrip_preserves_leader_length(self):
        """Record length in leader must account for all control fields."""
        marc_bytes = _build_record_with_two_007s_mrrc()
        original_length = int(marc_bytes[:5].decode("ascii"))

        reader = mrrc.MARCReader(io.BytesIO(marc_bytes))
        record = reader.read_record()

        output = io.BytesIO()
        writer = mrrc.MARCWriter(output)
        writer.write_record(record)
        writer.close()

        roundtrip_bytes = output.getvalue()
        roundtrip_length = int(roundtrip_bytes[:5].decode("ascii"))

        assert roundtrip_length == original_length, (
            f"Round-trip leader length {roundtrip_length} != original "
            f"{original_length} — likely a repeated control field was dropped"
        )

    def test_all_fields_returns_all_control_fields(self):
        """get_fields() with no args must include all repeated control fields."""
        marc_bytes = _build_record_with_two_007s_mrrc()

        reader = mrrc.MARCReader(io.BytesIO(marc_bytes))
        record = reader.read_record()

        all_fields = record.get_fields()
        tags_007 = [f for f in all_fields if f.tag == "007"]
        assert len(tags_007) == 2


# ============================================================================
# Pymarc parity tests (skipped when pymarc is not installed)
# ============================================================================

@requires_pymarc
class TestPymarcBaselineRepeatedControlFields:
    """Verify pymarc itself handles repeated control fields correctly.

    These tests establish the expected behavior that mrrc must match.
    """

    def test_pymarc_returns_two_007s(self):
        """Baseline: pymarc returns both 007 fields."""
        marc_bytes = _build_record_with_two_007s_pymarc()
        reader = pymarc.MARCReader(io.BytesIO(marc_bytes))
        record = next(reader)

        fields_007 = record.get_fields("007")
        assert len(fields_007) == 2
        assert fields_007[0].data == "cr|nn ||||||aa"
        assert fields_007[1].data == "fb|a bnnnn"

    def test_pymarc_returns_repeated_006_and_007(self):
        """Baseline: pymarc returns all repeated 006 and 007 fields."""
        marc_bytes = _build_record_with_repeated_006_007_pymarc()
        reader = pymarc.MARCReader(io.BytesIO(marc_bytes))
        record = next(reader)

        fields_006 = record.get_fields("006")
        assert len(fields_006) == 2

        fields_007 = record.get_fields("007")
        assert len(fields_007) == 3


@requires_pymarc
class TestMrrcPymarcControlFieldParity:
    """Direct comparison between pymarc and mrrc output for repeated fields."""

    def test_get_fields_returns_same_count(self):
        """get_fields('007') must return same count in pymarc and mrrc."""
        marc_bytes = _build_record_with_two_007s_pymarc()

        pymarc_reader = pymarc.MARCReader(io.BytesIO(marc_bytes))
        pymarc_record = next(pymarc_reader)

        mrrc_reader = mrrc.MARCReader(io.BytesIO(marc_bytes))
        mrrc_record = mrrc_reader.read_record()

        pymarc_007s = pymarc_record.get_fields("007")
        mrrc_007s = mrrc_record.get_fields("007")

        assert len(mrrc_007s) == len(pymarc_007s), (
            f"mrrc returned {len(mrrc_007s)} 007 fields but pymarc returned "
            f"{len(pymarc_007s)}"
        )

    def test_get_fields_returns_same_values(self):
        """get_fields('007') values must match between pymarc and mrrc."""
        marc_bytes = _build_record_with_two_007s_pymarc()

        pymarc_reader = pymarc.MARCReader(io.BytesIO(marc_bytes))
        pymarc_record = next(pymarc_reader)

        mrrc_reader = mrrc.MARCReader(io.BytesIO(marc_bytes))
        mrrc_record = mrrc_reader.read_record()

        pymarc_values = [f.data for f in pymarc_record.get_fields("007")]
        mrrc_values = [f.data for f in mrrc_record.get_fields("007")]

        assert mrrc_values == pymarc_values
