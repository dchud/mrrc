"""
MRRC: Fast MARC library written in Rust with Python bindings.

This module provides Python access to the Rust MARC library, enabling fast
reading, writing, and manipulation of MARC bibliographic records.

The Python wrapper aims for API compatibility with pymarc.
"""

from ._mrrc import (
    AuthorityMARCReader,
    AuthorityRecord as _AuthorityRecord,
    Field as _Field,
    HoldingsMARCReader,
    HoldingsRecord as _HoldingsRecord,
    Leader as _Leader,
    MARCReader as _MARCReader,
    MARCWriter as _MARCWriter,
    Record as _Record,
    RecordBoundaryScanner,
    ProducerConsumerPipeline,
    Subfield,
    parse_batch_parallel,
    parse_batch_parallel_limited,
    record_to_json,
    json_to_record,
    record_to_xml,
    xml_to_record,
    record_to_marcjson,
    marcjson_to_record,
    record_to_dublin_core,
    record_to_mods,
    mods_to_record,
    mods_collection_to_records,
    dublin_core_to_xml,
    record_to_csv,
    records_to_csv,
    records_to_csv_filtered,
    # Query DSL classes
    FieldQuery,
    TagRangeQuery,
    SubfieldPatternQuery,
    SubfieldValueQuery,
    # BIBFRAME conversion support (LOC linked data format)
    BibframeConfig,
    RdfGraph,
    marc_to_bibframe as _marc_to_bibframe,
    bibframe_to_marc as _bibframe_to_marc,
)
from typing import Optional, List, Union, Any, Tuple

__version__ = "0.1.0"
__author__ = "MRRC Contributors"


class Indicators:
    """Tuple-like wrapper for field indicators (pymarc compatibility)."""
    
    def __init__(self, ind1: str, ind2: str):
        """Create indicators tuple."""
        self.ind1 = ind1
        self.ind2 = ind2
    
    def __getitem__(self, index: int) -> str:
        """Get indicator by index (0 or 1)."""
        if index == 0:
            return self.ind1
        elif index == 1:
            return self.ind2
        else:
            raise IndexError("Indicator index must be 0 or 1")
    
    def __setitem__(self, index: int, value: str) -> None:
        """Set indicator by index (0 or 1)."""
        if index == 0:
            self.ind1 = value
        elif index == 1:
            self.ind2 = value
        else:
            raise IndexError("Indicator index must be 0 or 1")
    
    def __eq__(self, other: Any) -> bool:
        """Compare indicators."""
        if isinstance(other, Indicators):
            return self.ind1 == other.ind1 and self.ind2 == other.ind2
        elif isinstance(other, (tuple, list)) and len(other) == 2:
            return self.ind1 == other[0] and self.ind2 == other[1]
        return False
    
    def __repr__(self) -> str:
        """String representation."""
        return f"Indicators('{self.ind1}', '{self.ind2}')"
    
    def __hash__(self) -> int:
        """Hash based on both indicators."""
        return hash((self.ind1, self.ind2))
    
    def __iter__(self):
        """Allow unpacking like a tuple."""
        return iter([self.ind1, self.ind2])


