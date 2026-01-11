"""
Tests for the Query DSL - advanced field searching capabilities.

The Query DSL provides powerful field filtering that goes beyond pymarc's
simple get_fields(*tags) method. It supports:
- Indicator filtering (find 650 fields with ind2='0' for LCSH)
- Tag range queries (find all 6XX subject fields)
- Subfield requirements (fields must have specific subfield codes)
- Regex pattern matching on subfield values
- Exact/partial string matching on subfield values
"""

import pytest
from mrrc import (
    Record,
    Field,
    Leader,
    FieldQuery,
    TagRangeQuery,
    SubfieldPatternQuery,
    SubfieldValueQuery,
)


# =============================================================================
# Test Fixtures
# =============================================================================


def create_field(tag, ind1=" ", ind2=" ", **subfields):
    """Helper to create a field with subfields."""
    field = Field(tag, ind1, ind2)
    for code, value in subfields.items():
        field.add_subfield(code, value)
    return field


def create_test_record():
    """Create a bibliographic record with diverse fields for testing queries."""
    leader = Leader()
    record = Record(leader)

    # Control fields
    record.add_control_field("001", "12345")
    record.add_control_field("008", "210101s2021    nyu           000 0 eng d")

    # ISBN fields - mix of ISBN-10 and ISBN-13
    record.add_field(create_field("020", a="978-0-12-345678-9"))
    record.add_field(create_field("020", a="0-12-345678-X"))
    record.add_field(create_field("020", a="979-10-12-345678-7"))

    # Title field
    record.add_field(create_field("245", "1", "0", a="The Great Book", b="a subtitle", c="Author Name"))

    # Authors with dates
    record.add_field(create_field("100", "1", " ", a="Smith, John", d="1900-1980"))
    record.add_field(create_field("700", "1", " ", a="Jones, Mary", d="1950-"))

    # Subject headings - mix of LCSH (ind2=0), MeSH (ind2=2), and local (ind2=4)
    record.add_field(create_field("650", " ", "0", a="History", x="20th century", v="Periodicals"))
    record.add_field(create_field("650", " ", "0", a="Science", x="History"))
    record.add_field(create_field("650", " ", "2", a="Medical Subject", x="therapy"))
    record.add_field(create_field("650", " ", "4", a="Local Subject"))
    record.add_field(create_field("651", " ", "0", a="United States", x="History"))
    record.add_field(create_field("600", "1", "0", a="Lincoln, Abraham", d="1809-1865"))

    # Notes
    record.add_field(create_field("500", a="General note."))
    record.add_field(create_field("504", a="Includes bibliographical references."))

    return record


# =============================================================================
# FieldQuery Tests
# =============================================================================


