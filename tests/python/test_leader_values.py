"""
Unit tests for Leader value lookup helpers and validation.

Tests MARC 21 spec-compliant leader position value mappings and descriptions.
Ensures API parity with pymarc while providing enhanced functionality.
"""

import pytest
from mrrc import Leader


class TestLeaderValidValuesPosition5:
    """Test Leader valid values for position 5 (Record status)."""

    def test_get_valid_values_position_5(self):
        """Test retrieving valid values for record status position."""
        leader = Leader()
        values = leader.get_valid_values(5)

        assert isinstance(values, dict)
        assert len(values) > 0
        assert "a" in values
        assert "n" in values

    def test_record_status_values_mapped_correctly(self):
        """Test that record status values map to correct descriptions."""
        leader = Leader()
        values = leader.get_valid_values(5)

        assert values["a"] == "Increase in encoding level"
        assert values["c"] == "Corrected or revised"
        assert values["d"] == "Deleted"
        assert values["n"] == "New"
        assert values["p"] == "Increase in encoding level from prepublication"

    def test_describe_value_position_5(self):
        """Test describing a specific record status value."""
        desc = Leader.describe_value(5, "a")
        assert desc == "Increase in encoding level"

        desc = Leader.describe_value(5, "n")
        assert desc == "New"

    def test_describe_invalid_value_position_5(self):
        """Test that invalid values return None."""
        desc = Leader.describe_value(5, "x")
        assert desc is None


class TestLeaderValidValuesPosition6:
    """Test Leader valid values for position 6 (Type of record)."""

    def test_get_valid_values_position_6(self):
        """Test retrieving valid values for record type position."""
        leader = Leader()
        values = leader.get_valid_values(6)

        assert len(values) > 0
        assert "a" in values
        assert "m" in values

    def test_record_type_values_mapped_correctly(self):
        """Test that record type values map to correct descriptions."""
        leader = Leader()
        values = leader.get_valid_values(6)

        assert values["a"] == "Language material"
        assert values["c"] == "Notated music"
        assert values["m"] == "Computer file"
        assert values["t"] == "Manuscript language material"

    def test_describe_value_position_6(self):
        """Test describing record type values."""
        assert Leader.describe_value(6, "a") == "Language material"
        assert Leader.describe_value(6, "t") == "Manuscript language material"


class TestLeaderValidValuesPosition7:
    """Test Leader valid values for position 7 (Bibliographic level)."""

    def test_get_valid_values_position_7(self):
        """Test retrieving valid values for bibliographic level position."""
        leader = Leader()
        values = leader.get_valid_values(7)

        assert len(values) > 0
        assert "m" in values
        assert "s" in values

    def test_bibliographic_level_values_mapped_correctly(self):
        """Test that bibliographic level values map to correct descriptions."""
        values = Leader.get_valid_values(7)

        assert values["a"] == "Monographic component part"
        assert values["b"] == "Serial component part"
        assert values["m"] == "Monograph/Item"
        assert values["s"] == "Serial"
        assert values["i"] == "Integrating resource"

    def test_describe_value_position_7(self):
        """Test describing bibliographic level values."""
        assert Leader.describe_value(7, "m") == "Monograph/Item"
        assert Leader.describe_value(7, "s") == "Serial"


class TestLeaderValidValuesPosition8:
    """Test Leader valid values for position 8 (Type of control)."""

    def test_get_valid_values_position_8(self):
        """Test retrieving valid values for type of control position."""
        values = Leader.get_valid_values(8)

        assert "#" in values
        assert "a" in values

    def test_type_of_control_values_mapped_correctly(self):
        """Test that type of control values map to correct descriptions."""
        values = Leader.get_valid_values(8)

        assert values["#"] == "No specified type"
        assert values["a"] == "Archival"

    def test_describe_value_position_8(self):
        """Test describing type of control values."""
        assert Leader.describe_value(8, "#") == "No specified type"
        assert Leader.describe_value(8, "a") == "Archival"