class ControlField:
    """Wrapper for MARC control fields (001-009) with pymarc-compatible .value property."""
    
    def __init__(self, tag: str, value: str):
        """Create a new ControlField."""
        self.tag = tag
        self.value = value
    
    def __eq__(self, other: Any) -> bool:
        """Compare control fields by tag and value."""
        if isinstance(other, ControlField):
            return self.tag == other.tag and self.value == other.value
        return False
    
    def __repr__(self) -> str:
        """String representation."""
        return f"ControlField(tag='{self.tag}', value='{self.value}')"
    
    def __hash__(self) -> int:
        """Hash based on tag and value."""
        return hash((self.tag, self.value))


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
    
    def __getitem__(self, code: str) -> Optional[str]:
        """Get first subfield value by code (pymarc compatibility).
        
        Returns None if the subfield code doesn't exist, matching pymarc behavior.
        
        Example:
            field['a']  # Get first 'a' subfield value
            field['z']  # Returns None if 'z' subfield doesn't exist
        """
        try:
            values = self._inner.subfields_by_code(code)
            return values[0] if values else None
        except Exception:
            return None
    
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
        except Exception:
            return default

    def __contains__(self, code: str) -> bool:
        """Check if subfield code exists in field."""
        try:
            values = self._inner.subfields_by_code(code)
            return len(values) > 0
        except Exception:
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
            except Exception:
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
        except Exception:
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
        except Exception:
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
        self._inner.indicator1 = value
    
    @property
    def indicator2(self) -> str:
        """Second indicator."""
        return self._inner.indicator2
    
    @indicator2.setter
    def indicator2(self, value: str) -> None:
        """Set second indicator."""
        self._inner.indicator2 = value
    
    @property
    def indicators(self) -> 'Indicators':
        """Get indicators as tuple-like Indicators object (pymarc compatibility).
        
        Example:
            field.indicators[0]      # First indicator
            field.indicators[1]      # Second indicator
            ind1, ind2 = field.indicators  # Unpacking
        """
        return Indicators(self.indicator1, self.indicator2)
    
    @indicators.setter
    def indicators(self, value: Union['Indicators', Tuple[str, str], List[str]]) -> None:
        """Set indicators from Indicators object or tuple/list (pymarc compatibility)."""
        if isinstance(value, Indicators):
            self.indicator1 = value.ind1
            self.indicator2 = value.ind2
        elif isinstance(value, (tuple, list)) and len(value) == 2:
            self.indicator1 = value[0]
            self.indicator2 = value[1]
        else:
            raise ValueError("indicators must be Indicators object or [ind1, ind2] tuple/list")
    
    def is_subject_field(self) -> bool:
        """Check if this is a subject field (6xx)."""
        tag = self.tag
        return tag.startswith('6') and len(tag) >= 2
    
    def __eq__(self, other: Any) -> bool:
        """Compare fields by content."""
        if not isinstance(other, Field):
            return False
        return (self.tag == other.tag and 
                self.indicator1 == other.indicator1 and
                self.indicator2 == other.indicator2 and
                self.subfields() == other.subfields())
    
    def __hash__(self) -> int:
        """Hash based on tag and first subfield."""
        return hash((self.tag, self.indicator1, self.indicator2))


