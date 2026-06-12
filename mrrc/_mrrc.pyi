"""Type stubs for the mrrc native extension module."""

from collections.abc import Iterator
from typing import Any, final

__version__: str
__all__ = [
    "AuthorityMARCReader",
    "AuthorityRecord",
    "BibframeConfig",
    "Field",
    "FieldQuery",
    "HoldingsMARCReader",
    "HoldingsRecord",
    "Leader",
    "MARCReader",
    "MARCWriter",
    "ProducerConsumerPipeline",
    "RdfGraph",
    "Record",
    "RecordBoundaryScanner",
    "Subfield",
    "SubfieldPatternQuery",
    "SubfieldValueQuery",
    "TagRangeQuery",
    "__doc__",
    "__version__",
    "bibframe_to_marc",
    "dublin_core_to_xml",
    "json_to_record",
    "marc_to_bibframe",
    "marcjson_to_record",
    "mods_collection_to_records",
    "mods_to_record",
    "parse_batch_parallel",
    "parse_batch_parallel_limited",
    "record_to_csv",
    "record_to_dublin_core",
    "record_to_dublin_core_xml",
    "record_to_json",
    "record_to_marcjson",
    "record_to_mods",
    "record_to_xml",
    "records_to_csv",
    "records_to_csv_filtered",
    "xml_to_record",
    "xml_to_records",
]

# =============================================================================
# Core Data Types
# =============================================================================

@final
class Subfield:
    """A subfield within a MARC field.

    Subfields are named data elements within fields, consisting of a
    single-character code and a value.
    """
    code: str
    value: str
    def __new__(cls, code: str, value: str) -> Subfield: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __eq__(self, other: object, /) -> bool: ...

@final
class Leader:
    """MARC Leader - 24-byte record header.

    The MARC leader contains metadata about the record structure and content.
    All MARC records must begin with exactly 24 bytes of leader information.
    """
    @property
    def record_length(self) -> int:
        """Record length (5 digits) - positions 0-4"""
        ...
    @record_length.setter
    def record_length(self, value: int) -> None: ...
    @property
    def record_status(self) -> str:
        """Record status (1 char) - position 5"""
        ...
    @record_status.setter
    def record_status(self, value: str) -> None: ...
    @property
    def record_type(self) -> str:
        """Type of record (1 char) - position 6 (a=language, c=music, etc)"""
        ...
    @record_type.setter
    def record_type(self, value: str) -> None: ...
    @property
    def bibliographic_level(self) -> str:
        """Bibliographic level (1 char) - position 7 (m=monograph, s=serial, etc)"""
        ...
    @bibliographic_level.setter
    def bibliographic_level(self, value: str) -> None: ...
    @property
    def control_record_type(self) -> str:
        """Type of control record (1 char) - position 8"""
        ...
    @control_record_type.setter
    def control_record_type(self, value: str) -> None: ...
    @property
    def character_coding(self) -> str:
        """Character coding scheme (1 char) - position 9 (space=MARC-8, a=UTF-8)"""
        ...
    @character_coding.setter
    def character_coding(self, value: str) -> None: ...
    @property
    def indicator_count(self) -> int:
        """Indicator count (1 digit) - position 10 (usually 2)"""
        ...
    @indicator_count.setter
    def indicator_count(self, value: int) -> None: ...
    @property
    def subfield_code_count(self) -> int:
        """Subfield code count (1 digit) - position 11 (usually 2)"""
        ...
    @subfield_code_count.setter
    def subfield_code_count(self, value: int) -> None: ...
    @property
    def data_base_address(self) -> int:
        """Base address of data (5 digits) - positions 12-16"""
        ...
    @data_base_address.setter
    def data_base_address(self, value: int) -> None: ...
    @property
    def encoding_level(self) -> str:
        """Encoding level (1 char) - position 17"""
        ...
    @encoding_level.setter
    def encoding_level(self, value: str) -> None: ...
    @property
    def cataloging_form(self) -> str:
        """Cataloging form (1 char) - position 18"""
        ...
    @cataloging_form.setter
    def cataloging_form(self, value: str) -> None: ...
    @property
    def multipart_level(self) -> str:
        """Multipart resource record level (1 char) - position 19"""
        ...
    @multipart_level.setter
    def multipart_level(self, value: str) -> None: ...
    @property
    def reserved(self) -> str:
        """Reserved (4 chars) - positions 20-23 (usually "4500")"""
        ...
    @reserved.setter
    def reserved(self, value: str) -> None: ...

    def __init__(self) -> None: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __eq__(self, other: object, /) -> bool: ...
    @staticmethod
    def get_valid_values(position: int) -> dict[str, str] | None: ...
    @staticmethod
    def describe_value(position: int, value: str) -> str | None: ...
    @staticmethod
    def is_valid_value(position: int, value: str) -> bool: ...
    @staticmethod
    def get_value_description(position: int, value: str) -> str | None: ...

