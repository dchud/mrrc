"""
MRRC: Fast MARC library written in Rust with Python bindings.

This module provides Python access to the Rust MARC library, enabling fast
reading, writing, and manipulation of MARC bibliographic records.

The Python wrapper aims for API compatibility with pymarc.
"""

from . import _mrrc
from ._mrrc import (
    AuthorityMARCReader,
    AuthorityRecord,
    Field as _Field,
    HoldingsMARCReader,
    HoldingsRecord,
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
    json_to_record as _json_to_record,
    record_to_xml,
    xml_to_record as _xml_to_record,
    xml_to_records as _xml_to_records,
    record_to_marcjson,
    marcjson_to_record as _marcjson_to_record,
    record_to_dublin_core,
    record_to_dublin_core_xml,
    record_to_mods,
    mods_to_record as _mods_to_record,
    mods_collection_to_records as _mods_collection_to_records,
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

# Exception hierarchy — re-exported from mrrc.exceptions so the public API
# (`mrrc.InvalidIndicator`, `mrrc.RecordLeaderInvalid`, etc.) is unchanged.
# See mrrc/exceptions.py for the full hierarchy and per-class behavior.
from .exceptions import (  # noqa: F401, E402
    BadSubfieldCode,
    BadSubfieldCodeWarning,
    BaseAddressInvalid,
    BaseAddressNotFound,
    EncodingError,
    EndOfRecordNotFound,
    FatalReaderError,
    FieldNotFound,
    InvalidField,
    InvalidIndicator,
    JsonError,
    MrrcException,
    RecordDirectoryInvalid,
    RecordLeaderInvalid,
    RecordLengthInvalid,
    TruncatedRecord,
    WriterError,
    XmlError,
)

__version__ = _mrrc.__version__
__author__ = "MRRC Contributors"


def _is_control_tag(tag: str) -> bool:
    """Check if a tag is a control field tag (001-009).

    Matches pymarc's logic: tag < '010' and tag.isdigit().
    """
    return tag < '010' and tag.isdigit()


class StaleFieldError(MrrcException):
    """A field handle was invalidated by record modification.

    Fields obtained from a record are live handles: reads and writes go
    through to the record. Removing fields from the record invalidates
    all outstanding handles; re-fetch the field from the record (e.g.,
    ``record[tag]`` or ``record.get_fields(tag)``) and retry.
    """


def _wrap_field(rust_field, parent: 'Optional[Record]' = None, occurrence: int = 0) -> 'Field':
    """Wrap a Rust _Field in a Python Field wrapper.

    With ``parent``, the wrapper is a live handle: ``occurrence`` is the
    zero-based index among same-tag fields, and reads/writes go through
    to the parent record.
    """
    wrapper = Field.__new__(Field)
    wrapper._data = None
    wrapper._inner = rust_field
    wrapper._parent = parent
    wrapper._occurrence = occurrence
    wrapper._generation = parent._inner.generation if parent is not None else 0
    return wrapper


def _wrap_control_field(parent: 'Record', tag: str, occurrence: int, value: str) -> 'Field':
    """Create a live control-field handle bound to ``parent``."""
    wrapper = Field.__new__(Field)
    wrapper._data = value
    wrapper._inner = _Field(tag, ' ', ' ')
    wrapper._parent = parent
    wrapper._occurrence = occurrence
    wrapper._generation = parent._inner.generation
    return wrapper


def _field_value_key(rust_field) -> tuple:
    """Comparable value identity of a data field: tag, indicators, subfields."""
    return (
        rust_field.tag,
        rust_field.indicator1,
        rust_field.indicator2,
        tuple((sf.code, sf.value) for sf in rust_field.subfields()),
    )


# Control field tags for enumeration (when we need to iterate all possible control fields)
_CONTROL_TAGS = ('001', '002', '003', '004', '005', '006', '007', '008', '009')


# MARC format constants (pymarc compatibility)
LEADER_LEN = 24
DIRECTORY_ENTRY_LEN = 12
END_OF_FIELD = '\x1e'
END_OF_RECORD = '\x1d'
SUBFIELD_INDICATOR = '\x1f'
MARC_XML_NS = 'http://www.loc.gov/MARC21/slim'
MARC_XML_SCHEMA = 'http://www.loc.gov/standards/marcxml/schema/MARC21slim.xsd'


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
            return bool(self.ind1 == other[0] and self.ind2 == other[1])
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


class Field:
    """Enhanced Field wrapper with pymarc-compatible API.

    Supports both control fields and data fields (like pymarc.Field).
    Control fields are created with ``data=``: ``Field('001', data='12345')``.
    Data fields use indicators and subfields as before.
    """

    def __init__(self, tag: str, indicator1: str = ' ', indicator2: str = ' ', *, subfields: Optional[List[Subfield]] = None, indicators: Optional[List[str]] = None, data: Optional[str] = None):
        """Create a new Field.

        Args:
            tag: 3-character field tag.
            indicator1: First indicator (default ' ').
            indicator2: Second indicator (default ' ').
            subfields: Optional list of Subfield objects to add.
            indicators: Optional list/tuple of [ind1, ind2], overrides indicator1/indicator2.
            data: For control fields, the data string value.
        """
        self._data = data
        self._parent: Optional['Record'] = None
        self._occurrence = 0
        self._generation = 0
        if data is not None:
            # Control field: create a minimal _inner for tag access only
            self._inner = _Field(tag, ' ', ' ')
        else:
            self._inner = _Field(tag, indicator1, indicator2, subfields=subfields, indicators=indicators)

    def _refresh(self) -> None:
        """Re-sync a live handle from its record; no-op when detached.

        Raises StaleFieldError if the record's generation changed (a
        field was removed) or the handle's position no longer resolves.
        """
        parent = self._parent
        if parent is None:
            return
        if parent._inner.generation != self._generation:
            raise StaleFieldError(
                f"field handle for tag {self._inner.tag!r} was invalidated by "
                "record modification - re-fetch the field from the record"
            )
        if self._data is not None:
            value = parent._inner.control_field_value_at(self._inner.tag, self._occurrence)
            if value is None:
                raise StaleFieldError(
                    f"control field handle for tag {self._inner.tag!r} no longer "
                    "resolves - re-fetch the field from the record"
                )
            object.__setattr__(self, '_data', value)
        else:
            fresh = parent._inner.field_at(self._inner.tag, self._occurrence)
            if fresh is None:
                raise StaleFieldError(
                    f"field handle for tag {self._inner.tag!r} no longer "
                    "resolves - re-fetch the field from the record"
                )
            object.__setattr__(self, '_inner', fresh)

    def _writeback(self) -> None:
        """Push a live handle's state back to its record; no-op when detached."""
        parent = self._parent
        if parent is None:
            return
        if self._data is not None:
            ok = parent._inner.set_control_field_value_at(
                self._inner.tag, self._occurrence, self._data
            )
        else:
            ok = parent._inner.replace_field_at(
                self._inner.tag, self._occurrence, self._inner
            )
        if not ok:
            raise StaleFieldError(
                f"field handle for tag {self._inner.tag!r} no longer "
                "resolves - re-fetch the field from the record"
            )

    def __getattr__(self, name: str) -> Any:
        """Delegate attribute access to the inner Rust Field."""
        if name in ('_inner', '_data', '_parent', '_occurrence', '_generation'):
            raise AttributeError(name)
        if self._parent is not None:
            self._refresh()
        return getattr(self._inner, name)
    
    def __getitem__(self, code: str) -> Optional[str]:
        """Get first subfield value by code (pymarc compatibility).
        
        Returns None if the subfield code doesn't exist, matching pymarc behavior.
        
        Example:
            field['a']  # Get first 'a' subfield value
            field['z']  # Returns None if 'z' subfield doesn't exist
        """
        self._refresh()
        try:
            values = self._inner.subfields_by_code(code)
            return values[0] if values else None
        except Exception:
            return None

    def __setitem__(self, code: str, value: str) -> None:
        """Set subfield value (replace first occurrence)."""
        self._refresh()
        subfields = self._inner.subfields()
        code_char = code[0] if code else ''

        # Check if code already exists
        found_count = sum(1 for sf in subfields if sf.code == code_char)

        if found_count == 0:
            raise KeyError(code)
        elif found_count > 1:
            raise KeyError(f"Multiple subfields with code '{code}' exist")
        else:
            self._inner.delete_subfield(code)
            self._inner.add_subfield(code, value)
            self._writeback()

    def get(self, code: str, default: Optional[str] = None) -> Optional[str]:
        """Get first subfield value by code or return default."""
        self._refresh()
        try:
            values = self._inner.subfields_by_code(code)
            return values[0] if values else default
        except Exception:
            return default

    def __contains__(self, code: str) -> bool:
        """Check if subfield code exists in field."""
        self._refresh()
        try:
            values = self._inner.subfields_by_code(code)
            return len(values) > 0
        except Exception:
            return False

    @property
    def data(self) -> Optional[str]:
        """Control field data value (pymarc compatibility).

        Returns the data string for control fields, None for data fields.
        """
        self._refresh()
        return self._data

    @data.setter
    def data(self, value: str) -> None:
        """Set control field data; live handles write through to the record."""
        if self._data is None:
            raise AttributeError("data is only settable on control fields")
        self._refresh()
        object.__setattr__(self, '_data', value)
        self._writeback()

    def value(self) -> str:
        """Return the field's value (pymarc compatibility).
        For control fields, returns the data content.
        For data fields, returns space-joined subfield values.
        """
        self._refresh()
        if self.is_control_field():
            return self._data or ''
        return ' '.join(sf.value for sf in self._inner.subfields())

    def format_field(self) -> str:
        """Return human-readable text without indicators or subfield codes (pymarc compatibility)."""
        if self.is_control_field():
            return self._data or ''
        return ' '.join(sf.value for sf in self.subfields())

    def is_control_field(self) -> bool:
        """Check if this is a control field (pymarc compatibility)."""
        return self._data is not None

    def __str__(self) -> str:
        """MARC display format (pymarc compatibility).

        Data fields: =TAG  IND1IND2$aCONTENT$bCONTENT
        Control fields: =TAG  CONTENT
        """
        self._refresh()
        if self.is_control_field():
            return f'={self._inner.tag}  {self._data}'
        ind1 = self._inner.indicator1.replace(' ', '\\')
        ind2 = self._inner.indicator2.replace(' ', '\\')
        subfield_str = ''.join(f'${sf.code}{sf.value}' for sf in self._inner.subfields())
        return f'={self._inner.tag}  {ind1}{ind2}{subfield_str}'

    def __repr__(self) -> str:
        """Informative repr. Never raises: stale handles say so."""
        try:
            self._refresh()
        except StaleFieldError:
            return f"<Field {self._inner.tag} (stale handle)>"
        if self.is_control_field():
            return f"<Field {self._inner.tag}={self._data!r}>"
        return (
            f"<Field {self._inner.tag} {self._inner.indicator1}"
            f"{self._inner.indicator2} {len(self._inner.subfields())} subfields>"
        )

    def get_subfields(self, *codes: str) -> List[str]:
        """Get all subfield values for given codes (pymarc compatibility).

        Example:
            field.get_subfields('a', 'b')  # Get all 'a' and 'b' subfield values
        """
        self._refresh()
        result = []
        for code in codes:
            try:
                result.extend(self._inner.subfields_by_code(code))
            except Exception:
                pass
        return result

    def delete_subfield(self, code: str) -> Optional[str]:
        """Delete first subfield with given code and return its value.

        Matches pymarc's ``Field.delete_subfield()`` behavior: removes the
        first subfield with the given code and returns its value, or ``None``
        if no subfield with that code exists.
        """
        self._refresh()
        removed = self._inner.delete_subfield(code)
        self._writeback()
        return removed

    def subfields_as_dict(self) -> dict:
        """Return subfields as dictionary mapping code to list of values."""
        self._refresh()
        result: dict[str, list[str]] = {}
        try:
            for sf in self._inner.subfields():
                code = sf.code
                if code not in result:
                    result[code] = []
                result[code].append(sf.value)
        except Exception:
            pass
        return result
    
    def add_subfield(self, code: str, value: str, pos: Optional[int] = None) -> None:
        """Add a subfield, optionally at a specific position (pymarc compatibility)."""
        self._refresh()
        if pos is None:
            self._inner.add_subfield(code, value)
        else:
            current = list(self._inner.subfields())
            new_sf = Subfield(code, value)
            tag = self._inner.tag
            ind1 = self._inner.indicator1
            ind2 = self._inner.indicator2
            object.__setattr__(self, '_inner', _Field(tag, ind1, ind2))
            current.insert(pos, new_sf)
            for sf in current:
                self._inner.add_subfield(sf.code, sf.value)
        self._writeback()

    def subfields(self) -> List[Any]:
        """Get all subfields."""
        self._refresh()
        return self._inner.subfields()

    def subfields_by_code(self, code: str) -> List[str]:
        """Get subfield values by code."""
        self._refresh()
        return self._inner.subfields_by_code(code)

    @property
    def tag(self) -> str:
        """Field tag."""
        return self._inner.tag

    @property
    def indicator1(self) -> str:
        """First indicator."""
        self._refresh()
        return self._inner.indicator1

    @indicator1.setter
    def indicator1(self, value: str) -> None:
        """Set first indicator; live handles write through to the record."""
        self._refresh()
        self._inner.indicator1 = value
        self._writeback()

    @property
    def indicator2(self) -> str:
        """Second indicator."""
        self._refresh()
        return self._inner.indicator2

    @indicator2.setter
    def indicator2(self, value: str) -> None:
        """Set second indicator; live handles write through to the record."""
        self._refresh()
        self._inner.indicator2 = value
        self._writeback()
    
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

    def linkage_occurrence_num(self) -> Optional[str]:
        """Extract the occurrence number from subfield $6 linkage (pymarc compatibility)."""
        if self.is_control_field():
            return None
        sub6 = self['6']
        if sub6 is None:
            return None
        if '-' not in sub6:
            return None
        parts = sub6.split('-', 1)
        occ = parts[1]
        if '/' in occ:
            occ = occ.split('/')[0]
        return occ if occ else None

    def __eq__(self, other: Any) -> bool:
        """Compare fields by content."""
        if not isinstance(other, Field):
            return False
        if self.is_control_field() or other.is_control_field():
            return (self.tag == other.tag and
                    self._data == other._data)
        return (self.tag == other.tag and
                self.indicator1 == other.indicator1 and
                self.indicator2 == other.indicator2 and
                self.subfields() == other.subfields())
    
    def as_marc(self) -> bytes:
        """Serialize field to ISO 2709 binary format (pymarc compatibility)."""
        if self.is_control_field():
            return (self.data or '').encode('utf-8') + b'\x1e'
        return bytes(self._inner.to_marc21())

    def as_marc21(self) -> bytes:
        """Alias for as_marc() (pymarc compatibility)."""
        return self.as_marc()

    def __hash__(self) -> int:
        """Hash based on tag and first subfield."""
        if self._data is not None:
            return hash((self.tag, self._data))
        return hash((self.tag, self.indicator1, self.indicator2))

    @classmethod
    def convert_legacy_subfields(cls, subfields: list) -> list:
        """Convert legacy pymarc subfield list to Subfield objects.

        Converts the old format ['code', 'value', 'code', 'value', ...]
        to a list of Subfield objects.
        """
        result = []
        for i in range(0, len(subfields), 2):
            if i + 1 < len(subfields):
                result.append(Subfield(subfields[i], subfields[i + 1]))
        return result


class ControlField(Field):
    """Backward-compatible alias. In pymarc, both control and data fields are Field."""

    def __init__(self, tag: str, value: str):
        super().__init__(tag, data=value)


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
         return bool(self._rust_leader == other._rust_leader)
     
     def __hash__(self) -> int:
         """Hash based on rust leader."""
         return hash(id(self._rust_leader))


class Record:
    """Enhanced Record wrapper with pymarc-compatible API."""
    
    def __init__(self, leader: Optional[Leader] = None, *, fields: Optional[List[Field]] = None):
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

    def __repr__(self) -> str:
        return repr(self._inner)

    def __str__(self) -> str:
        return str(self._inner)

    def __contains__(self, tag: str) -> bool:
        """Check if a field with given tag exists in record."""
        if _is_control_tag(tag):
            return self._inner.control_field(tag) is not None
        return self.get_field(tag) is not None
    
    def __getitem__(self, tag: str) -> 'Field':
         """Get first field with given tag (pymarc compatibility).

         Returns a Field instance for both control and data fields.
         Raises KeyError if the tag is not present in the record.
         """
         # Check if this is a control field (001-009)
         if _is_control_tag(tag):
             value = self._inner.control_field(tag)
             if value is not None:
                 return _wrap_control_field(self, tag, 0, value)
             raise KeyError(tag)

         # For data fields, return a live handle
         field = self._inner.get_field(tag)
         if field:
             return _wrap_field(field, self, 0)
         raise KeyError(tag)
    
    def get_fields(self, *tags: str) -> List['Field']:
        """Get all fields with given tags.

        If no tags provided, returns all fields (control + data).
        Supports multiple tags: record.get_fields('245', '260')
        """
        result = []

        if not tags:
            # Return all control fields, then all data fields
            for tag in _CONTROL_TAGS:
                for i, value in enumerate(self._inner.control_field_values(tag)):
                    result.append(_wrap_control_field(self, tag, i, value))
            occurrences: dict[str, int] = {}
            for field in self._inner.fields():
                i = occurrences.get(field.tag, 0)
                occurrences[field.tag] = i + 1
                result.append(_wrap_field(field, self, i))
        else:
            # Return fields for specified tags
            for tag in tags:
                if _is_control_tag(tag):
                    for i, value in enumerate(self._inner.control_field_values(tag)):
                        result.append(_wrap_control_field(self, tag, i, value))
                else:
                    for i, field in enumerate(self._inner.get_fields(tag)):
                        result.append(_wrap_field(field, self, i))

        return result
    
    def add_field(self, *fields: 'Field') -> None:
        """Add one or more fields to the record."""
        for field in fields:
            if field.is_control_field():
                self._inner.add_control_field(field.tag, field.data or '')
            else:
                self._inner.add_field(field._inner)
    
    def get(self, tag: str, default=None):
        """Get first field with given tag, or default (pymarc compatibility)."""
        try:
            return self[tag]
        except KeyError:
            return default

    def get_field(self, tag: str) -> Optional['Field']:
        """Get first field with given tag."""
        field = self._inner.get_field(tag)
        if field:
            return _wrap_field(field, self, 0)
        return None

    def get_field_or_err(self, tag: str) -> 'Field':
        """Get first field with given tag, raising :class:`mrrc.FieldNotFound`
        (E105) when the tag is not present.

        Strict-mode counterpart to :meth:`get_field`. Use this when an
        absent field is a programming error worth the typed exception
        and its diagnostic context (``record_control_number``,
        ``field_tag``); use :meth:`get_field` when ``None`` is the
        expected sentinel.
        """
        return _wrap_field(self._inner.get_field_or_err(tag), self, 0)

    def remove_field(self, *fields: Union['Field', str]) -> None:
        """Remove one or more fields from the record (pymarc compatibility).

        A Field argument removes exactly that field: a live handle
        obtained from this record removes its precise occurrence; a
        detached field removes the first value-equal field, raising
        ValueError when nothing matches (as pymarc does). A tag string
        removes all fields with that tag, control tags included.
        """
        for field in fields:
            if isinstance(field, Field):
                self._remove_one(field)
            else:
                self._remove_tag(field)

    def remove_fields(self, *tags: str) -> None:
        """Remove all fields with the given tags (pymarc compatibility)."""
        for tag in tags:
            self._remove_tag(tag)

    def _remove_tag(self, tag: str) -> None:
        """Remove all fields with the given tag, control tags included."""
        if _is_control_tag(tag):
            self._inner.remove_control_field(tag)
        else:
            self._inner.remove_field(tag)

    def _remove_one(self, field: 'Field') -> None:
        """Remove exactly one field.

        Live handles bound to this record remove their occurrence
        (raising StaleFieldError if the handle was invalidated).
        Detached or foreign fields remove the first value-equal field,
        raising ValueError when no field matches.
        """
        if field._parent is not None:
            field._refresh()
        tag = field._inner.tag
        if field._parent is self:
            if field.is_control_field():
                self._inner.remove_control_field_at(tag, field._occurrence)
            else:
                self._inner.remove_field_at(tag, field._occurrence)
            return
        if field.is_control_field():
            for i, value in enumerate(self._inner.control_field_values(tag)):
                if value == field._data:
                    self._inner.remove_control_field_at(tag, i)
                    return
        else:
            wanted = _field_value_key(field._inner)
            for i, candidate in enumerate(self._inner.get_fields(tag)):
                if _field_value_key(candidate) == wanted:
                    self._inner.remove_field_at(tag, i)
                    return
        raise ValueError(f"field not in record: {field!r}")

    def _rebuild_fields(self, field_list) -> None:
        """Replace all data fields with the given list (internal helper)."""
        existing_tags = set(f.tag for f in self._inner.fields())
        for tag in existing_tags:
            self._inner.remove_field(tag)
        for f in field_list:
            self._inner.add_field(f)

    def add_ordered_field(self, *fields: 'Field') -> None:
        """Add fields maintaining tag sort order (pymarc compatibility)."""
        for field in fields:
            if field.is_control_field():
                self._inner.add_control_field(field.tag, field.data or '')
            else:
                existing = list(self._inner.fields())
                insert_idx = len(existing)
                for i, f in enumerate(existing):
                    if f.tag > field.tag:
                        insert_idx = i
                        break
                existing.insert(insert_idx, field._inner)
                self._rebuild_fields(existing)

    def add_grouped_field(self, *fields: 'Field') -> None:
        """Add fields after the last field with the same tag (pymarc compatibility)."""
        for field in fields:
            if field.is_control_field():
                self._inner.add_control_field(field.tag, field.data or '')
                continue
            existing = list(self._inner.fields())
            last_idx = None
            for i, f in enumerate(existing):
                if f.tag == field.tag:
                    last_idx = i
            if last_idx is None:
                self.add_ordered_field(field)
            else:
                existing.insert(last_idx + 1, field._inner)
                self._rebuild_fields(existing)

    def add_control_field(self, tag: str, value: str) -> None:
        """Add a control field."""
        self._inner.add_control_field(tag, value)
    
    def control_field(self, tag: str) -> Optional[str]:
        """Get a control field value."""
        return self._inner.control_field(tag)
    
    def fields(self) -> List['Field']:
        """Get all fields (control + data), as live handles.

        Enumerates identically to no-arg :meth:`get_fields`: repeated
        control tags (e.g. multiple 007s) yield one field per value.
        """
        return self.get_fields()

    def fields_by_tag(self, tag: str) -> List['Field']:
        """Get all data fields with the given tag, as live handles."""
        return [
            _wrap_field(field, self, i)
            for i, field in enumerate(self._inner.fields_by_tag(tag))
        ]
    
    @property
    def title(self) -> Optional[str]:
        """Title from 245 field."""
        return self._inner.title()

    @property
    def author(self) -> Optional[str]:
        """Author from 100/110/111 field."""
        return self._inner.author()

    @property
    def isbn(self) -> Optional[str]:
        """ISBN from 020 field."""
        return self._inner.isbn()

    @property
    def issn(self) -> Optional[str]:
        """ISSN from 022 field."""
        return self._inner.issn()

    @property
    def subjects(self) -> List[str]:
        """All subject headings from 6XX subject fields."""
        return self._inner.subjects()

    @property
    def location(self) -> List[str]:
        """All location fields (852)."""
        return self._inner.location()

    @property
    def notes(self) -> List[str]:
        """All notes from 5xx fields."""
        return self._inner.notes()

    @property
    def publisher(self) -> Optional[str]:
        """Publisher from 260 or 264 (RDA) field."""
        return self._inner.publisher()

    @property
    def uniform_title(self) -> Optional[str]:
        """Uniform title from 130 field."""
        return self._inner.uniform_title()

    @property
    def sudoc(self) -> Optional[str]:
        """SuDoc from 086 field."""
        return self._inner.sudoc()

    @property
    def issn_title(self) -> Optional[str]:
        """ISSN title from 222 field."""
        return self._inner.issn_title()

    @property
    def issnl(self) -> Optional[str]:
        """ISSN-L from 024 field."""
        return self._inner.issnl()

    @property
    def pubyear(self) -> Optional[str]:
        """Publication year (returns str, matching pymarc)."""
        result = self._inner.pubyear()
        return str(result) if result is not None else None

    @property
    def series(self) -> Optional[str]:
        """Series from 490 field."""
        return self._inner.series()

    @property
    def physical_description(self) -> Optional[str]:
        """Physical description from 300 field."""
        return self._inner.physical_description()

    @property
    def physicaldescription(self) -> Optional[str]:
        """Physical description (pymarc-compatible name)."""
        return self.physical_description

    @property
    def uniformtitle(self) -> Optional[str]:
        """Uniform title (pymarc-compatible name)."""
        return self.uniform_title

    @property
    def addedentries(self) -> list:
        """Added entries from 700/710/711/730 fields (pymarc compatibility)."""
        return self.get_fields('700', '710', '711', '730')
    
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
    # Linked field navigation (880 alternate graphic representation)
    # =========================================================================

    def get_linked_fields(self, field: 'Field') -> List['Field']:
        """Find all 880 fields linked to a given field via subfield $6.

        Given a non-880 field that has a $6 linkage subfield, returns all 880
        fields whose $6 occurrence number matches.  This is pymarc-compatible.

        Args:
            field: A Field object with a $6 linkage subfield.

        Returns:
            List of linked 880 Field objects (empty if no linkage or no match).

        Example:
            >>> f245 = record.get_fields('245')[0]
            >>> linked = record.get_linked_fields(f245)
            >>> if linked:
            ...     print(f"Vernacular title: {linked[0]['a']}")
        """
        result = []
        for f in self._inner.get_linked_fields(field._inner):
            wrapper = Field(f.tag, f.indicator1, f.indicator2)
            wrapper._inner = f
            result.append(wrapper)
        return result

    def get_linked_field(self, field: 'Field') -> Optional['Field']:
        """Find the single 880 field linked to a given field via subfield $6.

        Like get_linked_fields() but returns only the first match.

        Args:
            field: A Field object with a $6 linkage subfield.

        Returns:
            The linked 880 Field, or None.
        """
        f = self._inner.get_linked_field(field._inner)
        if f is not None:
            wrapper = Field(f.tag, f.indicator1, f.indicator2)
            wrapper._inner = f
            return wrapper
        return None

    def get_original_field(self, field_880: 'Field') -> Optional['Field']:
        """Find the original field linked from a given 880 field.

        Args:
            field_880: An 880 Field object.

        Returns:
            The linked original Field, or None.
        """
        f = self._inner.get_original_field(field_880._inner)
        if f is not None:
            wrapper = Field(f.tag, f.indicator1, f.indicator2)
            wrapper._inner = f
            return wrapper
        return None

    def get_field_pairs(self, tag: str) -> List[Tuple['Field', Optional['Field']]]:
        """Get field pairs of original fields with their linked 880 counterparts.

        Args:
            tag: The field tag to pair (e.g., '245', '100').

        Returns:
            List of (original Field, linked 880 Field or None) tuples.

        Example:
            >>> for orig, linked in record.get_field_pairs('245'):
            ...     print(f"Romanized: {orig['a']}")
            ...     if linked:
            ...         print(f"Vernacular: {linked['a']}")
        """
        result = []
        for orig, linked in self._inner.get_field_pairs(tag):
            orig_wrapper = Field(orig.tag, orig.indicator1, orig.indicator2)
            orig_wrapper._inner = orig
            linked_wrapper = None
            if linked is not None:
                linked_wrapper = Field(linked.tag, linked.indicator1, linked.indicator2)
                linked_wrapper._inner = linked
            result.append((orig_wrapper, linked_wrapper))
        return result

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
            result.append(_wrap_field(field))
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
            result.append(_wrap_field(field))
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
            result.append(_wrap_field(field))
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
            result.append(_wrap_field(field))
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
            result.append(_wrap_field(field))
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
            result.append(_wrap_field(field))
        return result
    
    def to_json(self) -> str:
        """Serialize to JSON."""
        return self._inner.to_json()
    
    def to_xml(self) -> str:
        """Serialize to MARCXML."""
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
    
    def as_dict(self) -> dict:
        """Return pymarc-compatible MARC-in-JSON dict (code4lib schema)."""
        fields_list: list[dict[str, Any]] = []
        for tag, value in self._inner.control_fields():
            fields_list.append({tag: value})
        for field in self._inner.fields():
            subfields_list = [{sf.code: sf.value} for sf in field.subfields()]
            fields_list.append({
                field.tag: {
                    'ind1': field.indicator1,
                    'ind2': field.indicator2,
                    'subfields': subfields_list,
                }
            })
        return {
            'leader': self.leader()._get_leader_as_string(),
            'fields': fields_list,
        }

    def as_json(self, **kwargs) -> str:
        """Serialize to pymarc-compatible MARC-in-JSON string."""
        import json as _json
        return _json.dumps(self.as_dict(), **kwargs)

    def as_marc(self) -> bytes:
        """Serialize record to ISO 2709 binary MARC (pymarc compatibility)."""
        self._sync_leader()
        return bytes(self._inner.to_marc21())

    def as_marc21(self) -> bytes:
        """Alias for as_marc() (pymarc compatibility)."""
        return self.as_marc()

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
        self_cfs = self._inner.control_fields()
        other_cfs = other._inner.control_fields()
        if self_cfs != other_cfs:
            return False
        
        return True
    
    def __hash__(self) -> int:
        """Hash based on leader."""
        return hash(id(self._inner))


class MARCReader:
    """MARC Reader wrapper.

    Args:
        file_obj: File path (str), pathlib.Path, bytes/bytearray, or file-like object.
        to_unicode: Accepted for pymarc compatibility. mrrc always converts
            MARC-8 to UTF-8; passing ``False`` emits a warning.
        permissive: When ``True``, yields ``None`` for records that fail to
            parse instead of raising, matching pymarc's ``permissive`` behavior.
            Setting this flag implicitly defaults ``recovery_mode`` back to
            ``"strict"`` (pymarc-shape: inner raises, outer swallows) unless
            an explicit ``recovery_mode`` is passed.
        recovery_mode: mrrc-native error handling. ``"permissive"`` (the
            default for the Python user surface) accepts partial data and
            yields records with diagnostics attached on ``record.errors``;
            unsalvageable records yield as ``None``. ``"lenient"`` salvages
            valid fields and also attaches diagnostics on ``record.errors``.
            ``"strict"`` raises on the first malformation. Cannot be combined
            with ``permissive=True`` (which implies the strict-raises + outer-
            swallows shape).
        validation_level: What counts as an error during parsing — orthogonal
            to ``recovery_mode``. ``"structural"`` (default) fires only ISO
            2709 structural errors; UTF-8 decode is lossy across all readers.
            ``"strict_marc"`` adds universal byte-level MARC 21 checks
            (E201 indicator, E202 subfield code, E301 strict UTF-8).
        max_errors: Optional cap on accumulated recovered errors in
            lenient/permissive mode. ``None`` (default) disables the
            wrapper-level cap. ``0`` matches the Rust API's
            no-cap sentinel. Any positive ``N`` raises
            :class:`mrrc.FatalReaderError` (E099) once recovered errors
            across records exceed ``N``. Observationally inert in strict
            mode (the first error fires before any recovery accumulates).
    """

    def __init__(self, file_obj: Any, to_unicode: bool = True, permissive: bool = False,
                 recovery_mode: Optional[str] = None,
                 validation_level: str = "structural",
                 max_errors: Optional[int] = None):
        """Create a new MARC reader."""
        if not to_unicode:
            import warnings
            warnings.warn(
                "mrrc always converts MARC-8 to UTF-8; to_unicode=False has no effect",
                stacklevel=2,
            )
        # ``recovery_mode`` defaults are a two-level sentinel: a caller
        # who passes ``permissive=True`` (pymarc-shape) without naming a
        # recovery_mode gets the inner reader in strict so the outer
        # wrapper can swallow per pymarc semantics; everyone else gets
        # the permissive default. Explicit combinations that contradict
        # (``permissive=True`` paired with a non-strict recovery_mode)
        # still raise below.
        if recovery_mode is None:
            recovery_mode = "strict" if permissive else "permissive"
        if permissive and recovery_mode != "strict":
            raise ValueError(
                "Cannot combine permissive=True with recovery_mode other than "
                "'strict' — they represent different error-handling strategies"
            )
        self._permissive = permissive
        self._inner = _MARCReader(
            file_obj,
            recovery_mode=recovery_mode,
            validation_level=validation_level,
            max_errors=max_errors,
        )
        # pymarc-compat accessors populated by __next__. `current_chunk`
        # tracks the bytes of the most recent record chunk read from the
        # source (regardless of whether the parse succeeded);
        # `current_exception` is the typed mrrc exception caught from
        # the most recent parse attempt, or None on a clean read. Both
        # mirror pymarc.MARCReader semantics so existing pymarc-shape
        # error-diagnosis code (``if reader.current_exception: ...``)
        # works against mrrc unchanged.
        self.current_exception: Optional[Exception] = None
        self.current_chunk: Optional[bytes] = None

    def __iter__(self):
        """Iterate over records."""
        return self

    def __next__(self) -> Optional[Record]:
        """Get next record.

        When ``permissive=True``, returns ``None`` for records that fail
        to parse instead of raising, matching pymarc behavior. After
        each call, ``self.current_chunk`` holds the bytes just read
        from the source, and ``self.current_exception`` holds the
        exception caught (when a permissive-mode parse failed) or
        ``None`` on a clean read.
        """
        try:
            record = next(self._inner)
        except StopIteration:
            raise
        except Exception as e:
            if self._permissive:
                self.current_chunk = self._inner.last_chunk
                self.current_exception = e
                return None
            raise
        self.current_chunk = self._inner.last_chunk
        self.current_exception = None
        return _wrap_record(record)

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

    def iter_with_errors(self):
        """Iterate yielding ``(record, errors)`` tuples.

        Equivalent to iterating with ``__next__`` and reading
        ``record.errors`` from each yielded record — same data, more
        ergonomic destructuring at the call site, and observable in
        ``permissive=True`` mode for records that ``__next__`` would
        otherwise swallow as ``None``.

        In ``recovery_mode="strict"`` the errors list is always empty
        (the parser raises before the record is yielded). In
        ``lenient`` / ``permissive`` it carries any diagnostics
        captured during the record's parse.

        With ``permissive=True``, records whose parse failed entirely
        yield as ``(None, [exception])`` instead of being silently
        skipped, so even unsalvageable records are observable.

        Example::

            for rec, errs in reader.iter_with_errors():
                if rec is None:
                    log.error(f"unsalvageable: {errs[0]}")
                elif errs:
                    log.warning(f"{len(errs)} issues parsing record")
        """
        while True:
            try:
                inner_record = next(self._inner)
            except StopIteration:
                return
            except Exception as e:
                if self._permissive:
                    yield (None, [e])
                    continue
                raise
            wrapped = _wrap_record(inner_record)
            yield (wrapped, wrapped.errors)


class MARCWriter:
    """MARC Writer wrapper."""
    
    def __init__(self, file_obj: Any):
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


def _wrap_record(rust_record) -> Record:
    """Wrap a raw Rust PyRecord in the Python Record wrapper."""
    wrapper = Record(None)
    wrapper._inner = rust_record
    leader = Leader()
    leader._rust_leader = rust_record.leader()
    leader._parent_record = wrapper
    wrapper._leader = leader
    wrapper._leader_modified = False
    return wrapper


def json_to_record(json_str: str) -> Record:
    """Convert a MARC JSON string to a Record."""
    return _wrap_record(_json_to_record(json_str))


def xml_to_record(xml_str: str) -> Record:
    """Convert a MARCXML string to a Record."""
    return _wrap_record(_xml_to_record(xml_str))


def xml_to_records(xml_str: str) -> List[Record]:
    """Convert a MARCXML collection string to a list of Records."""
    return [_wrap_record(r) for r in _xml_to_records(xml_str)]


def marcjson_to_record(marcjson_str: str) -> Record:
    """Convert a MARCJSON string to a Record."""
    return _wrap_record(_marcjson_to_record(marcjson_str))


def mods_to_record(mods_str: str) -> Record:
    """Convert a MODS XML string to a Record."""
    return _wrap_record(_mods_to_record(mods_str))


def mods_collection_to_records(mods_str: str) -> List[Record]:
    """Convert a MODS collection XML string to a list of Records."""
    return [_wrap_record(r) for r in _mods_collection_to_records(mods_str)]


def parse_xml_to_array(xml_file) -> List[Record]:
    """Parse MARCXML to a list of Records (pymarc compatibility).

    Accepts file paths (str/Path), open file handles, or XML strings.
    """
    import os
    if isinstance(xml_file, (str, os.PathLike)):
        path = str(xml_file)
        if os.path.isfile(path):
            with open(path, 'r', encoding='utf-8') as f:
                xml_str = f.read()
        else:
            xml_str = path
    elif hasattr(xml_file, 'read'):
        xml_str = xml_file.read()
    else:
        xml_str = str(xml_file)
    return xml_to_records(xml_str)


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


# Mirror module-level MARC leader spec helpers onto the Leader class so
# both `Leader.describe_value(...)` and `mrrc.get_leader_value_description(...)`
# resolve to the same implementation (which delegates to the Rust Leader).
# This is a deliberate override of the equivalently-named @classmethods on
# the Leader class above; the runtime behavior chooses the Rust spec as the
# single source of truth. mypy / pyright can't see through this assignment
# pattern, so the per-line ignores below are intentional.
Leader.get_valid_values = staticmethod(get_leader_valid_values)  # type: ignore[method-assign]
Leader.describe_value = staticmethod(get_leader_value_description)  # type: ignore[attr-defined]
Leader.is_valid_value = staticmethod(get_leader_is_valid_value)  # type: ignore[method-assign]
Leader.get_value_description = staticmethod(get_leader_value_description)  # type: ignore[method-assign]


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

def marc_to_bibframe(record, config: Optional[BibframeConfig] = None) -> RdfGraph:
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


def map_records(func, *files: str) -> None:
    """Apply a function to each record in one or more MARC files (pymarc compatibility)."""
    for path in files:
        reader = MARCReader(open(path, 'rb'))
        for record in reader:
            func(record)


def parse_json_to_array(json_str: str) -> List[Record]:
    """Parse a JSON array of pymarc-format records (pymarc compatibility)."""
    import json as _json
    data = _json.loads(json_str)
    if not isinstance(data, list):
        data = [data]
    records = []
    for item in data:
        record = Record()
        if 'leader' in item:
            record.leader()._update_leader_from_string(str(item['leader']))
        if 'fields' in item:
            for field_dict in item['fields']:
                for tag, value in field_dict.items():
                    if isinstance(value, str):
                        record.add_control_field(tag, value)
                    elif isinstance(value, dict):
                        ind1 = value.get('ind1', ' ')
                        ind2 = value.get('ind2', ' ')
                        subfields = []
                        for sf_dict in value.get('subfields', []):
                            for code, val in sf_dict.items():
                                subfields.append(Subfield(code, val))
                        f = Field(tag, ind1, ind2, subfields=subfields)
                        record.add_field(f)
        records.append(record)
    return records


__all__ = [
    # MARC format constants
    "LEADER_LEN",
    "DIRECTORY_ENTRY_LEN",
    "END_OF_FIELD",
    "END_OF_RECORD",
    "SUBFIELD_INDICATOR",
    "MARC_XML_NS",
    "MARC_XML_SCHEMA",
    # Exception hierarchy
    "MrrcException",
    "RecordLengthInvalid",
    "RecordLeaderInvalid",
    "BaseAddressInvalid",
    "BaseAddressNotFound",
    "RecordDirectoryInvalid",
    "EndOfRecordNotFound",
    "FieldNotFound",
    "FatalReaderError",
    "BadSubfieldCodeWarning",
    "StaleFieldError",
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
    "xml_to_records",
    "parse_xml_to_array",
    "record_to_marcjson",
    "marcjson_to_record",
    "record_to_dublin_core",
    "record_to_dublin_core_xml",
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
    # Convenience functions
    "map_records",
    "parse_json_to_array",
]