class TestFieldQuery:
    """Test the FieldQuery builder pattern."""

    def test_empty_query_matches_all(self):
        """An empty query should match all fields."""
        record = create_test_record()
        query = FieldQuery()
        results = record.fields_matching(query)
        # Should get all fields (excluding control fields)
        assert len(results) > 0

    def test_query_by_tag(self):
        """Query by tag only."""
        record = create_test_record()
        query = FieldQuery().tag("650")
        results = record.fields_matching(query)
        assert len(results) == 4  # 4 650 fields in our test record
        for field in results:
            assert field.tag == "650"

    def test_query_by_indicator1(self):
        """Query by first indicator."""
        record = create_test_record()
        query = FieldQuery().tag("100").indicator1("1")
        results = record.fields_matching(query)
        assert len(results) == 1
        assert results[0].indicator1 == "1"

    def test_query_by_indicator2(self):
        """Query by second indicator - common pattern for subject thesaurus."""
        record = create_test_record()
        # Find LCSH subjects (indicator2 = '0')
        query = FieldQuery().tag("650").indicator2("0")
        results = record.fields_matching(query)
        assert len(results) == 2  # "History" and "Science"
        for field in results:
            assert field.indicator2 == "0"

    def test_query_with_required_subfield(self):
        """Query with required subfield presence."""
        record = create_test_record()
        # Find 650 fields that have subdivision ($x)
        query = FieldQuery().tag("650").has_subfield("x")
        results = record.fields_matching(query)
        assert len(results) == 3  # History, Science, Medical Subject have $x
        for field in results:
            values = field.subfields_by_code("x")
            assert len(values) > 0

    def test_query_with_multiple_required_subfields(self):
        """Query requiring multiple subfields (AND logic)."""
        record = create_test_record()
        # Find 650 fields that have both $a and $v
        query = FieldQuery().tag("650").has_subfield("a").has_subfield("v")
        results = record.fields_matching(query)
        assert len(results) == 1  # Only "History" has $v (Periodicals)

    def test_query_with_has_subfields_list(self):
        """Test has_subfields() with a list of codes."""
        record = create_test_record()
        query = FieldQuery().tag("650").has_subfields(["a", "x"])
        results = record.fields_matching(query)
        assert len(results) == 3

    def test_combined_query(self):
        """Query combining tag, indicator, and subfield requirements."""
        record = create_test_record()
        # Find LCSH 650 fields with $x subdivision
        query = FieldQuery().tag("650").indicator2("0").has_subfield("x")
        results = record.fields_matching(query)
        assert len(results) == 2  # History and Science have ind2=0 and $x

    def test_query_indicator_wildcard(self):
        """None indicator means wildcard (match any)."""
        record = create_test_record()
        query = FieldQuery().tag("650").indicator1(None).indicator2("0")
        results = record.fields_matching(query)
        assert len(results) == 2  # ind1 can be anything, ind2 must be '0'

    def test_query_repr(self):
        """Test string representation of FieldQuery."""
        query = FieldQuery().tag("650").indicator2("0").has_subfield("a")
        repr_str = repr(query)
        assert "FieldQuery" in repr_str
        assert "650" in repr_str
        assert "0" in repr_str


# =============================================================================
# TagRangeQuery Tests
# =============================================================================


class TestTagRangeQuery:
    """Test tag range queries for finding groups of related fields."""

    def test_subject_range(self):
        """Find all subject fields (600-699)."""
        record = create_test_record()
        query = TagRangeQuery("600", "699")
        results = record.fields_matching_range(query)
        # 4 650s + 1 651 + 1 600 = 6 subject fields
        assert len(results) == 6
        for field in results:
            assert "600" <= field.tag <= "699"

    def test_range_with_indicator_filter(self):
        """Find all 6XX fields with indicator2='0' (LCSH)."""
        record = create_test_record()
        query = TagRangeQuery("600", "699", indicator2="0")
        results = record.fields_matching_range(query)
        # 600, 650 (2), 651 with ind2=0 = 4
        assert len(results) == 4
        for field in results:
            assert field.indicator2 == "0"

    def test_range_with_required_subfields(self):
        """Find range with required subfields."""
        record = create_test_record()
        query = TagRangeQuery("600", "699", required_subfields=["x"])
        results = record.fields_matching_range(query)
        # Fields with $x: 650 (3) + 651 (1) = 4
        assert len(results) == 4

    def test_tag_in_range_method(self):
        """Test the tag_in_range() method."""
        query = TagRangeQuery("600", "699")
        assert query.tag_in_range("600") is True
        assert query.tag_in_range("650") is True
        assert query.tag_in_range("699") is True
        assert query.tag_in_range("599") is False
        assert query.tag_in_range("700") is False

    def test_range_getters(self):
        """Test property getters."""
        query = TagRangeQuery("600", "699", indicator1="1", indicator2="0")
        assert query.start_tag == "600"
        assert query.end_tag == "699"
        assert query.indicator1 == "1"
        assert query.indicator2 == "0"

    def test_range_from_field_query(self):
        """Test creating TagRangeQuery from FieldQuery.tag_range()."""
        query = FieldQuery().indicator2("0").has_subfield("a").tag_range("600", "699")
        record = create_test_record()
        results = record.fields_matching_range(query)
        # All 6XX fields with ind2=0 and $a
        assert len(results) == 4


# =============================================================================
# SubfieldPatternQuery Tests
# =============================================================================


