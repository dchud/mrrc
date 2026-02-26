"""
Advanced unit tests for mrrc Python wrapper.
Tests for edge cases, format conversions, and comprehensive API coverage.
"""

import pytest
from mrrc import MARCReader, MARCWriter, Record, Field, Leader
import io


def create_field(tag, ind1='0', ind2='0', **subfields):
    """Helper to create a field with subfields."""
    field = Field(tag, ind1, ind2)
    for code, value in subfields.items():
        field.add_subfield(code, value)
    return field


class TestRecordEdgeCases:
    """Test edge cases for records."""

    def test_empty_record_serialization(self):
        """Test serializing an empty record."""
        record = Record()
        
        # Should serialize without error
        json_str = record.to_json()
        assert json_str is not None
        
        xml_str = record.to_xml()
        assert xml_str is not None
        
        marc_bytes = record.to_marc21()
        assert isinstance(marc_bytes, bytes)
        assert len(marc_bytes) >= 24  # At least leader


class TestRecordMultipleFields:
    """Test records with multiple fields."""

    def test_add_multiple_fields_same_tag(self):
        """Test adding multiple fields with the same tag."""
        record = Record()
        
        for i in range(5):
            field = Field('650', ' ', '0')
            field.add_subfield('a', f'Subject {i}')
            record.add_field(field)
        
        subjects = record.subjects()
        assert len(subjects) >= 5


class TestFieldOperations:
    """Test field-level operations."""

    def test_field_subfields_iteration(self):
        """Test iterating over field subfields."""
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Main title')
        field.add_subfield('b', 'Subtitle')
        field.add_subfield('c', 'Responsibility')
        
        subfields = field.subfields()
        assert len(subfields) == 3
        
        codes = [sf.code for sf in subfields]
        assert 'a' in codes
        assert 'b' in codes
        assert 'c' in codes


    def test_field_subfields_by_code(self):
        """Test getting subfields by code."""
        field = Field('300', ' ', ' ')
        field.add_subfield('a', 'Pages')
        field.add_subfield('c', 'Height')
        
        a_values = field.subfields_by_code('a')
        assert len(a_values) == 1
        assert a_values[0] == 'Pages'
        
        c_values = field.subfields_by_code('c')
        assert len(c_values) == 1
        assert c_values[0] == 'Height'


    def test_field_dict_like_access(self):
        """Test dictionary-like access to subfield values."""
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Title Value')
        field.add_subfield('b', 'Subtitle Value')
        
        # Test __getitem__
        assert field['a'] == 'Title Value'
        assert field['b'] == 'Subtitle Value'
        
        # Test __contains__
        assert 'a' in field
        assert 'b' in field
        assert 'z' not in field
        
        # Test get with default
        assert field.get('a') == 'Title Value'
        assert field.get('missing', 'default') == 'default'


    def test_field_get_subfields(self):
        """Test getting multiple subfields at once."""
        field = Field('260', ' ', ' ')
        field.add_subfield('a', 'New York')
        field.add_subfield('b', 'Publisher')
        field.add_subfield('c', '2023')
        field.add_subfield('d', 'Distributor')
        
        # Get multiple codes at once
        values = field.get_subfields('a', 'b')
        assert 'New York' in values
        assert 'Publisher' in values


    def test_field_indicators_mutation(self):
        """Test modifying field indicators."""
        field = Field('245', '0', '0')
        assert field.indicator1 == '0'
        assert field.indicator2 == '0'
        
        # Modify indicators
        field.indicator1 = '1'
        field.indicator2 = '4'
        
        assert field.indicator1 == '1'
        assert field.indicator2 == '4'


