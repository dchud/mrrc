#!/usr/bin/env python3
"""Example: Analyzing MARC records with Polars.

This example demonstrates how to use mrrc.analytics to analyze MARC
records using Polars DataFrames for fast data manipulation.

Requirements:
    pip install polars pyarrow
"""

import mrrc
import mrrc.analytics as analytics

try:
    import polars as pl
except ImportError:
    print("This example requires polars: pip install polars")
    exit(1)


def main():
    # Load sample records
    print("Loading MARC records...")
    records = list(mrrc.read("tests/data/fixtures/1k_records.mrc"))
    print(f"Loaded {len(records)} records\n")

    # Convert to Polars DataFrame
    print("Converting to Polars DataFrame...")
    df = analytics.to_polars(records)
    print(f"DataFrame shape: {df.shape}")
    print(f"Columns: {df.columns}\n")

    # Example 1: Basic statistics
    print("=" * 60)
    print("Example 1: DataFrame overview")
    print("=" * 60)
    print(df.head(5))
    print()

    # Example 2: Field tag distribution
    print("=" * 60)
    print("Example 2: Field tag distribution")
    print("=" * 60)
    tag_counts = (
        df.group_by("field_tag")
        .agg(pl.len().alias("count"))
        .sort("count", descending=True)
        .head(10)
    )
    print(tag_counts)
    print()

    # Example 3: Extract titles
    print("=" * 60)
    print("Example 3: Extract titles (245$a)")
    print("=" * 60)
    titles = (
        df.filter((pl.col("field_tag") == "245") & (pl.col("subfield_code") == "a"))
        .select("record_index", "subfield_value")
        .rename({"subfield_value": "title"})
        .head(5)
    )
    print(titles)
    print()

    # Example 4: Count records per subject
    print("=" * 60)
    print("Example 4: Subject analysis (650$a)")
    print("=" * 60)
    subjects = (
        df.filter((pl.col("field_tag") == "650") & (pl.col("subfield_code") == "a"))
        .group_by("subfield_value")
        .agg(pl.col("record_index").n_unique().alias("num_records"))
        .sort("num_records", descending=True)
        .head(5)
    )
    print(subjects)
    print()

    # Example 5: Join title and author data
    print("=" * 60)
    print("Example 5: Title + Author join")
    print("=" * 60)
    titles_df = (
        df.filter((pl.col("field_tag") == "245") & (pl.col("subfield_code") == "a"))
        .select(pl.col("record_index"), pl.col("subfield_value").alias("title"))
    )
    authors_df = (
        df.filter((pl.col("field_tag") == "100") & (pl.col("subfield_code") == "a"))
        .select(pl.col("record_index"), pl.col("subfield_value").alias("author"))
    )
    combined = titles_df.join(authors_df, on="record_index", how="left").head(5)
    print(combined)
    print()

    # Example 6: Lazy evaluation for large datasets
    print("=" * 60)
    print("Example 6: Lazy evaluation (efficient for large data)")
    print("=" * 60)
    lazy_result = (
        df.lazy()
        .filter(pl.col("field_tag").is_in(["245", "100", "650"]))
        .group_by("field_tag")
        .agg(
            pl.len().alias("total_rows"),
            pl.col("record_index").n_unique().alias("unique_records"),
        )
        .sort("field_tag")
        .collect()
    )
    print(lazy_result)


if __name__ == "__main__":
    main()