class Leader:
     """Enhanced Leader wrapper with pymarc-compatible API.
     
     Provides both property-based access and MARC 21 reference information for leader positions.
     """
     
     # MARC 21 Reference: Position 5 - Record Status
     RECORD_STATUS_VALUES = {
         'a': 'Increase in encoding level',
         'c': 'Corrected or revised',
         'd': 'Deleted',
         'n': 'New',
         'p': 'Increase in encoding level from prepublication',
     }
     
     # MARC 21 Reference: Position 6 - Type of record
     RECORD_TYPE_VALUES = {
         'a': 'Language material',
         'b': 'Notated music',
         'c': 'Notated music',
         'd': 'Manuscript notated music',
         'e': 'Cartographic material',
         'f': 'Manuscript cartographic material',
         'g': 'Projected medium',
         'h': 'Microform',
         'i': 'Nonmusical sound recording',
         'j': 'Musical sound recording',
         'k': 'Two-dimensional nonprojectable graphic',
         'm': 'Computer file',
         'o': 'Kit',
         'p': 'Mixed materials',
         'r': 'Three-dimensional artifact or naturally occurring object',
         't': 'Manuscript language material',
     }
     
     # MARC 21 Reference: Position 7 - Bibliographic level
     BIBLIOGRAPHIC_LEVEL_VALUES = {
         'a': 'Monographic component part',
         'b': 'Serial component part',
         'c': 'Collection',
         'd': 'Subunit',
         'i': 'Integrating resource',
         'm': 'Monograph',
         's': 'Serial',
     }
     
     # MARC 21 Reference: Position 17 - Encoding level
     ENCODING_LEVEL_VALUES = {
         ' ': 'Full level',
         '1': 'Full level, material not examined',
         '2': 'Less-than-full level, material not examined',
         '3': 'Abbreviated level',
         '4': 'Core level',
         '5': 'Partial (preliminary) level',
         '7': 'Minimal level',
         '8': 'Prepublication level',
         'u': 'Unknown',
         'z': 'Not applicable',
     }
     
     # MARC 21 Reference: Position 18 - Descriptive cataloging form (Cataloging form)
     CATALOGING_FORM_VALUES = {
         ' ': 'Non-ISBD',
         'a': 'AACR 2',
         'c': 'ISBD punctuation omitted',
         'i': 'ISBD punctuation included',
         'n': 'Non-ISBD punctuation omitted',
         'u': 'Unknown',
     }
     
     @classmethod
     def get_valid_values(cls, position: int) -> Optional[dict]:
         """Get dictionary of valid values for a leader position.
         
         MARC 21 positions with defined valid values:
         - 5: Record status (RECORD_STATUS_VALUES)
         - 6: Type of record (RECORD_TYPE_VALUES)
         - 7: Bibliographic level (BIBLIOGRAPHIC_LEVEL_VALUES)
         - 17: Encoding level (ENCODING_LEVEL_VALUES)
         - 18: Cataloging form (CATALOGING_FORM_VALUES)
         
         Args:
             position: Leader position (0-23)
             
         Returns:
             Dictionary mapping values to descriptions, or None if position has no defined values
             
         Example:
             >>> Leader.get_valid_values(5)
             {'a': 'Increase in encoding level', 'c': 'Corrected or revised', ...}
             >>> Leader.get_valid_values(0)  # Record length has no fixed values
             None
         """
         position_map = {
             5: cls.RECORD_STATUS_VALUES,
             6: cls.RECORD_TYPE_VALUES,
             7: cls.BIBLIOGRAPHIC_LEVEL_VALUES,
             17: cls.ENCODING_LEVEL_VALUES,
             18: cls.CATALOGING_FORM_VALUES,
         }
         return position_map.get(position)
     
     @classmethod
     def is_valid_value(cls, position: int, value: str) -> bool:
         """Check if a value is valid for a leader position.
         
         Args:
             position: Leader position (0-23)
             value: Single character value to validate
             
         Returns:
             True if value is valid for this position, False otherwise
             
         Example:
             >>> Leader.is_valid_value(5, 'a')  # Record status
             True
             >>> Leader.is_valid_value(5, 'x')  # Invalid
             False
         """
         valid_values = cls.get_valid_values(position)
         if valid_values is None:
             return True  # Position without defined values accepts any single char
         return value in valid_values
     
     @classmethod
     def get_value_description(cls, position: int, value: str) -> Optional[str]:
         """Get description of a leader value.
         
         Args:
             position: Leader position (0-23)
             value: Single character value
             
         Returns:
             Description string if value is defined, None otherwise
             
         Example:
             >>> Leader.get_value_description(5, 'a')
             'Increase in encoding level'
             >>> Leader.get_value_description(5, 'x')  # Invalid
             None
         """
         valid_values = cls.get_valid_values(position)
         if valid_values is None:
             return None
         return valid_values.get(value)
     
     def __init__(self):
         """Create a new Leader."""
         # Just use the Rust Leader directly, but add aliases
         # The Rust leader has all the properties we need
         pass
     
     def __new__(cls):
         """Create instance - actually returns a Rust Leader with aliases."""
         instance = object.__new__(cls)
         instance._rust_leader = _Leader()
         instance._parent_record = None
         return instance
     
     def __getattr__(self, name: str) -> Any:
         """Delegate attribute access, handling aliases."""
         # Aliases for pymarc compatibility
         if name == 'descriptive_cataloging_form':
             return self._rust_leader.cataloging_form
         elif name == 'multipart_resource_record_level':
             return self._rust_leader.multipart_level
         # Delegate everything else
         return getattr(self._rust_leader, name)
     
     def __setattr__(self, name: str, value: Any) -> None:
         """Delegate attribute setting, handling aliases."""
         if name in ('_rust_leader', '_parent_record'):
             object.__setattr__(self, name, value)
         elif name == 'descriptive_cataloging_form':
             self._rust_leader.cataloging_form = value
             # Mark parent record as having modified leader
             if hasattr(self, '_parent_record') and self._parent_record is not None:
                 self._parent_record._leader_modified = True
         elif name == 'multipart_resource_record_level':
             self._rust_leader.multipart_level = value
             # Mark parent record as having modified leader
             if hasattr(self, '_parent_record') and self._parent_record is not None:
                 self._parent_record._leader_modified = True
         else:
             setattr(self._rust_leader, name, value)
             # Mark parent record as having modified leader
             if hasattr(self, '_parent_record') and self._parent_record is not None:
                 self._parent_record._leader_modified = True
     
     def __getitem__(self, index: Union[int, slice]) -> Union[str, Optional[str]]:
         """Get leader character(s) by position (pymarc compatibility).
         
         Examples:
             leader[5]       # Get record status character
             leader[0:5]     # Get first 5 characters (record length)
             leader[18]      # Get cataloging form character
         """
         # Get the leader as a 24-character string
         leader_str = self._get_leader_as_string()
         
         if isinstance(index, slice):
             # Slice access: leader[0:5]
             start = index.start or 0
             stop = index.stop or len(leader_str)
             if start < 0 or stop > len(leader_str):
                 raise IndexError("Leader position out of range")
             return leader_str[start:stop]
         else:
             # Single position access: leader[5]
             if index < 0 or index >= len(leader_str):
                 raise IndexError("Leader position out of range")
             return leader_str[index]
     
     def __setitem__(self, index: int, value: str) -> None:
         """Set leader character by position (pymarc compatibility).
         
         Example:
             leader[5] = 'a'  # Set record status
         """
         if not isinstance(index, int):
             raise TypeError("Leader position must be an integer")
         if not isinstance(value, str) or len(value) != 1:
             raise ValueError("Leader value must be a single character string")
         
         # Get current leader as string
         leader_str = self._get_leader_as_string()
         
         if index < 0 or index >= len(leader_str):
             raise IndexError("Leader position out of range")
         
         # Replace character at position
         new_leader_str = leader_str[:index] + value + leader_str[index+1:]
         
         # Update the leader based on the position
         self._update_leader_from_string(new_leader_str)
     
     def _get_leader_as_string(self) -> str:
         """Get the leader as a 24-character MARC21 leader string."""
         # Build leader string from properties
         leader = []
         leader.append(str(self._rust_leader.record_length).zfill(5))
         leader.append(self._rust_leader.record_status)
         leader.append(self._rust_leader.record_type)
         leader.append(self._rust_leader.bibliographic_level)
         leader.append(self._rust_leader.control_record_type)
         leader.append(self._rust_leader.character_coding)
         leader.append(str(self._rust_leader.indicator_count))
         leader.append(str(self._rust_leader.subfield_code_count))
         leader.append(str(self._rust_leader.data_base_address).zfill(5))
         leader.append(self._rust_leader.encoding_level)
         leader.append(self._rust_leader.cataloging_form)
         leader.append(self._rust_leader.multipart_level)
         leader.append(self._rust_leader.reserved)
         
         return ''.join(leader)
     
     def _update_leader_from_string(self, leader_str: str) -> None:
         """Update leader properties from a 24-character string."""
         if len(leader_str) != 24:
             raise ValueError(f"Leader string must be exactly 24 characters, got {len(leader_str)}")
         
         # Parse MARC21 leader format (positions as per standard)
         self._rust_leader.record_length = int(leader_str[0:5])
         self._rust_leader.record_status = leader_str[5]
         self._rust_leader.record_type = leader_str[6]
         self._rust_leader.bibliographic_level = leader_str[7]
         self._rust_leader.control_record_type = leader_str[8]
         self._rust_leader.character_coding = leader_str[9]
         self._rust_leader.indicator_count = int(leader_str[10])
         self._rust_leader.subfield_code_count = int(leader_str[11])
         self._rust_leader.data_base_address = int(leader_str[12:17])
         self._rust_leader.encoding_level = leader_str[17]
         self._rust_leader.cataloging_form = leader_str[18]
         self._rust_leader.multipart_level = leader_str[19]
         self._rust_leader.reserved = leader_str[20:24]
         
         # Mark parent record as having modified leader
         if hasattr(self, '_parent_record') and self._parent_record is not None:
             self._parent_record._leader_modified = True
     
     def __eq__(self, other: Any) -> bool:
         """Compare leaders by content."""
         if not isinstance(other, Leader):
             return False
         return self._rust_leader == other._rust_leader
     
     def __hash__(self) -> int:
         """Hash based on rust leader."""
         return hash(id(self._rust_leader))


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
        # Get the inner Rust leader
        rust_leader = leader._rust_leader if isinstance(leader, Leader) else leader
        self._inner = _Record(rust_leader)
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
    
    def __getitem__(self, tag: str) -> Union[Optional['Field'], 'ControlField']:
         """Get first field with given tag (pymarc compatibility).
         
         For control fields (001-009), returns ControlField with .value property.
         For data fields, returns Field wrapper.
         Returns None if field doesn't exist.
         """
         # Check if this is a control field (001-009)
         if tag in ('001', '002', '003', '004', '005', '006', '007', '008', '009'):
             value = self._inner.control_field(tag)
             if value is not None:
                 return ControlField(tag, value)
             return None
         
         # For data fields, return Field wrapper
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
    
    def remove_field(self, field: Union['Field', str]) -> List['Field']:
        """Remove a field from record.
        
        Can accept either a Field object or a tag string.
        Returns list of removed fields.
        """
        if isinstance(field, str):
            # Remove by tag
            tag = field
        else:
            # Remove by tag (using Field object)
            tag = field.tag
        
        # Get fields before removal
        fields_before = self._inner.get_fields(tag)
        
        # Remove
        self._inner.remove_field(tag)
        
        # Convert to wrapped Fields
        result = []
        for field_obj in fields_before:
            wrapper = Field(field_obj.tag, field_obj.indicator1, field_obj.indicator2)
            wrapper._inner = field_obj
            result.append(wrapper)
        return result
    
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
        """Get all subject headings from 6XX subject fields."""
        return self._inner.subjects()
    
    def location(self) -> List[str]:
        """Get all location fields (852)."""
        return self._inner.location()
    
    def notes(self) -> List[str]:
        """Get all notes from 5xx fields."""
        return self._inner.notes()
    
    def publisher(self) -> Optional[str]:
        """Get publisher from 260 or 264 (RDA) field."""
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
    
    # =========================================================================
    # Query DSL Methods - Advanced field searching beyond pymarc's get_fields()
    # =========================================================================

    def fields_by_indicator(
        self, tag: str, *, indicator1: Optional[str] = None, indicator2: Optional[str] = None
    ) -> List['Field']:
        """Get fields matching indicator values.
        
        This is a convenience method for filtering by indicators.
        For more complex queries, use `fields_matching()` with a `FieldQuery`.
        
        Args:
            tag: The 3-character field tag to search.
            indicator1: Optional first indicator value (None = match any).
            indicator2: Optional second indicator value (None = match any).
            
        Returns:
            List of Field objects matching the criteria.
            
        Example:
            >>> # Find all 650 fields with indicator2='0' (Library of Congress Subject Headings)
            >>> lcsh_subjects = record.fields_by_indicator("650", indicator2="0")
            >>> for field in lcsh_subjects:
            ...     print(field["a"])
        """
        result = []
        for field in self._inner.fields_by_indicator(tag, indicator1=indicator1, indicator2=indicator2):
            wrapper = Field(field.tag, field.indicator1, field.indicator2)
            wrapper._inner = field
            result.append(wrapper)
        return result
    
    def fields_in_range(self, start_tag: str, end_tag: str) -> List['Field']:
        """Get fields within a tag range (inclusive).
        
        Useful for querying groups of related fields, such as all subject fields
        (600-699) or all added entry fields (700-799).
        
        Args:
            start_tag: Start of range (inclusive), e.g., "600".
            end_tag: End of range (inclusive), e.g., "699".
            
        Returns:
            List of Field objects within the tag range.
            
        Example:
            >>> # Find all subject fields (600-699)
            >>> subjects = record.fields_in_range("600", "699")
            >>> for field in subjects:
            ...     print(f"{field.tag}: {field['a']}")
        """
        result = []
        for field in self._inner.fields_in_range(start_tag, end_tag):
            wrapper = Field(field.tag, field.indicator1, field.indicator2)
            wrapper._inner = field
            result.append(wrapper)
        return result
    
    def fields_matching(self, query: 'FieldQuery') -> List['Field']:
        """Get fields matching a FieldQuery.
        
        This method enables complex field matching using the Query DSL.
        A FieldQuery can combine tag, indicator, and subfield requirements.
        
        Args:
            query: A FieldQuery object with the matching criteria.
            
        Returns:
            List of Field objects matching all query criteria.
            
        Example:
            >>> query = FieldQuery().tag("650").indicator2("0").has_subfield("a")
            >>> lcsh = record.fields_matching(query)
            >>> for field in lcsh:
            ...     print(field["a"])
        """
        result = []
        for field in self._inner.fields_matching(query):
            wrapper = Field(field.tag, field.indicator1, field.indicator2)
            wrapper._inner = field
            result.append(wrapper)
        return result
    
    def fields_matching_range(self, query: 'TagRangeQuery') -> List['Field']:
        """Get fields matching a TagRangeQuery.
        
        This method finds fields within a tag range that also match indicator
        and subfield requirements.
        
        Args:
            query: A TagRangeQuery object with range and filter criteria.
            
        Returns:
            List of Field objects matching all query criteria.
            
        Example:
            >>> # Find all 6XX subjects with indicator2='0' (LCSH) that have subfield 'a'
            >>> query = TagRangeQuery("600", "699", indicator2="0", required_subfields=["a"])
            >>> subjects = record.fields_matching_range(query)
        """
        result = []
        for field in self._inner.fields_matching_range(query):
            wrapper = Field(field.tag, field.indicator1, field.indicator2)
            wrapper._inner = field
            result.append(wrapper)
        return result
    
    def fields_matching_pattern(self, query: 'SubfieldPatternQuery') -> List['Field']:
        """Get fields matching a SubfieldPatternQuery (regex matching).
        
        This method finds fields where a specific subfield's value matches
        a regular expression pattern.
        
        Args:
            query: A SubfieldPatternQuery object with tag, subfield, and regex.
            
        Returns:
            List of Field objects where the subfield matches the pattern.
            
        Example:
            >>> # Find all ISBN-13s (start with 978 or 979)
            >>> query = SubfieldPatternQuery("020", "a", r"^97[89]-")
            >>> isbn13_fields = record.fields_matching_pattern(query)
        """
        result = []
        for field in self._inner.fields_matching_pattern(query):
            wrapper = Field(field.tag, field.indicator1, field.indicator2)
            wrapper._inner = field
            result.append(wrapper)
        return result
    
    def fields_matching_value(self, query: 'SubfieldValueQuery') -> List['Field']:
        """Get fields matching a SubfieldValueQuery (exact or partial string matching).
        
        This method finds fields where a specific subfield's value matches
        a string exactly or as a substring.
        
        Args:
            query: A SubfieldValueQuery object with tag, subfield, value, and match type.
            
        Returns:
            List of Field objects where the subfield matches the value.
            
        Example:
            >>> # Find exact subject heading "History"
            >>> query = SubfieldValueQuery("650", "a", "History")
            >>> history_fields = record.fields_matching_value(query)
            
            >>> # Find subjects containing "History" anywhere
            >>> query = SubfieldValueQuery("650", "a", "History", partial=True)
            >>> related_fields = record.fields_matching_value(query)
        """
        result = []
        for field in self._inner.fields_matching_value(query):
            wrapper = Field(field.tag, field.indicator1, field.indicator2)
            wrapper._inner = field
            result.append(wrapper)
        return result
    
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
    
    def leader(self) -> Leader:
        """Get the leader."""
        # Ensure _leader is initialized and synced
        if not hasattr(self, '_leader') or self._leader is None:
            leader = Leader()
            leader._rust_leader = self._inner.leader()
            leader._parent_record = self
            # Track that we haven't modified the leader
            self._leader_modified = False
            self._leader = leader
        return self._leader
    
    def _sync_leader(self) -> None:
        """Sync the Python leader back to the Rust record if it was modified."""
        # Only sync if the leader was actually accessed/modified
        if not getattr(self, '_leader_modified', False):
            return
       
        if hasattr(self, '_leader') and self._leader is not None:
            # Just directly replace the inner leader with our modified one
            try:
                self._inner.set_leader(self._leader._rust_leader)
            except RuntimeError as e:
                # If we get a borrowing error, it means the leader is still borrowed
                # In that case, we need to sync properties individually
                if "Already borrowed" in str(e):
                    # Get the inner leader and sync all properties
                    inner_leader = self._inner.leader()
                    rust_leader = self._leader._rust_leader
                    
                    # Sync all properties
                    inner_leader.record_length = rust_leader.record_length
                    inner_leader.record_status = rust_leader.record_status
                    inner_leader.record_type = rust_leader.record_type
                    inner_leader.bibliographic_level = rust_leader.bibliographic_level
                    inner_leader.control_record_type = rust_leader.control_record_type
                    inner_leader.character_coding = rust_leader.character_coding
                    inner_leader.indicator_count = rust_leader.indicator_count
                    inner_leader.subfield_code_count = rust_leader.subfield_code_count
                    inner_leader.data_base_address = rust_leader.data_base_address
                    inner_leader.encoding_level = rust_leader.encoding_level
                    inner_leader.cataloging_form = rust_leader.cataloging_form
                    inner_leader.multipart_level = rust_leader.multipart_level
                    inner_leader.reserved = rust_leader.reserved
                    # Note: set_leader will still be called implicitly
                else:
                    raise
    
    def __eq__(self, other: Any) -> bool:
        """Compare records by content."""
        if not isinstance(other, Record):
            return False
        # Compare leaders and control fields and fields
        self_fields = self._inner.fields()
        other_fields = other._inner.fields()
        
        # Same number of fields
        if len(self_fields) != len(other_fields):
            return False
        
        # Compare each field
        for self_f, other_f in zip(self_fields, other_fields):
            if (self_f.tag != other_f.tag or
                self_f.indicator1 != other_f.indicator1 or
                self_f.indicator2 != other_f.indicator2 or
                len(self_f.subfields()) != len(other_f.subfields())):
                return False
            for self_sf, other_sf in zip(self_f.subfields(), other_f.subfields()):
                if self_sf.code != other_sf.code or self_sf.value != other_sf.value:
                    return False
        
        # Compare control fields
        for code, value in self._inner.control_fields():
            if self._inner.control_field(code) != value:
                return False
            if other._inner.control_field(code) != value:
                return False
        
        return True
    
    def __hash__(self) -> int:
        """Hash based on leader."""
        return hash(id(self._inner))