@final
class Field:
    """A MARC data field (tags 010 and higher).

    A field consists of a 3-character tag, two indicators, and one or more subfields.
    """
    tag: str
    indicator1: str
    indicator2: str

    def __new__(
        cls,
        tag: str,
        indicator1: str | None = None,
        indicator2: str | None = None,
        *,
        subfields: list[Subfield] | None = None,
        indicators: list[str] | None = None,
    ) -> Field: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __eq__(self, other: object, /) -> bool: ...
    def add_subfield(self, code: str, value: str) -> None:
        """Add a subfield to the field.

        Args:
            code: Subfield code (single character)
            value: Subfield value

        Raises:
            ValueError: If code is empty
        """
        ...
    def subfields(self) -> list[Subfield]:
        """Get all subfields in this field."""
        ...
    def subfields_by_code(self, code: str) -> list[str]:
        """Get all subfield values for a given code.

        Args:
            code: Subfield code to search for

        Returns:
            List of subfield values matching the code

        Raises:
            ValueError: If code is empty
        """
        ...
    def delete_subfield(self, code: str) -> str | None:
        """Delete the first subfield with the given code, returning its value.

        Args:
            code: Subfield code (single character)

        Returns:
            The deleted subfield's value, or None if no matching subfield was
            present.

        Raises:
            ValueError: If code is empty.
        """
        ...
    def to_marc21(self) -> bytes:
        """Serialize field to ISO 2709 binary format.

        Returns:
            Bytes containing the field's binary MARC 21 representation
            (indicators + subfield data + field terminator).
        """
        ...

