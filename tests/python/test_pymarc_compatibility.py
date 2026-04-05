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
from mrrc import Record, Field, Leader, MARCReader, MARCWriter, Subfield, ControlField, Indicators
import io
import json
from pathlib import Path

# Test data directory relative to this file
TEST_DATA_DIR = Path(__file__).parent.parent / 'data'


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

    def test_control_field_dict_access(self):
         """Test accessing control fields via dict-style access (pymarc compatibility)."""
         record = Record(Leader())
         record.add_control_field('001', '12345')
         record.add_control_field('003', 'DLC')

         # Access via dict notation should return Field with is_control_field()
         field_001 = record['001']
         assert isinstance(field_001, Field)
         assert field_001.data == '12345'
         assert field_001.tag == '001'

    def test_control_field_value_property(self):
         """Test control field .data property (pymarc compatibility)."""
         record = Record(Leader())
         record.add_control_field('005', '20210315120000.0')

         # Access via dict notation and .data property
         assert record['005'].data == '20210315120000.0'

    def test_control_field_backward_compat(self):
         """Test that record.control_field() still works after adding dict access."""
         record = Record(Leader())
         record.add_control_field('001', 'test-id')

         # Both access patterns should work and return same value
         assert record['001'].data == record.control_field('001')
         assert record['001'].data == 'test-id'

    def test_missing_control_field_raises_keyerror(self):
         """Test that missing control fields raise KeyError via dict access."""
         record = Record(Leader())
         with pytest.raises(KeyError):
             record['001']
         with pytest.raises(KeyError):
             record['008']

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
        record.remove_field('245')

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

    def test_field_getitem_returns_value(self):
        """Test Field.__getitem__ returns subfield value (pymarc compatibility)."""
        field = create_field('245', '1', '0', a='Title')
        assert field['a'] == 'Title'

    def test_field_getitem_returns_none_for_missing(self):
        """Test Field.__getitem__ returns None for missing subfield (pymarc compatibility)."""
        field = create_field('245', '1', '0', a='Title')
        # Should return None, not raise KeyError
        assert field['z'] is None

    def test_field_getitem_with_multiple_same_code(self):
        """Test Field.__getitem__ returns first value when multiple subfields have same code."""
        field = Field('300', ' ', ' ')
        field.add_subfield('a', '256 pages')
        field.add_subfield('a', '24 cm')
        # Should return first 'a' value
        assert field['a'] == '256 pages'

    def test_field_indicators_tuple_access(self):
       """Test Field.indicators property returns Indicators tuple-like object (pymarc compatibility)."""
       field = Field('245', '1', '0')
       
       # Access via indicators property
       indicators = field.indicators
       assert isinstance(indicators, Indicators)
       assert indicators[0] == '1'
       assert indicators[1] == '0'

    def test_field_indicators_unpacking(self):
       """Test Field.indicators can be unpacked like a tuple (pymarc compatibility)."""
       field = Field('245', '1', '0')
       
       # Unpacking
       ind1, ind2 = field.indicators
       assert ind1 == '1'
       assert ind2 == '0'

    def test_field_indicators_backward_compat(self):
       """Test that field.indicator1/indicator2 still work alongside indicators property."""
       field = Field('245', '1', '0')
       
       # Both patterns should work
       assert field.indicator1 == field.indicators[0]
       assert field.indicator2 == field.indicators[1]

    def test_field_indicators_setter(self):
       """Test Field.indicators setter (pymarc compatibility)."""
       field = Field('245', '0', '0')
       
       # Set via Indicators object
       field.indicators = Indicators('1', '4')
       assert field.indicator1 == '1'
       assert field.indicator2 == '4'
       
       # Set via tuple
       field.indicators = ('1', '0')
       assert field.indicator1 == '1'
       assert field.indicator2 == '0'


