"""
MRRC: A fast MARC library written in Rust with Python bindings.

This package provides Python bindings to the Rust mrrc library,
enabling high-performance MARC data processing.

Classes:
    Leader: MARC record leader (24-byte header)
    Subfield: Named data element within a field
    Field: MARC field (tag, indicators, subfields)
    Record: Complete MARC bibliographic record
    MARCReader: Stream reader for MARC binary files
    MARCWriter: Stream writer for MARC binary files

Functions:
    record_to_json: Convert Record to JSON
    json_to_record: Parse JSON back to Record
    record_to_xml: Convert Record to XML
    xml_to_record: Parse XML back to Record
    record_to_marcjson: Convert Record to MARCJSON format
    marcjson_to_record: Parse MARCJSON back to Record
    record_to_dublin_core: Extract Dublin Core metadata
    record_to_mods: Convert Record to MODS XML format
    dublin_core_to_xml: Convert Dublin Core dict to XML

Example:
    >>> from mrrc import Leader, Record, Field
    >>> leader = Leader()
    >>> leader.set_record_type('a')
    >>> record = Record(leader)
    >>> field = Field('245', '1', '0')
    >>> field.add_subfield('a', 'Title')
    >>> record.add_field(field)
    >>> json_str = record.to_json()
"""

__version__ = "0.1.0"
__author__ = "MRRC Contributors"

from ._mrrc import (
    Leader,
    Subfield,
    Field,
    Record,
    MARCReader,
    MARCWriter,
    record_to_json,
    json_to_record,
    record_to_xml,
    xml_to_record,
    record_to_marcjson,
    marcjson_to_record,
    record_to_dublin_core,
    record_to_mods,
    dublin_core_to_xml,
)

__all__ = [
    "Leader",
    "Subfield",
    "Field",
    "Record",
    "MARCReader",
    "MARCWriter",
    "record_to_json",
    "json_to_record",
    "record_to_xml",
    "xml_to_record",
    "record_to_marcjson",
    "marcjson_to_record",
    "record_to_dublin_core",
    "record_to_mods",
    "dublin_core_to_xml",
]
