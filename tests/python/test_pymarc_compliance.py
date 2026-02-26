"""
PyMARC Compliance Tests
========================

These tests are ported directly from the pymarc test suite to ensure
complete API compatibility. The logic is preserved strictly from pymarc
to help identify API gaps and ensure our Python wrapper achieves 100%
compatibility with the original pymarc library.

Reference: https://gitlab.com/pymarc/pymarc/-/tree/main/test
"""

import pytest
from mrrc import MARCReader, Record, Field, Leader
import io


# ============================================================================
# Test Fixtures - Sample data and helpers
# ============================================================================

def create_field(tag, ind1=' ', ind2=' ', **subfields):
    """PYMARC COMPAT: Helper to create a field with subfields."""
    field = Field(tag, ind1, ind2)
    for code, value in subfields.items():
        field.add_subfield(code, value)
    return field


# ============================================================================
# Record Tests (from pymarc test_record.py)
# ============================================================================

class TestRecordAddField:
    """PYMARC COMPAT: test_add_field"""
    
    def test_add_field(self):
        """Test basic field addition to a record."""
        record = Record()
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Python')
        field.add_subfield('c', 'Guido')
        record.add_field(field)
        
        assert field in record.fields()


class TestRecordFields:
    """PYMARC COMPAT: test_fields"""
    
    def test_fields_access(self):
        """Test accessing fields via dictionary-like syntax."""
        record = Record()
        
        field1 = Field('245', '1', '0')
        field1.add_subfield('a', 'Python')
        field1.add_subfield('c', 'Guido')
        record.add_field(field1)
        
        field2 = Field('260', ' ', ' ')
        field2.add_subfield('a', 'Amsterdam')
        record.add_field(field2)
        
        assert record['245'] is not None
        assert record['245']['a'] == 'Python'
        assert record['260']['a'] == 'Amsterdam'


class TestRecordRemoveField:
    """PYMARC COMPAT: test_remove_field"""
    
    def test_remove_existing_field(self):
        """Test removing a field that exists."""
        record = Record()
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Python')
        field.add_subfield('c', 'Guido')
        record.add_field(field)
        
        assert record['245']['a'] == 'Python'
        
        # Remove the field
        record.remove_field(field)
        assert record.get_field('245') is None


class TestRecordQuickAccess:
    """PYMARC COMPAT: test_quick_access"""
    
    def test_quick_access_syntax(self):
        """Test record[tag] quick access."""
        record = Record()
        title = Field('245', '1', '0')
        title.add_subfield('a', 'Python')
        title.add_subfield('c', 'Guido')
        record.add_field(title)
        
        assert record['245'] == title
        assert record.get_field('999') is None


class TestRecordMembership:
    """PYMARC COMPAT: test_membership"""
    
    def test_in_operator(self):
        """Test 'in' operator for tag membership."""
        record = Record()
        title = Field('245', '1', '0')
        title.add_subfield('a', 'Python')
        record.add_field(title)
        
        assert '245' in record
        assert '999' not in record


class TestRecordFind:
    """PYMARC COMPAT: test_find"""
    
    def test_get_fields_by_tag(self):
        """Test get_fields() to retrieve multiple fields by tag."""
        record = Record()
        
        subject1 = Field('650', ' ', '0')
        subject1.add_subfield('a', 'Programming Language')
        record.add_field(subject1)
        
        subject2 = Field('650', ' ', '0')
        subject2.add_subfield('a', 'Object Oriented')
        record.add_field(subject2)
        
        found = record.get_fields('650')
        assert len(found) == 2
        assert found[0].subfields_by_code('a')[0] == 'Programming Language'
        assert found[1].subfields_by_code('a')[0] == 'Object Oriented'
        
        # Test get_fields() with no tag (returns all)
        found_all = record.get_fields()
        assert len(found_all) == 2


class TestRecordMultiFind:
    """PYMARC COMPAT: test_multi_find"""
    
    def test_get_fields_multiple_tags(self):
        """Test get_fields() with multiple tags."""
        record = Record()
        
        subject1 = Field('650', ' ', '0')
        subject1.add_subfield('a', 'Programming Language')
        record.add_field(subject1)
        
        subject2 = Field('651', ' ', '0')
        subject2.add_subfield('a', 'Object Oriented')
        record.add_field(subject2)
        
        found = record.get_fields('650', '651')
        assert len(found) == 2


