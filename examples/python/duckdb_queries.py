#!/usr/bin/env python3
"""Example: Querying MARC records with DuckDB.

This example demonstrates how to use mrrc.analytics to query MARC
records using SQL with DuckDB.

Requirements:
    pip install duckdb pyarrow
"""

import mrrc
import mrrc.analytics as analytics


def main():
    # Load sample records
    print("Loading MARC records...")
    records = list(mrrc.read("tests/data/fixtures/1k_records.mrc"))
    print(f"Loaded {len(records)} records\n")

    # Create DuckDB connection with records
    print("Creating DuckDB table...")
    conn = analytics.to_duckdb(records)

    # Example 1: Count unique records
    print("=" * 60)
    print("Query 1: Count unique records")
    print("=" * 60)
    result = conn.execute("""
        SELECT COUNT(DISTINCT record_index) as num_records
        FROM marc_records
    """).fetchone()
    print(f"Total unique records: {result[0]}\n")

    # Example 2: Field tag distribution
    print("=" * 60)
    print("Query 2: Top 10 field tags by frequency")
    print("=" * 60)
    results = conn.execute("""
        SELECT
            field_tag,
            COUNT(*) as count,
            COUNT(DISTINCT record_index) as records_with_field
        FROM marc_records
        GROUP BY field_tag
        ORDER BY count DESC
        LIMIT 10
    """).fetchall()
    print(f"{'Tag':<6} {'Count':<10} {'Records':<10}")
    print("-" * 26)
    for tag, count, records in results:
        print(f"{tag:<6} {count:<10} {records:<10}")
    print()

    # Example 3: Find titles (field 245, subfield a)
    print("=" * 60)
    print("Query 3: Sample titles (field 245$a)")
    print("=" * 60)
    results = conn.execute("""
        SELECT DISTINCT subfield_value
        FROM marc_records
        WHERE field_tag = '245' AND subfield_code = 'a'
        LIMIT 5
    """).fetchall()
    for (title,) in results:
        print(f"  - {title[:60]}...")
    print()

    # Example 4: Records with specific subject (650 field)
    print("=" * 60)
    print("Query 4: Subject headings (field 650)")
    print("=" * 60)
    results = conn.execute("""
        SELECT
            subfield_value,
            COUNT(DISTINCT record_index) as num_records
        FROM marc_records
        WHERE field_tag = '650' AND subfield_code = 'a'
        GROUP BY subfield_value
        ORDER BY num_records DESC
        LIMIT 5
    """).fetchall()
    for subject, count in results:
        print(f"  {subject}: {count} records")
    print()

    # Example 5: Using query_duckdb() shortcut
    print("=" * 60)
    print("Query 5: Using query_duckdb() shortcut")
    print("=" * 60)
    # One-liner for quick queries
    pub_years = analytics.query_duckdb(
        records,
        """
        SELECT
            SUBSTR(subfield_value, 8, 4) as pub_year,
            COUNT(*) as count
        FROM marc_records
        WHERE field_tag = '008' AND LENGTH(subfield_value) >= 11
        GROUP BY pub_year
        ORDER BY count DESC
        LIMIT 5
        """
    )
    print("Top publication years (from 008 field):")
    for year, count in pub_years:
        if year and year.strip():
            print(f"  {year}: {count}")


if __name__ == "__main__":
    main()
