"""ISO 2709 binary MARC format support.

This is the baseline MARC format defined by ISO 2709. It is the standard
interchange format for bibliographic records.
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