class TestLeaderValidValuesPosition9:
    """Test Leader valid values for position 9 (Character coding scheme)."""

    def test_get_valid_values_position_9(self):
        """Test retrieving valid values for character coding position."""
        values = Leader.get_valid_values(9)

        assert " " in values
        assert "a" in values

    def test_character_coding_values_mapped_correctly(self):
        """Test that character coding values map to correct descriptions."""
        values = Leader.get_valid_values(9)

        assert values[" "] == "MARC-8"
        assert values["a"] == "UCS/Unicode"

    def test_describe_value_position_9(self):
        """Test describing character coding values."""
        assert Leader.describe_value(9, " ") == "MARC-8"
        assert Leader.describe_value(9, "a") == "UCS/Unicode"


class TestLeaderValidValuesPosition17:
    """Test Leader valid values for position 17 (Encoding level)."""

    def test_get_valid_values_position_17(self):
        """Test retrieving valid values for encoding level position."""
        values = Leader.get_valid_values(17)

        assert len(values) > 0
        assert " " in values
        assert "8" in values

    def test_encoding_level_values_mapped_correctly(self):
        """Test that encoding level values map to correct descriptions."""
        values = Leader.get_valid_values(17)

        assert values[" "] == "Full level"
        assert values["3"] == "Abbreviated level"
        assert values["8"] == "Prepublication level"

    def test_describe_value_position_17(self):
        """Test describing encoding level values."""
        assert Leader.describe_value(17, " ") == "Full level"
        assert Leader.describe_value(17, "8") == "Prepublication level"


class TestLeaderValidValuesPosition18:
    """Test Leader valid values for position 18 (Cataloging form)."""

    def test_get_valid_values_position_18(self):
        """Test retrieving valid values for cataloging form position."""
        values = Leader.get_valid_values(18)

        assert " " in values
        assert "a" in values

    def test_cataloging_form_values_mapped_correctly(self):
        """Test that cataloging form values map to correct descriptions."""
        values = Leader.get_valid_values(18)

        assert values[" "] == "Non-ISBD"
        assert values["a"] == "AACR 2"
        assert values["c"] == "ISBD punctuation omitted"
        assert values["i"] == "ISBD punctuation included"
        assert values["n"] == "Non-ISBD punctuation omitted"

    def test_describe_value_position_18(self):
        """Test describing cataloging form values."""
        assert Leader.describe_value(18, " ") == "Non-ISBD"
        assert Leader.describe_value(18, "a") == "AACR 2"
        assert Leader.describe_value(18, "i") == "ISBD punctuation included"


class TestLeaderValidValuesPosition19:
    """Test Leader valid values for position 19 (Multipart level)."""

    def test_get_valid_values_position_19(self):
        """Test retrieving valid values for multipart level position."""
        values = Leader.get_valid_values(19)

        assert " " in values
        assert "a" in values

    def test_multipart_level_values_mapped_correctly(self):
        """Test that multipart level values map to correct descriptions."""
        values = Leader.get_valid_values(19)

        assert values[" "] == "Not specified or not applicable"
        assert values["a"] == "Set"
        assert values["b"] == "Part with independent title"

    def test_describe_value_position_19(self):
        """Test describing multipart level values."""
        assert (
            Leader.describe_value(19, " ") == "Not specified or not applicable"
        )
        assert Leader.describe_value(19, "a") == "Set"


class TestLeaderValidValuesInvalidPosition:
    """Test Leader behavior with invalid positions."""

    def test_get_valid_values_invalid_position(self):
        """Test that invalid positions return None."""
        values = Leader.get_valid_values(0)
        assert values is None

        values = Leader.get_valid_values(99)
        assert values is None

    def test_describe_value_invalid_position(self):
        """Test that invalid positions return None."""
        desc = Leader.describe_value(0, "a")
        assert desc is None

        desc = Leader.describe_value(99, "x")
        assert desc is None


