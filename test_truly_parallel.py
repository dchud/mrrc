#!/usr/bin/env python3
"""
TRUE Parallel Test - Completely separate BytesIO objects per thread
"""

import io
import time
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
import threading
from mrrc import MARCReader


def load_fixture() -> bytes:
    """Load a test fixture."""
    fixture_path = Path(__file__).parent / "tests" / "data" / "fixtures" / "10k_records.mrc"
    return fixture_path.read_bytes()


def main():
    fixture = load_fixture()
    
    # Test with truly independent BytesIO objects
    def read_file_independent(file_id: int) -> tuple[int, float]:
        """Each thread gets its own BytesIO."""
        # Create a NEW BytesIO for this thread
        data = io.BytesIO(fixture)
        start = time.perf_counter()
        reader = MARCReader(data)
        count = 0
        while record := reader.read_record():
            count += 1
        elapsed = time.perf_counter() - start
        print(f"  [Thread {file_id}] {count} records in {elapsed:.3f}s")
        return count, elapsed
    
    print("=" * 70)
    print("Parallel Read Test - Truly Independent BytesIO Objects")
    print("=" * 70)
    print()
    
    # Sequential
    print("Sequential (main thread reads twice):")
    start_seq = time.perf_counter()
    r1 = read_file_independent(1)
    r2 = read_file_independent(2)
    total_seq = time.perf_counter() - start_seq
    print(f"Total: {total_seq:.3f}s")
    print()
    
    # Parallel
    print("Parallel (2 threads, each with own BytesIO):")
    start_par = time.perf_counter()
    with ThreadPoolExecutor(max_workers=2) as executor:
        futures = [executor.submit(read_file_independent, i) for i in range(1, 3)]
        results = [f.result() for f in futures]
    total_par = time.perf_counter() - start_par
    print(f"Total: {total_par:.3f}s")
    print()
    
    speedup = total_seq / total_par
    print("=" * 70)
    print(f"SPEEDUP: {speedup:.2f}x")
    print(f"  Sequential: {total_seq:.3f}s")
    print(f"  Parallel:   {total_par:.3f}s")
    print()
    
    if speedup >= 1.5:
        print("✓ GIL Release Working Well: Good parallelism")
    elif speedup >= 1.2:
        print("⚠ Partial GIL Release: Some parallelism")
    else:
        print("✗ GIL NOT Released Effectively: No speedup")
    
    return 0 if speedup >= 1.5 else 1


if __name__ == "__main__":
    exit(main())