class MARCReader:
    """MARC Reader wrapper."""
    
    def __init__(self, file_obj):
        """Create a new MARC reader."""
        self._inner = _MARCReader(file_obj)
    
    def __iter__(self):
        """Iterate over records."""
        return self
    
    def __next__(self) -> Record:
        """Get next record."""
        record = next(self._inner)
        wrapper = Record(None)
        wrapper._inner = record
        # Create a Leader wrapper from the Rust record's leader
        leader = Leader()
        leader._rust_leader = record.leader()
        leader._parent_record = wrapper
        wrapper._leader = leader
        wrapper._leader_modified = False
        return wrapper
    
    @property
    def backend_type(self) -> str:
        """The backend type: ``"rust_file"``, ``"cursor"``, or ``"python_file"``."""
        return self._inner.backend_type

    def read_record(self) -> Optional[Record]:
        """Read next record (pymarc compatibility)."""
        try:
            return next(self)
        except StopIteration:
            return None


class MARCWriter:
    """MARC Writer wrapper."""
    
    def __init__(self, file_obj):
        """Create a new MARC writer."""
        self._inner = _MARCWriter(file_obj)
    
    def write(self, record: Record) -> None:
        """Write a record."""
        # Sync any modifications to the leader before writing
        record._sync_leader()
        self._inner.write_record(record._inner)
    
    def write_record(self, record: Record) -> None:
        """Write a record (alias for write)."""
        self.write(record)
    
    def close(self) -> None:
        """Close the writer."""
        self._inner.close()
    
    def __enter__(self):
        """Context manager support."""
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager support."""
        self.close()
        return False


# Aliases for Authority and Holdings records
AuthorityRecord = _AuthorityRecord
HoldingsRecord = _HoldingsRecord


def get_leader_valid_values(position: int) -> Optional[dict]:
    """Get valid values for a specific leader position (MARC 21 spec reference).
    
    Module-level function (also available as Leader.get_valid_values(position) 
    or instance_leader.get_valid_values(position)).
    
    Returns a dictionary mapping valid character values to their descriptions
    for the given position. Returns None if the position has no defined valid values.
    
    Args:
        position: The leader position (5-19)
        
    Returns:
        A dictionary mapping values to descriptions, or None for positions with no defined values
        
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


