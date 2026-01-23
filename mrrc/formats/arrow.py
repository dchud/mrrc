"""Apache Arrow IPC format support (Tier 2).

Arrow provides columnar data representation ideal for analytics
integration with tools like DuckDB, Polars, Pandas, and DataFusion.

Arrow stores data in a columnar format optimized for analytical queries,
enabling efficient SQL-like operations over MARC collections.

Use Cases
---------
- Large-scale MARC analytics (field frequency, pattern analysis)
- SQL queries over MARC data via DuckDB
- DataFrame operations with Polars or Pandas
- Data science workflows and machine learning pipelines

Performance
-----------
- Read: ~865,000 records/second
- Write: ~800,000 records/second
- Size: ~96% smaller than ISO 2709 (excellent compression)

Examples
--------
Read records from an Arrow file:

>>> from mrrc.formats import arrow
>>> for record in arrow.read("records.arrow"):
...     print(record.title())

Write records to an Arrow file:

>>> from mrrc.formats import arrow
>>> count = arrow.write(records, "output.arrow")

Export to Parquet for analytics tools:

>>> from mrrc.formats import arrow
>>> arrow.export_to_parquet("records.arrow", "records.parquet")

Query with DuckDB:

>>> import duckdb
>>> conn = duckdb.connect()
>>> result = conn.execute('''
...     SELECT title, author FROM "records.parquet"
...     WHERE publication_year > 2000
... ''').fetchall()

See Also
--------
- ArrowReader: Streaming reader class
- ArrowWriter: Streaming writer class
- export_to_parquet: Convert to Parquet format
"""

from mrrc import ArrowReader, ArrowWriter

__all__ = [
    "ArrowReader",
    "ArrowWriter",
    "read",
    "write",
    "export_to_parquet",
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


def export_to_parquet(source, dest, compression="snappy"):
    """Export MARC records from Arrow IPC to Parquet format.

    Converts an Arrow IPC file to Parquet format for use with analytics
    tools like DuckDB, Polars, Spark, and other data lake systems.

    Args:
        source: Source Arrow IPC file path (str) or file with MARC records.
        dest: Destination Parquet file path (str).
        compression: Parquet compression codec. Options: "snappy" (default),
            "gzip", "brotli", "lz4", "zstd", or None for no compression.

    Returns:
        Number of records exported.

    Raises:
        ImportError: If pyarrow is not installed.

    Example:
        >>> mrrc.formats.arrow.export_to_parquet("records.arrow", "records.parquet")
        1000
    """
    try:
        import pyarrow as pa
        import pyarrow.parquet as pq
    except ImportError:
        raise ImportError(
            "pyarrow is required for Parquet export. "
            "Install with: pip install pyarrow"
        )

    # Read the Arrow IPC stream
    with pa.ipc.open_stream(source) as reader:
        table = reader.read_all()

    # Write to Parquet
    pq.write_table(table, dest, compression=compression)

    return table.num_rows
