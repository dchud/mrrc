#!/usr/bin/env python3
"""
Test to verify PyO3 allow_threads() behavior with assume_gil_acquired().

This helps us understand if the pattern we're using actually releases the GIL.
"""

import threading
import time


def simple_test():
    """Simple test of GIL release via event signaling."""
    print("Testing if allow_threads() actually releases the GIL...")
    print()
    
    from mrrc import MARCReader
    import io
    from pathlib import Path
    
    fixture_path = Path(__file__).parent / "tests" / "data" / "fixtures" / "10k_records.mrc"
    fixture = fixture_path.read_bytes()
    
    # Create a reader that will parse slowly (many records)
    reader = MARCReader(io.BytesIO(fixture))
    
    # Flag to track if other thread ran
    other_thread_ran = threading.Event()
    parsing_started = threading.Event()
    
    def background_thread():
        """This thread should be able to run if GIL is released."""
        time.sleep(0.05)  # Wait for main thread to start parsing
        print("Background thread: Attempting to acquire GIL...")
        # Simply trying to call a Python function should block if GIL not released
        try:
            time.sleep(0.1)  # Small sleep to allow main thread to be in Phase 2
            other_thread_ran.set()
            print("Background thread: Successfully ran while main thread was parsing!")
        except Exception as e:
            print(f"Background thread: Failed with {e}")
    
    # Start background thread
    t = threading.Thread(target=background_thread)
    t.start()
    
    # Read records - this should trigger Phase 2 GIL release
    print("Main thread: Starting to read records...")
    count = 0
    start = time.perf_counter()
    while record := reader.read_record():
        count += 1
        if count == 1:
            parsing_started.set()
    elapsed = time.perf_counter() - start
    
    print(f"Main thread: Read {count} records in {elapsed:.3f}s")
    
    t.join(timeout=5)
    
    if other_thread_ran.is_set():
        print()
        print("✓ GIL WAS Released: Other thread ran during parsing")
        return 0
    else:
        print()
        print("✗ GIL NOT Released: Other thread did not run")
        return 1


if __name__ == "__main__":
    exit(simple_test())