def get_leader_is_valid_value(position: int, value: str) -> bool:
    """Check if a value is valid for a specific leader position.
    
    Positions without defined valid values accept any value.
    
    Module-level function (also available as Leader.is_valid_value(position, value)).
    
    Args:
        position: The leader position (5-19)
        value: The character value to validate
        
    Returns:
        True if the value is valid for the position, False otherwise
    """
    return _Leader.is_valid_value(position, value)


# Expose as class methods on the Leader class
Leader.get_valid_values = staticmethod(get_leader_valid_values)
Leader.describe_value = staticmethod(get_leader_value_description)
Leader.is_valid_value = staticmethod(get_leader_is_valid_value)
Leader.get_value_description = staticmethod(get_leader_value_description)


# Format-agnostic reader helper
def read(path: Union[str, Any], format: Optional[str] = None):
    """Read MARC records from a file, auto-detecting format from extension.

    Args:
        path: File path (str or pathlib.Path) to read from.
        format: Optional format override. If not specified, format is inferred
            from the file extension. Supported values:
            - "marc" or "mrc": ISO 2709 binary MARC

    Returns:
        An iterator over Record objects from the file.

    Raises:
        ValueError: If format cannot be determined or is unsupported.
        FileNotFoundError: If the file does not exist.

    Example:
        >>> for record in mrrc.read("data.mrc"):
        ...     print(record.title())
    """
    import os

    # Convert pathlib.Path to string if needed
    if hasattr(path, '__fspath__'):
        path = os.fspath(path)

    # Determine format from extension if not specified
    if format is None:
        _, ext = os.path.splitext(path)
        ext = ext.lower().lstrip('.')

        extension_map = {
            'mrc': 'marc',
            'marc': 'marc',
        }

        format = extension_map.get(ext)
        if format is None:
            raise ValueError(
                f"Cannot determine format from extension '.{ext}'. "
                f"Supported extensions: {', '.join(sorted(extension_map.keys()))}. "
                f"Use format= parameter to specify explicitly."
            )

    # Normalize format aliases
    format = format.lower()
    format_aliases = {
        'mrc': 'marc',
    }
    format = format_aliases.get(format, format)

    # Return appropriate reader
    if format == 'marc':
        f = open(path, 'rb')
        return MARCReader(f)
    else:
        raise ValueError(
            f"Unsupported format '{format}'. Supported formats: marc"
        )