class TestSubfieldPatternQuery:
    """Test regex pattern matching on subfield values."""

    def test_isbn13_pattern(self):
        """Find ISBN-13s starting with 978."""
        record = create_test_record()
        query = SubfieldPatternQuery("020", "a", r"^978-")
        results = record.fields_matching_pattern(query)
        assert len(results) == 1
        assert "978-0-12-345678-9" in results[0].subfields_by_code("a")

    def test_isbn13_both_prefixes(self):
        """Find all ISBN-13s (978 or 979 prefix)."""
        record = create_test_record()
        query = SubfieldPatternQuery("020", "a", r"^97[89]-")
        results = record.fields_matching_pattern(query)
        assert len(results) == 2  # 978 and 979 ISBNs

    def test_date_range_pattern(self):
        """Find personal names with death dates (date ranges)."""
        record = create_test_record()
        # Match YYYY-YYYY pattern (birth-death)
        query = SubfieldPatternQuery("100", "d", r"\d{4}-\d{4}")
        results = record.fields_matching_pattern(query)
        assert len(results) == 1
        assert "1900-1980" in results[0].subfields_by_code("d")

    def test_open_date_pattern(self):
        """Find persons with open dates (living or unknown death)."""
        record = create_test_record()
        # Match YYYY- pattern (birth only, no death)
        query = SubfieldPatternQuery("700", "d", r"^\d{4}-$")
        results = record.fields_matching_pattern(query)
        assert len(results) == 1
        assert "1950-" in results[0].subfields_by_code("d")

    def test_invalid_regex_raises_error(self):
        """Invalid regex pattern should raise ValueError."""
        with pytest.raises(ValueError) as excinfo:
            SubfieldPatternQuery("020", "a", r"[invalid")
        assert "Invalid regex pattern" in str(excinfo.value)

    def test_empty_subfield_code_raises_error(self):
        """Empty subfield code should raise ValueError."""
        with pytest.raises(ValueError):
            SubfieldPatternQuery("020", "", r"^978-")

    def test_pattern_getters(self):
        """Test property getters."""
        query = SubfieldPatternQuery("020", "a", r"^978-")
        assert query.tag == "020"
        assert query.subfield_code == "a"


# =============================================================================
# SubfieldValueQuery Tests
# =============================================================================


class TestSubfieldValueQuery:
    """Test exact and partial string matching on subfield values."""

    def test_exact_match(self):
        """Find exact subject heading match."""
        record = create_test_record()
        query = SubfieldValueQuery("650", "a", "History")
        results = record.fields_matching_value(query)
        assert len(results) == 1
        assert "History" in results[0].subfields_by_code("a")

    def test_exact_match_case_sensitive(self):
        """Exact match is case-sensitive."""
        record = create_test_record()
        query = SubfieldValueQuery("650", "a", "history")  # lowercase
        results = record.fields_matching_value(query)
        assert len(results) == 0  # No match - "History" vs "history"

    def test_partial_match(self):
        """Find subjects containing a substring."""
        record = create_test_record()
        query = SubfieldValueQuery("650", "a", "Subject", partial=True)
        results = record.fields_matching_value(query)
        # "Medical Subject" and "Local Subject"
        assert len(results) == 2

    def test_partial_match_in_subdivision(self):
        """Find partial match in subdivision."""
        record = create_test_record()
        query = SubfieldValueQuery("650", "x", "History", partial=True)
        results = record.fields_matching_value(query)
        assert len(results) == 1  # Science > History

    def test_empty_subfield_code_raises_error(self):
        """Empty subfield code should raise ValueError."""
        with pytest.raises(ValueError):
            SubfieldValueQuery("650", "", "History")

    def test_value_getters(self):
        """Test property getters."""
        query = SubfieldValueQuery("650", "a", "History", partial=True)
        assert query.tag == "650"
        assert query.subfield_code == "a"
        assert query.value == "History"
        assert query.partial is True


# =============================================================================
# Convenience Method Tests
# =============================================================================


class TestConvenienceMethods:
    """Test convenience methods on Record."""

    def test_fields_by_indicator(self):
        """Test the fields_by_indicator() convenience method."""
        record = create_test_record()
        # Find LCSH 650 subjects
        results = record.fields_by_indicator("650", indicator2="0")
        assert len(results) == 2
        for field in results:
            assert field.tag == "650"
            assert field.indicator2 == "0"

    def test_fields_by_indicator_both(self):
        """Test filtering by both indicators."""
        record = create_test_record()
        # Find personal name added entry with ind1='1' (surname first)
        results = record.fields_by_indicator("700", indicator1="1")
        assert len(results) == 1

    def test_fields_in_range(self):
        """Test the fields_in_range() convenience method."""
        record = create_test_record()
        # Find all 5XX notes
        results = record.fields_in_range("500", "599")
        assert len(results) == 2  # 500 and 504
        for field in results:
            assert "500" <= field.tag <= "599"