class TestConvenienceMethods:
    """Test all pymarc-style convenience methods."""

    def test_title(self):
        """Test title() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('245', '1', '0', a='Test Title'))
        assert record.title == 'Test Title'

    def test_author(self):
        """Test author() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('100', '1', ' ', a='Author, Test'))
        assert 'Author' in record.author

    def test_isbn(self):
        """Test isbn() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('020', ' ', ' ', a='0201616165'))
        assert record.isbn == '0201616165'

    def test_issn(self):
        """Test issn() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('022', ' ', ' ', a='0028-0836'))
        assert record.issn == '0028-0836'

    def test_publisher(self):
        """Test publisher() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('260', ' ', ' ', b='Test Publisher'))
        assert 'Publisher' in record.publisher or 'Test' in record.publisher

    def test_subjects(self):
        """Test subjects() convenience method."""
        record = Record(Leader())
        for i in range(3):
            record.add_field(create_field('650', ' ', '0', a=f'Subject {i}'))

        subjects = record.subjects
        assert len(subjects) == 3

    def test_location(self):
        """Test location() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('852', ' ', ' ', a='Main Library'))

        locations = record.location
        assert 'Main Library' in locations

    def test_notes(self):
        """Test notes() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('500', ' ', ' ', a='General note'))

        notes = record.notes
        assert 'General note' in notes

    def test_series(self):
        """Test series() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('490', ' ', ' ', a='Series Name'))

        series = record.series
        assert series is not None

    def test_physical_description(self):
        """Test physical_description() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('300', ' ', ' ', a='256 pages'))

        phys_desc = record.physical_description
        assert '256' in phys_desc or phys_desc is not None

    def test_uniform_title(self):
        """Test uniform_title() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('130', ' ', '0', a='Uniform Title'))

        uniform = record.uniform_title
        assert 'Uniform' in uniform

    def test_sudoc(self):
        """Test sudoc() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('086', ' ', ' ', a='I 19.2:En 3'))

        sudoc = record.sudoc
        assert sudoc == 'I 19.2:En 3'

    def test_issn_title(self):
        """Test issn_title() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('222', ' ', ' ', a='Key Title'))

        issn_title = record.issn_title
        assert 'Key Title' in issn_title

    def test_pubyear(self):
        """Test pubyear() convenience method."""
        record = Record(Leader())
        record.add_field(create_field('260', ' ', ' ', c='2023'))

        year = record.pubyear
        assert year == '2023'


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
        json_str = record.to_json()
        assert json_str is not None

    def test_record_with_many_fields(self):
        """Test record with many fields."""
        record = Record(Leader())

        # Add many fields
        for i in range(20):
            record.add_field(create_field('650', ' ', '0', a=f'Subject {i}'))

        subjects = record.subjects
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

        marcjson = record.to_marcjson()
        assert marcjson is not None
        assert len(marcjson) > 0


class TestFieldCreation:
    """Test Field creation and basic properties (from pymarc test_field.py)."""

    def test_field_subfields_created(self):
        """Test subfields are properly created."""
        field = Field('245', '0', '1')
        field.add_subfield('a', 'Title')
        field.add_subfield('b', 'Subtitle')
        assert len(field.subfields()) == 2

    def test_field_indicators(self):
        """Test indicator access."""
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Test Title')
        assert field.indicator1 == '1'
        assert field.indicator2 == '0'

    def test_field_reassign_indicators(self):
        """Test changing indicators."""
        field = Field('245', '0', '1')
        field.indicator1 = '1'
        field.indicator2 = '0'
        assert field.indicator1 == '1'
        assert field.indicator2 == '0'

    def test_field_subfield_get_multiple(self):
        """Test getting multiple subfields by code."""
        field = Field('650', ' ', '0')
        field.add_subfield('a', 'First Subject')
        field.add_subfield('v', 'Subdivision')
        result = field.subfields_by_code('a')
        assert 'First Subject' in result

    def test_field_add_subfield(self):
        """Test adding subfields."""
        field = Field('245', '0', '1')
        field.add_subfield('a', 'foo')
        field.add_subfield('b', 'bar')
        subfields = field.subfields()
        assert len(subfields) == 2
        assert subfields[0].value == 'foo'

    def test_field_is_subject_field(self):
        """Test identifying subject fields."""
        subject_field = Field('650', ' ', '0')
        subject_field.add_subfield('a', 'Python')
        title_field = Field('245', '1', '0')
        title_field.add_subfield('a', 'Title')
        assert subject_field.is_subject_field()
        assert not title_field.is_subject_field()


