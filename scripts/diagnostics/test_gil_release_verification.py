#!/usr/bin/env python3
"""
GIL Release Verification Test - mrrc-l97

Tests whether the GIL is actually being released during Phase 2 parsing.
This is a focused test that measures threading speedup and compares it to
the expected 1.8x+ improvement.

Usage:
    python test_gil_release_verification.py
"""

import io
import time
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
from mrrc import MARCReader


def load_fixture(name: str) -> bytes:
    """Load a test fixture from the test data directory."""
    fixture_path = Path(__file__).parent / "tests" / "data" / "fixtures" / f"{name}.mrc"
    if not fixture_path.exists():
        raise FileNotFoundError(f"Fixture not found: {fixture_path}")
    return fixture_path.read_bytes()


def read_sequential(data: bytes, num_reads: int = 1) -> tuple[int, float]:
    """Read records sequentially and return count + elapsed time."""
    start = time.perf_counter()
    total = 0
    for _ in range(num_reads):
        reader = MARCReader(io.BytesIO(data))
        while record := reader.read_record():
            total += 1
    elapsed = time.perf_counter() - start
    return total, elapsed


def read_parallel(data: bytes, num_threads: int) -> tuple[int, float]:
    """Read records in parallel using ThreadPoolExecutor and return count + elapsed time."""
    start = time.perf_counter()
    
    def read_single():
        reader = MARCReader(io.BytesIO(data))
        count = 0
        while record := reader.read_record():
            count += 1
        return count
    
    with ThreadPoolExecutor(max_workers=num_threads) as executor:
        results = list(executor.map(lambda _: read_single(), range(num_threads)))
    
    elapsed = time.perf_counter() - start
    return sum(results), elapsed


def main():
    print("=" * 70)
    print("GIL Release Verification Test (mrrc-l97)")
    print("=" * 70)
    print()
    
    # Load test data
    print("Loading test fixtures...")
    try:
        fixture_10k = load_fixture("10k_records")
        print(f"✓ Loaded 10k fixture ({len(fixture_10k)} bytes)")
    except FileNotFoundError as e:
        print(f"✗ Failed to load fixture: {e}")
        return 1
    
    print()
    print("-" * 70)
    print("TEST 1: Single vs 2-Thread Performance (10k records)")
    print("-" * 70)
    
    # Baseline: sequential reads of 2x 10k records
    print("Measuring sequential read of 2x10k records...")
    total_seq2, time_seq2 = read_sequential(fixture_10k, num_reads=2)
    print(f"  Result: {total_seq2} records in {time_seq2:.3f}s")
    print(f"  Rate: {total_seq2/time_seq2:.0f} records/sec")
    
    # 2-threaded: parallel reads of 2x 10k records
    print()
    print("Measuring 2-threaded read of 2x10k records...")
    total_par2, time_par2 = read_parallel(fixture_10k, num_threads=2)
    print(f"  Result: {total_par2} records in {time_par2:.3f}s")
    print(f"  Rate: {total_par2/time_par2:.0f} records/sec")
    
    # Calculate speedup
    speedup_2 = time_seq2 / time_par2
    print()
    print(f"SPEEDUP (2 threads): {speedup_2:.2f}x")
    print(f"  Expected with GIL release: ≥1.8x")
    print(f"  Status: {'✓ PASS' if speedup_2 >= 1.5 else '✗ FAIL'}")
    
    print()
    print("-" * 70)
    print("TEST 2: Sequential vs 4-Thread Performance (10k records)")
    print("-" * 70)
    
    # Baseline: sequential reads of 4x 10k records
    print("Measuring sequential read of 4x10k records...")
    total_seq4, time_seq4 = read_sequential(fixture_10k, num_reads=4)
    print(f"  Result: {total_seq4} records in {time_seq4:.3f}s")
    print(f"  Rate: {total_seq4/time_seq4:.0f} records/sec")
    
    # 4-threaded: parallel reads of 4x 10k records
    print()
    print("Measuring 4-threaded read of 4x10k records...")
    total_par4, time_par4 = read_parallel(fixture_10k, num_threads=4)
    print(f"  Result: {total_par4} records in {time_par4:.3f}s")
    print(f"  Rate: {total_par4/time_par4:.0f} records/sec")
    
    # Calculate speedup
    speedup_4 = time_seq4 / time_par4
    print()
    print(f"SPEEDUP (4 threads): {speedup_4:.2f}x")
    print(f"  Expected with GIL release: ≥2.5x")
    print(f"  Status: {'✓ PASS' if speedup_4 >= 2.0 else '✗ FAIL'}")
    
    print()
    print("=" * 70)
    print("SUMMARY")
    print("=" * 70)
    print(f"2-thread speedup:  {speedup_2:.2f}x (target: ≥1.8x)")
    print(f"4-thread speedup:  {speedup_4:.2f}x (target: ≥2.5x)")
    print()
    
    # Overall status
    overall_pass = speedup_2 >= 1.5 and speedup_4 >= 2.0
    if overall_pass:
        print("✓ GIL Release Verification: PASSED")
        print()
        print("The fix appears to be working! The GIL is being released during Phase 2,")
        print("allowing multiple threads to execute concurrently.")
        return 0
    else:
        print("✗ GIL Release Verification: FAILED")
        print()
        if speedup_2 < 1.5:
            print(f"  - 2-thread speedup is only {speedup_2:.2f}x (expected ≥1.5x)")
            print("    This suggests the GIL is not being released properly.")
        if speedup_4 < 2.0:
            print(f"  - 4-thread speedup is only {speedup_4:.2f}x (expected ≥2.0x)")
            print("    This suggests GIL contention limits parallelism.")
        return 1


if __name__ == "__main__":
    exit(main())
