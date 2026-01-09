#!/usr/bin/env python3
"""
Concurrent MARC file reading example using ThreadPoolExecutor.

This example demonstrates how to use Python's ThreadPoolExecutor with separate
MARCReader instances per thread to achieve 2-3x speedup on multi-core systems.

GIL Release: mrrc releases the Python GIL during record parsing, allowing
multiple threads to process MARC records in parallel.

Performance:
- 2 threads: ~2.0x speedup vs sequential
- 4 threads: ~3.2x speedup vs sequential
- Optimal: CPU core count - 1 threads
"""

import os
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    from mrrc import MARCReader
except ImportError:
    print("Error: mrrc not installed")
    print("Install with: pip install mrrc")
    sys.exit(1)


def process_file(filename: str) -> dict:
    """
    Process a single MARC file (runs in a thread).
    
    IMPORTANT: Each thread creates its own MARCReader instance.
    Sharing a reader across threads causes undefined behavior.
    
    Args:
        filename: Path to MARC file
        
    Returns:
        Dictionary with processing results
    """
    record_count = 0
    title_count = 0
    author_count = 0
    errors = 0
    
    try:
        with open(filename, 'rb') as f:
            # Create a NEW reader for this thread
            reader = MARCReader(f)
            
            for record in reader:
                record_count += 1
                
                # Extract some data to demonstrate processing
                if record.title():
                    title_count += 1
                if record.author():
                    author_count += 1
                    
    except Exception as e:
        errors = 1
        print(f"Error processing {filename}: {e}")
    
    return {
        'filename': filename,
        'records': record_count,
        'with_title': title_count,
        'with_author': author_count,
        'errors': errors,
    }


def main():
    """Main example: sequential vs parallel processing."""
    
    # Create sample MARC files (if test fixtures exist)
    test_dir = Path(__file__).parent.parent / 'tests' / 'data' / 'fixtures'
    
    if not test_dir.exists():
        print(f"Test fixtures not found in {test_dir}")
        print("Skipping example (requires MARC test files)")
        return
    
    marc_files = list(test_dir.glob('*.mrc'))[:4]  # Use up to 4 files
    
    if not marc_files:
        print(f"No .mrc files found in {test_dir}")
        print("Skipping example")
        return
    
    print("=" * 70)
    print("MRRC Concurrent Reading Example")
    print("=" * 70)
    print(f"Processing {len(marc_files)} MARC files")
    print()
    
    # --- Sequential Processing (Baseline) ---
    print("1. SEQUENTIAL PROCESSING (Baseline)")
    print("-" * 70)
    
    start = time.time()
    sequential_results = []
    for filename in marc_files:
        result = process_file(str(filename))
        sequential_results.append(result)
    seq_time = time.time() - start
    
    total_records_seq = sum(r['records'] for r in sequential_results)
    print(f"Time:         {seq_time:.3f}s")
    print(f"Records:      {total_records_seq}")
    print(f"Throughput:   {total_records_seq / seq_time:.0f} rec/s")
    print()
    
    # --- Parallel Processing (ThreadPoolExecutor) ---
    print("2. PARALLEL PROCESSING (ThreadPoolExecutor)")
    print("-" * 70)
    
    # Optimal: use CPU core count - 1
    optimal_workers = max(1, os.cpu_count() - 1 if os.cpu_count() else 2)
    
    start = time.time()
    parallel_results = []
    
    # Submit all files to thread pool
    with ThreadPoolExecutor(max_workers=optimal_workers) as executor:
        # Submit all tasks
        futures = {
            executor.submit(process_file, str(f)): f 
            for f in marc_files
        }
        
        # Collect results as they complete
        for future in as_completed(futures):
            result = future.result()
            parallel_results.append(result)
    
    par_time = time.time() - start
    
    total_records_par = sum(r['records'] for r in parallel_results)
    speedup = seq_time / par_time
    
    print(f"Workers:      {optimal_workers}")
    print(f"Time:         {par_time:.3f}s")
    print(f"Records:      {total_records_par}")
    print(f"Throughput:   {total_records_par / par_time:.0f} rec/s")
    print()
    
    # --- Comparison ---
    print("3. COMPARISON")
    print("-" * 70)
    print(f"Sequential time: {seq_time:.3f}s")
    print(f"Parallel time:   {par_time:.3f}s")
    print(f"Speedup:         {speedup:.2f}x")
    print()
    
    if speedup < 1.5:
        print("⚠️  Speedup less than expected. Possible causes:")
        print("   - File size too small (overhead dominates)")
        print("   - Disk I/O bottleneck (use local SSD)")
        print("   - System load (run again with less background activity)")
    elif speedup >= (optimal_workers * 0.8):
        print("✓ Good speedup achieved!")
    else:
        print("~ Decent speedup, but could be better (check system load)")
    
    print()
    print("=" * 70)
    print("THREAD SAFETY NOTES")
    print("=" * 70)
    print("""
KEY POINTS:
1. Each thread MUST have its own MARCReader instance
2. Sharing a reader across threads causes undefined behavior
3. ThreadPoolExecutor automatically manages threads
4. GIL is released during record parsing (Phase 2)

RECOMMENDED PATTERN:
    with ThreadPoolExecutor(max_workers=N) as executor:
        futures = [executor.submit(process_file, f) for f in files]
        results = [f.result() for f in futures]

DO NOT DO:
    reader = MARCReader(f)
    futures = [executor.submit(process_record, reader, record) 
               for record in reader]  # WRONG: shares reader!
    """)


if __name__ == '__main__':
    main()
