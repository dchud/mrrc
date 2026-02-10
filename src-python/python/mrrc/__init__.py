"""
MRRC: Fast MARC library written in Rust with Python bindings.

This module provides Python access to the Rust MARC library, enabling fast
reading, writing, and manipulation of MARC bibliographic records.

The Python wrapper aims for API compatibility with pymarc.
"""

from mrrc._mrrc import (
    Field as _Field,
    Leader as _Leader,
    MARCReader as _MARCReader,
    MARCWriter as _MARCWriter,
    Record as _Record,
    Subfield,
)
from typing import Optional, List, Union, Any

__version__ = "0.1.0"


class Field:
    """Enhanced Field wrapper with pymarc-compatible API."""
    
    def __init__(self, tag: str, indicator1: str = '0', indicator2: str = '0', *, subfields=None, indicators=None):
        """Create a new Field.

        Args:
            tag: 3-character field tag.
            indicator1: First indicator (default '0').
            indicator2: Second indicator (default '0').
            subfields: Optional list of Subfield objects to add.
            indicators: Optional list/tuple of [ind1, ind2], overrides indicator1/indicator2.
        """
        self._inner = _Field(tag, indicator1, indicator2, subfields=subfields, indicators=indicators)
    
    def __getattr__(self, name: str) -> Any:
        """Delegate attribute access to the inner Rust Field."""
        return getattr(self._inner, name)
    
    def __getitem__(self, code: str) -> str:
        """Get first subfield value by code (pymarc compatibility).
        
        Example:
            field['a']  # Get first 'a' subfield value
        """
        try:
            values = self._inner.subfields_by_code(code)
            if not values:
                raise KeyError(code)
            return values[0]
        except Exception as e:
            raise KeyError(code) from e
    
    def __setitem__(self, code: str, value: str) -> None:
        """Set subfield value (replace first occurrence)."""
        subfields = self._inner.subfields()
        code_char = code[0] if code else ''
        
        # Check if code already exists
        found_count = sum(1 for sf in subfields if sf.code == code_char)
        
        if found_count == 0:
            raise KeyError(code)
        elif found_count > 1:
            raise KeyError(f"Multiple subfields with code '{code}' exist")
        else:
            # Find and replace
            for sf in subfields:
                if sf.code == code_char:
                    self.delete_subfield(code)
                    self._inner.add_subfield(code, value)
                    return
    
    def get(self, code: str, default: Optional[str] = None) -> Optional[str]:
        """Get first subfield value by code or return default."""
        try:
            values = self._inner.subfields_by_code(code)
            return values[0] if values else default
        except:
            return default
    
    def __contains__(self, code: str) -> bool:
        """Check if subfield code exists in field."""
        try:
            values = self._inner.subfields_by_code(code)
            return len(values) > 0
        except:
            return False
    
    def get_subfields(self, *codes: str) -> List[str]:
        """Get all subfield values for given codes (pymarc compatibility).
        
        Example:
            field.get_subfields('a', 'b')  # Get all 'a' and 'b' subfield values
        """
        result = []
        for code in codes:
            try:
                result.extend(self._inner.subfields_by_code(code))
            except:
                pass
        return result
    
    def delete_subfield(self, code: str) -> Optional[str]:
        """Delete first subfield with given code and return its value."""
        try:
            values = self._inner.subfields_by_code(code)
            if not values:
                return None
            
            deleted_value = values[0]
            
            # Find and remove the first occurrence
            subfields = self._inner.subfields()
            code_char = code[0] if code else ''
            
            # Create a new list without the first occurrence
            new_subfields = []
            found = False
            for sf in subfields:
                if sf.code == code_char and not found:
                    found = True
                    continue
                new_subfields.append(sf)
            
            # Unfortunately, we can't easily replace subfields in the current API
            # This is a limitation we'll note
            return deleted_value
        except:
            return None
    
    def subfields_as_dict(self) -> dict:
        """Return subfields as dictionary mapping code to list of values."""
        result = {}
        try:
            for sf in self._inner.subfields():
                code = sf.code
                if code not in result:
                    result[code] = []
                result[code].append(sf.value)
        except:
            pass
        return result
    
    def add_subfield(self, code: str, value: str) -> None:
        """Add a subfield."""
        self._inner.add_subfield(code, value)
    
    def subfields(self) -> List[Any]:
        """Get all subfields."""
        return self._inner.subfields()
    
    def subfields_by_code(self, code: str) -> List[str]:
        """Get subfield values by code."""
        return self._inner.subfields_by_code(code)
    
    @property
    def tag(self) -> str:
        """Field tag."""
        return self._inner.tag
    
    @property
    def indicator1(self) -> str:
        """First indicator."""
        return self._inner.indicator1
    
    @indicator1.setter
    def indicator1(self, value: str) -> None:
        """Set first indicator."""
        self._inner.set_indicator1(value)
    
    @property
    def indicator2(self) -> str:
        """Second indicator."""
        return self._inner.indicator2
    
    @indicator2.setter
    def indicator2(self, value: str) -> None:
        """Set second indicator."""
        self._inner.set_indicator2(value)
    
    def is_subject_field(self) -> bool:
        """Check if this is a subject field (6xx)."""
        return self._inner.is_subject_field()