class TestRecordAdvanced:
    """Advanced record tests (from pymarc test_record.py)."""

    def test_record_add_field(self):
        """Test adding fields to records."""
        record = Record(Leader())
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Python')
        field.add_subfield('c', 'Guido')
        record.add_field(field)
        assert field in record.fields()

    def test_record_quick_access(self):
        """Test quick access via tags."""
        record = Record(Leader())
        title = Field('245', '1', '0')
        title.add_subfield('a', 'Python')
        record.add_field(title)
        assert record['245'] == title

    def test_record_getitem_missing_tag(self):
        """Test Record.__getitem__ raises KeyError for missing tag (pymarc compatibility)."""
        record = Record(Leader())
        with pytest.raises(KeyError):
            record['999']
        with pytest.raises(KeyError):
            record['245']

    def test_record_membership(self):
        """Test checking if tag exists in record."""
        record = Record(Leader())
        title = Field('245', '1', '0')
        title.add_subfield('a', 'Python')
        record.add_field(title)
        assert '245' in record
        assert '999' not in record

    def test_record_get_fields_multi(self):
        """Test retrieving multiple field types."""
        record = Record(Leader())
        subject1 = Field('650', ' ', '0')
        subject1.add_subfield('a', 'Programming')
        subject2 = Field('651', ' ', '0')
        subject2.add_subfield('a', 'Computer Science')
        record.add_field(subject1)
        record.add_field(subject2)
        fields = record.get_fields('650', '651')
        assert len(fields) == 2

    def test_record_remove_field(self):
        """Test removing fields."""
        record = Record(Leader())
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Title')
        record.add_field(field)
        assert '245' in record
        record.remove_field(field)
        assert record.get_field('245') is None


class TestReaderWriter:
    """Test MARC reading and writing (from pymarc test_reader.py, test_writer.py)."""

    def test_reader_from_file(self):
        """Test reading MARC records from file using direct path passing."""
        test_file = TEST_DATA_DIR / 'simple_book.mrc'
        # Pass path directly to MARCReader (recommended - allows Rust to handle I/O)
        reader = MARCReader(test_file)
        record = next(reader)
        assert record is not None
        assert len(record.fields()) > 0

    def test_reader_iteration(self):
        """Test iterating through records using direct path passing."""
        test_file = TEST_DATA_DIR / 'simple_book.mrc'
        # Pass path directly to MARCReader (recommended - allows Rust to handle I/O)
        reader = MARCReader(test_file)
        count = 0
        for record in reader:
            count += 1
            assert record is not None
        assert count > 0

    def test_writer_to_bytes(self):
        """Test writing records to bytes."""
        record = Record(Leader())
        record.add_control_field('001', 'test-id')
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Test Title')
        record.add_field(field)

        output = io.BytesIO()
        writer = MARCWriter(output)
        writer.write(record)
        written_bytes = output.getvalue()
        assert len(written_bytes) > 0

    def test_roundtrip_record(self):
        """Test writing then reading a record."""
        original = Record(Leader())
        original.add_control_field('001', 'test-123')
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Test Title')
        original.add_field(field)

        # Write to bytes
        output = io.BytesIO()
        writer = MARCWriter(output)
        writer.write(original)

        # Read back
        output.seek(0)
        reader = MARCReader(output)
        read_record = next(reader)

        # Verify content
        assert read_record is not None
        assert read_record.control_field('001') == 'test-123'


