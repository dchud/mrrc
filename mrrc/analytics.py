"""Analytics helpers for MARC data analysis.

This module provides integration with analytics tools like DuckDB and Polars
for querying and analyzing MARC record data via Arrow format.
"""

import tempfile
import os
from typing import Optional, List, Any

__all__ = [
    "to_duckdb",
    "to_polars",
    "query_duckdb",
    "to_arrow_table",
]


def to_arrow_table(records):
    """Convert MARC records to a PyArrow Table.

    Args:
        records: Iterable of Record objects.

    Returns:
        pyarrow.Table containing the flattened record data.

    Raises:
        ImportError: If pyarrow is not installed.
    """
    try:
        import pyarrow as pa
    except ImportError:
        raise ImportError(
            "pyarrow is required for Arrow table conversion. "
            "Install with: pip install pyarrow"
        )

    from mrrc import ArrowWriter

    # Write records to a temporary Arrow IPC file
    with tempfile.NamedTemporaryFile(suffix=".arrow", delete=False) as f:
        temp_path = f.name

    try:
        writer = ArrowWriter(temp_path)
        for record in records:
            inner = record._inner if hasattr(record, "_inner") else record
            writer.write_record(inner)
        writer.close()

        # Read back as Arrow table
        with pa.ipc.open_stream(temp_path) as reader:
            return reader.read_all()
    finally:
        os.unlink(temp_path)


def to_duckdb(records, table_name: str = "marc_records", connection=None):
    """Load MARC records into a DuckDB table.

    Creates a DuckDB table from MARC records for SQL querying. The table
    contains denormalized data with columns for record, field, and subfield
    information.

    Args:
        records: Iterable of Record objects.
        table_name: Name for the DuckDB table (default: "marc_records").
        connection: Optional existing DuckDB connection. If None, creates
            an in-memory database.

    Returns:
        DuckDB connection with the loaded table.

    Raises:
        ImportError: If duckdb is not installed.

    Example:
        >>> conn = mrrc.analytics.to_duckdb(records)
        >>> result = conn.execute("SELECT DISTINCT field_tag FROM marc_records").fetchall()
    """
    try:
        import duckdb
    except ImportError:
        raise ImportError(
            "duckdb is required for DuckDB integration. "
            "Install with: pip install duckdb"
        )

    # Convert to Arrow table
    table = to_arrow_table(records)

    # Create connection if not provided
    if connection is None:
        connection = duckdb.connect(":memory:")

    # Register the Arrow table
    connection.register(table_name, table)

    return connection


def query_duckdb(records, sql: str, table_name: str = "marc_records"):
    """Query MARC records using DuckDB SQL.

    Convenience function that loads records and executes a query in one step.

    Args:
        records: Iterable of Record objects.
        sql: SQL query string. Use table_name to reference the data.
        table_name: Name for the temporary table (default: "marc_records").

    Returns:
        Query results as a list of tuples.

    Raises:
        ImportError: If duckdb is not installed.

    Example:
        >>> results = mrrc.analytics.query_duckdb(
        ...     records,
        ...     "SELECT COUNT(DISTINCT record_index) FROM marc_records"
        ... )
    """
    conn = to_duckdb(records, table_name)
    return conn.execute(sql).fetchall()


def to_polars(records):
    """Convert MARC records to a Polars DataFrame.

    Creates a Polars DataFrame from MARC records for data analysis.
    The DataFrame contains denormalized data with columns for record,
    field, and subfield information.

    Args:
        records: Iterable of Record objects.

    Returns:
        polars.DataFrame containing the record data.

    Raises:
        ImportError: If polars is not installed.

    Example:
        >>> df = mrrc.analytics.to_polars(records)
        >>> titles = df.filter(pl.col("field_tag") == "245")
    """
    try:
        import polars as pl
    except ImportError:
        raise ImportError(
            "polars is required for Polars integration. "
            "Install with: pip install polars"
        )

    # Convert to Arrow table first
    table = to_arrow_table(records)

    # Convert Arrow table to Polars DataFrame
    return pl.from_arrow(table)