class TestLeaderValueValidationIntegration:
    """Integration tests for leader values with actual leader manipulation."""

    def test_set_and_describe_record_status(self):
        """Test setting record status and getting its description."""
        leader = Leader()
        leader.record_status = "a"

        # Get the actual value set
        assert leader.record_status == "a"

        # Get its description
        desc = Leader.describe_value(5, leader.record_status)
        assert desc == "Increase in encoding level"

    def test_set_and_describe_record_type(self):
        """Test setting record type and getting its description."""
        leader = Leader()
        leader.record_type = "t"

        assert leader.record_type == "t"

        desc = Leader.describe_value(6, leader.record_type)
        assert desc == "Manuscript language material"

    def test_set_and_describe_bibliographic_level(self):
        """Test setting bibliographic level and getting its description."""
        leader = Leader()
        leader.bibliographic_level = "s"

        assert leader.bibliographic_level == "s"

        desc = Leader.describe_value(7, leader.bibliographic_level)
        assert desc == "Serial"

    def test_describe_all_valid_values_position_5(self):
        """Test that all values in the valid_values dict are describable."""
        valid_values = Leader.get_valid_values(5)

        for value in valid_values.keys():
            desc = Leader.describe_value(5, value)
            assert desc is not None
            assert desc == valid_values[value]

    def test_describe_all_valid_values_position_6(self):
        """Test that all values in the valid_values dict are describable."""
        valid_values = Leader.get_valid_values(6)

        for value in valid_values.keys():
            desc = Leader.describe_value(6, value)
            assert desc is not None
            assert desc == valid_values[value]


class TestLeaderStaticMethods:
    """Test that helper methods are accessible as both static and instance methods."""

    def test_static_method_access(self):
        """Test accessing helpers via static method syntax."""
        values = Leader.get_valid_values(5)
        assert len(values) > 0

        desc = Leader.describe_value(5, "n")
        assert desc == "New"

    def test_instance_method_access(self):
        """Test accessing get_valid_values via instance method."""
        leader = Leader()
        values = leader.get_valid_values(5)
        assert len(values) > 0

    def test_describe_value_static_method(self):
        """Test describe_value as static method on instance."""
        leader = Leader()
        # Static methods should be callable on instances too
        desc = leader.describe_value(5, "n")
        assert desc == "New"


class TestLeaderAPICompatibility:
    """Test API compatibility with pymarc expectations."""

    def test_leader_values_not_regress_existing_api(self):
        """Test that new methods don't break existing leader API."""
        leader = Leader()

        # Existing API should still work
        leader.record_status = "a"
        assert leader.record_status == "a"

        leader.record_type = "t"
        assert leader.record_type == "t"

        leader.bibliographic_level = "m"
        assert leader.bibliographic_level == "m"

    def test_descriptor_returns_consistent_data(self):
        """Test that describe_value always returns consistent data."""
        desc1 = Leader.describe_value(5, "a")
        desc2 = Leader.describe_value(5, "a")
        assert desc1 == desc2

    def test_valid_values_dict_format(self):
        """Test that get_valid_values returns proper dictionary format."""
        values = Leader.get_valid_values(5)

        assert isinstance(values, dict)
        for key, val in values.items():
            assert isinstance(key, str)
            assert isinstance(val, str)


class TestLeaderDocumentation:
    """Test that helper methods have proper documentation."""

    def test_get_valid_values_docstring(self):
        """Test that get_valid_values has docstring."""
        leader = Leader()
        assert leader.get_valid_values.__doc__ is not None
        assert len(leader.get_valid_values.__doc__) > 0

    def test_describe_value_docstring(self):
        """Test that describe_value has docstring."""
        assert Leader.describe_value.__doc__ is not None
        assert len(Leader.describe_value.__doc__) > 0