@final
class Record:
    """A MARC bibliographic record.

    A MARC record consists of a leader, control fields (000-009), and data fields (010+).
    """
    def __new__(cls, leader: Leader) -> Record: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __eq__(self, other: object, /) -> bool: ...
    @property
    def errors(self) -> list[Exception]:
        """Non-fatal errors accumulated while parsing this record.

        Always empty in ``recovery_mode="strict"``. Populated in
        ``lenient`` / ``permissive`` with one ``MrrcException`` instance
        per recovered defect, carrying the same positional context as
        if it had been raised directly.
        """
        ...
    @property
    def leader(self) -> Leader:
        """The record leader (attribute, matching pymarc's record.leader)."""
        ...
    def set_leader(self, leader: Leader) -> None: ...
    def add_field(self, field: Field) -> None:
        """Add a data field to the record."""
        ...
    def add_control_field(self, tag: str, value: str) -> None:
        """Add a control field (000-009).

        Args:
            tag: 3-character field tag
            value: Field value

        Raises:
            ValueError: If tag is not 3 characters
        """
        ...
    def control_field(self, tag: str) -> str | None:
        """Get a control field value.

        Args:
            tag: 3-character field tag

        Returns:
            The field value, or None if not found
        """
        ...
    def control_fields(self) -> list[tuple[str, str]]:
        """Get all control fields as (tag, value) tuples.

        Repeated tags (e.g., multiple 007 fields) produce multiple entries.
        """
        ...
    def control_field_values(self, tag: str) -> list[str]:
        """Get all values for a control field tag.

        Returns all values for tags that may be repeated (e.g., 006, 007).
        Returns an empty list if the tag doesn't exist.
        """
        ...
    def get_field(self, tag: str) -> Field | None: ...
    def get_field_or_err(self, tag: str) -> Field:
        """Get first field with given tag, raising ``mrrc.FieldNotFound``
        (E105) when the tag is not present."""
        ...
    def get_fields(self, tag: str) -> list[Field]: ...
    def fields(self) -> list[Field]:
        """Get all fields in the record."""
        ...
    def remove_field(self, tag: str) -> list[Field]:
        """Remove all fields with a given tag, returning them.

        Bumps ``generation`` when anything was removed, invalidating
        outstanding field handles.
        """
        ...
    def remove_field_at(self, tag: str, occurrence: int) -> Field | None:
        """Remove the single data field at (tag, occurrence), where
        occurrence is the zero-based index among fields with that tag.

        Returns the removed field, or None if no field exists at that
        position. Bumps ``generation`` on removal.
        """
        ...
    def remove_control_field(self, tag: str) -> list[str]:
        """Remove all values for a control field tag, returning them.

        Bumps ``generation`` when anything was removed, invalidating
        outstanding field handles.
        """
        ...
    def remove_control_field_at(self, tag: str, occurrence: int) -> str | None:
        """Remove the control field value at (tag, occurrence).

        Returns the removed value, or None if no value exists at that
        position. Bumps ``generation`` on removal.
        """
        ...
    @property
    def generation(self) -> int:
        """Modification counter bumped by every field removal.

        Field handles capture this at creation and raise
        ``mrrc.StaleFieldError`` once it changes.
        """
        ...
    def field_at(self, tag: str, occurrence: int) -> Field | None:
        """Get a copy of the field at (tag, occurrence), where occurrence
        is the zero-based index among fields with that tag."""
        ...
    def replace_field_at(self, tag: str, occurrence: int, field: Field) -> bool:
        """Replace the field at (tag, occurrence). Returns False if no
        field exists at that position."""
        ...
    def control_field_value_at(self, tag: str, occurrence: int) -> str | None:
        """Get the control field value at (tag, occurrence)."""
        ...
    def set_control_field_value_at(self, tag: str, occurrence: int, value: str) -> bool:
        """Replace the control field value at (tag, occurrence). Returns
        False if no value exists at that position."""
        ...
    def fields_by_indicator(
        self, tag: str, *, indicator1: str | None = None, indicator2: str | None = None
    ) -> list[Field]: ...
    def fields_in_range(self, start_tag: str, end_tag: str) -> list[Field]: ...
    def get_linked_fields(self, field: Field) -> list[Field]:
        """Find all 880 fields linked to a given field via subfield $6."""
        ...
    def get_linked_field(self, field: Field) -> Field | None:
        """Find the single 880 field linked to a given field via subfield $6."""
        ...
    def get_original_field(self, field_880: Field) -> Field | None:
        """Find the original field linked from a given 880 field."""
        ...
    def get_field_pairs(self, tag: str) -> list[tuple[Field, Field | None]]:
        """Get field pairs of original fields with their linked 880 counterparts."""
        ...
    def fields_matching(self, query: FieldQuery) -> list[Field]: ...
    def fields_matching_range(self, query: TagRangeQuery) -> list[Field]: ...
    def fields_matching_pattern(self, query: SubfieldPatternQuery) -> list[Field]: ...
    def fields_matching_value(self, query: SubfieldValueQuery) -> list[Field]: ...
    def title(self) -> str | None:
        """Get title from 245 field (first subfield $a)."""
        ...
    def author(self) -> str | None:
        """Get author from 100/110/111 field."""
        ...
    def isbn(self) -> str | None:
        """Get ISBN from 020 field."""
        ...
    def issn(self) -> str | None: ...
    def subjects(self) -> list[str]: ...
    def location(self) -> list[str]: ...
    def notes(self) -> list[str]: ...
    def publisher(self) -> str | None: ...
    def uniform_title(self) -> str | None: ...
    def sudoc(self) -> str | None: ...
    def issn_title(self) -> str | None: ...
    def issnl(self) -> str | None: ...
    def pubyear(self) -> int | None: ...
    def series(self) -> str | None: ...
    def physical_description(self) -> str | None: ...
    def is_book(self) -> bool: ...
    def is_serial(self) -> bool: ...
    def is_music(self) -> bool: ...
    def is_audiovisual(self) -> bool: ...
    def to_json(self) -> str: ...
    def to_xml(self) -> str: ...
    def to_dublin_core(self) -> str: ...
    def to_marcjson(self) -> str: ...
    def to_mods(self) -> str: ...
    def to_marc21(self) -> bytes:
        """Serialize the record to ISO 2709 binary format."""
        ...