class TestRecordRoundTrip:
    """Test round-trip serialization and deserialization."""

    def test_roundtrip_with_control_fields(self):
        """Test round-trip with control fields."""
        original = Record()
        original.add_control_field('001', 'control-123')
        original.add_control_field('003', 'ABC')
        original.add_control_field('005', '20231231120000.0')
        
        # Serialize
        marc_bytes = original.to_marc21()
        
        # Deserialize
        reader = MARCReader(io.BytesIO(marc_bytes))
        restored = reader.read_record()
        
        assert restored is not None
        assert restored.control_field('001') == 'control-123'
        assert restored.control_field('003') == 'ABC'


    def test_roundtrip_with_multiple_fields(self):
        """Test round-trip with multiple data fields."""
        original = Record()
        original.add_control_field('001', 'id-456')
        
        # Add various fields
        original.add_field(create_field('245', '1', '0', 
                                        a='Title', b='Subtitle'))
        original.add_field(create_field('100', '1', ' ', a='Author'))
        original.add_field(create_field('020', ' ', ' ', a='ISBN123'))
        original.add_field(create_field('650', ' ', '0', a='Subject 1'))
        original.add_field(create_field('650', ' ', '0', a='Subject 2'))
        
        # Serialize and deserialize
        marc_bytes = original.to_marc21()
        reader = MARCReader(io.BytesIO(marc_bytes))
        restored = reader.read_record()
        
        assert restored is not None
        assert restored.title() is not None
        assert restored.author() is not None
        assert restored.isbn() is not None
        assert len(restored.subjects()) >= 2


    def test_roundtrip_preserves_indicators(self):
        """Test that round-trip preserves field indicators."""
        original = Record()
        
        # Add field with specific indicators
        field = Field('245', '1', '4')
        field.add_subfield('a', 'The title')
        original.add_field(field)
        
        # Serialize and deserialize
        marc_bytes = original.to_marc21()
        reader = MARCReader(io.BytesIO(marc_bytes))
        restored = reader.read_record()
        
        restored_field = restored['245']
        assert restored_field is not None
        assert restored_field.indicator1 == '1'
        assert restored_field.indicator2 == '4'


class TestFormatConversions:
    """Test conversions to various formats."""

    def test_to_json_format(self):
        """Test JSON serialization produces valid output."""
        record = Record()
        record.add_field(create_field('245', '1', '0', a='Test Title'))
        
        json_str = record.to_json()
        assert isinstance(json_str, str)
        assert len(json_str) > 0
        
        # Should contain field data
        assert '245' in json_str or 'Test Title' in json_str


    def test_to_xml_format(self):
        """Test XML serialization produces valid output."""
        record = Record()
        record.add_field(create_field('245', '1', '0', a='Test Title'))
        
        xml_str = record.to_xml()
        assert isinstance(xml_str, str)
        assert len(xml_str) > 0
        assert '<' in xml_str


    def test_to_marcjson_format(self):
        """Test MARCJSON serialization."""
        record = Record()
        record.add_control_field('001', 'test-id')
        record.add_field(create_field('245', '1', '0', a='Title'))
        
        marcjson_str = record.to_marcjson()
        assert isinstance(marcjson_str, str)
        assert len(marcjson_str) > 0


    def test_dublin_core_conversion(self):
        """Test Dublin Core metadata conversion."""
        record = Record()
        record.add_field(create_field('245', '1', '0', a='Test Title'))
        record.add_field(create_field('100', '1', ' ', a='Test Author'))
        record.add_field(create_field('260', ' ', ' ', 
                                      b='Test Publisher', c='2023'))
        
        dc = record.to_dublin_core()
        
        assert isinstance(dc, dict)
        assert 'title' in dc
        assert 'creator' in dc
        assert 'publisher' in dc