class TestRecordFieldNotFound:
    """PYMARC COMPAT: test_field_not_found"""
    
    def test_empty_record_fields(self):
        """Test that an empty record has no fields."""
        record = Record()
        assert len(record.fields()) == 0


class TestRecordAuthor:
    """PYMARC COMPAT: test_author"""
    
    def test_author_from_100_field(self):
        """Test getting author from 100 field."""
        record = Record()
        assert record.author() is None
        
        record.add_field(create_field('100', '1', ' ', 
                                      a='Bletch, Foobie,',
                                      d='1979-1981.'))
        assert record.author() is not None
        assert 'Bletch' in record.author()


class TestRecordUniformTitle:
    """PYMARC COMPAT: test_uniformtitle"""
    
    def test_uniform_title_from_130_field(self):
        """Test getting uniform title from 130 field."""
        record = Record()
        assert record.uniform_title() is None
        
        record.add_field(create_field('130', '0', ' ',
                                      a='Tosefta.',
                                      l='English.',
                                      f='1977.'))
        uniform_title = record.uniform_title()
        assert uniform_title is not None


class TestRecordSubjects:
    """PYMARC COMPAT: test_subjects"""

    def test_subjects_from_650_fields(self):
        """Test getting subjects from 650 fields."""
        record = Record()

        record.add_field(create_field('650', ' ', '0', a='Computer science'))
        record.add_field(create_field('650', ' ', '0', a='Python language'))

        subjects = record.subjects()
        assert len(subjects) >= 2
        assert any('Computer' in s for s in subjects)

    def test_subjects_from_all_6xx_fields(self):
        """Test that subjects() returns entries from all 6xx fields, matching pymarc."""
        record = Record()

        record.add_field(create_field('600', '1', '0', a='Maimonides, Moses,'))
        record.add_field(create_field('610', '2', '0', a='United Nations'))
        record.add_field(create_field('611', '2', '0', a='Vatican Council'))
        record.add_field(create_field('630', '0', '4', a='Talmud Bavli.'))
        record.add_field(create_field('648', ' ', '0', a='20th century'))
        record.add_field(create_field('650', ' ', '0', a='Jewish law.'))
        record.add_field(create_field('651', ' ', '0', a='Jerusalem'))
        record.add_field(create_field('655', ' ', '7', a='Commentaries.'))

        subjects = record.subjects()
        assert len(subjects) == 8
        assert 'Maimonides, Moses,' in subjects
        assert 'United Nations' in subjects
        assert 'Vatican Council' in subjects
        assert 'Talmud Bavli.' in subjects
        assert '20th century' in subjects
        assert 'Jewish law.' in subjects
        assert 'Jerusalem' in subjects
        assert 'Commentaries.' in subjects


class TestRecordPublisher:
    """PYMARC COMPAT: test_publisher"""
    
    def test_publisher_from_260_field(self):
        """Test getting publisher from 260 field."""
        record = Record()
        assert record.publisher() is None
        
        record.add_field(create_field('260', ' ', ' ',
                                      a='Paris :',
                                      b='Gauthier-Villars ;',
                                      c='1955.'))
        assert record.publisher() is not None
        assert 'Villars' in record.publisher()


    def test_publisher_from_264_rda_field(self):
        """Test getting publisher from 264 field (RDA cataloging)."""
        record = Record()
        record.add_field(create_field('264', ' ', '1',
                                      a='Cambridge, Massachusetts :',
                                      b='The MIT Press,',
                                      c='[2022]'))
        assert record.publisher() == 'The MIT Press,'

    def test_publisher_prefers_260_over_264(self):
        """Test that 260 is preferred when both 260 and 264 exist."""
        record = Record()
        record.add_field(create_field('260', ' ', ' ',
                                      b='Old Publisher,'))
        record.add_field(create_field('264', ' ', '1',
                                      b='New Publisher,'))
        assert record.publisher() == 'Old Publisher,'

    def test_publisher_ignores_264_non_publication(self):
        """Test that 264 with ind2 != '1' is not used for publisher."""
        record = Record()
        record.add_field(create_field('264', ' ', '3',
                                      b='Some Printer,'))
        assert record.publisher() is None