class TestLeader:
    """Test Leader manipulation (from pymarc test_leader.py)."""

    def test_leader_defaults(self):
        """Test default leader values."""
        leader = Leader()
        assert leader is not None

    def test_leader_record_type(self):
        """Test setting record type."""
        leader = Leader()
        leader.record_type = 'a'
        assert leader.record_type == 'a'

    def test_leader_bibliographic_level(self):
        """Test setting bibliographic level."""
        leader = Leader()
        leader.bibliographic_level = 'c'
        assert leader.bibliographic_level == 'c'

    def test_leader_encoding_level(self):
        """Test setting encoding level."""
        leader = Leader()
        leader.encoding_level = '4'
        assert leader.encoding_level == '4'

    def test_leader_descriptor_cataloging_form(self):
        """Test descriptor cataloging form."""
        leader = Leader()
        leader.descriptive_cataloging_form = 'c'
        assert leader.descriptive_cataloging_form == 'c'

    def test_leader_multipart_resource_record_level(self):
        """Test multipart resource level."""
        leader = Leader()
        leader.multipart_resource_record_level = 'a'
        assert leader.multipart_resource_record_level == 'a'

    def test_leader_single_position_access(self):
        """Test Leader position-based access (pymarc compatibility)."""
        leader = Leader()
        leader.record_status = 'c'
        # Access by position 5
        assert leader[5] == 'c'

    def test_leader_slice_access(self):
        """Test Leader slice-based access (pymarc compatibility)."""
        leader = Leader()
        # leader[0:5] should return the record length as a string
        record_len_str = leader[0:5]
        assert isinstance(record_len_str, str)
        assert len(record_len_str) == 5

    def test_leader_setitem_by_position(self):
        """Test Leader position-based setting (pymarc compatibility)."""
        leader = Leader()
        leader[5] = 'a'  # Set record status at position 5
        assert leader[5] == 'a'
        assert leader.record_status == 'a'

    def test_leader_position_and_property_stay_in_sync(self):
        """Test that position-based and property-based access stay synchronized."""
        leader = Leader()
        
        # Set via property
        leader.record_status = 'd'
        assert leader[5] == 'd'
        
        # Set via position
        leader[6] = 'a'
        assert leader.record_type == 'a'

    def test_leader_get_valid_values(self):
        """Test getting valid values for leader positions."""
        # Position 5: Record status
        values = Leader.get_valid_values(5)
        assert values is not None
        assert 'a' in values
        assert 'c' in values
        assert 'd' in values
        assert 'n' in values
        assert 'p' in values
        
        # Position 6: Type of record
        values = Leader.get_valid_values(6)
        assert values is not None
        assert 'a' in values
        assert 'm' in values
        
        # Position 7: Bibliographic level
        values = Leader.get_valid_values(7)
        assert values is not None
        assert 'm' in values
        assert 's' in values
        
        # Position 17: Encoding level
        values = Leader.get_valid_values(17)
        assert values is not None
        assert ' ' in values
        assert '1' in values
        
        # Position 18: Cataloging form
        values = Leader.get_valid_values(18)
        assert values is not None
        assert 'a' in values
        
        # Position 0: No defined values
        values = Leader.get_valid_values(0)
        assert values is None

    def test_leader_is_valid_value(self):
        """Test validating values for leader positions."""
        # Position 5: Record status
        assert Leader.is_valid_value(5, 'a') is True
        assert Leader.is_valid_value(5, 'c') is True
        assert Leader.is_valid_value(5, 'x') is False
        
        # Position 6: Type of record
        assert Leader.is_valid_value(6, 'a') is True
        assert Leader.is_valid_value(6, 'm') is True
        assert Leader.is_valid_value(6, 'z') is False
        
        # Position 0: No validation (any value accepted)
        assert Leader.is_valid_value(0, '0') is True
        assert Leader.is_valid_value(0, 'x') is True

    def test_leader_get_value_description(self):
        """Test getting descriptions of leader values."""
        # Position 5: Record status
        desc = Leader.get_value_description(5, 'a')
        assert desc is not None
        assert 'Increase in encoding level' in desc
        
        desc = Leader.get_value_description(5, 'c')
        assert desc is not None
        assert 'Corrected or revised' in desc
        
        # Invalid value
        desc = Leader.get_value_description(5, 'x')
        assert desc is None
        
        # Position 6: Type of record
        desc = Leader.get_value_description(6, 'a')
        assert desc is not None
        assert 'Language material' in desc
        
        # Position 0: No descriptions
        desc = Leader.get_value_description(0, '5')
        assert desc is None


class TestEncoding:
    """Test encoding handling (from pymarc test_utf8.py, test_marc8.py)."""

    def test_utf8_record_creation(self):
        """Test creating records with UTF-8 data."""
        record = Record(Leader())
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Rū Harison no wārudo')  # UTF-8 string
        record.add_field(field)
        assert record.get_field('245') is not None

    def test_special_characters(self):
        """Test handling special characters."""
        record = Record(Leader())
        field = Field('650', ' ', '0')
        field.add_subfield('a', 'Müller')  # Umlaut
        field.add_subfield('a', 'Café')    # Accent
        record.add_field(field)
        assert record.get_field('650') is not None

    def test_encoding_to_marc(self):
        """Test encoding record to MARC."""
        record = Record(Leader())
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Test')
        record.add_field(field)
        encoded = record.to_marc21()
        assert encoded is not None