class TestFormatConversionWrapping:
    """Test that format conversion functions return properly wrapped Python objects."""

    def _make_marcjson(self):
        """Create a MARCJSON string for testing."""
        import json
        return json.dumps([
            {"leader": "01826cam a2200421 a 4500"},
            {"001": "12345"},
            {"245": {"ind1": "1", "ind2": "0", "subfields": [{"a": "Test title /"}]}},
            {"650": {"ind1": " ", "ind2": "0", "subfields": [{"a": "Testing."}]}},
        ])

    def test_marcjson_to_record_returns_wrapped_record(self):
        """Test that marcjson_to_record returns a Python Record, not a raw Rust object."""
        from mrrc import marcjson_to_record
        record = marcjson_to_record(self._make_marcjson())
        assert type(record).__name__ == 'Record'
        assert type(record).__module__ == 'mrrc'

    def test_marcjson_to_record_fields_are_wrapped(self):
        """Test that fields from marcjson_to_record records support subscript access."""
        from mrrc import marcjson_to_record
        record = marcjson_to_record(self._make_marcjson())
        fields = record.get_fields('245')
        assert len(fields) == 1
        f = fields[0]
        assert type(f).__name__ == 'Field'
        assert type(f).__module__ == 'mrrc'
        # Subscript access should work
        assert f['a'] == 'Test title /'

    def test_marcjson_to_record_leader_is_wrapped(self):
        """Test that leader from marcjson_to_record records supports indexing."""
        from mrrc import marcjson_to_record
        record = marcjson_to_record(self._make_marcjson())
        ldr = record.leader()
        assert type(ldr).__name__ == 'Leader'
        assert type(ldr).__module__ == 'mrrc'
        assert ldr[9] is not None

    def test_marcjson_to_record_helper_methods_work(self):
        """Test that helper methods work on marcjson_to_record records."""
        from mrrc import marcjson_to_record
        record = marcjson_to_record(self._make_marcjson())
        assert record.title() == 'Test title /'
        assert record.subjects() == ['Testing.']

    def test_json_to_record_returns_wrapped_record(self):
        """Test that json_to_record returns a wrapped Python Record."""
        from mrrc import json_to_record
        record = Record()
        record.add_field(create_field('245', '1', '0', a='Test Title'))
        json_str = record.to_json()
        restored = json_to_record(json_str)
        assert type(restored).__name__ == 'Record'
        assert type(restored).__module__ == 'mrrc'
        assert restored.title() == 'Test Title'
        fields = restored.get_fields('245')
        assert fields[0]['a'] == 'Test Title'

    def test_xml_to_record_returns_wrapped_record(self):
        """Test that xml_to_record returns a wrapped Python Record."""
        from mrrc import xml_to_record
        record = Record()
        record.add_field(create_field('245', '1', '0', a='Test Title'))
        xml_str = record.to_xml()
        restored = xml_to_record(xml_str)
        assert type(restored).__name__ == 'Record'
        assert type(restored).__module__ == 'mrrc'
        assert restored.title() == 'Test Title'
        fields = restored.get_fields('245')
        assert fields[0]['a'] == 'Test Title'


class TestControlFields:
    """Test control field operations."""

    def test_control_field_roundtrip(self):
        """Test control field preservation in round-trip."""
        record = Record()
        
        test_fields = {
            '001': '12345',
            '003': 'DLC',
            '005': '20231201',
            '006': 'fixed006value',
            '007': 'fixed007value',
            '008': '230601s2023    xxu||||||||||||eng d'
        }
        
        for tag, value in test_fields.items():
            record.add_control_field(tag, value)
        
        # Verify they're there
        for tag, expected_value in test_fields.items():
            actual = record.control_field(tag)
            assert actual == expected_value


    def test_multiple_control_fields(self):
        """Test getting all control fields."""
        record = Record()
        record.add_control_field('001', 'id1')
        record.add_control_field('003', 'source1')
        
        cfs = record.control_fields()
        assert len(cfs) >= 2


class TestRecordTypeDetection:
    """Test record type detection methods."""

    def test_is_book_detection(self):
        """Test book detection."""
        leader = Leader()
        leader.record_type = 'a'
        leader.bibliographic_level = 'm'
        record = Record(leader)
        
        assert record.is_book() is True


    def test_is_serial_detection(self):
        """Test serial detection."""
        leader = Leader()
        leader.record_type = 'a'
        leader.bibliographic_level = 's'
        record = Record(leader)
        
        assert record.is_serial() is True


    def test_is_music_detection(self):
        """Test music detection."""
        leader = Leader()
        leader.record_type = 'c'
        record = Record(leader)
        
        assert record.is_music() is True


    def test_is_audiovisual_detection(self):
        """Test audiovisual detection."""
        leader = Leader()
        leader.record_type = 'g'
        record = Record(leader)
        
        assert record.is_audiovisual() is True


class TestRecordRemoval:
    """Test field removal operations."""

    def test_remove_field_by_tag(self):
        """Test removing a field by tag."""
        record = Record()
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Test')
        record.add_field(field)
        
        assert record['245'] is not None
        
        # Remove the field
        record.remove_field('245')
        
        # Verify it's gone
        assert record['245'] is None


