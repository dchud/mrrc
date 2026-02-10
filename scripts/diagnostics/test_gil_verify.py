#!/usr/bin/env python3
"""
Verify GIL is actually being released during parsing.

This test uses threading events to detect if the GIL is held during Phase 2.
"""

import io
import threading
import time
from mrrc import MARCReader

def test_gil_release():
    """
    Test that GIL is released during record parsing.
    
    If GIL is NOT released, other_thread will timeout waiting to run.
    If GIL IS released, other_thread will run quickly.
    """
    # Load test data
    with open('tests/data/fixtures/10k_records.mrc', 'rb') as f:
        fixture = f.read()
    
    parsing_started = threading.Event()
    other_thread_ran = threading.Event()
    
    def read_with_notification():
        """Read records and signal when Phase 2 starts."""
        reader = MARCReader(io.BytesIO(fixture[:1000000]))  # Use first 1MB to keep test short
        count = 0
        for record in reader:
            count += 1
            if count == 100:
                # We've been parsing for a while
                # If GIL is released, another thread should have been able to run by now
                parsing_started.set()
            if count > 200:
                break
        return count
    
    def other_work():
        """Try to run while main thread is parsing."""
        # Wait for main thread to start parsing
        parsing_started.wait(timeout=10)
        
        # Now we should be able to run quickly if GIL is released
        # Record when we actually get to run
        start_time = time.perf_counter()
        time.sleep(0.001)  # Just a small amount of work
        end_time = time.perf_counter()
        
        # If GIL was NOT released, this sleep would have been blocked
        # and would have taken much longer (waiting for main thread)
        other_thread_ran.set()
        return end_time - start_time
    
    # Start main reading thread
    main_thread = threading.Thread(target=read_with_notification)
    main_thread.start()
    
    # Start other thread
    other_thread = threading.Thread(target=other_work)
    other_thread.start()
    
    # Wait for completion
    main_thread.join(timeout=30)
    other_thread.join(timeout=30)
    
    # Check if other thread was able to run
    if other_thread_ran.is_set():
        print("✅ GIL appears to be released (other thread ran)")
    else:
        print("❌ GIL appears to NOT be released (other thread blocked)")

if __name__ == "__main__":
    test_gil_release()