class TestSerialization:
    """Test serialization formats (from pymarc test_json.py, test_xml.py)."""

    def test_json_serialization(self):
        """Test JSON serialization."""
        record = Record(Leader())
        record.add_control_field('001', 'test-id')
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Title')
        record.add_field(field)

        json_str = record.to_json()
        assert json_str is not None
        parsed = json.loads(json_str)
        assert parsed is not None

    def test_xml_serialization(self):
        """Test XML serialization."""
        record = Record(Leader())
        record.add_control_field('001', 'test-id')
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Title')
        record.add_field(field)

        xml_str = record.to_xml()
        assert xml_str is not None
        assert '<' in xml_str

    def test_dublin_core_serialization(self):
        """Test Dublin Core serialization."""
        record = Record(Leader())
        record.add_control_field('001', 'test-id')
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Title')
        record.add_field(field)

        dc_xml = record.to_dublin_core()
        assert dc_xml is not None


class TestConstructorKwargs:
    """Test Field and Record constructor keyword arguments for pymarc parity."""

    def test_field_with_indicators_kwarg(self):
        """Test Field with indicators= kwarg."""
        field = Field('245', indicators=['1', '0'])
        assert field.indicator1 == '1'
        assert field.indicator2 == '0'

    def test_field_with_subfields_kwarg(self):
        """Test Field with subfields= kwarg."""
        field = Field('245', '1', '0', subfields=[Subfield('a', 'Test Title')])
        assert field['a'] == 'Test Title'
        assert len(field.subfields()) == 1

    def test_field_with_indicators_and_subfields(self):
        """Test Field with both indicators= and subfields= kwargs."""
        field = Field('245', indicators=['1', '0'], subfields=[
            Subfield('a', 'Pragmatic Programmer'),
            Subfield('c', 'Hunt and Thomas'),
        ])
        assert field.indicator1 == '1'
        assert field.indicator2 == '0'
        assert field['a'] == 'Pragmatic Programmer'
        assert field['c'] == 'Hunt and Thomas'
        assert len(field.subfields()) == 2

    def test_record_with_fields_kwarg(self):
        """Test Record with fields= kwarg."""
        title = Field('245', '1', '0', subfields=[Subfield('a', 'My Book')])
        author = Field('100', '1', ' ', subfields=[Subfield('a', 'Doe, John')])
        record = Record(fields=[title, author])
        assert record.title == 'My Book'
        assert record.get_field('100') is not None

    def test_full_inline_construction(self):
        """Test the exact pattern from the issue: full inline construction."""
        record = Record(fields=[
            Field('245', indicators=['0', '1'], subfields=[
                Subfield('a', 'Pragmatic Programmer'),
            ]),
            Field('100', '1', ' ', subfields=[
                Subfield('a', 'Hunt, Andrew'),
            ]),
            Field('650', ' ', '0', subfields=[
                Subfield('a', 'Computer programming'),
            ]),
        ])
        assert record.title == 'Pragmatic Programmer'
        assert len(record.get_fields('650')) == 1

    def test_field_backward_compat_positional_indicators(self):
        """Test backward compatibility: Field('245', '0', '1') still works."""
        field = Field('245', '0', '1')
        assert field.indicator1 == '0'
        assert field.indicator2 == '1'

    def test_record_backward_compat_no_args(self):
        """Test backward compatibility: Record() with no args still works."""
        record = Record()
        assert record is not None
        assert len(record.fields()) == 0

    def test_record_with_leader_and_fields(self):
        """Test Record with both leader and fields kwargs."""
        leader = Leader()
        leader.record_type = 'a'
        leader.bibliographic_level = 'm'
        record = Record(leader, fields=[
            Field('245', '1', '0', subfields=[Subfield('a', 'Title')]),
        ])
        assert record.leader().record_type == 'a'
        assert record.title == 'Title'