# =============================================================================
# Real-World Use Case Tests
# =============================================================================


class TestRealWorldUseCases:
    """Test practical library cataloging scenarios."""

    def test_find_lcsh_subjects_with_subdivisions(self):
        """
        Common cataloging task: Find all LCSH subject headings that include
        topical subdivisions ($x) for subject analysis.
        """
        record = create_test_record()
        query = (
            FieldQuery()
            .tag("650")
            .indicator2("0")  # LCSH
            .has_subfield("x")  # Has topical subdivision
        )
        results = record.fields_matching(query)
        assert len(results) == 2  # History and Science
        for field in results:
            subdivisions = field.subfields_by_code("x")
            assert len(subdivisions) > 0

    def test_find_all_subjects_for_export(self):
        """
        Export scenario: Gather all subject headings (6XX) regardless of
        thesaurus for conversion to another format.
        """
        record = create_test_record()
        results = record.fields_in_range("600", "699")
        assert len(results) == 6
        tags = {f.tag for f in results}
        assert tags == {"600", "650", "651"}

    def test_find_isbn13_for_linking(self):
        """
        Linking scenario: Find ISBN-13 for resolving works across systems.
        """
        record = create_test_record()
        query = SubfieldPatternQuery("020", "a", r"^978-")
        results = record.fields_matching_pattern(query)
        assert len(results) == 1

    def test_find_persons_with_death_dates(self):
        """
        Authority control: Find personal names with complete life dates
        for enhanced authority linking.
        """
        record = create_test_record()
        # Match birth-death pattern
        query = SubfieldPatternQuery("100", "d", r"^\d{4}-\d{4}$")
        results = record.fields_matching_pattern(query)
        assert len(results) == 1

    def test_combine_queries_for_complex_analysis(self):
        """
        Complex analysis: Chain multiple query types for detailed analysis.
        """
        record = create_test_record()

        # Step 1: Find all 6XX subject fields
        all_subjects = record.fields_in_range("600", "699")
        assert len(all_subjects) == 6

        # Step 2: Filter to LCSH only
        lcsh_query = TagRangeQuery("600", "699", indicator2="0")
        lcsh_subjects = record.fields_matching_range(lcsh_query)
        assert len(lcsh_subjects) == 4

        # Step 3: Among LCSH, find those with subdivisions
        with_subdivisions = [
            f for f in lcsh_subjects if len(f.subfields_by_code("x")) > 0
        ]
        assert len(with_subdivisions) == 3  # History, Science, USA


# =============================================================================
# Edge Cases and Error Handling
# =============================================================================


class TestEdgeCases:
    """Test edge cases and error handling."""

    def test_query_empty_record(self):
        """Query against a record with no fields."""
        leader = Leader()
        record = Record(leader)
        query = FieldQuery().tag("650")
        results = record.fields_matching(query)
        assert len(results) == 0

    def test_query_nonexistent_tag(self):
        """Query for a tag that doesn't exist."""
        record = create_test_record()
        query = FieldQuery().tag("999")
        results = record.fields_matching(query)
        assert len(results) == 0

    def test_range_no_matches(self):
        """Range query that matches nothing."""
        record = create_test_record()
        query = TagRangeQuery("800", "899")
        results = record.fields_matching_range(query)
        assert len(results) == 0

    def test_pattern_no_matches(self):
        """Pattern query that matches nothing."""
        record = create_test_record()
        query = SubfieldPatternQuery("020", "a", r"^NOTFOUND")
        results = record.fields_matching_pattern(query)
        assert len(results) == 0

    def test_value_no_matches(self):
        """Value query that matches nothing."""
        record = create_test_record()
        query = SubfieldValueQuery("650", "a", "NonexistentSubject")
        results = record.fields_matching_value(query)
        assert len(results) == 0
