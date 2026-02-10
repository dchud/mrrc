#!/usr/bin/env python3
"""
Concurrent MARC file writing example using ThreadPoolExecutor.

This example demonstrates how to use separate MARCWriter instances per thread
to achieve parallel writing. While writing has less GIL release benefit than
reading (due to Python file object I/O), separate writers still prevent
contention and allow concurrent processing.

Performance:
- Each thread writes to its own file
- Writing throughput comparable to reading
- Demonstrates error handling and thread-safe patterns
"""

import os
import sys
import time
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    from mrrc import MARCReader, MARCWriter
except ImportError:
    print("Error: mrrc not installed")
    print("Install with: pip install mrrc")
    sys.exit(1)


def copy_records(input_file: str, output_file: str) -> dict:
    """
    Copy all records from input file to output file.
    
    IMPORTANT: Creates separate reader AND writer for this thread.
    Each thread must have its own reader and writer instances.
    
    Args:
        input_file: Path to source MARC file
        output_file: Path to destination MARC file
        
    Returns:
        Dictionary with processing results
    """
    records_read = 0
    records_written = 0
    errors = 0
    
    try:
        with open(input_file, 'rb') as infile:
            with open(output_file, 'wb') as outfile:
                # Create separate reader and writer for this thread
                reader = MARCReader(infile)
                writer = MARCWriter(outfile)
                
                for record in reader:
                    records_read += 1
                    writer.write_record(record)
                    records_written += 1
                    
        # Verify file was created
        if not os.path.exists(output_file):
            errors = 1
            
    except Exception as e:
        errors = 1
        print(f"Error processing {input_file} → {output_file}: {e}")
    
    return {
        'input': input_file,
        'output': output_file,
        'read': records_read,
        'written': records_written,
        'errors': errors,
    }


def main():
    """Main example: sequential vs parallel writing."""
    
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
    print("MRRC Concurrent Writing Example")
    print("=" * 70)
    print(f"Processing {len(marc_files)} MARC files")
    print()
    
    # Create temporary directory for output files
    with tempfile.TemporaryDirectory() as tmpdir:
        output_files = [
            os.path.join(tmpdir, f"output_{i}.mrc")
            for i in range(len(marc_files))
        ]
        
        # --- Sequential Writing (Baseline) ---
        print("1. SEQUENTIAL WRITING (Baseline)")
        print("-" * 70)
        
        start = time.time()
        sequential_results = []
        for input_file, output_file in zip(marc_files, output_files):
            result = copy_records(str(input_file), output_file)
            sequential_results.append(result)
        seq_time = time.time() - start
        
        total_written_seq = sum(r['written'] for r in sequential_results)
        print(f"Time:         {seq_time:.3f}s")
        print(f"Records:      {total_written_seq}")
        print(f"Throughput:   {total_written_seq / seq_time:.0f} rec/s")
        print()
        
        # --- Parallel Writing (ThreadPoolExecutor) ---
        print("2. PARALLEL WRITING (ThreadPoolExecutor)")
        print("-" * 70)
        
        # Optimal: use CPU core count - 1
        optimal_workers = max(1, os.cpu_count() - 1 if os.cpu_count() else 2)
        
        start = time.time()
        parallel_results = []
        
        with ThreadPoolExecutor(max_workers=optimal_workers) as executor:
            futures = [
                executor.submit(copy_records, str(input_file), output_file)
                for input_file, output_file in zip(marc_files, output_files)
            ]
            
            for future in futures:
                result = future.result()
                parallel_results.append(result)
        
        par_time = time.time() - start
        
        total_written_par = sum(r['written'] for r in parallel_results)
        speedup = seq_time / par_time if par_time > 0 else 0
        
        print(f"Workers:      {optimal_workers}")
        print(f"Time:         {par_time:.3f}s")
        print(f"Records:      {total_written_par}")
        print(f"Throughput:   {total_written_par / par_time:.0f} rec/s")
        print()
        
        # --- Verify Results ---
        print("3. VERIFICATION")
        print("-" * 70)
        
        # Check that output files match input files
        all_match = True
        for input_file, output_file in zip(marc_files, output_files):
            with open(input_file, 'rb') as f1:
                input_data = f1.read()
            
            if os.path.exists(output_file):
                with open(output_file, 'rb') as f2:
                    output_data = f2.read()
                
                if input_data == output_data:
                    print(f"✓ {Path(input_file).name} → {Path(output_file).name}")
                else:
                    print(f"✗ {Path(input_file).name} → {Path(output_file).name} (data mismatch)")
                    all_match = False
            else:
                print(f"✗ {Path(output_file).name} (not created)")
                all_match = False
        
        print()
        
        # --- Comparison ---
        print("4. PERFORMANCE COMPARISON")
        print("-" * 70)
        print(f"Sequential time: {seq_time:.3f}s")
        print(f"Parallel time:   {par_time:.3f}s")
        print(f"Speedup:         {speedup:.2f}x")
        print(f"Round-trip:      All files {'✓ match' if all_match else '✗ differ'}")
        print()
    
    print("=" * 70)
    print("THREAD SAFETY NOTES")
    print("=" * 70)
    print("""
KEY POINTS:
1. Each thread MUST have its own MARCReader AND MARCWriter instances
2. Sharing readers/writers across threads causes undefined behavior
3. Writing to separate output files (different file paths) is safe
4. Each thread holds its own file handles

RECOMMENDED PATTERN:
    with ThreadPoolExecutor(max_workers=N) as executor:
        futures = [executor.submit(copy_file, input_f, output_f)
                   for input_f, output_f in zip(inputs, outputs)]
        results = [f.result() for f in futures]

ROUND-TRIP VERIFICATION:
    After writing, you can read back and compare:
    - Same record count
    - Same binary data (byte-for-byte)
    - Successful re-parsing without errors
    """)


if __name__ == '__main__':
    main()