class TestFieldUnification:
    """Test unified Field class for both control and data fields (pymarc compatibility)."""

    def test_field_with_data_creates_control_field(self):
        """Field('001', data='12345') creates a control field."""
        field = Field('001', data='12345')
        assert field.is_control_field()
        assert field.tag == '001'
        assert field.data == '12345'

    def test_data_field_is_not_control(self):
        """Field('245', '1', '0') is not a control field."""
        field = Field('245', '1', '0')
        assert not field.is_control_field()
        assert field.data is None

    def test_control_field_isinstance(self):
        """ControlField is a subclass of Field."""
        cf = ControlField('001', '12345')
        assert isinstance(cf, Field)
        assert cf.is_control_field()

    def test_control_field_backward_compat_class(self):
        """ControlField class still works as backward-compatible alias."""
        cf = ControlField('003', 'DLC')
        assert cf.tag == '003'
        assert cf.data == 'DLC'
        assert cf.is_control_field()

    def test_record_getitem_returns_field_for_control(self):
        """Record['001'] returns a Field instance (not ControlField)."""
        record = Record(Leader())
        record.add_control_field('001', 'test-id')
        field = record['001']
        assert isinstance(field, Field)
        assert field.data == 'test-id'

    def test_record_getitem_raises_keyerror(self):
        """Record['999'] raises KeyError."""
        record = Record(Leader())
        with pytest.raises(KeyError):
            record['999']

    def test_record_get_returns_none_for_missing(self):
        """record.get('999') returns None."""
        record = Record(Leader())
        assert record.get('999') is None
        assert record.get('001') is None

    def test_record_get_returns_field_for_existing(self):
        """record.get('001') returns a Field for existing control fields."""
        record = Record(Leader())
        record.add_control_field('001', 'test-id')
        field = record.get('001')
        assert isinstance(field, Field)
        assert field.data == 'test-id'

    def test_get_fields_includes_control_fields(self):
        """get_fields() returns Field instances for both control and data fields."""
        record = Record(Leader())
        record.add_control_field('001', 'test-id')
        record.add_control_field('003', 'DLC')
        record.add_field(Field('245', '1', '0', subfields=[Subfield('a', 'Title')]))

        all_fields = record.get_fields()
        tags = [f.tag for f in all_fields]
        assert '001' in tags
        assert '003' in tags
        assert '245' in tags
        for f in all_fields:
            assert isinstance(f, Field)

    def test_get_fields_by_control_tag(self):
        """get_fields('001') returns a list with the control field."""
        record = Record(Leader())
        record.add_control_field('001', 'test-id')
        fields = record.get_fields('001')
        assert len(fields) == 1
        assert fields[0].data == 'test-id'

    def test_add_field_with_control_field(self):
        """add_field() accepts a Field created with data=."""
        record = Record(Leader())
        record.add_field(Field('001', data='12345'))
        assert record.control_field('001') == '12345'

    def test_fields_method_includes_control_fields(self):
        """record.fields() includes control fields as Field instances."""
        record = Record(Leader())
        record.add_control_field('001', 'test-id')
        record.add_field(Field('245', '1', '0', subfields=[Subfield('a', 'Title')]))

        all_fields = record.fields()
        tags = [f.tag for f in all_fields]
        assert '001' in tags
        assert '245' in tags

    def test_default_indicators_are_spaces(self):
        """Default indicators should be spaces (matching pymarc)."""
        field = Field('245')
        assert field.indicator1 == ' '
        assert field.indicator2 == ' '


class TestFieldStringRepresentation:
    """Test Field.__str__ and __repr__ match pymarc format."""

    def test_data_field_str(self):
        """str(field) returns pymarc MARC display format for data fields."""
        field = Field('245', '1', '0', subfields=[
            Subfield('a', 'The Great Gatsby'),
            Subfield('c', 'F. Scott Fitzgerald'),
        ])
        assert str(field) == '=245  10$aThe Great Gatsby$cF. Scott Fitzgerald'

    def test_control_field_str(self):
        """str(field) returns pymarc format for control fields."""
        field = Field('001', data='12345')
        assert str(field) == '=001  12345'

    def test_data_field_str_blank_indicators(self):
        """Blank indicators display as backslash."""
        field = Field('650', ' ', '0', subfields=[Subfield('a', 'Python')])
        assert str(field) == '=650  \\0$aPython'

    def test_field_repr(self):
        """repr(field) should be informative."""
        field = Field('245', '1', '0', subfields=[Subfield('a', 'Title')])
        r = repr(field)
        assert '245' in r

    def test_control_field_repr(self):
        """repr for control field."""
        field = Field('001', data='12345')
        r = repr(field)
        assert '001' in r


