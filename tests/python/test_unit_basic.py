"""
Unit tests for basic mrrc Python wrapper functionality.
Tests API compatibility with pymarc.
"""

import pytest
from mrrc import MARCReader, Record, Field, Leader, Subfield
import io


def create_field(tag, ind1='0', ind2='0', **subfields):
    """Helper to create a field with subfields."""
    field = Field(tag, ind1, ind2)
    for code, value in subfields.items():
        field.add_subfield(code, value)
    return field


class TestLeaderBasics:
    """Test Leader functionality."""
    
    def test_create_default_leader(self):
        """Test creating a default Leader."""
        leader = Leader()
        assert leader.record_type == 'a'
        assert leader.bibliographic_level == 'm'
        assert leader.record_status == 'n'
    
    def test_leader_properties(self):
        """Test setting/getting leader properties."""
        leader = Leader()
        leader.record_type = 'c'
        assert leader.record_type == 'c'
        
        leader.bibliographic_level = 'd'
        assert leader.bibliographic_level == 'd'


class TestRecordBasics:
    """Test Record creation and basic operations."""
    
    def test_create_empty_record(self):
        """Test creating an empty Record."""
        leader = Leader()
        record = Record(leader)
        assert len(record.fields()) == 0
    
    def test_add_control_field(self):
        """Test adding a control field."""
        leader = Leader()
        record = Record(leader)
        record.add_control_field('001', '12345')
        
        value = record.control_field('001')
        assert value == '12345'
    
    def test_control_field_not_found(self):
        """Test getting non-existent control field."""
        leader = Leader()
        record = Record(leader)
        value = record.control_field('999')
        assert value is None
    
    def test_get_all_control_fields(self):
        """Test getting all control fields."""
        leader = Leader()
        record = Record(leader)
        record.add_control_field('001', '12345')
        record.add_control_field('003', 'ABC')
        
        cfs = record.control_fields()
        assert len(cfs) >= 2


class TestFieldCreation:
    """Test Field and Subfield creation."""
    
    def test_create_field_with_subfields(self):
        """Test creating a field with subfields."""
        field = Field('245', '1', '0')
        field.add_subfield('a', 'The pragmatic programmer :')
        field.add_subfield('b', 'from journeyman to master /')
        assert field.tag == '245'
        assert len(field.subfields()) == 2
    
    def test_subfield_creation(self):
        """Test creating individual Subfield."""
        sf = Subfield('a', 'test value')
        assert sf.code == 'a'
        assert sf.value == 'test value'


class TestRecordFieldOperations:
    """Test adding and retrieving fields from records."""
    
    def test_add_field_to_record(self):
        """Test adding a field to a record."""
        leader = Leader()
        record = Record(leader)
        
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Test Title')
        record.add_field(field)
        
        fields = record.fields_by_tag('245')
        assert len(fields) == 1
    
    def test_get_fields_by_tag(self):
        """Test retrieving fields by tag."""
        leader = Leader()
        record = Record(leader)
        
        # Add multiple 650 fields
        for i in range(3):
            field = Field('650', ' ', '0')
            field.add_subfield('a', f'Subject {i}')
            record.add_field(field)
        
        fields = record.fields_by_tag('650')
        assert len(fields) == 3
    
    def test_get_all_fields(self):
        """Test getting all fields from a record."""
        leader = Leader()
        record = Record(leader)
        
        for i in range(2):
            field = Field('245', '1', '0')
            field.add_subfield('a', f'Title {i}')
            record.add_field(field)
        
        all_fields = record.fields()
        assert len(all_fields) >= 2


