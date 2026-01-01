#!/usr/bin/env python3
"""
Realistic Threading Test for GIL Release

Tests actual parallel reading with SEPARATE file handles per thread.
This is the intended use case: multiple threads reading different files.
"""

import io
import time
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
from mrrc import MARCReader


def load_fixture() -> bytes:
    """Load a test fixture."""
    fixture_path = Path(__file__).parent / "tests" / "data" / "fixtures" / "10k_records.mrc"
    return fixture_path.read_bytes()


def main():
    print("=" * 70)
    print("Realistic Threading Test - Parallel File Reading")
    print("=" * 70)
    print()
    
    fixture = load_fixture()
    
    def read_file(file_data: bytes) -> tuple[int, float]:
        """Read all records from file data and measure time."""
        start = time.perf_counter()
        reader = MARCReader(io.BytesIO(file_data))
        count = 0
        while record := reader.read_record():
            count += 1
        elapsed = time.perf_counter() - start
        return count, elapsed
    
    # Test 1: Sequential - read 2 files one after another
    print("TEST 1: Sequential Read of 2 Files")
    print("-" * 70)
    start_seq = time.perf_counter()
    count1, time1 = read_file(fixture)
    count2, time2 = read_file(fixture)
    total_seq = time.perf_counter() - start_seq
    print(f"  File 1: {count1} records in {time1:.3f}s")
    print(f"  File 2: {count2} records in {time2:.3f}s")
    print(f"  Total sequential: {total_seq:.3f}s")
    print()
    
    # Test 2: Parallel - read 2 files using ThreadPoolExecutor
    print("TEST 2: Parallel Read of 2 Files (2 threads)")
    print("-" * 70)
    start_par = time.perf_counter()
    with ThreadPoolExecutor(max_workers=2) as executor:
        results = list(executor.map(read_file, [fixture, fixture]))
    total_par = time.perf_counter() - start_par
    
    for i, (count, elapsed) in enumerate(results):
        print(f"  File {i+1}: {count} records in {elapsed:.3f}s")
    print(f"  Total parallel: {total_par:.3f}s")
    print()
    
    # Calculate speedup
    speedup = total_seq / total_par
    print("=" * 70)
    print(f"SPEEDUP: {speedup:.2f}x")
    print(f"  Sequential total: {total_seq:.3f}s")
    print(f"  Parallel total:   {total_par:.3f}s")
    print()
    
    if speedup >= 1.5:
        print("✓ GIL Release Working: Speedup >= 1.5x (good threading performance)")
    elif speedup >= 1.2:
        print("⚠ Partial GIL Release: Speedup 1.2-1.5x (some parallelism)")
    else:
        print("✗ GIL NOT Released: Speedup < 1.2x (no parallelism)")
    
    return 0 if speedup >= 1.5 else 1


if __name__ == "__main__":
    exit(main())
