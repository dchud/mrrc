"""Type stubs for the mrrc native extension module."""

from typing import Iterator, Optional, List, BinaryIO, Any

# =============================================================================
# Core Data Types
# =============================================================================

class Subfield:
    """A subfield within a MARC field.

    Subfields are named data elements within fields, consisting of a
    single-character code and a value.
    """
    code: str
    value: str
    def __init__(self, code: str, value: str) -> None: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

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
    def __eq__(self, other: object) -> bool: ...
    @staticmethod
    def get_valid_values(position: int) -> Optional[dict[str, str]]: ...
    @staticmethod
    def describe_value(position: int, value: str) -> Optional[str]: ...
    @staticmethod
    def is_valid_value(position: int, value: str) -> bool: ...

class Field:
    """A MARC data field (tags 010 and higher).

    A field consists of a 3-character tag, two indicators, and one or more subfields.
    """
    tag: str
    indicator1: str
    indicator2: str

    def __init__(self, tag: str, indicator1: str = " ", indicator2: str = " ", *, subfields: list[Subfield] | None = None, indicators: list[str] | None = None) -> None: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def add_subfield(self, code: str, value: str) -> None:
        """Add a subfield to the field.

        Args:
            code: Subfield code (single character)
            value: Subfield value

        Raises:
            ValueError: If code is empty
        """
        ...
    def subfields(self) -> List[Subfield]:
        """Get all subfields in this field."""
        ...
    def subfields_by_code(self, code: str) -> List[str]:
        """Get all subfield values for a given code.

        Args:
            code: Subfield code to search for

        Returns:
            List of subfield values matching the code

        Raises:
            ValueError: If code is empty
        """
        ...

class Record:
    """A MARC bibliographic record.

    A MARC record consists of a leader, control fields (000-009), and data fields (010+).
    """
    def __init__(self, leader: Optional[Leader] = None, *, fields: list[Field] | None = None) -> None: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def leader(self) -> Leader:
        """Get the record's leader."""
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
    def control_field(self, tag: str) -> Optional[str]:
        """Get a control field value.

        Args:
            tag: 3-character field tag

        Returns:
            The field value, or None if not found
        """
        ...
    def control_fields(self) -> List[tuple[str, str]]:
        """Get all control fields as (tag, value) tuples."""
        ...
    def get_field(self, tag: str) -> Optional[Field]: ...
    def get_fields(self, tag: str) -> List[Field]: ...
    def fields(self) -> List[Field]:
        """Get all fields in the record."""
        ...
    def remove_field(self, tag: str) -> None: ...
    def fields_by_indicator(
        self, tag: str, *, indicator1: Optional[str] = None, indicator2: Optional[str] = None
    ) -> List[Field]: ...
    def fields_in_range(self, start_tag: str, end_tag: str) -> List[Field]: ...
    def fields_matching(self, query: FieldQuery) -> List[Field]: ...
    def fields_matching_range(self, query: TagRangeQuery) -> List[Field]: ...
    def fields_matching_pattern(self, query: SubfieldPatternQuery) -> List[Field]: ...
    def fields_matching_value(self, query: SubfieldValueQuery) -> List[Field]: ...
    def title(self) -> Optional[str]:
        """Get title from 245 field (first subfield $a)."""
        ...
    def author(self) -> Optional[str]:
        """Get author from 100/110/111 field."""
        ...
    def isbn(self) -> Optional[str]:
        """Get ISBN from 020 field."""
        ...
    def issn(self) -> Optional[str]: ...
    def subjects(self) -> List[str]: ...
    def location(self) -> List[str]: ...
    def notes(self) -> List[str]: ...
    def publisher(self) -> Optional[str]: ...
    def uniform_title(self) -> Optional[str]: ...
    def sudoc(self) -> Optional[str]: ...
    def issn_title(self) -> Optional[str]: ...
    def issnl(self) -> Optional[str]: ...
    def pubyear(self) -> Optional[int]: ...
    def series(self) -> Optional[str]: ...
    def physical_description(self) -> Optional[str]: ...
    def is_book(self) -> bool: ...
    def is_serial(self) -> bool: ...
    def is_music(self) -> bool: ...
    def is_audiovisual(self) -> bool: ...
    def to_json(self) -> str: ...
    def to_xml(self) -> str: ...
    def to_dublin_core(self) -> str: ...
    def to_marcjson(self) -> str: ...

class AuthorityRecord:
    """A MARC authority record."""
    def leader(self) -> Leader: ...
    def fields(self) -> List[Field]: ...
    def get_field(self, tag: str) -> Optional[Field]: ...
    def get_fields(self, tag: str) -> List[Field]: ...
    def control_field(self, tag: str) -> Optional[str]: ...

class HoldingsRecord:
    """A MARC holdings record."""
    def leader(self) -> Leader: ...
    def fields(self) -> List[Field]: ...
    def get_field(self, tag: str) -> Optional[Field]: ...
    def get_fields(self, tag: str) -> List[Field]: ...
    def control_field(self, tag: str) -> Optional[str]: ...

