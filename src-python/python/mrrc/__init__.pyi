"""
Type stubs for MRRC - Fast MARC library written in Rust with Python bindings.

This module provides Python access to the Rust MARC library, enabling fast
reading, writing, and manipulation of MARC bibliographic records.
"""

from typing import Any, Iterator

__version__: str

class Leader:
    """MARC Leader - 24-byte record header.

    The MARC leader contains metadata about the record structure and content.
    All MARC records must begin with exactly 24 bytes of leader information.
    """

    def __new__(cls) -> Leader: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
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

class Subfield:
    """A subfield within a MARC field.

    Subfields are named data elements within fields, consisting of a
    single-character code and a value.
    """

    def __new__(cls, code: str, value: str) -> Subfield: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    @property
    def code(self) -> str:
        """Subfield code (single character)"""
        ...

    @property
    def value(self) -> str:
        """Subfield value"""
        ...

    @value.setter
    def value(self, val: str) -> None: ...

class Field:
    """A MARC field (data fields with tag 010 and higher).

    A field consists of a 3-character tag, two indicators, and one or more subfields.
    """

    def __new__(cls, tag: str, indicator1: str = " ", indicator2: str = " ") -> Field: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    @property
    def tag(self) -> str:
        """Field tag (3 digits)"""
        ...

    @property
    def indicator1(self) -> str:
        """First indicator"""
        ...

    @indicator1.setter
    def indicator1(self, value: str) -> None: ...
    @property
    def indicator2(self) -> str:
        """Second indicator"""
        ...

    @indicator2.setter
    def indicator2(self, value: str) -> None: ...
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

class Record:
    """A MARC bibliographic record.

    A MARC record consists of a leader, control fields (000-009), and data fields (010+).
    """

    def __new__(cls, leader: Leader) -> Record: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def leader(self) -> Leader:
        """Get the record's leader."""
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

    def add_field(self, field: Field) -> None:
        """Add a data field to the record.

        Args:
            field: Field instance to add
        """
        ...

    def fields_by_tag(self, tag: str) -> list[Field]:
        """Get all fields with a given tag.

        Args:
            tag: 3-character field tag

        Returns:
            List of Field instances matching the tag
        """
        ...

    def fields(self) -> list[Field]:
        """Get all fields in the record."""
        ...

    def control_fields(self) -> list[tuple[str, str]]:
        """Get all control fields as (tag, value) tuples."""
        ...

    def title(self) -> str | None:
        """Get title from 245 field (first subfield $a).

        Returns:
            The title value, or None if not found
        """
        ...

    def author(self) -> str | None:
        """Get author from 100/110/111 field.

        Returns:
            The author value, or None if not found
        """
        ...

    def isbn(self) -> str | None:
        """Get ISBN from 020 field.

        Returns:
            The ISBN value, or None if not found
        """
        ...

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
        Sharing a reader across threads causes undefined behavior.

    Concurrency:
        Use ThreadPoolExecutor with separate readers per thread for 2-3x speedup:
        - 2 threads: 2.04x speedup vs sequential
        - 4 threads: 3.20x speedup vs sequential
        - GIL is released during Phase 2 (record parsing)

    Examples:
        Simple sequential reading:

        ```python
        import mrrc

        with open('records.mrc', 'rb') as f:
            reader = mrrc.MARCReader(f)
            for record in reader:
                print(record.title())
        ```

        Parallel processing with ThreadPoolExecutor:

        ```python
        from concurrent.futures import ThreadPoolExecutor
        
        def process_file(filename):
            count = 0
            with open(filename, 'rb') as f:
                reader = mrrc.MARCReader(f)  # New reader per thread
                for record in reader:
                    count += 1
            return count
        
        with ThreadPoolExecutor(max_workers=4) as executor:
            futures = [executor.submit(process_file, f) for f in file_list]
            results = [f.result() for f in futures]
            # Expected: 3-4x faster than sequential on 4-core system
        ```
    """

    def __new__(cls, file: Any) -> MARCReader: ...
    def __repr__(self) -> str: ...
    def __iter__(self) -> Iterator[Record]: ...
    def __next__(self) -> Record:
        """Get next record from the file.

        Implements three-phase GIL release for parallelism:
        - Phase 1: Read record bytes (GIL held if needed)
        - Phase 2: Parse MARC record (GIL released)
        - Phase 3: Convert to Python Record (GIL re-acquired)

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

class MARCWriter:
    """Writer for ISO 2709 binary MARC format.

    Writes MARC records one at a time to a Python file-like object.
    Supports context manager protocol for proper resource cleanup.

    Examples:
        Writing records to a file:

        ```python
        import mrrc

        with open('output.mrc', 'wb') as f:
            with mrrc.MARCWriter(f) as writer:
                writer.write_record(record)
        ```
    """

    def __new__(cls, file: Any) -> MARCWriter: ...
    def __repr__(self) -> str: ...
    def __enter__(self) -> MARCWriter: ...
    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> bool: ...
    def write_record(self, record: Record) -> None:
        """Write a record to the file.

        Serializes the record to ISO 2709 binary format.

        Args:
            record: Record instance to write

        Raises:
            ValueError: If the record is invalid
            IOError: If an I/O error occurs
        """
        ...

    def close(self) -> None:
        """Close the writer and flush the buffer.

        This is automatically called when using the context manager.
        """
        ...