class TestConvenienceMethods:
    """Test pymarc-style convenience methods."""
    
    def test_title_method(self):
        """Test the title() convenience method."""
        leader = Leader()
        record = Record(leader)
        
        field = Field('245', '1', '0')
        field.add_subfield('a', 'The pragmatic programmer :')
        field.add_subfield('b', 'from journeyman to master /')
        record.add_field(field)
        
        title = record.title()
        assert title is not None
        assert 'pragmatic' in title.lower()
    
    def test_author_method(self):
        """Test the author() convenience method."""
        leader = Leader()
        record = Record(leader)
        record.add_field(create_field('100', '1', ' ', a='Hunt, Andrew'))
        
        author = record.author()
        assert author is not None
        assert 'Hunt' in author
    
    def test_isbn_method(self):
        """Test the isbn() convenience method."""
        leader = Leader()
        record = Record(leader)
        record.add_field(create_field('020', ' ', ' ', a='0201616165'))
        
        isbn = record.isbn()
        assert isbn is not None
        assert '0201616165' in isbn
    
    def test_subjects_method(self):
        """Test the subjects() convenience method."""
        leader = Leader()
        record = Record(leader)
        
        # Add multiple subject fields (650)
        for i in range(3):
            record.add_field(create_field('650', ' ', '0', a=f'Subject {i}'))
        
        subjects = record.subjects()
        assert len(subjects) == 3
        assert 'Subject 0' in subjects
    
    def test_location_method(self):
        """Test the location() convenience method."""
        leader = Leader()
        record = Record(leader)
        record.add_field(create_field('852', ' ', ' ', a='Main Library'))
        
        locations = record.location()
        assert len(locations) >= 1
        assert 'Main Library' in locations
    
    def test_notes_method(self):
        """Test the notes() convenience method."""
        leader = Leader()
        record = Record(leader)
        record.add_field(create_field('500', ' ', ' ', a='General note'))
        
        notes = record.notes()
        assert len(notes) >= 1
        assert 'General note' in notes
    
    def test_uniform_title_method(self):
        """Test the uniform_title() convenience method."""
        leader = Leader()
        record = Record(leader)
        record.add_field(create_field('130', ' ', '0', a='Standardized Title'))
        
        uniform_title = record.uniform_title()
        assert uniform_title is not None
        assert uniform_title == 'Standardized Title'
    
    def test_sudoc_method(self):
        """Test the sudoc() convenience method."""
        leader = Leader()
        record = Record(leader)
        record.add_field(create_field('086', ' ', ' ', a='I 19.2:En 3'))
        
        sudoc = record.sudoc()
        assert sudoc is not None
        assert sudoc == 'I 19.2:En 3'
    
    def test_issn_title_method(self):
        """Test the issn_title() convenience method."""
        leader = Leader()
        record = Record(leader)
        record.add_field(create_field('222', ' ', ' ', a='Key Title'))
        
        issn_title = record.issn_title()
        assert issn_title is not None
        assert issn_title == 'Key Title'
    
    def test_issnl_method(self):
        """Test the issnl() convenience method for ISSN-L."""
        leader = Leader()
        record = Record(leader)
        record.add_field(create_field('024', ' ', ' ', a='1234-5678'))
        
        issnl = record.issnl()
        assert issnl is not None
        assert issnl == '1234-5678'
    
    def test_pubyear_method(self):
        """Test the pubyear() convenience method (alias for publication_year)."""
        leader = Leader()
        record = Record(leader)
        
        # Add publication info
        field = Field('260', ' ', ' ')
        field.add_subfield('a', 'New York :')
        field.add_subfield('b', 'Penguin,')
        field.add_subfield('c', '2023')
        record.add_field(field)
        
        # pubyear() should work as alias
        year = record.pubyear()
        assert year is not None
        assert year == 2023
    
    def test_publisher_method(self):
        """Test the publisher() convenience method."""
        leader = Leader()
        record = Record(leader)
        
        field = Field('260', ' ', ' ')
        field.add_subfield('a', 'New York :')
        field.add_subfield('b', 'Addison-Wesley,')
        field.add_subfield('c', '2000')
        record.add_field(field)
        
        publisher = record.publisher()
        assert publisher is not None
        assert 'Addison-Wesley' in publisher
    
    def test_issn_method(self):
        """Test the issn() convenience method."""
        leader = Leader()
        record = Record(leader)
        record.add_field(create_field('022', ' ', ' ', a='0028-0836'))
        
        issn = record.issn()
        assert issn is not None
        assert issn == '0028-0836'
    
    def test_series_method(self):
        """Test the series() convenience method."""
        leader = Leader()
        record = Record(leader)
        record.add_field(create_field('490', ' ', ' ', a='Programming Patterns'))
        
        series = record.series()
        assert series is not None
        assert 'Programming' in series
    
    def test_physical_description_method(self):
        """Test the physical_description() convenience method."""
        leader = Leader()
        record = Record(leader)
        
        field = Field('300', ' ', ' ')
        field.add_subfield('a', '256 pages ;')
        field.add_subfield('c', '24 cm')
        record.add_field(field)
        
        phys_desc = record.physical_description()
        assert phys_desc is not None
        assert '256' in phys_desc