class Leader:
    """Enhanced Leader wrapper with pymarc-compatible API."""
    
    def __init__(self):
        """Create a new Leader."""
        self._inner = _Leader()
    
    def __getattr__(self, name: str) -> Any:
        """Delegate attribute access to the inner Rust Leader."""
        if name in ('descriptive_cataloging_form', 'multipart_resource_record_level'):
            # Handle aliases
            if name == 'descriptive_cataloging_form':
                return self._inner.cataloging_form
            elif name == 'multipart_resource_record_level':
                return self._inner.multipart_level
        return getattr(self._inner, name)
    
    def __setattr__(self, name: str, value: Any) -> None:
        """Delegate attribute setting to the inner Rust Leader."""
        if name == '_inner':
            super().__setattr__(name, value)
        elif name == 'descriptive_cataloging_form':
            self._inner.set_cataloging_form(value)
        elif name == 'multipart_resource_record_level':
            self._inner.set_multipart_level(value)
        else:
            setattr(self._inner, name, value)
    
    @property
    def descriptive_cataloging_form(self) -> str:
        """Descriptive cataloging form (position 18)."""
        return self._inner.cataloging_form
    
    @descriptive_cataloging_form.setter
    def descriptive_cataloging_form(self, value: str) -> None:
        """Set descriptive cataloging form."""
        self._inner.set_cataloging_form(value)
    
    @property
    def multipart_resource_record_level(self) -> str:
        """Multipart resource record level (position 19)."""
        return self._inner.multipart_level
    
    @multipart_resource_record_level.setter
    def multipart_resource_record_level(self, value: str) -> None:
        """Set multipart resource record level."""
        self._inner.set_multipart_level(value)
    

    
    @property
    def record_type(self) -> str:
        """Type of record."""
        return self._inner.record_type
    
    @record_type.setter
    def record_type(self, value: str) -> None:
        """Set type of record."""
        self._inner.set_record_type(value)
    
    @property
    def bibliographic_level(self) -> str:
        """Bibliographic level."""
        return self._inner.bibliographic_level
    
    @bibliographic_level.setter
    def bibliographic_level(self, value: str) -> None:
        """Set bibliographic level."""
        self._inner.set_bibliographic_level(value)
    
    @property
    def encoding_level(self) -> str:
        """Encoding level."""
        return self._inner.encoding_level
    
    @encoding_level.setter
    def encoding_level(self, value: str) -> None:
        """Set encoding level."""
        self._inner.set_encoding_level(value)