# =============================================================================
# BIBFRAME Conversion (LOC Linked Data Format)
# =============================================================================

class BibframeConfig:
    """Configuration for BIBFRAME conversion.

    Controls how MARC records are converted to BIBFRAME entities and how
    the resulting RDF graph is serialized.

    Examples:
        Default configuration:

        ```python
        import mrrc

        config = mrrc.BibframeConfig()
        ```

        Custom configuration:

        ```python
        config = mrrc.BibframeConfig()
        config.set_base_uri("http://example.org/")
        config.set_output_format("turtle")
        config.set_authority_linking(True)
        ```
    """

    def __new__(cls) -> BibframeConfig: ...
    def __repr__(self) -> str: ...
    def set_base_uri(self, uri: str) -> None:
        """Set the base URI for generated resources.

        When set, entities are given minted URIs like `{base}/work/{id}`.
        When not set, blank nodes are used.

        Args:
            uri: The base URI string
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
        """Enable or disable linking to external authority URIs.

        When True, agents and subjects with identifiable authority control
        numbers link to external URIs like http://id.loc.gov/authorities/names/.

        Args:
            enabled: Whether to enable authority linking
        """
        ...

    @property
    def authority_linking(self) -> bool:
        """Get the current authority linking setting."""
        ...

    def set_include_bflc(self, enabled: bool) -> None:
        """Enable or disable BFLC extensions.

        BFLC extensions are required for practical LOC compatibility.

        Args:
            enabled: Whether to include BFLC extensions
        """
        ...

    @property
    def include_bflc(self) -> bool:
        """Get the current BFLC extension setting."""
        ...

    def set_strict(self, enabled: bool) -> None:
        """Enable or disable strict validation mode.

        When True, questionable data causes errors.
        When False (default), best-effort conversion is attempted.

        Args:
            enabled: Whether to enable strict mode
        """
        ...

    @property
    def strict(self) -> bool:
        """Get the current strict mode setting."""
        ...

    def set_fail_fast(self, enabled: bool) -> None:
        """Enable or disable fail-fast error handling.

        When True, conversion stops at the first error.
        When False (default), errors are collected and conversion continues.

        Args:
            enabled: Whether to enable fail-fast mode
        """
        ...

    @property
    def fail_fast(self) -> bool:
        """Get the current fail-fast setting."""
        ...

class RdfGraph:
    """An RDF graph containing BIBFRAME triples.

    This class wraps the RDF graph produced by MARC-to-BIBFRAME conversion
    and provides serialization to various RDF formats.

    Examples:
        ```python
        import mrrc

        record = mrrc.Record(leader=mrrc.Leader())
        config = mrrc.BibframeConfig()
        graph = mrrc.marc_to_bibframe(record, config)

        # Get number of triples
        print(f"Graph has {len(graph)} triples")

        # Serialize to different formats
        rdf_xml = graph.serialize("rdf-xml")
        jsonld = graph.serialize("jsonld")
        turtle = graph.serialize("turtle")
        ```
    """

    def __new__(cls) -> RdfGraph: ...
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
        """Get all triples as a list of (subject, predicate, object) tuples.

        Returns:
            A list of tuples where each tuple is (subject_str, predicate_str, object_str)
        """
        ...

def marc_to_bibframe(record: Record, config: BibframeConfig) -> RdfGraph:
    """Convert a MARC record to a BIBFRAME RDF graph.

    This function transforms a MARC bibliographic record into a BIBFRAME 2.0
    RDF graph containing Work, Instance, and optionally Item entities.

    Args:
        record: The MARC record to convert
        config: Configuration options for the conversion

    Returns:
        An RdfGraph containing the BIBFRAME representation

    Examples:
        ```python
        import mrrc

        record = mrrc.Record(leader=mrrc.Leader())
        record.add_control_field("001", "12345")
        record.add_control_field("008", "040520s2023    xxu           000 0 eng  ")

        config = mrrc.BibframeConfig()
        config.set_base_uri("http://example.org/")

        graph = mrrc.marc_to_bibframe(record, config)
        print(graph.serialize("jsonld"))
        ```
    """
    ...

def bibframe_to_marc(graph: RdfGraph) -> Record:
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

    Examples:
        ```python
        import mrrc

        # Round-trip conversion
        record = mrrc.Record(leader=mrrc.Leader())
        config = mrrc.BibframeConfig()
        graph = mrrc.marc_to_bibframe(record, config)
        recovered = mrrc.bibframe_to_marc(graph)
        ```
    """
    ...