@final
class AuthorityRecord:
    """A MARC authority record. Returned by ``AuthorityMARCReader``."""
    @property
    def leader(self) -> Leader: ...
    @property
    def errors(self) -> list[Exception]:
        """Non-fatal errors accumulated while parsing this record.
        See :attr:`Record.errors`.
        """
        ...
    def record_type(self) -> str:
        """Leader byte 06 (type of record) as a single-character string."""
        ...
    def heading(self) -> Field | None:
        """The main heading field (1XX), or ``None`` if absent."""
        ...
    def heading_text(self) -> str | None:
        """Subfield ``a`` of the main heading field, or ``None``."""
        ...
    def see_from_tracings(self) -> list[Field]:
        """See-from tracings (4XX fields)."""
        ...
    def see_also_tracings(self) -> list[Field]:
        """See-also-from tracings (5XX fields)."""
        ...
    def notes(self) -> list[Field]:
        """Note fields: 6XX fields other than the subject tracings 650/651/655."""
        ...
    def linking_entries(self) -> list[Field]:
        """Heading linking entries (7XX fields)."""
        ...
    def get_fields(self, tag: str) -> list[Field] | None:
        """All fields matching ``tag``, or ``None`` when none match."""
        ...
    def get_field(self, tag: str) -> Field | None: ...
    def get_field_or_err(self, tag: str) -> Field:
        """Get first field with given tag, raising ``mrrc.FieldNotFound``
        (E105) when the tag is not present."""
        ...
    def get_control_field(self, tag: str) -> str | None: ...
    def to_json(self) -> str: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

@final
class HoldingsRecord:
    """A MARC holdings record. Returned by ``HoldingsMARCReader``."""
    @property
    def leader(self) -> Leader: ...
    @property
    def errors(self) -> list[Exception]:
        """Non-fatal errors accumulated while parsing this record.
        See :attr:`Record.errors`.
        """
        ...
    def record_type(self) -> str:
        """Leader byte 06 (type of record) as a single-character string."""
        ...
    def locations(self) -> list[Field]:
        """Location fields (852)."""
        ...
    def captions_basic(self) -> list[Field]:
        """Basic caption and pattern fields (853)."""
        ...
    def captions_supplements(self) -> list[Field]:
        """Supplementary-material caption and pattern fields (854)."""
        ...
    def captions_indexes(self) -> list[Field]:
        """Index caption and pattern fields (855)."""
        ...
    def enumeration_basic(self) -> list[Field]:
        """Basic enumeration and chronology fields (863)."""
        ...
    def enumeration_supplements(self) -> list[Field]:
        """Supplementary-material enumeration and chronology fields (864)."""
        ...
    def enumeration_indexes(self) -> list[Field]:
        """Index enumeration and chronology fields (865)."""
        ...
    def textual_holdings_basic(self) -> list[Field]:
        """Basic textual holdings fields (866)."""
        ...
    def textual_holdings_supplements(self) -> list[Field]:
        """Supplementary-material textual holdings fields (867)."""
        ...
    def textual_holdings_indexes(self) -> list[Field]:
        """Index textual holdings fields (868)."""
        ...
    def get_fields(self, tag: str) -> list[Field] | None:
        """All fields matching ``tag``, or ``None`` when none match."""
        ...
    def get_field(self, tag: str) -> Field | None: ...
    def get_field_or_err(self, tag: str) -> Field:
        """Get first field with given tag, raising ``mrrc.FieldNotFound``
        (E105) when the tag is not present."""
        ...
    def get_control_field(self, tag: str) -> str | None: ...
    def to_json(self) -> str: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

# =============================================================================
# Query DSL Classes
# =============================================================================

@final
class FieldQuery:
    """Query builder for matching fields by tag, indicators, and subfields."""
    def __init__(self) -> None: ...
    def tag(self, tag: str) -> FieldQuery: ...
    def indicator1(self, indicator: str | None = None) -> FieldQuery: ...
    def indicator2(self, indicator: str | None = None) -> FieldQuery: ...
    def has_subfield(self, code: str) -> FieldQuery: ...
    def has_subfields(self, codes: list[str]) -> FieldQuery: ...
    def tag_range(self, start_tag: str, end_tag: str) -> TagRangeQuery: ...

