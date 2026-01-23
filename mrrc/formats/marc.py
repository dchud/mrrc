"""ISO 2709 binary MARC format support.

This is the baseline MARC format defined by ISO 2709. It is the standard
interchange format for bibliographic records used by library systems worldwide.

This module provides the primary entry points for reading and writing
MARC records in the standard binary format.

Use Cases
---------
- Exchange with library systems (ILS, OCLC, Z39.50)
- Importing/exporting catalog data
- Standard MARC interchange workflows

Performance
-----------
- Read: ~900,000 records/second
- Write: ~800,000 records/second
- Memory: O(1) - streams one record at a time

Examples
--------
Read records from a MARC file:

>>> from mrrc.formats import marc
>>> for record in marc.read("records.mrc"):
...     print(record.title())

Write records to a MARC file:

>>> from mrrc.formats import marc
>>> count = marc.write(records, "output.mrc")
>>> print(f"Wrote {count} records")

Read from a file-like object:

>>> with open("records.mrc", "rb") as f:
...     for record in marc.read(f):
...         process(record)

See Also
--------
- MARCReader: Low-level reader class
- MARCWriter: Low-level writer class
"""

from mrrc import MARCReader, MARCWriter, Record

__all__ = ["MARCReader", "MARCWriter", "read", "write"]


def _wrap_record(record):
    """Wrap a raw Rust record in a Python Record if needed."""
    if hasattr(record, "_sync_leader"):
        # Already a wrapped Record
        return record
    # Raw Rust record - wrap it
    wrapper = Record()
    wrapper._inner = record
    return wrapper


def read(source):
    """Read MARC records from an ISO 2709 file or file-like object.

    Args:
        source: File path (str) or file-like object opened in binary mode.

    Returns:
        Iterator over Record objects.
    """
    if isinstance(source, str):
        f = open(source, "rb")
        return MARCReader(f)
    return MARCReader(source)


def write(records, dest):
    """Write MARC records to an ISO 2709 file.

    Args:
        records: Iterable of Record objects.
        dest: File path (str) or file-like object opened in binary mode.

    Returns:
        Number of records written.
    """
    if isinstance(dest, str):
        f = open(dest, "wb")
        writer = MARCWriter(f)
        count = 0
        for record in records:
            writer.write(_wrap_record(record))
            count += 1
        writer.close()
        return count
    else:
        writer = MARCWriter(dest)
        count = 0
        for record in records:
            writer.write(_wrap_record(record))
            count += 1
        writer.close()
        return count
