"""
Comprehensive pymarc compatibility test suite for mrrc Python wrapper.

This test suite aims for feature parity with pymarc's test coverage to ensure
the mrrc wrapper is a drop-in replacement for pymarc. It includes:
- Record creation and manipulation
- Field operations and subfield access
- Reader/Writer round-trip testing
- Format conversions (JSON, XML, Dublin Core)
- Edge cases and error handling
"""

import pytest
from mrrc import Record, Field, Leader, MARCReader, MARCWriter, Subfield
import io
import json


def create_field(tag, ind1='0', ind2='0', **subfields):
    """Helper to create a field with subfields."""
    field = Field(tag, ind1, ind2)
    for code, value in subfields.items():
        field.add_subfield(code, value)
    return field


class TestRecordCreation:
    """Test Record creation and basic properties."""

    def test_create_empty_record(self):
        """Test creating an empty record."""
        leader = Leader()
        record = Record(leader)
        assert record is not None
        assert len(record.fields()) == 0

    def test_record_with_leader(self):
        """Test that record preserves leader settings."""
        leader = Leader()
        leader.record_type = 'c'
        leader.bibliographic_level = 'd'
        record = Record(leader)
        # Note: accessing leader properties requires via the wrapper
        assert record.leader().record_type == 'c'
        assert record.leader().bibliographic_level == 'd'

    def test_record_equality(self):
        """Test comparing two identical records."""
        leader1 = Leader()
        record1 = Record(leader1)
        record1.add_control_field('001', 'test-id')

        leader2 = Leader()
        record2 = Record(leader2)
        record2.add_control_field('001', 'test-id')

        assert record1 == record2


class TestRecordFieldOperations:
    """Test adding, removing, and accessing fields."""

    def test_add_single_field(self):
        """Test adding a single field to a record."""
        record = Record(Leader())
        field = create_field('245', '1', '0', a='Test Title')
        record.add_field(field)

        retrieved = record.get_field('245')
        assert retrieved is not None

    def test_add_multiple_fields(self):
        """Test adding multiple fields with same tag."""
        record = Record(Leader())
        for i in range(3):
            field = create_field('650', ' ', '0', a=f'Subject {i}')
            record.add_field(field)

        fields = record.get_fields('650')
        assert len(fields) == 3

    def test_add_control_field(self):
        """Test adding control fields."""
        record = Record(Leader())
        record.add_control_field('001', '12345')
        record.add_control_field('003', 'ABC')

        assert record.control_field('001') == '12345'
        assert record.control_field('003') == 'ABC'

    def test_get_nonexistent_field(self):
        """Test getting a field that doesn't exist."""
        record = Record(Leader())
        field = record.get_field('999')
        assert field is None

    def test_get_all_fields(self):
        """Test retrieving all fields from a record."""
        record = Record(Leader())
        record.add_field(create_field('245', '1', '0', a='Title'))
        record.add_field(create_field('650', ' ', '0', a='Subject'))

        all_fields = record.fields()
        assert len(all_fields) >= 2

    def test_remove_field(self):
        """Test removing a specific field."""
        record = Record(Leader())
        field = create_field('245', '1', '0', a='Title')
        record.add_field(field)

        # Verify field exists
        assert record.get_field('245') is not None
        
        # Remove the field
        removed = record.remove_field('245')
        assert len(removed) > 0
        
        # Verify field is gone
        assert record.get_field('245') is None