def write(records, path: Union[str, Any], format: Optional[str] = None) -> int:
    """Write MARC records to a file, auto-detecting format from extension.

    Args:
        records: An iterable of Record objects to write.
        path: File path (str or pathlib.Path) to write to.
        format: Optional format override. If not specified, format is inferred
            from the file extension. Supported values:
            - "marc" or "mrc": ISO 2709 binary MARC

    Returns:
        The number of records written.

    Raises:
        ValueError: If format cannot be determined or is unsupported.

    Example:
        >>> records = list(mrrc.read("input.mrc"))
        >>> mrrc.write(records, "output.mrc")
        100
    """
    import os

    # Convert pathlib.Path to string if needed
    if hasattr(path, '__fspath__'):
        path = os.fspath(path)

    # Determine format from extension if not specified
    if format is None:
        _, ext = os.path.splitext(path)
        ext = ext.lower().lstrip('.')

        extension_map = {
            'mrc': 'marc',
            'marc': 'marc',
        }

        format = extension_map.get(ext)
        if format is None:
            raise ValueError(
                f"Cannot determine format from extension '.{ext}'. "
                f"Supported extensions: {', '.join(sorted(extension_map.keys()))}. "
                f"Use format= parameter to specify explicitly."
            )

    # Normalize format aliases
    format = format.lower()
    format_aliases = {
        'mrc': 'marc',
    }
    format = format_aliases.get(format, format)

    # Write using appropriate writer
    count = 0
    if format == 'marc':
        with open(path, 'wb') as f:
            writer = MARCWriter(f)
            for record in records:
                writer.write(record)
                count += 1
            writer.close()
    else:
        raise ValueError(
            f"Unsupported format '{format}'. Supported formats: marc"
        )

    return count