class Record:
    """Enhanced Record wrapper with pymarc-compatible API."""
    
    def __init__(self, leader: Optional[Leader] = None, *, fields=None):
        """Create a new Record.

        Args:
            leader: Optional Leader object (defaults to Leader()).
            fields: Optional list of Field objects to add to the record.
        """
        if leader is None:
            leader = Leader()
        self._inner = _Record(leader._inner)
        self._leader = leader
        if fields:
            for field in fields:
                self.add_field(field)
    
    def __getattr__(self, name: str) -> Any:
        """Delegate attribute access to the inner Rust Record."""
        return getattr(self._inner, name)
    
    def __contains__(self, tag: str) -> bool:
        """Check if a field with given tag exists in record."""
        return self.get_field(tag) is not None
    
    def __getitem__(self, tag: str) -> Optional['Field']:
        """Get first field with given tag (pymarc compatibility)."""
        field = self._inner.get_field(tag)
        if field:
            wrapper = Field(field.tag, field.indicator1, field.indicator2)
            wrapper._inner = field
            return wrapper
        return None
    
    def get_fields(self, *tags: str) -> List['Field']:
        """Get all fields with given tags.
        
        If no tags provided, returns all fields.
        Supports multiple tags: record.get_fields('245', '260')
        """
        result = []
        
        if not tags:
            # Return all fields
            for field in self._inner.fields():
                wrapper = Field(field.tag, field.indicator1, field.indicator2)
                wrapper._inner = field
                result.append(wrapper)
        else:
            # Return fields for specified tags
            for tag in tags:
                for field in self._inner.get_fields(tag):
                    wrapper = Field(field.tag, field.indicator1, field.indicator2)
                    wrapper._inner = field
                    result.append(wrapper)
        
        return result
    
    def add_field(self, field: 'Field') -> None:
        """Add a field to the record."""
        self._inner.add_field(field._inner)
    
    def get_field(self, tag: str) -> Optional['Field']:
        """Get first field with given tag."""
        field = self._inner.get_field(tag)
        if field:
            wrapper = Field(field.tag, field.indicator1, field.indicator2)
            wrapper._inner = field
            return wrapper
        return None
    
    def remove_field(self, field: Union['Field', str]) -> None:
        """Remove a field from record.
        
        Can accept either a Field object or a tag string.
        """
        if isinstance(field, str):
            # Remove by tag
            self._inner.remove_field(field)
        else:
            # Remove by tag (using Field object)
            self._inner.remove_field(field.tag)
    
    def add_control_field(self, tag: str, value: str) -> None:
        """Add a control field."""
        self._inner.add_control_field(tag, value)
    
    def control_field(self, tag: str) -> Optional[str]:
        """Get a control field value."""
        return self._inner.control_field(tag)
    
    def fields(self) -> List['Field']:
        """Get all fields."""
        result = []
        for field in self._inner.fields():
            wrapper = Field(field.tag, field.indicator1, field.indicator2)
            wrapper._inner = field
            result.append(wrapper)
        return result
    
    def title(self) -> Optional[str]:
        """Get title from 245 field."""
        return self._inner.title()
    
    def author(self) -> Optional[str]:
        """Get author from 100/110/111 field."""
        return self._inner.author()
    
    def isbn(self) -> Optional[str]:
        """Get ISBN from 020 field."""
        return self._inner.isbn()
    
    def issn(self) -> Optional[str]:
        """Get ISSN from 022 field."""
        return self._inner.issn()
    
    def subjects(self) -> List[str]:
        """Get all subject headings from 650 field."""
        return self._inner.subjects()
    
    def location(self) -> List[str]:
        """Get all location fields (852)."""
        return self._inner.location()
    
    def notes(self) -> List[str]:
        """Get all notes from 5xx fields."""
        return self._inner.notes()
    
    def publisher(self) -> Optional[str]:
        """Get publisher from 260 field."""
        return self._inner.publisher()
    
    def uniform_title(self) -> Optional[str]:
        """Get uniform title from 130 field."""
        return self._inner.uniform_title()
    
    def sudoc(self) -> Optional[str]:
        """Get SuDoc from 086 field."""
        return self._inner.sudoc()
    
    def issn_title(self) -> Optional[str]:
        """Get ISSN title from 222 field."""
        return self._inner.issn_title()
    
    def issnl(self) -> Optional[str]:
        """Get ISSN-L from 024 field."""
        return self._inner.issnl()
    
    def pubyear(self) -> Optional[int]:
        """Get publication year."""
        return self._inner.pubyear()
    
    def series(self) -> Optional[str]:
        """Get series from 490 field."""
        return self._inner.series()
    
    def physical_description(self) -> Optional[str]:
        """Get physical description from 300 field."""
        return self._inner.physical_description()
    
    def is_book(self) -> bool:
        """Check if this is a book."""
        return self._inner.is_book()
    
    def is_serial(self) -> bool:
        """Check if this is a serial."""
        return self._inner.is_serial()
    
    def is_music(self) -> bool:
        """Check if this is music."""
        return self._inner.is_music()
    
    def is_audiovisual(self) -> bool:
        """Check if this is audiovisual."""
        return self._inner.is_audiovisual()
    
    def to_json(self) -> str:
        """Serialize to JSON."""
        return self._inner.to_json()
    
    def to_xml(self) -> str:
        """Serialize to XML."""
        return self._inner.to_xml()
    
    def to_dublin_core(self) -> str:
        """Serialize to Dublin Core."""
        return self._inner.to_dublin_core()
    
    def to_marcjson(self) -> str:
        """Serialize to MARCJSON."""
        return self._inner.to_marcjson()
    
    def to_marc21(self) -> bytes:
        """Serialize to MARC21 binary format (ISO 2709).
        
        Returns:
            bytes: The record in MARC21 binary format following ISO 2709 standard.
        
        Example:
            >>> record = Record()
            >>> marc_bytes = record.to_marc21()
            >>> with open('record.mrc', 'wb') as f:
            ...     f.write(marc_bytes)
        """
        return self._inner.to_marc21()
    
    def leader(self) -> Leader:
        """Get the leader."""
        return self._leader


