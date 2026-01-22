"""Apache Arrow IPC format support (Tier 2).

Arrow provides columnar data representation ideal for analytics
integration with tools like DuckDB, Polars, and DataFusion.
"""

from mrrc import ArrowReader, ArrowWriter

__all__ = [
    "ArrowReader",
    "ArrowWriter",
    "read",
    "write",
]


def read(source):
    """Read MARC records from an Arrow IPC file or bytes.

    Args:
        source: File path (str) or bytes containing Arrow IPC data.

    Returns:
        Iterator over Record objects.
    """
    return ArrowReader(source)


def write(records, dest):
    """Write MARC records to an Arrow IPC file.

    Args:
        records: Iterable of Record objects.
        dest: File path (str).

    Returns:
        Number of records written.
    """
    writer = ArrowWriter(dest)
    count = 0
    for record in records:
        inner = record._inner if hasattr(record, "_inner") else record
        writer.write_record(inner)
        count += 1
    writer.close()
    return count