class TestFieldDictLike:
    """Test dictionary-like access to fields (pymarc style)."""
    
    def test_field_dict_access_not_implemented(self):
        """
        NOTE: This tests a missing feature.
        pymarc allows: record['245']['a']
        mrrc currently does not support this syntax.
        """
        leader = Leader()
        record = Record(leader)
        
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Test Title')
        record.add_field(field)
        
        # This should work in pymarc but doesn't in mrrc yet
        try:
            title_a = record['245']['a']
            assert title_a == 'Test Title'
        except (TypeError, KeyError):
            pytest.skip("Dictionary-like field access not yet implemented")


class TestFieldSubfieldDict:
    """Test subfield access patterns."""
    
    def test_get_subfield_values(self):
        """Test getting subfield values from a field."""
        field = Field('245', '1', '0')
        field.add_subfield('a', 'value_a')
        field.add_subfield('b', 'value_b')
        field.add_subfield('c', 'value_c')
        
        # This pattern works in pymarc
        # field.get_subfields('a', 'b')
        # We need to test what mrrc currently supports
        assert len(field.subfields()) == 3


class TestRecordEquality:
    """Test record equality comparison."""
    
    def test_record_equality(self):
        """Test comparing two identical records."""
        leader1 = Leader()
        record1 = Record(leader1)
        record1.add_control_field('001', 'test-id')
        
        leader2 = Leader()
        record2 = Record(leader2)
        record2.add_control_field('001', 'test-id')
        
        # Equality should work
        assert record1 == record2


class TestRecordSerialization:
    """Test converting records to various formats."""
    
    def test_to_json(self):
        """Test JSON serialization."""
        leader = Leader()
        record = Record(leader)
        record.add_control_field('001', 'test-id')
        
        json_str = record.to_json()
        assert json_str is not None
        assert '001' in json_str or 'test-id' in json_str
    
    def test_to_xml(self):
        """Test XML serialization."""
        leader = Leader()
        record = Record(leader)
        record.add_control_field('001', 'test-id')
        
        xml_str = record.to_xml()
        assert xml_str is not None
        assert 'xml' in xml_str.lower() or '<' in xml_str
    
    def test_to_dublin_core(self):
        """Test Dublin Core serialization."""
        leader = Leader()
        record = Record(leader)
        
        # Add title
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Test Title')
        record.add_field(field)
        
        dc = record.to_dublin_core()
        assert isinstance(dc, dict)
        assert 'title' in dc
    
    def test_to_marc21(self):
        """Test MARC21 binary serialization."""
        leader = Leader()
        record = Record(leader)
        record.add_control_field('001', 'test-id-001')
        
        # Add a title field
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Test Title')
        record.add_field(field)
        
        # Serialize to MARC21
        marc_bytes = record.to_marc21()
        
        # Verify it's bytes and not empty
        assert isinstance(marc_bytes, bytes)
        assert len(marc_bytes) > 0
        
        # MARC21 should start with a leader (24 bytes)
        assert len(marc_bytes) >= 24
        
        # Verify it can be read back
        reader = MARCReader(io.BytesIO(marc_bytes))
        read_record = reader.read_record()
        assert read_record is not None
        assert read_record.control_field('001') == 'test-id-001'


class TestReadingFromFile:
    """Test reading MARC records from file."""
    
    def test_marc_reader_basic(self, fixture_1k):
        """Test basic MARCReader functionality."""
        data = io.BytesIO(fixture_1k)
        reader = MARCReader(data)
        
        count = 0
        while record := reader.read_record():
            assert record is not None
            count += 1
            if count >= 5:  # Just test first 5
                break
        
        assert count > 0


class TestIndicators:
    """Test field indicators."""
    
    def test_field_indicators(self):
        """Test setting and getting field indicators."""
        # pymarc allows: indicators=['1', '0']
        # mrrc uses positional args: Field(tag, ind1, ind2)
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Test')
        assert field.tag == '245'
        assert field.indicator1 == '1'
        assert field.indicator2 == '0'


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