@final
class TagRangeQuery:
    """Query for fields within a tag range."""
    def __new__(
        cls,
        start_tag: str,
        end_tag: str,
        *,
        indicator1: str | None = None,
        indicator2: str | None = None,
        required_subfields: list[str] | None = None,
    ) -> TagRangeQuery: ...
    @property
    def start_tag(self) -> str: ...
    @property
    def end_tag(self) -> str: ...
    @property
    def indicator1(self) -> str | None: ...
    @property
    def indicator2(self) -> str | None: ...
    def tag_in_range(self, tag: str) -> bool: ...

@final
class SubfieldPatternQuery:
    """Query for fields where a subfield matches a regex pattern."""
    def __new__(
        cls, tag: str, subfield_code: str, pattern: str, *, negate: bool = False
    ) -> SubfieldPatternQuery: ...
    @property
    def tag(self) -> str: ...
    @property
    def subfield_code(self) -> str: ...
    @property
    def pattern(self) -> str:
        """The regex pattern string this query matches against."""
        ...
    @property
    def negate(self) -> bool: ...

@final
class SubfieldValueQuery:
    """Query for fields where a subfield matches a value."""
    def __new__(
        cls,
        tag: str,
        subfield_code: str,
        value: str,
        *,
        partial: bool = False,
        negate: bool = False,
    ) -> SubfieldValueQuery: ...
    @property
    def tag(self) -> str: ...
    @property
    def subfield_code(self) -> str: ...
    @property
    def value(self) -> str: ...
    @property
    def partial(self) -> bool: ...
    @property
    def negate(self) -> bool: ...

# =============================================================================
# ISO 2709 MARC Readers and Writers
# =============================================================================

@final
class MARCReader:
    """Reader for ISO 2709 binary MARC format with GIL-released I/O operations.

    Reads MARC records from file paths, bytes, or file-like objects. Supports iteration
    protocol for easy use in for-loops. GIL is released during record parsing, enabling
    true multi-thread parallelism.

    Accepts multiple input types:
    - str or pathlib.Path: File path (pure Rust I/O, zero GIL overhead)
    - bytes or bytearray: In-memory data
    - file object: Python file-like object (GIL managed)

    Thread Safety:
        NOT thread-safe. Each thread must create its own reader instance.

    Concurrency:
        Use ThreadPoolExecutor with separate readers per thread for 2-3x speedup.
        GIL is released during record parsing (Phase 2).

    Examples:
        Simple sequential reading (path string uses Rust I/O, releases GIL)::

            reader = mrrc.MARCReader('records.mrc')
            for record in reader:
                print(record.title())

        Parallel processing with ThreadPoolExecutor::

            from concurrent.futures import ThreadPoolExecutor

            def process_file(filename):
                reader = mrrc.MARCReader(filename)  # New reader per thread
                return sum(1 for _ in reader)

            with ThreadPoolExecutor(max_workers=4) as executor:
                futures = [executor.submit(process_file, f) for f in file_list]
                results = [f.result() for f in futures]
    """
    def __new__(
        cls,
        source: Any,
        *,
        recovery_mode: str = "permissive",
        validation_level: str = "structural",
        max_errors: int | None = None,
    ) -> MARCReader: ...
    def __repr__(self) -> str: ...
    def __iter__(self) -> Iterator[Record]: ...
    def __next__(self) -> Record:
        """Get next record from the file.

        Returns:
            A Record instance

        Raises:
            StopIteration: When end of file is reached
            ValueError: If the binary data is malformed
        """
        ...
    def read_record(self) -> Record | None:
        """Read the next record from the file.

        Returns:
            A Record instance, or None if EOF reached

        Raises:
            ValueError: If the binary data is malformed
            IOError: If an I/O error occurs
        """
        ...
    @property
    def backend_type(self) -> str:
        """The backend type: ``"rust_file"``, ``"cursor"``, or ``"python_file"``.

        Raises:
            RuntimeError: If the reader has been consumed.
        """
        ...
    @property
    def last_chunk(self) -> bytes | None:
        """Bytes of the most recent record chunk read from the source.

        Populated on every successful chunk read, regardless of whether
        the parse step then succeeds or fails. The ``mrrc.MARCReader``
        Python wrapper exposes this as the pymarc-compatible
        ``current_chunk`` accessor used to diagnose records swallowed
        by ``permissive=True``.
        """
        ...

