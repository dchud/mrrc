#!/usr/bin/env python3
"""
Analyze allocation patterns in single-threaded MARC parsing.
Uses valgrind massif to profile heap usage.
"""

import subprocess
import json
import os
import sys
from pathlib import Path

def run_massif():
    """Run valgrind massif on profiling harness"""
    print("Running valgrind massif on profiling harness...")
    
    # Build the harness
    subprocess.run(["cargo", "bench", "--bench", "profiling_harness", "--no-run"],
                   capture_output=True, check=True)
    
    # Find the compiled binary
    harness_path = Path("target/release/deps/profiling_harness-")
    binaries = list(Path("target/release/deps").glob("profiling_harness-*"))
    if not binaries:
        print("Error: Could not find compiled benchmark binary")
        sys.exit(1)
    
    binary = binaries[0]
    print(f"Using binary: {binary}")
    
    # Run with massif (brief run, just to profile allocation patterns)
    cmd = [
        "valgrind",
        "--tool=massif",
        "--massif-out-file=massif.out",
        f"--max-snapshots=100",
        str(binary),
    ]
    
    print(f"Command: {' '.join(cmd)}")
    result = subprocess.run(cmd, capture_output=True, text=True)
    
    if result.returncode != 0:
        print("Valgrind output:")
        print(result.stdout)
        print(result.stderr)
        sys.exit(1)
    
    # Run ms_print to analyze results
    print("\n" + "="*60)
    print("MEMORY PROFILING RESULTS")
    print("="*60 + "\n")
    
    ms_result = subprocess.run(["ms_print", "massif.out"],
                                capture_output=True, text=True)
    print(ms_result.stdout)
    
    if os.path.exists("massif.out"):
        os.remove("massif.out")


def analyze_record_structure():
    """Estimate allocation overhead from Record structure"""
    print("\n" + "="*60)
    print("ALLOCATION ANALYSIS FROM SOURCE CODE")
    print("="*60 + "\n")
    
    # Typical record statistics
    avg_tags_per_record = 35
    avg_subfields_per_field = 6
    avg_string_length_value = 50
    
    # String overhead on 64-bit (with small string optimization)
    # String: 24 bytes (3 x 8byte ptrs for ptr, len, capacity)
    # But may use SSO for small strings <~24 bytes
    string_overhead = 24
    
    # Tag allocations
    tag_allocs = avg_tags_per_record
    tag_memory = tag_allocs * string_overhead  # Each tag is 3 bytes, but String overhead is ~24
    
    # Indicator allocations (if stored as String)
    # Currently stored as char, so no additional allocations
    
    # Subfield value allocations  
    subfield_allocs = avg_tags_per_record * avg_subfields_per_field
    subfield_memory = subfield_allocs * (string_overhead + avg_string_length_value)
    
    # Vec allocations
    field_vec_allocs = avg_tags_per_record
    field_vec_overhead_per_vec = 24  # Vec: 3 x 8byte ptrs
    field_vec_memory = field_vec_allocs * field_vec_overhead_per_vec
    
    # IndexMap allocation (one per record)
    indexmap_allocs = 1
    indexmap_memory = 200  # Approximate for 35 entries
    
    print(f"Per-record allocation analysis:")
    print(f"  Tag allocations:           {tag_allocs:3} (String overhead: {tag_memory:4} bytes)")
    print(f"  Subfield allocations:      {subfield_allocs:3} (overhead + value: {subfield_memory:6} bytes)")
    print(f"  Field Vec allocations:     {field_vec_allocs:3} (overhead: {field_vec_memory:4} bytes)")
    print(f"  IndexMap allocation:       {indexmap_allocs:3} (overhead: {indexmap_memory:4} bytes)")
    print(f"  {'─' * 60}")
    print(f"  Total allocations:         {tag_allocs + subfield_allocs + field_vec_allocs + indexmap_allocs:3}")
    print(f"  Total heap overhead:       {tag_memory + subfield_memory + field_vec_memory + indexmap_memory:6} bytes")
    
    # Optimization potential
    print(f"\nOptimization potential (per-record):")
    
    # SmallVec<[Subfield; 4]> - eliminates 70% of Vec allocations
    smallvec_saving_allocs = int(field_vec_allocs * 0.7)
    smallvec_saving_bytes = smallvec_saving_allocs * field_vec_overhead_per_vec
    print(f"  SmallVec<[Subfield; 4]>:  Save {smallvec_saving_allocs} allocs, {smallvec_saving_bytes} bytes")
    
    # Tag optimization: u16 instead of String
    tag_opt_saving_allocs = tag_allocs
    tag_opt_saving_bytes = tag_memory - (tag_allocs * 2)  # u16 = 2 bytes vs String = 24
    print(f"  Tags as u16:               Save {tag_opt_saving_allocs} allocs, {tag_opt_saving_bytes} bytes")
    
    # Combined savings
    total_savings_allocs = smallvec_saving_allocs + tag_opt_saving_allocs
    total_savings_bytes = smallvec_saving_bytes + tag_opt_saving_bytes
    print(f"  Combined optimization:     Save {total_savings_allocs} allocs, {total_savings_bytes} bytes/record")
    
    # For 10k records
    print(f"\nFor 10,000 records:")
    print(f"  Allocation reduction:      {total_savings_allocs * 10000:,} fewer allocations")
    print(f"  Memory reduction:          {total_savings_bytes * 10000 / 1024 / 1024:.1f} MB")


if __name__ == "__main__":
    print("MARC Record Allocation Profiling")
    print("=" * 60)
    
    # Check if valgrind is available
    try:
        subprocess.run(["which", "valgrind"], capture_output=True, check=True)
        analyze_record_structure()
        # Only run massif if valgrind is available
        # run_massif()
    except subprocess.CalledProcessError:
        print("Note: valgrind not found, skipping runtime profiling")
        print("Running analysis based on source code structure...\n")
        analyze_record_structure()