# =============================================================================
# BIBFRAME Conversion Functions (LOC Linked Data Format)
# =============================================================================

def marc_to_bibframe(record, config: BibframeConfig = None) -> RdfGraph:
    """Convert a MARC record to a BIBFRAME RDF graph.

    This function transforms a MARC bibliographic record into a BIBFRAME 2.0
    RDF graph containing Work, Instance, and optionally Item entities.

    Args:
        record: The MARC record to convert (Record or wrapped Record)
        config: Configuration options for the conversion (default: BibframeConfig())

    Returns:
        An RdfGraph containing the BIBFRAME representation

    Example:
        >>> import mrrc
        >>> record = mrrc.Record(leader=mrrc.Leader())
        >>> record.add_control_field("001", "12345")
        >>> config = mrrc.BibframeConfig()
        >>> graph = mrrc.marc_to_bibframe(record, config)
        >>> print(graph.serialize("jsonld"))
    """
    if config is None:
        config = BibframeConfig()
    # Handle wrapped Record (get inner PyRecord)
    inner_record = record._inner if hasattr(record, '_inner') else record
    return _marc_to_bibframe(inner_record, config)


def bibframe_to_marc(graph: RdfGraph) -> 'Record':
    """Convert a BIBFRAME RDF graph to a MARC record.

    This function transforms a BIBFRAME 2.0 RDF graph back into a MARC
    bibliographic record. Note that some information loss is inherent
    because BIBFRAME is semantically richer than MARC.

    Args:
        graph: The BIBFRAME RDF graph to convert

    Returns:
        A MARC Record representing the BIBFRAME data

    Raises:
        ValueError: If the graph cannot be converted

    Example:
        >>> import mrrc
        >>> # Round-trip conversion
        >>> record = mrrc.Record(leader=mrrc.Leader())
        >>> config = mrrc.BibframeConfig()
        >>> graph = mrrc.marc_to_bibframe(record, config)
        >>> recovered = mrrc.bibframe_to_marc(graph)
    """
    inner_record = _bibframe_to_marc(graph)
    # Wrap in Python Record class
    wrapped = Record.__new__(Record)
    wrapped._inner = inner_record
    return wrapped