@final
class MARCWriter:
    """Writer for ISO 2709 binary MARC format.

    Writes MARC records one at a time to a Python file-like object.
    Supports context manager protocol for proper resource cleanup.

    Examples:
        Writing records to a file::

            with open('output.mrc', 'wb') as f:
                with mrrc.MARCWriter(f) as writer:
                    writer.write_record(record)
    """
    def __new__(cls, source: Any) -> MARCWriter: ...
    def __repr__(self) -> str: ...
    def __enter__(self) -> MARCWriter: ...
    def __exit__(
        self, _exc_type: Any = None, _exc_val: Any = None, _exc_tb: Any = None
    ) -> bool: ...
    def write_record(self, record: Record) -> None:
        """Write a record to the file.

        Args:
            record: Record instance to write

        Raises:
            ValueError: If the record is invalid
            IOError: If an I/O error occurs
        """
        ...
    def write(self, record: Record) -> None: ...
    def close(self) -> None:
        """Close the writer and flush the buffer.

        This is automatically called when using the context manager.
        """
        ...

@final
class AuthorityMARCReader:
    """Reader for MARC authority records."""
    def __new__(
        cls,
        source: Any,
        *,
        recovery_mode: str = "permissive",
        validation_level: str = "structural",
    ) -> AuthorityMARCReader: ...
    def __iter__(self) -> Iterator[AuthorityRecord]: ...
    def __next__(self) -> AuthorityRecord: ...
    def __enter__(self) -> AuthorityMARCReader: ...
    def __exit__(self, _exc_type: Any, _exc_val: Any, _exc_tb: Any) -> bool: ...
    def read_record(self) -> AuthorityRecord | None: ...

@final
class HoldingsMARCReader:
    """Reader for MARC holdings records."""
    def __new__(
        cls,
        source: Any,
        *,
        recovery_mode: str = "permissive",
        validation_level: str = "structural",
    ) -> HoldingsMARCReader: ...
    def __iter__(self) -> Iterator[HoldingsRecord]: ...
    def __next__(self) -> HoldingsRecord: ...
    def __enter__(self) -> HoldingsMARCReader: ...
    def __exit__(self, _exc_type: Any, _exc_val: Any, _exc_tb: Any) -> bool: ...
    def read_record(self) -> HoldingsRecord | None: ...

# =============================================================================
# Format Conversion Functions
# =============================================================================

def record_to_json(record: Record) -> str: ...
def json_to_record(json_str: str) -> Record: ...
def record_to_xml(record: Record) -> str: ...
def xml_to_record(xml_str: str) -> Record: ...
def xml_to_records(xml_str: str) -> list[Record]: ...
def record_to_marcjson(record: Record) -> str: ...
def marcjson_to_record(marcjson_str: str) -> Record: ...
def record_to_dublin_core(record: Record) -> str: ...
def record_to_dublin_core_xml(record: Record) -> str: ...
def record_to_mods(record: Record) -> str: ...
def mods_to_record(xml_str: str) -> Record: ...
def mods_collection_to_records(xml_str: str) -> list[Record]: ...
def dublin_core_to_xml(dublin_core: str) -> str: ...
def record_to_csv(record: Record) -> str: ...
def records_to_csv(records: list[Record]) -> str: ...
def records_to_csv_filtered(records: list[Record], filter_fn: Any) -> str: ...

# =============================================================================
# BIBFRAME Conversion (LOC Linked Data Format)
# =============================================================================

@final
class BibframeConfig:
    """Configuration for BIBFRAME conversion.

    Controls how MARC records are converted to BIBFRAME entities and how
    the resulting RDF graph is serialized.

    Examples:
        Default configuration::

            config = mrrc.BibframeConfig()

        Custom configuration::

            config = mrrc.BibframeConfig()
            config.set_base_uri("http://example.org/")
            config.set_output_format("turtle")
            config.set_authority_linking(True)
    """
    def __init__(self) -> None: ...
    def __repr__(self) -> str: ...
    def set_base_uri(self, uri: str) -> None:
        """Set the base URI for generated resources.

        When set, entities are given minted URIs like ``{base}/work/{id}``.
        When not set, blank nodes are used.
        """
        ...
    @property
    def base_uri(self) -> str | None:
        """Get the current base URI."""
        ...
    def set_output_format(self, format: str) -> None:
        """Set the output format for RDF serialization.

        Args:
            format: One of: "rdf-xml", "jsonld", "turtle", "ntriples"

        Raises:
            ValueError: If format is not recognized
        """
        ...
    @property
    def output_format(self) -> str:
        """Get the current output format."""
        ...
    def set_authority_linking(self, enabled: bool) -> None:
        """Enable or disable linking to external authority URIs."""
        ...
    @property
    def authority_linking(self) -> bool:
        """Get the current authority linking setting."""
        ...
    def set_include_bflc(self, enabled: bool) -> None:
        """Enable or disable BFLC extensions."""
        ...
    @property
    def include_bflc(self) -> bool:
        """Get the current BFLC extension setting."""
        ...
    def set_strict(self, enabled: bool) -> None:
        """Enable or disable strict validation mode."""
        ...
    @property
    def strict(self) -> bool:
        """Get the current strict mode setting."""
        ...
    def set_fail_fast(self, enabled: bool) -> None:
        """Enable or disable fail-fast error handling."""
        ...
    @property
    def fail_fast(self) -> bool:
        """Get the current fail-fast setting."""
        ...