class TestRecordPropertyAccessors:
    """Verify Record convenience accessors are properties (not methods)."""

    def _make_record(self):
        """Create a record with various fields for testing."""
        record = Record()
        record.add_field(create_field('245', '1', '0', a='Test Title'))
        record.add_field(create_field('100', '1', ' ', a='Smith, John'))
        record.add_field(create_field('020', ' ', ' ', a='0201616165'))
        record.add_field(create_field('022', ' ', ' ', a='0028-0836'))
        record.add_field(create_field('260', ' ', ' ', a='Place :', b='Publisher,', c='2023'))
        record.add_field(create_field('650', ' ', '0', a='Testing.'))
        record.add_field(create_field('852', ' ', ' ', a='Library'))
        record.add_field(create_field('500', ' ', ' ', a='A note.'))
        record.add_field(create_field('130', ' ', ' ', a='Uniform'))
        record.add_field(create_field('086', ' ', ' ', a='Y 1.1/2:'))
        record.add_field(create_field('222', ' ', ' ', a='ISSN Title'))
        record.add_field(create_field('024', '8', ' ', a='1234-5678'))
        record.add_field(create_field('490', '1', ' ', a='Series Name'))
        record.add_field(create_field('300', ' ', ' ', a='100 p.'))
        record.add_field(create_field('700', '1', ' ', a='Jones, Mary'))
        return record

    def test_title_is_property(self):
        """record.title returns a value, not a bound method."""
        record = self._make_record()
        assert record.title == 'Test Title'
        assert not callable(record.title) or isinstance(record.title, str)

    def test_author_is_property(self):
        record = self._make_record()
        assert 'Smith' in record.author

    def test_isbn_is_property(self):
        record = self._make_record()
        assert record.isbn == '0201616165'

    def test_issn_is_property(self):
        record = self._make_record()
        assert record.issn == '0028-0836'

    def test_subjects_is_property(self):
        record = self._make_record()
        assert isinstance(record.subjects, list)
        assert 'Testing.' in record.subjects

    def test_publisher_is_property(self):
        record = self._make_record()
        assert record.publisher is not None

    def test_location_is_property(self):
        record = self._make_record()
        assert isinstance(record.location, list)

    def test_notes_is_property(self):
        record = self._make_record()
        assert isinstance(record.notes, list)

    def test_uniform_title_is_property(self):
        record = self._make_record()
        assert record.uniform_title is not None

    def test_sudoc_is_property(self):
        record = self._make_record()
        assert record.sudoc is not None

    def test_issn_title_is_property(self):
        record = self._make_record()
        assert record.issn_title is not None

    def test_issnl_is_property(self):
        record = self._make_record()
        # issnl may or may not match based on how 024 is parsed
        result = record.issnl
        assert result is None or isinstance(result, str)

    def test_pubyear_returns_str(self):
        """pubyear must return str, not int, matching pymarc."""
        record = self._make_record()
        year = record.pubyear
        assert year is not None
        assert isinstance(year, str)
        assert year == '2023'

    def test_pubyear_none_returns_none(self):
        """pubyear returns None when no year field exists."""
        record = Record()
        assert record.pubyear is None

    def test_series_is_property(self):
        record = self._make_record()
        assert record.series is not None

    def test_physical_description_is_property(self):
        record = self._make_record()
        assert record.physical_description is not None

    def test_physicaldescription_alias(self):
        """physicaldescription is an alias for physical_description."""
        record = self._make_record()
        assert record.physicaldescription == record.physical_description

    def test_uniformtitle_alias(self):
        """uniformtitle is an alias for uniform_title."""
        record = self._make_record()
        assert record.uniformtitle == record.uniform_title

    def test_addedentries(self):
        """addedentries returns 700/710/711/730 fields."""
        record = self._make_record()
        entries = record.addedentries
        assert isinstance(entries, list)
        assert len(entries) >= 1
        assert any('Jones' in str(e) for e in entries)