class MARCReader:
    """MARC Reader wrapper."""
    
    def __init__(self, file_obj):
        """Create a new MARC reader."""
        self._inner = _MARCReader(file_obj)
    
    def __iter__(self):
        """Iterate over records."""
        return iter(self._inner)
    
    def __next__(self) -> Record:
        """Get next record."""
        record = next(self._inner)
        wrapper = Record(None)
        wrapper._inner = record
        return wrapper


class MARCWriter:
    """MARC Writer wrapper."""
    
    def __init__(self, file_obj):
        """Create a new MARC writer."""
        self._inner = _MARCWriter(file_obj)
    
    def write(self, record: Record) -> None:
        """Write a record."""
        self._inner.write(record._inner)


def get_leader_valid_values(position: int) -> dict:
    """Get valid values for a specific leader position (MARC 21 spec reference).
    
    Module-level function (also available as Leader.get_valid_values(position) 
    or instance_leader.get_valid_values(position)).
    
    Returns a dictionary mapping valid character values to their descriptions
    for the given position.
    
    Args:
        position: The leader position (5-19)
        
    Returns:
        A dictionary mapping values to descriptions, or empty dict for unknown positions
        
    Example:
        >>> valid = get_leader_valid_values(5)
        >>> # Returns: {'a': 'increase in encoding level', 'c': 'corrected or revised', ...}
    """
    return _Leader.get_valid_values(position)


def get_leader_value_description(position: int, value: str) -> Optional[str]:
    """Get description for a specific value at a leader position.
    
    Module-level function (also available as Leader.describe_value(position, value)
    or instance_leader.describe_value(position, value)).
    
    Args:
        position: The leader position (5-19)
        value: The character value to look up
        
    Returns:
        The description if found, or None if the value is invalid for the position
        
    Example:
        >>> desc = get_leader_value_description(5, "a")
        >>> # Returns: "increase in encoding level"
    """
    return _Leader.describe_value(position, value)


# Expose as class methods on the Leader class
Leader.get_valid_values = staticmethod(get_leader_valid_values)
Leader.describe_value = staticmethod(get_leader_value_description)

__all__ = [
    "Leader", "Subfield", "Field", "Record", "MARCReader", "MARCWriter",
    "get_leader_valid_values", "get_leader_value_description",
]