class TestFieldSerialization:
    """Test field-level serialization."""

    def test_field_with_repeated_subfields(self):
        """Test field with repeated subfield codes."""
        field = Field('300', ' ', ' ')
        field.add_subfield('a', 'Part 1')
        field.add_subfield('a', 'Part 2')
        field.add_subfield('c', 'Size')
        
        # Get all 'a' values
        a_values = field.subfields_by_code('a')
        assert len(a_values) == 2
        assert 'Part 1' in a_values
        assert 'Part 2' in a_values


class TestLeaderProperties:
    """Test Leader property access and modification."""

    def test_leader_defaults(self):
        """Test default leader values."""
        leader = Leader()
        
        assert leader.record_type == 'a'
        assert leader.bibliographic_level == 'm'
        assert leader.record_status == 'n'
        assert leader.character_coding == ' '


    def test_leader_modification(self):
        """Test modifying leader properties."""
        leader = Leader()
        
        leader.record_type = 'c'
        assert leader.record_type == 'c'
        
        leader.bibliographic_level = 'd'
        assert leader.bibliographic_level == 'd'
        
        leader.record_status = 'a'
        assert leader.record_status == 'a'


    def test_leader_encoding_level(self):
        """Test leader encoding level setting."""
        leader = Leader()
        leader.encoding_level = '4'
        assert leader.encoding_level == '4'


    def test_leader_cataloging_form(self):
        """Test leader cataloging form (descriptor_cataloging_form)."""
        leader = Leader()
        
        # Test via descriptor_cataloging_form property
        leader.descriptive_cataloging_form = 'a'
        assert leader.descriptive_cataloging_form == 'a'


class TestUnicodeAndEncoding:
    """Test unicode and special character handling."""

    def test_field_with_unicode_subfields(self):
        """Test fields with unicode characters."""
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Tïtlé wíth üñíçödé')
        
        subfields = field.subfields_by_code('a')
        assert 'üñíçödé' in subfields[0]


    def test_record_with_unicode_fields(self):
        """Test record with unicode content."""
        record = Record()
        
        field = Field('245', '1', '0')
        field.add_subfield('a', '日本語タイトル')
        record.add_field(field)
        
        title = record.title()
        assert title is not None
        assert '日本語' in title


    def test_roundtrip_preserves_unicode(self):
        """Test that round-trip preserves unicode characters."""
        original = Record()
        
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Titel in Français')
        original.add_field(field)
        
        # Serialize and deserialize
        marc_bytes = original.to_marc21()
        reader = MARCReader(io.BytesIO(marc_bytes))
        restored = reader.read_record()
        
        restored_field = restored['245']
        assert restored_field is not None
        a_values = restored_field.subfields_by_code('a')
        assert 'Français' in a_values[0]


class TestMARCWriterIntegration:
    """Test MARCWriter integration."""

    def test_write_multiple_records(self):
        """Test writing multiple records to a stream."""
        buffer = io.BytesIO()
        writer = MARCWriter(buffer)
        
        # Write 3 records
        for i in range(3):
            record = Record()
            record.add_control_field('001', f'id-{i}')
            field = Field('245', '1', '0')
            field.add_subfield('a', f'Title {i}')
            record.add_field(field)
            writer.write(record)
        
        # Read them back
        buffer.seek(0)
        reader = MARCReader(buffer)
        
        count = 0
        for record in reader:
            assert record is not None
            count += 1
        
        assert count == 3


class TestFieldConveniences:
    """Test convenience methods for common fields."""

    def test_get_multiple_fields_by_tags(self):
        """Test getting multiple fields by multiple tags."""
        record = Record()
        
        record.add_field(create_field('245', '1', '0', a='Title'))
        record.add_field(create_field('100', '1', ' ', a='Author'))
        record.add_field(create_field('260', ' ', ' ', b='Publisher'))
        record.add_field(create_field('300', ' ', ' ', a='Pages'))
        
        # Get multiple tags at once
        fields = record.get_fields('245', '100', '260')
        assert len(fields) >= 3


    def test_all_fields_access(self):
        """Test getting all fields at once."""
        record = Record()
        
        record.add_field(create_field('245', '1', '0', a='Title'))
        record.add_field(create_field('100', '1', ' ', a='Author'))
        record.add_field(create_field('650', ' ', '0', a='Subject'))
        
        all_fields = record.get_fields()
        assert len(all_fields) >= 3


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
