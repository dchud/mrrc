#!/usr/bin/env python3
"""
Concurrent MARC file reading example using ProducerConsumerPipeline.

This example demonstrates how to use ProducerConsumerPipeline for maximum
throughput when reading a single large MARC file. It uses a producer thread
to read from disk, a bounded channel for backpressure, and Rayon parallel
parsing for CPU-intensive record parsing.

Performance (on 4-core system):
- Single file with ThreadPoolExecutor: 3.0x speedup
- Single file with ProducerConsumerPipeline: 3.74x speedup

Use ProducerConsumerPipeline when:
✓ Processing a single large MARC file
✓ Maximum throughput is needed
✓ You want the most sophisticated concurrency pattern

Use ThreadPoolExecutor when:
✓ Processing multiple separate files
✓ Simpler API is preferred
✓ You already have file list available

Architecture:
  Disk
   |
   └─ Producer Thread (reads chunks, identifies records)
       |
       ├─ Bounded Channel (1000-record buffer, provides backpressure)
       |
       └─ Rayon Parallel Batch Processor (parses 100 records at a time)
           |
           └─ Consumer Thread (your code, receives parsed records)
"""

import os
import sys
import time
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    from mrrc import ProducerConsumerPipeline
except ImportError:
    print("Error: mrrc not installed")
    print("Install with: pip install mrrc")
    sys.exit(1)


def main():
    """Main example: sequential vs ProducerConsumerPipeline processing."""
    
    # Create sample MARC file path
    test_dir = Path(__file__).parent.parent / 'tests' / 'data' / 'fixtures'
    
    if not test_dir.exists():
        print(f"Test fixtures not found in {test_dir}")
        print("Skipping example (requires MARC test files)")
        return
    
    marc_files = list(test_dir.glob('*.mrc'))
    
    if not marc_files:
        print(f"No .mrc files found in {test_dir}")
        print("Skipping example")
        return
    
    # For this example, use the largest file (simulates "large" file scenario)
    marc_file = max(marc_files, key=lambda f: f.stat().st_size)
    
    print("=" * 70)
    print("MRRC ProducerConsumerPipeline Example")
    print("=" * 70)
    print(f"File: {marc_file.name}")
    print(f"Size: {marc_file.stat().st_size / 1024:.1f} KB")
    print()
    
    # --- Sequential Processing (Baseline) ---
    print("1. SEQUENTIAL PROCESSING (Baseline)")
    print("-" * 70)
    
    from mrrc import MARCReader
    
    start = time.time()
    record_count = 0
    title_count = 0
    author_count = 0
    
    with open(marc_file, 'rb') as f:
        reader = MARCReader(f)
        for record in reader:
            record_count += 1
            if record.title():
                title_count += 1
            if record.author():
                author_count += 1
    
    seq_time = time.time() - start
    print(f"Time:           {seq_time:.3f}s")
    print(f"Records:        {record_count}")
    print(f"With title:     {title_count}")
    print(f"With author:    {author_count}")
    print(f"Throughput:     {record_count / seq_time:.0f} rec/s")
    print()
    
    # --- ProducerConsumerPipeline Processing ---
    print("2. PRODUCER-CONSUMER PIPELINE PROCESSING")
    print("-" * 70)
    
    # Create pipeline with default config
    # Default: buffer_size=1000, batch_size=100, chunk_size=512KB
    start = time.time()
    record_count_pc = 0
    title_count_pc = 0
    author_count_pc = 0
    
    try:
        # Create pipeline with default config (512 KB buffer, 1000-record channel)
        pipeline = ProducerConsumerPipeline.from_file(str(marc_file))

        # Iterate over records from pipeline
        for record in pipeline:
            record_count_pc += 1
            if record.title():
                title_count_pc += 1
            if record.author():
                author_count_pc += 1
        
        pc_time = time.time() - start
        speedup = seq_time / pc_time
        
        print(f"Time:           {pc_time:.3f}s")
        print(f"Records:        {record_count_pc}")
        print(f"With title:     {title_count_pc}")
        print(f"With author:    {author_count_pc}")
        print(f"Throughput:     {record_count_pc / pc_time:.0f} rec/s")
        print()
        
        # --- Comparison ---
        print("3. COMPARISON")
        print("-" * 70)
        print(f"Sequential time:  {seq_time:.3f}s")
        print(f"Pipeline time:    {pc_time:.3f}s")
        print(f"Speedup:          {speedup:.2f}x")
        print()
        
        if speedup > 3.0:
            print("✓ Excellent speedup! ProducerConsumerPipeline working well.")
        elif speedup > 2.0:
            print("✓ Good speedup achieved!")
        else:
            print("~ Modest speedup. Possible causes:")
            print("   - File too small for overhead to amortize")
            print("   - System load (run with less background activity)")
        
    except Exception as e:
        print(f"Error: {e}")
        print()
        print("Note: ProducerConsumerPipeline may not be available in all builds.")
        print("Ensure MRRC is built with Rayon support enabled.")
    
    print()
    print("=" * 70)
    print("ARCHITECTURE OVERVIEW")
    print("=" * 70)
    print("""
PRODUCER-CONSUMER PIPELINE:

1. Producer Thread
   - Reads file in 512 KB chunks (configurable)
   - Identifies MARC record boundaries
   - Puts raw record data into bounded channel

2. Bounded Channel
   - Default capacity: 1000 records
   - Provides backpressure (producer waits if full)
   - Decouples producer from parser

3. Rayon Parallel Batch Parser
   - Consumes batches of 100 records from channel
   - Parses each batch in parallel using all CPU cores
   - Puts parsed records back into output channel

4. Consumer (Your Code)
   - Receives fully-parsed Record objects
   - No parsing overhead here
   - Processes records at maximum throughput

ADVANTAGES:
✓ Maximizes CPU core utilization
✓ Overlaps disk I/O with parallel parsing
✓ Automatic backpressure handling
✓ Simple Pythonic iteration (for record in pipeline)

CONFIGURATION:
# Default configuration
pipeline = ProducerConsumerPipeline.from_file(filename)

# Custom configuration
pipeline = ProducerConsumerPipeline.from_file(
    filename,
    buffer_size=1024*1024,  # 1 MB I/O buffer (default: 512 KB)
    channel_capacity=500    # Records in channel (default: 1000)
)
    """)
    
    print()
    print("=" * 70)
    print("WHEN TO USE")
    print("=" * 70)
    print("""
Use ProducerConsumerPipeline when:
  • Processing a single large MARC file (100MB+)
  • Maximum throughput is critical
  • You want true parallelism with sophisticated pipelining

Use ThreadPoolExecutor when:
  • Processing multiple separate files
  • Simpler API is preferred
  • File sizes are moderate (10-100MB)

See examples/concurrent_reading.py for ThreadPoolExecutor pattern.
    """)


if __name__ == '__main__':
    main()