class TestRecordPublicationYear:
    """PYMARC COMPAT: test_pubyear"""

    def test_publication_year_from_260_field(self):
        """Test getting publication year from 260 field."""
        record = Record()
        assert record.pubyear() is None

        record.add_field(create_field('260', ' ', ' ',
                                      a='Paris :',
                                      b='Gauthier-Villars ;',
                                      c='1955.'))
        year = record.pubyear()
        assert year is not None

    def test_publication_year_from_264_rda_field(self):
        """Test getting publication year from 264 field (RDA cataloging)."""
        record = Record()
        record.add_field(create_field('264', ' ', '1',
                                      c='[2022]'))
        assert record.pubyear() == 2022


class TestRecordISBN:
    """PYMARC COMPAT: test_isbn"""
    
    def test_isbn_from_020_field(self):
        """Test getting ISBN from 020 field."""
        record = Record()
        assert record.isbn() is None
        
        record.add_field(create_field('020', ' ', ' ', a='0914378287'))
        assert record.isbn() is not None


class TestRecordISSN:
    """PYMARC COMPAT: test_issn"""
    
    def test_issn_from_022_field(self):
        """Test getting ISSN from 022 field."""
        record = Record()
        assert record.issn() is None
        
        record.add_field(create_field('022', ' ', ' ', a='0028-0836'))
        issn = record.issn()
        assert issn is not None
        assert '0028-0836' in issn


class TestRecordTitle:
    """PYMARC COMPAT: test_title"""
    
    def test_title_from_245_field(self):
        """Test getting title from 245 field."""
        record = Record()
        assert record.title() is None
        
        record.add_field(create_field('245', '1', '0', a='Python Programming'))
        title = record.title()
        assert title is not None
        assert 'Python' in title


class TestRecordAsMarc:
    """PYMARC COMPAT: test_as_marc"""
    
    def test_as_marc_serialization(self):
        """Test as_marc() serialization (now to_marc21())."""
        record = Record()
        
        record.add_field(create_field('245', '0', '1', a='The pragmatic programmer'))
        
        # Get binary MARC format
        marc_bytes = record.to_marc21()
        
        assert isinstance(marc_bytes, bytes)
        assert len(marc_bytes) > 0
        
        # First 24 bytes are leader
        assert len(marc_bytes) >= 24


class TestRecordAsJsonXml:
    """PYMARC COMPAT: test_as_json, test_as_xml"""
    
    def test_as_json_format(self):
        """Test as_json() serialization."""
        record = Record()
        record.add_field(create_field('245', '1', '0', a='Title'))
        
        json_str = record.to_json()
        assert json_str is not None
        assert isinstance(json_str, str)


    def test_as_xml_format(self):
        """Test as_xml() serialization."""
        record = Record()
        record.add_field(create_field('245', '1', '0', a='Title'))
        
        xml_str = record.to_xml()
        assert xml_str is not None
        assert isinstance(xml_str, str)
        assert '<' in xml_str


class TestRecordPhysicalDescription:
    """PYMARC COMPAT: test_physicaldescription"""
    
    def test_physical_description(self):
        """Test physical description property."""
        record = Record()
        assert record.physical_description() is None
        
        record.add_field(create_field('300', ' ', ' ',
                                      a='1 photographic print :',
                                      b='gelatin silver ;',
                                      c='10 x 56 in.'))
        desc = record.physical_description()
        assert desc is not None


class TestRecordLocation:
    """PYMARC COMPAT: test_location"""
    
    def test_location_field(self):
        """Test location fields (852)."""
        record = Record()
        assert record.location() == []
        
        record.add_field(create_field('852', ' ', ' ',
                                      a='Main Library',
                                      b='Reference'))
        locs = record.location()
        assert len(locs) >= 1


class TestRecordNotes:
    """PYMARC COMPAT: test_notes"""
    
    def test_notes_field(self):
        """Test notes from 5xx fields."""
        record = Record()
        assert record.notes() == []
        
        record.add_field(create_field('500', ' ', ' ',
                                      a='This is a general note.'))
        notes = record.notes()
        assert len(notes) >= 1


# ============================================================================
# Reader Tests (from pymarc test_reader.py)
# ============================================================================

