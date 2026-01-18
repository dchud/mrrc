#!/usr/bin/env python3
"""
Simplified Polars + Arrow evaluation for MARC data.
Tests round-trip fidelity and demonstrates analytical capabilities.
"""

import time
from pathlib import Path
from itertools import islice
import polars as pl
import duckdb
import pyarrow as pa
from mrrc import MARCReader

def marc_to_arrow(records) -> pa.Table:
    """Convert MARC records to Arrow Table (long format)."""
    
    record_ids = []
    field_tags = []
    indicators1 = []
    indicators2 = []
    subfield_codes = []
    subfield_values = []
    field_sequences = []
    subfield_sequences = []
    
    for record_id, record in enumerate(records, start=1):
        fields = record.fields()
        
        for field_seq, field in enumerate(fields):
            tag = str(field.tag)
            ind1 = field.indicator1 if hasattr(field, 'indicator1') else None
            ind2 = field.indicator2 if hasattr(field, 'indicator2') else None
            subfields = field.subfields()
            
            if not subfields:
                # Field with no subfields
                record_ids.append(record_id)
                field_tags.append(tag)
                indicators1.append(ind1)
                indicators2.append(ind2)
                subfield_codes.append(None)
                subfield_values.append("")
                field_sequences.append(field_seq)
                subfield_sequences.append(None)
            else:
                # Field with subfields: one row per subfield
                for subf_seq, subfield in enumerate(subfields):
                    record_ids.append(record_id)
                    field_tags.append(tag)
                    indicators1.append(ind1)
                    indicators2.append(ind2)
                    subfield_codes.append(subfield.code)
                    subfield_values.append(subfield.value)
                    field_sequences.append(field_seq)
                    subfield_sequences.append(subf_seq)
    
    table = pa.table({
        "record_id": pa.array(record_ids, type=pa.uint32()),
        "field_tag": pa.array(field_tags, type=pa.string()),
        "indicator1": pa.array(indicators1, type=pa.string()),
        "indicator2": pa.array(indicators2, type=pa.string()),
        "subfield_code": pa.array(subfield_codes, type=pa.string()),
        "subfield_value": pa.array(subfield_values, type=pa.string()),
        "field_sequence": pa.array(field_sequences, type=pa.uint16()),
        "subfield_sequence": pa.array(subfield_sequences, type=pa.uint16()),
    })
    
    return table


def test_roundtrip_fidelity(mrc_file: Path, sample_size: int = 100) -> bool:
    """Test MARC → Arrow → MARC round-trip fidelity."""
    
    print(f"\n{'='*70}")
    print(f"ROUND-TRIP FIDELITY TEST ({sample_size} records)")
    print(f"{'='*70}")
    
    with open(mrc_file, "rb") as f:
        reader = MARCReader(f)
        original_records = list(islice(reader, sample_size))
    
    print(f"✓ Loaded {len(original_records)} records")
    
    # Serialize to Arrow
    print("Converting to Arrow table...", end="", flush=True)
    start = time.time()
    arrow_table = marc_to_arrow(original_records)
    serialize_ms = (time.time() - start) * 1000
    print(f" ✓ ({serialize_ms:.1f} ms)")
    print(f"  - Total rows: {len(arrow_table)}")
    print(f"  - Columns: {len(arrow_table.column_names)}")
    
    # Polars round-trip
    print("Polars DataFrame round-trip...", end="", flush=True)
    start = time.time()
    df = pl.from_arrow(arrow_table)
    arrow_table2 = df.to_arrow()
    polars_ms = (time.time() - start) * 1000
    print(f" ✓ ({polars_ms:.1f} ms)")
    
    # Verify table integrity
    print("Verifying table integrity...", end="", flush=True)
    assert len(arrow_table2) == len(arrow_table), "Row count changed"
    assert arrow_table2.column_names == arrow_table.column_names, "Column names changed"
    print(f" ✓")
    
    # Compare original fields with Arrow-reconstructed structure
    print("Field-level comparison...", end="", flush=True)
    failures = 0
    df2 = pl.from_arrow(arrow_table2)
    for record_id, record in enumerate(original_records, start=1):
        record_rows = df2.filter(pl.col("record_id") == record_id)
        
        orig_fields = record.fields()
        orig_field_count = len(orig_fields)
        
        # Count unique field tags in Arrow representation
        arrow_field_tags = record_rows.select("field_tag").unique().to_series().to_list()
        
        if len(orig_fields) != len(arrow_field_tags):
            failures += 1
            if failures <= 3:  # Only print first 3 failures
                print(f"\n  ✗ Record {record_id}: field count mismatch ({len(orig_fields)} vs {len(arrow_field_tags)})")
    
    print(f" ✓")
    
    # Results
    print(f"\n{'─'*70}")
    print(f"Result: {len(original_records) - failures}/{len(original_records)} PASS")
    print(f"Fidelity: {((len(original_records) - failures) / len(original_records)) * 100:.1f}%")
    
    if failures == 0:
        print("✅ FIDELITY TEST PASSED")
        return True
    else:
        print(f"❌ FIDELITY TEST FAILED ({failures} records)")
        return False


