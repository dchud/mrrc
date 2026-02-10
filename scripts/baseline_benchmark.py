#!/usr/bin/env python3
"""
Baseline benchmark for GIL Release Phase B.

Measures current single-thread and 2-thread performance BEFORE any GIL release changes.
This baseline gates the Phase C deferral decision:
- If Phase F speedup >= 2.0x vs baseline, Phase C optional
- If Phase F speedup < 2.0x, Phase C required before release
"""

import time
import threading
import shutil
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
import mrrc

def single_thread_benchmark(fixture_path):
    """Measure time to read all records from a file sequentially."""
    start = time.time()
    with open(fixture_path, 'rb') as f:
        reader = mrrc.MARCReader(f)
        count = 0
        for record in reader:
            count += 1
    elapsed = time.time() - start
    return elapsed, count

def two_thread_benchmark(fixture_5k_path):
    """
    Measure time for two threads to read different 5K files simultaneously.
    Uses ThreadPoolExecutor to start both threads at roughly the same time.
    """
    def read_file(path):
        with open(path, 'rb') as f:
            reader = mrrc.MARCReader(f)
            count = 0
            for record in reader:
                count += 1
        return count
    
    # Create two identical copies of the 5K fixture
    copy_a = fixture_5k_path.parent / "fixture_5k_copy_a.mrc"
    copy_b = fixture_5k_path.parent / "fixture_5k_copy_b.mrc"
    
    shutil.copy(fixture_5k_path, copy_a)
    shutil.copy(fixture_5k_path, copy_b)
    
    try:
        start = time.time()
        with ThreadPoolExecutor(max_workers=2) as executor:
            future_a = executor.submit(read_file, copy_a)
            future_b = executor.submit(read_file, copy_b)
            count_a = future_a.result()
            count_b = future_b.result()
        elapsed = time.time() - start
        return elapsed, count_a + count_b
    finally:
        copy_a.unlink(missing_ok=True)
        copy_b.unlink(missing_ok=True)

def main():
    fixtures_dir = Path(__file__).parent.parent / "tests" / "data" / "fixtures"
    fixture_10k = fixtures_dir / "10k_records.mrc"
    fixture_5k = fixtures_dir / "5k_records.mrc"
    
    # Check fixtures exist
    if not fixture_10k.exists():
        print(f"ERROR: {fixture_10k} not found")
        return False
    
    if not fixture_5k.exists():
        print(f"ERROR: {fixture_5k} not found")
        return False
    
    print("=" * 70)
    print("BASELINE BENCHMARK - GIL Release Phase B")
    print("=" * 70)
    print()
    
    # Single-thread baseline
    print("Testing single-thread baseline (10K records)...")
    st_elapsed, st_count = single_thread_benchmark(fixture_10k)
    st_ops_per_sec = st_count / st_elapsed
    print(f"  Time: {st_elapsed:.3f}s")
    print(f"  Records: {st_count}")
    print(f"  Ops/sec: {st_ops_per_sec:.0f}")
    print()
    
    # Two-thread baseline
    print("Testing 2-thread baseline (2x 5K records)...")
    mt_elapsed, mt_count = two_thread_benchmark(fixture_5k)
    mt_ops_per_sec = mt_count / mt_elapsed
    speedup = st_elapsed / mt_elapsed
    print(f"  Time: {mt_elapsed:.3f}s")
    print(f"  Records: {mt_count}")
    print(f"  Ops/sec: {mt_ops_per_sec:.0f}")
    print(f"  Speedup: {speedup:.2f}x")
    print()
    
    # Write results
    benchmarks_dir = Path(__file__).parent.parent / ".benchmarks"
    benchmarks_dir.mkdir(exist_ok=True)
    
    result_file = benchmarks_dir / "baseline_before_gil_release.txt"
    with open(result_file, 'w') as f:
        f.write("BASELINE BENCHMARK - GIL Release Phase B\n")
        f.write("=" * 70 + "\n")
        f.write("\n")
        f.write("SINGLE-THREAD BASELINE (10K records)\n")
        f.write("-" * 70 + "\n")
        f.write(f"Time:      {st_elapsed:.3f}s\n")
        f.write(f"Records:   {st_count}\n")
        f.write(f"Ops/sec:   {st_ops_per_sec:.0f}\n")
        f.write("\n")
        f.write("TWO-THREAD BASELINE (2x 5K records)\n")
        f.write("-" * 70 + "\n")
        f.write(f"Time:      {mt_elapsed:.3f}s\n")
        f.write(f"Records:   {mt_count}\n")
        f.write(f"Ops/sec:   {mt_ops_per_sec:.0f}\n")
        f.write(f"Speedup:   {speedup:.2f}x\n")
        f.write("\n")
        f.write("PHASE C DEFERRAL GATE\n")
        f.write("-" * 70 + "\n")
        f.write(f"Baseline speedup:      {speedup:.2f}x\n")
        f.write(f"Target (after Phase F): >= 2.0x\n")
        f.write(f"Decision:\n")
        if speedup >= 2.0:
            f.write(f"  If Phase F >= 2.0x: Phase C OPTIONAL (skip)\n")
            f.write(f"  If Phase F < 2.0x: Phase C REQUIRED\n")
        else:
            f.write(f"  Current speedup ({speedup:.2f}x) < 2.0x goal\n")
            f.write(f"  Phase C optimization will be REQUIRED if Phase F < 2.0x\n")
    
    print(f"Results saved to: {result_file}")
    print()
    print("=" * 70)
    print("BASELINE COMPLETE - Ready for Phase B implementation")
    print("=" * 70)
    
    return True

if __name__ == '__main__':
    import sys
    success = main()
    sys.exit(0 if success else 1)