class TestMARCReaderBasic:
    """PYMARC COMPAT: Basic MARCReader functionality"""
    
    def test_reader_iteration(self, fixture_1k):
        """Test iterating over MARC records."""
        data = io.BytesIO(fixture_1k)
        reader = MARCReader(data)
        
        count = 0
        for record in reader:
            if record:
                count += 1
            if count >= 5:
                break
        
        assert count > 0


class TestMARCReaderRecordCount:
    """PYMARC COMPAT: Count records in file"""
    
    def test_reader_count(self, fixture_1k):
        """Test counting all records."""
        data = io.BytesIO(fixture_1k)
        reader = MARCReader(data)
        
        count = 0
        for record in reader:
            if record:
                count += 1
        
        assert count > 0


# ============================================================================
# Field Tests (from pymarc test_field.py)
# ============================================================================

class TestFieldCreation:
    """PYMARC COMPAT: test_field.py - Field creation"""
    
    def test_field_with_subfields(self):
        """Test creating a field with subfields."""
        field = Field('245', '1', '0')
        field.add_subfield('a', 'The pragmatic programmer :')
        field.add_subfield('b', 'from journeyman to master /')
        
        assert field.tag == '245'
        assert len(field.subfields()) == 2


    def test_field_indicators(self):
        """Test field indicators."""
        field = Field('245', '1', '0')
        assert field.indicator1 == '1'
        assert field.indicator2 == '0'


class TestFieldSubfieldAccess:
    """PYMARC COMPAT: test_field.py - Subfield access"""
    
    def test_subfield_by_code(self):
        """Test getting subfield values by code."""
        field = Field('245', '1', '0')
        field.add_subfield('a', 'Main title')
        field.add_subfield('b', 'Subtitle')
        
        a_values = field.subfields_by_code('a')
        assert len(a_values) == 1
        assert a_values[0] == 'Main title'


    def test_get_subfields_multiple_codes(self):
        """Test getting multiple subfield codes at once."""
        field = Field('260', ' ', ' ')
        field.add_subfield('a', 'New York')
        field.add_subfield('b', 'Publisher')
        field.add_subfield('c', '2023')
        
        values = field.get_subfields('a', 'b')
        assert 'New York' in values
        assert 'Publisher' in values


# ============================================================================
# Leader Tests (from pymarc test_leader.py)
# ============================================================================

class TestLeaderBasics:
    """PYMARC COMPAT: test_leader.py - Leader basics"""
    
    def test_leader_creation(self):
        """Test creating a leader."""
        leader = Leader()
        assert leader.record_type == 'a'
        assert leader.bibliographic_level == 'm'
        assert leader.record_status == 'n'


    def test_leader_properties(self):
        """Test setting leader properties."""
        leader = Leader()
        
        leader.record_type = 'c'
        assert leader.record_type == 'c'
        
        leader.bibliographic_level = 'd'
        assert leader.bibliographic_level == 'd'


# ============================================================================
# Round-Trip and Integration Tests
# ============================================================================

class TestRoundTripSerialization:
    """PYMARC COMPAT: Round-trip serialization tests"""
    
    def test_record_roundtrip(self):
        """Test creating, serializing, and deserializing a record."""
        # Create original
        original = Record()
        original.add_control_field('001', 'original-id-001')
        original.add_field(create_field('245', '1', '0', a='Test Title'))
        original.add_field(create_field('100', '1', ' ', a='Test Author'))
        
        # Serialize
        marc_bytes = original.to_marc21()
        
        # Deserialize
        reader = MARCReader(io.BytesIO(marc_bytes))
        restored = reader.read_record()
        
        assert restored is not None
        assert restored.title() is not None
        assert restored.author() is not None


class TestMultipleFormats:
    """PYMARC COMPAT: Test output formats"""
    
    def test_json_roundtrip_concept(self):
        """Test JSON serialization."""
        record = Record()
        record.add_field(create_field('245', '1', '0', a='Title'))
        
        json_str = record.to_json()
        assert isinstance(json_str, str)
        assert len(json_str) > 0


    def test_xml_serialization(self):
        """Test XML serialization."""
        record = Record()
        record.add_field(create_field('245', '1', '0', a='Title'))
        
        xml_str = record.to_xml()
        assert isinstance(xml_str, str)
        assert len(xml_str) > 0


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