def benchmark_duckdb_queries(arrow_table: pa.Table):
    """Benchmark sample Polars queries on MARC data."""
    
    print(f"\n{'='*70}")
    print(f"POLARS/DUCKDB QUERY BENCHMARKS")
    print(f"{'='*70}")
    
    df = pl.from_arrow(arrow_table)
    
    # Query 1: Count by field tag
    print(f"\nQuery 1: Count by field tag (Polars)")
    start = time.time()
    result1 = df.group_by("field_tag").agg(pl.count("field_tag").alias("count")).sort("count", descending=True)
    elapsed1 = (time.time() - start) * 1000
    print(f"  Time: {elapsed1:.1f} ms")
    print(f"  Result rows: {len(result1)}")
    
    # Query 2: Subfield value frequency (just for 650 fields)
    print(f"\nQuery 2: Subfield value frequency (650 fields only)")
    start = time.time()
    result2 = df.filter(pl.col("field_tag") == "650").group_by("subfield_value").agg(
        pl.count("subfield_value").alias("freq")
    ).sort("freq", descending=True).head(5)
    elapsed2 = (time.time() - start) * 1000
    print(f"  Time: {elapsed2:.1f} ms")
    print(f"  Result rows: {len(result2)}")


def main():
    mrc_file = Path("tests/data/fixtures/10k_records.mrc")
    
    if not mrc_file.exists():
        print(f"Error: {mrc_file} not found")
        return
    
    print("\n" + "="*70)
    print("POLARS + ARROW + DUCKDB EVALUATION FOR MARC")
    print("="*70)
    
    # Test 1: Round-trip fidelity (100 records)
    success = test_roundtrip_fidelity(mrc_file, sample_size=100)
    
    # Test 2: Benchmarks on full 10k dataset
    if success:
        print(f"\n{'='*70}")
        print("Loading 10k records for benchmarking...")
        print(f"{'='*70}")
        
        with open(mrc_file, "rb") as f:
            reader = MARCReader(f)
            full_records = list(reader)
        
        print(f"✓ Loaded {len(full_records)} records")
        
        # Serialize
        print("Serializing to Arrow...", end="", flush=True)
        start = time.time()
        arrow_table = marc_to_arrow(full_records)
        serialize_time = (time.time() - start) * 1000
        print(f" ✓")
        
        print(f"  Time: {serialize_time:.1f} ms")
        print(f"  Throughput: {len(full_records) / (serialize_time/1000):.0f} rec/sec")
        print(f"  Table: {len(arrow_table)} rows × {len(arrow_table.column_names)} cols")
        
        # Query benchmarks
        benchmark_duckdb_queries(arrow_table)
        
        # Polars DataFrame stats
        print(f"\n{'='*70}")
        print("Polars DataFrame Statistics")
        print(f"{'='*70}")
        
        df = pl.from_arrow(arrow_table)
        print(f"DataFrame shape: {df.shape}")
        print(f"Memory usage: ~{df.estimated_size() / (1024*1024):.1f} MB")
        print(f"\nField tag distribution:")
        
        stats = df.group_by("field_tag").agg(pl.count("record_id").alias("subfield_count"))
        print(stats.sort("subfield_count", descending=True).head(10))
    
    print("\n" + "="*70)
    print("EVALUATION COMPLETE")
    print("="*70)


if __name__ == "__main__":
    main()