class TestBulkFieldOperations:
    """Test bulk add/remove field operations (pymarc compatibility)."""

    def test_add_multiple_fields(self):
        """add_field(*fields) accepts multiple fields at once."""
        record = Record(Leader())
        f1 = Field('100', '1', ' ', subfields=[Subfield('a', 'Author')])
        f2 = Field('245', '1', '0', subfields=[Subfield('a', 'Title')])
        f3 = Field('650', ' ', '0', subfields=[Subfield('a', 'Subject')])
        record.add_field(f1, f2, f3)
        assert record.get_field('100') is not None
        assert record.get_field('245') is not None
        assert record.get_field('650') is not None

    def test_add_field_single_still_works(self):
        """add_field still works with a single argument (backward compat)."""
        record = Record(Leader())
        f = Field('245', '1', '0', subfields=[Subfield('a', 'Title')])
        record.add_field(f)
        assert record.get_field('245') is not None

    def test_remove_field_by_object(self):
        """remove_field accepts a Field object."""
        record = Record(Leader())
        f = Field('245', '1', '0', subfields=[Subfield('a', 'Title')])
        record.add_field(f)
        record.remove_field(f)
        assert record.get_field('245') is None

    def test_remove_field_multiple(self):
        """remove_field accepts multiple Field objects."""
        record = Record(Leader())
        f1 = Field('100', '1', ' ', subfields=[Subfield('a', 'Author')])
        f2 = Field('650', ' ', '0', subfields=[Subfield('a', 'Subject')])
        record.add_field(f1, f2)
        record.remove_field(f1, f2)
        assert record.get_field('100') is None
        assert record.get_field('650') is None

    def test_remove_field_returns_none(self):
        """remove_field returns None (not a list)."""
        record = Record(Leader())
        f = Field('245', '1', '0', subfields=[Subfield('a', 'Title')])
        record.add_field(f)
        result = record.remove_field(f)
        assert result is None

    def test_remove_fields_by_tags(self):
        """remove_fields(*tags) removes all fields with given tags."""
        record = Record(Leader())
        record.add_field(Field('650', ' ', '0', subfields=[Subfield('a', 'Subject')]))
        record.add_field(Field('700', '1', ' ', subfields=[Subfield('a', 'Author')]))
        record.remove_fields('650', '700')
        assert record.get_field('650') is None
        assert record.get_field('700') is None

    def test_remove_fields_returns_none(self):
        """remove_fields returns None."""
        record = Record(Leader())
        record.add_field(Field('650', ' ', '0', subfields=[Subfield('a', 'Subject')]))
        result = record.remove_fields('650')
        assert result is None


class TestOrderedFieldInsertion:
    """Test ordered and grouped field insertion (pymarc compatibility)."""

    def test_add_ordered_field(self):
        """add_ordered_field inserts field in tag-sorted position."""
        record = Record(fields=[
            Field('100', '1', ' ', subfields=[Subfield('a', 'Author')]),
            Field('650', ' ', '0', subfields=[Subfield('a', 'Subject')]),
        ])
        f245 = Field('245', '1', '0', subfields=[Subfield('a', 'Title')])
        record.add_ordered_field(f245)
        tags = [f.tag for f in record.get_fields()]
        # Filter to just data field tags (not control fields)
        data_tags = [t for t in tags if t >= '010']
        assert data_tags == ['100', '245', '650']

    def test_add_ordered_field_at_end(self):
        record = Record(fields=[
            Field('100', '1', ' ', subfields=[Subfield('a', 'Author')]),
        ])
        f650 = Field('650', ' ', '0', subfields=[Subfield('a', 'Subject')])
        record.add_ordered_field(f650)
        data_tags = [f.tag for f in record.get_fields() if f.tag >= '010']
        assert data_tags == ['100', '650']

    def test_add_grouped_field(self):
        record = Record(fields=[
            Field('650', ' ', '0', subfields=[Subfield('a', 'Subject 1')]),
            Field('650', ' ', '0', subfields=[Subfield('a', 'Subject 2')]),
            Field('700', '1', ' ', subfields=[Subfield('a', 'Author')]),
        ])
        f650 = Field('650', ' ', '0', subfields=[Subfield('a', 'Subject 3')])
        record.add_grouped_field(f650)
        data_tags = [f.tag for f in record.get_fields() if f.tag >= '010']
        assert data_tags == ['650', '650', '650', '700']

    def test_add_grouped_field_no_existing(self):
        record = Record(fields=[
            Field('100', '1', ' ', subfields=[Subfield('a', 'Author')]),
            Field('650', ' ', '0', subfields=[Subfield('a', 'Subject')]),
        ])
        f245 = Field('245', '1', '0', subfields=[Subfield('a', 'Title')])
        record.add_grouped_field(f245)
        data_tags = [f.tag for f in record.get_fields() if f.tag >= '010']
        assert data_tags == ['100', '245', '650']


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
