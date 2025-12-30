"""
Unit tests for basic mrrc Python wrapper functionality.
Tests API compatibility with pymarc.
"""

import pytest
from mrrc import MARCReader, Record, Field, Leader, Subfield
import io


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
        leader.set_record_type('c')
        assert leader.record_type == 'c'
        
        leader.set_bibliographic_level('d')
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
        subfields = [
            Subfield('a', 'The pragmatic programmer :'),
            Subfield('b', 'from journeyman to master /')
        ]
        field = Field('245', subfields=subfields)
        assert field.tag == '245'
    
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
        
        subfields = [Subfield('a', 'Test Title')]
        field = Field('245', subfields=subfields)
        record.add_field(field)
        
        fields = record.fields_by_tag('245')
        assert len(fields) == 1
    
    def test_get_fields_by_tag(self):
        """Test retrieving fields by tag."""
        leader = Leader()
        record = Record(leader)
        
        # Add multiple 650 fields
        for i in range(3):
            subfields = [Subfield('a', f'Subject {i}')]
            field = Field('650', subfields=subfields)
            record.add_field(field)
        
        fields = record.fields_by_tag('650')
        assert len(fields) == 3
    
    def test_get_all_fields(self):
        """Test getting all fields from a record."""
        leader = Leader()
        record = Record(leader)
        
        for i in range(2):
            subfields = [Subfield('a', f'Title {i}')]
            field = Field('245', subfields=subfields)
            record.add_field(field)
        
        all_fields = record.fields()
        assert len(all_fields) >= 2


class TestConvenienceMethods:
    """Test pymarc-style convenience methods."""
    
    def test_title_method(self):
        """Test the title() convenience method."""
        leader = Leader()
        record = Record(leader)
        
        subfields = [
            Subfield('a', 'The pragmatic programmer :'),
            Subfield('b', 'from journeyman to master /')
        ]
        field = Field('245', subfields=subfields)
        record.add_field(field)
        
        title = record.title()
        assert title is not None
        assert 'pragmatic' in title.lower()
    
    def test_author_method(self):
        """Test the author() convenience method."""
        leader = Leader()
        record = Record(leader)
        
        subfields = [Subfield('a', 'Hunt, Andrew')]
        field = Field('100', subfields=subfields)
        record.add_field(field)
        
        author = record.author()
        assert author is not None
        assert 'Hunt' in author
    
    def test_isbn_method(self):
        """Test the isbn() convenience method."""
        leader = Leader()
        record = Record(leader)
        
        subfields = [Subfield('a', '0201616165')]
        field = Field('020', subfields=subfields)
        record.add_field(field)
        
        isbn = record.isbn()
        assert isbn is not None
        assert '0201616165' in isbn


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
        
        subfields = [Subfield('a', 'Test Title')]
        field = Field('245', subfields=subfields)
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
        subfields = [
            Subfield('a', 'value_a'),
            Subfield('b', 'value_b'),
            Subfield('c', 'value_c')
        ]
        field = Field('245', subfields=subfields)
        
        # This pattern works in pymarc
        # field.get_subfields('a', 'b')
        # We need to test what mrrc currently supports
        assert len(field.subfields) == 3


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
        subfields = [Subfield('a', 'Test Title')]
        field = Field('245', subfields=subfields)
        record.add_field(field)
        
        dc = record.to_dublin_core()
        assert isinstance(dc, dict)
        assert 'title' in dc


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
        subfields = [Subfield('a', 'Test')]
        field = Field('245', indicators=['1', '0'], subfields=subfields)
        assert field.tag == '245'
        # Note: need to verify indicator access methods


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