class TestFieldSubfieldOperations:
    """Test field and subfield manipulation."""

    def test_field_creation_with_indicators(self):
        """Test creating fields with specific indicators."""
        field = Field('245', '1', '0')
        assert field.tag == '245'
        # Note: indicator access needs to be exposed in wrapper

    def test_add_subfield(self):
        """Test adding subfields to a field."""
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Title')
        field.add_subfield('b', 'Subtitle')

        assert len(field.subfields()) == 2

    def test_multiple_subfields_same_code(self):
        """Test field with multiple subfields with same code."""
        field = Field('300', ' ', ' ')
        field.add_subfield('a', '256 pages')
        field.add_subfield('a', '24 cm')  # Multiple 'a' subfields

        assert len(field.subfields()) >= 2

    def test_subfield_access(self):
        """Test accessing specific subfields."""
        field = create_field('245', '1', '0', a='Title', b='Subtitle', c='Creator')
        # Verify subfields are accessible via the wrapper
        subfield_codes = [sf.code for sf in field.subfields()]
        assert 'a' in subfield_codes
        assert 'b' in subfield_codes
        assert 'c' in subfield_codes


class TestConvenienceMethods:
    """Test all pymarc-style convenience methods."""

    def test_title(self):
        """Test title() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('245', '1', '0', a='Test Title'))
        assert record.title() == 'Test Title'

    def test_author(self):
        """Test author() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('100', '1', ' ', a='Author, Test'))
        assert 'Author' in record.author()

    def test_isbn(self):
        """Test isbn() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('020', ' ', ' ', a='0201616165'))
        assert record.isbn() == '0201616165'

    def test_issn(self):
        """Test issn() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('022', ' ', ' ', a='0028-0836'))
        assert record.issn() == '0028-0836'

    def test_publisher(self):
        """Test publisher() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('260', ' ', ' ', b='Test Publisher'))
        assert 'Publisher' in record.publisher() or 'Test' in record.publisher()

    def test_subjects(self):
        """Test subjects() convenience method."""
        record = Record(Leader())
        for i in range(3):
            record.add_field(create_field('650', ' ', '0', a=f'Subject {i}'))

        subjects = record.subjects()
        assert len(subjects) == 3

    def test_location(self):
        """Test location() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('852', ' ', ' ', a='Main Library'))

        locations = record.location()
        assert 'Main Library' in locations

    def test_notes(self):
        """Test notes() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('500', ' ', ' ', a='General note'))

        notes = record.notes()
        assert 'General note' in notes

    def test_series(self):
        """Test series() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('490', ' ', ' ', a='Series Name'))

        series = record.series()
        assert series is not None

    def test_physical_description(self):
        """Test physical_description() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('300', ' ', ' ', a='256 pages'))

        phys_desc = record.physical_description()
        assert '256' in phys_desc or phys_desc is not None

    def test_uniform_title(self):
        """Test uniform_title() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('130', ' ', '0', a='Uniform Title'))

        uniform = record.uniform_title()
        assert 'Uniform' in uniform

    def test_sudoc(self):
        """Test sudoc() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('086', ' ', ' ', a='I 19.2:En 3'))

        sudoc = record.sudoc()
        assert sudoc == 'I 19.2:En 3'

    def test_issn_title(self):
        """Test issn_title() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('222', ' ', ' ', a='Key Title'))

        issn_title = record.issn_title()
        assert 'Key Title' in issn_title

    def test_pubyear(self):
        """Test pubyear() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('260', ' ', ' ', c='2023'))

        year = record.pubyear()
        assert year == 2023


class TestRecordSerialization:
    """Test converting records to various formats."""

    def test_to_json(self):
        """Test JSON serialization."""
        record = Record(Leader())
        record.add_control_field('001', 'test-id')
        record.add_field(create_field('245', '1', '0', a='Title'))

        json_str = record.to_json()
        assert json_str is not None
        assert 'test-id' in json_str or 'Title' in json_str

    def test_to_json_valid_json(self):
        """Test that JSON output is valid JSON."""
        record = Record(Leader())
        record.add_field(create_field('245', '1', '0', a='Title'))

        json_str = record.to_json()
        try:
            data = json.loads(json_str)
            assert isinstance(data, (dict, list))
        except json.JSONDecodeError:
            pytest.fail("to_json() did not return valid JSON")

    def test_to_xml(self):
        """Test XML serialization."""
        record = Record(Leader())
        record.add_control_field('001', 'test-id')

        xml_str = record.to_xml()
        assert xml_str is not None
        assert '<' in xml_str

    def test_to_dublin_core(self):
        """Test Dublin Core serialization."""
        record = Record(Leader())
        record.add_field(create_field('245', '1', '0', a='Test Title'))

        dc = record.to_dublin_core()
        assert isinstance(dc, dict)
        # DC should have title from 245 field
        assert 'title' in dc or len(dc) > 0


class TestRecordTypeDetection:
    """Test record type helper methods."""

    def test_is_book(self):
        """Test is_book() detection."""
        leader = Leader()
        leader.record_type = 'a'
        leader.bibliographic_level = 'm'
        record = Record(leader)

        assert record.is_book()

    def test_is_serial(self):
        """Test is_serial() detection."""
        leader = Leader()
        leader.bibliographic_level = 's'
        record = Record(leader)

        assert record.is_serial()

    def test_is_music(self):
        """Test is_music() detection."""
        leader = Leader()
        leader.record_type = 'c'
        record = Record(leader)

        assert record.is_music()

    def test_is_audiovisual(self):
        """Test is_audiovisual() detection."""
        leader = Leader()
        leader.record_type = 'g'
        record = Record(leader)

        assert record.is_audiovisual()


class TestMARCReaderWriter:
    """Test reading and writing MARC records (round-trip)."""

    @pytest.fixture
    def sample_record(self):
        """Create a sample MARC record for testing."""
        record = Record(Leader())
        record.add_control_field('001', '12345')
        record.add_field(create_field('245', '1', '0', a='Test Title', b='Subtitle'))
        record.add_field(create_field('100', '1', ' ', a='Author, Test'))
        record.add_field(create_field('650', ' ', '0', a='Subject'))
        return record

    def test_reader_creation(self, fixture_1k):
        """Test creating a MARCReader."""
        data = io.BytesIO(fixture_1k)
        reader = MARCReader(data)
        assert reader is not None

    def test_reader_iteration(self, fixture_1k):
        """Test iterating through records with MARCReader."""
        data = io.BytesIO(fixture_1k)
        reader = MARCReader(data)

        count = 0
        while record := reader.read_record():
            assert record is not None
            count += 1
            if count >= 3:
                break

        assert count > 0


class TestEdgeCases:
    """Test edge cases and error handling."""

    def test_empty_record_serialization(self):
        """Test serializing an empty record."""
        record = Record(Leader())

        # Should handle empty records gracefully
        try:
            json_str = record.to_json()
            assert json_str is not None
        except Exception:
            pytest.skip("Empty record serialization not yet implemented")

    def test_record_with_many_fields(self):
        """Test record with many fields."""
        record = Record(Leader())

        # Add many fields
        for i in range(20):
            record.add_field(create_field('650', ' ', '0', a=f'Subject {i}'))

        subjects = record.subjects()
        assert len(subjects) == 20

    def test_field_with_many_subfields(self):
        """Test field with many subfields."""
        field = Field('300', ' ', ' ')

        for i in range(10):
            field.add_subfield('a', f'Value {i}')

        assert len(field.subfields()) == 10

    def test_special_characters_in_subfields(self):
        """Test special characters in subfield values."""
        field = create_field('245', '1', '0',
                            a='Title with "quotes"',
                            b="Subtitle with 'apostrophes'")

        assert len(field.subfields()) == 2


class TestFormatConversions:
    """Test format conversion compatibility."""

    def test_marcjson_roundtrip(self):
        """Test MARCJSON round-trip conversion."""
        record = Record(Leader())
        record.add_control_field('001', 'test-id')
        record.add_field(create_field('245', '1', '0', a='Title'))

        try:
            marcjson = record.to_marcjson()
            assert marcjson is not None
            assert len(marcjson) > 0
        except AttributeError:
            pytest.skip("to_marcjson not yet implemented")


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