# =============================================================================
# Query DSL Classes
# =============================================================================

class FieldQuery:
    """Query builder for matching fields by tag, indicators, and subfields."""
    def __init__(self) -> None: ...
    def tag(self, tag: str) -> FieldQuery: ...
    def indicator1(self, value: str) -> FieldQuery: ...
    def indicator2(self, value: str) -> FieldQuery: ...
    def has_subfield(self, code: str) -> FieldQuery: ...

class TagRangeQuery:
    """Query for fields within a tag range."""
    def __init__(
        self,
        start_tag: str,
        end_tag: str,
        *,
        indicator1: Optional[str] = None,
        indicator2: Optional[str] = None,
        required_subfields: Optional[List[str]] = None,
    ) -> None: ...

class SubfieldPatternQuery:
    """Query for fields where a subfield matches a regex pattern."""
    def __init__(self, tag: str, subfield_code: str, pattern: str) -> None: ...

class SubfieldValueQuery:
    """Query for fields where a subfield matches a value."""
    def __init__(
        self, tag: str, subfield_code: str, value: str, *, partial: bool = False
    ) -> None: ...

# =============================================================================
# ISO 2709 MARC Readers and Writers
# =============================================================================

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
        Simple sequential reading::

            with open('records.mrc', 'rb') as f:
                reader = mrrc.MARCReader(f)
                for record in reader:
                    print(record.title())

        Parallel processing with ThreadPoolExecutor::

            from concurrent.futures import ThreadPoolExecutor

            def process_file(filename):
                with open(filename, 'rb') as f:
                    reader = mrrc.MARCReader(f)  # New reader per thread
                    return sum(1 for _ in reader)

            with ThreadPoolExecutor(max_workers=4) as executor:
                futures = [executor.submit(process_file, f) for f in file_list]
                results = [f.result() for f in futures]
    """
    def __init__(self, file: Any) -> None: ...
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
    def read_record(self) -> Optional[Record]:
        """Read the next record from the file.

        Returns:
            A Record instance, or None if EOF reached

        Raises:
            ValueError: If the binary data is malformed
            IOError: If an I/O error occurs
        """
        ...
    def records_read(self) -> int: ...

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
    def __init__(self, file: Any) -> None: ...
    def __repr__(self) -> str: ...
    def __enter__(self) -> MARCWriter: ...
    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> bool: ...
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
    def records_written(self) -> int: ...

class AuthorityMARCReader:
    """Reader for MARC authority records."""
    def __init__(self, file: BinaryIO) -> None: ...
    def __iter__(self) -> Iterator[AuthorityRecord]: ...
    def __next__(self) -> AuthorityRecord: ...

class HoldingsMARCReader:
    """Reader for MARC holdings records."""
    def __init__(self, file: BinaryIO) -> None: ...
    def __iter__(self) -> Iterator[HoldingsRecord]: ...
    def __next__(self) -> HoldingsRecord: ...

# =============================================================================
# Format Conversion Functions
# =============================================================================

def record_to_json(record: Record) -> str: ...
def json_to_record(json_str: str) -> Record: ...
def record_to_xml(record: Record) -> str: ...
def xml_to_record(xml_str: str) -> Record: ...
def record_to_marcjson(record: Record) -> str: ...
def marcjson_to_record(json_str: str) -> Record: ...
def record_to_dublin_core(record: Record) -> str: ...
def record_to_dublin_core_xml(record: Record) -> str: ...
def record_to_mods(record: Record) -> str: ...
def mods_to_record(xml_str: str) -> Record: ...
def mods_collection_to_records(xml_str: str) -> list[Record]: ...
def dublin_core_to_xml(dc_str: str) -> str: ...
def record_to_csv(record: Record, fields: List[str]) -> str: ...
def records_to_csv(records: List[Record], fields: List[str]) -> str: ...
def records_to_csv_filtered(
    records: List[Record], fields: List[str], filter_fn: Any
) -> str: ...

# =============================================================================
# BIBFRAME Conversion (LOC Linked Data Format)
# =============================================================================

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
    def with_base_uri(self, uri: str) -> BibframeConfig: ...
    def with_output_format(self, format: str) -> BibframeConfig: ...
    def with_authority_linking(self, enabled: bool) -> BibframeConfig: ...
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
    def len(self) -> int: ...
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

class RecordBoundaryScanner:
    """Scanner for finding record boundaries in MARC data."""
    def __init__(self, data: bytes) -> None: ...
    def scan(self) -> List[tuple[int, int]]: ...

class ProducerConsumerPipeline:
    """Pipeline for parallel record processing."""
    def __init__(self, num_workers: int = 4) -> None: ...

def parse_batch_parallel(data: bytes) -> List[Record]: ...
def parse_batch_parallel_limited(data: bytes, max_records: int) -> List[Record]: ...