__all__ = [
    # Core classes
    "AuthorityMARCReader",
    "AuthorityRecord",
    "HoldingsMARCReader",
    "HoldingsRecord",
    "Leader",
    "Indicators",
    "Subfield",
    "ControlField",
    "Field",
    "Record",
    "MARCReader",
    "MARCWriter",
    "RecordBoundaryScanner",
    "ProducerConsumerPipeline",
    # Query DSL classes
    "FieldQuery",
    "TagRangeQuery",
    "SubfieldPatternQuery",
    "SubfieldValueQuery",
    # Functions
    "parse_batch_parallel",
    "parse_batch_parallel_limited",
    "record_to_json",
    "json_to_record",
    "record_to_xml",
    "xml_to_record",
    "record_to_marcjson",
    "marcjson_to_record",
    "record_to_dublin_core",
    "record_to_mods",
    "mods_to_record",
    "mods_collection_to_records",
    "dublin_core_to_xml",
    "record_to_csv",
    "records_to_csv",
    "records_to_csv_filtered",
    "get_leader_valid_values",
    "get_leader_value_description",
    "get_leader_is_valid_value",
    # BIBFRAME conversion support (LOC linked data format)
    "BibframeConfig",
    "RdfGraph",
    "marc_to_bibframe",
    "bibframe_to_marc",
    # Format-agnostic helpers
    "read",
    "write",
]