@final
class RdfGraph:
    """An RDF graph containing BIBFRAME triples.

    Wraps the RDF graph produced by MARC-to-BIBFRAME conversion
    and provides serialization to various RDF formats.

    Examples:
        ::

            record = mrrc.Record(leader=mrrc.Leader())
            config = mrrc.BibframeConfig()
            graph = mrrc.marc_to_bibframe(record, config)

            print(f"Graph has {len(graph)} triples")
            turtle = graph.serialize("turtle")
    """
    def __init__(self) -> None: ...
    def __repr__(self) -> str: ...
    def __len__(self) -> int:
        """Get the number of triples in the graph."""
        ...
    def is_empty(self) -> bool:
        """Check if the graph is empty."""
        ...
    def serialize(self, format: str) -> str:
        """Serialize the graph to a string in the specified format.

        Args:
            format: One of: "rdf-xml", "jsonld", "turtle", "ntriples"

        Returns:
            The serialized RDF as a string

        Raises:
            ValueError: If format is not recognized or serialization fails
        """
        ...
    @staticmethod
    def parse(data: str, format: str) -> RdfGraph:
        """Parse an RDF graph from a string.

        Args:
            data: The RDF data as a string
            format: One of: "rdf-xml", "jsonld", "turtle", "ntriples"

        Returns:
            A new RdfGraph instance

        Raises:
            ValueError: If format is not recognized or parsing fails
        """
        ...
    def triples(self) -> list[tuple[str, str, str]]:
        """Get all triples as a list of (subject, predicate, object) tuples."""
        ...

def marc_to_bibframe(record: Record, config: BibframeConfig) -> RdfGraph:
    """Convert a MARC record to a BIBFRAME RDF graph.

    Args:
        record: The MARC record to convert
        config: Configuration options for the conversion

    Returns:
        An RdfGraph containing the BIBFRAME representation
    """
    ...

def bibframe_to_marc(graph: RdfGraph) -> Record:
    """Convert a BIBFRAME RDF graph to a MARC record.

    Args:
        graph: The BIBFRAME RDF graph to convert

    Returns:
        A MARC Record representing the BIBFRAME data

    Raises:
        ValueError: If the graph cannot be converted
    """
    ...

# =============================================================================
# Parallel Processing Utilities
# =============================================================================

@final
class RecordBoundaryScanner:
    """Scanner for finding record boundaries in MARC data."""
    def __init__(self) -> None: ...
    def scan(self, data: bytes) -> list[tuple[int, int]]: ...
    def scan_limited(self, data: bytes, limit: int) -> list[tuple[int, int]]: ...
    def count_records(self, data: bytes) -> int: ...
    def clear(self) -> None: ...
    def capacity(self) -> int: ...

@final
class ProducerConsumerPipeline:
    """Pipeline for parallel record processing."""
    def __init__(self) -> None: ...
    @staticmethod
    def from_file(
        path: str,
        buffer_size: int | None = None,
        channel_capacity: int | None = None,
    ) -> ProducerConsumerPipeline: ...
    def next(self) -> Record | None: ...
    def try_next(self) -> Record | None: ...

def parse_batch_parallel(
    boundaries: list[tuple[int, int]],
    buffer: bytes | bytearray,
) -> list[Record]: ...
def parse_batch_parallel_limited(
    boundaries: list[tuple[int, int]],
    buffer: bytes | bytearray,
    limit: int,
) -> list[Record]: ...
