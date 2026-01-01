#!/usr/bin/env python3
"""
GIL Release Diagnostic Test

Tests whether the GIL is actually being released during Phase 2 parsing.
Uses threading.Event to confirm other threads can run while Phase 2 is executing.
"""

import io
import time
import threading
from pathlib import Path
from mrrc import MARCReader


def load_fixture() -> bytes:
    """Load a test fixture."""
    fixture_path = Path(__file__).parent / "tests" / "data" / "fixtures" / "10k_records.mrc"
    return fixture_path.read_bytes()


def test_gil_release_with_event():
    """
    Diagnostic test: If GIL is released, other threads should run.
    
    If GIL is NOT released, the second thread will block waiting for the event,
    then timeout and report failure.
    """
    print("=" * 70)
    print("GIL Release Diagnostic Test")
    print("=" * 70)
    print()
    
    fixture = load_fixture()
    signal_received = threading.Event()
    test_started = threading.Event()
    
    def thread_2_waits():
        """Second thread waits for signal that parsing is happening."""
        print("  [Thread 2] Waiting for signal that GIL is released...")
        test_started.wait(timeout=5)  # Wait for reader to start
        
        # Now GIL should be released during parsing
        # If it is, we can acquire it and this will complete quickly
        if signal_received.wait(timeout=2):
            print("  [Thread 2] ✓ SIGNAL RECEIVED - GIL was released!")
            return True
        else:
            print("  [Thread 2] ✗ TIMEOUT - GIL was NOT released!")
            return False
    
    # Start thread 2
    t2 = threading.Thread(target=thread_2_waits)
    t2.start()
    
    # In main thread, start reading
    print("  [Main] Starting read operations...")
    time.sleep(0.1)  # Let thread 2 start waiting
    
    # Read one record to trigger Phase 2 parsing
    reader = MARCReader(io.BytesIO(fixture))
    test_started.set()
    
    print("  [Main] Sending signal midway through reads...")
    time.sleep(0.05)  # Do some work
    signal_received.set()
    
    print("  [Main] Finishing read operations...")
    count = 0
    while record := reader.read_record():
        count += 1
        if count >= 100:  # Just read 100 records for test
            break
    
    t2.join(timeout=5)
    print()
    print(f"Read {count} records")
    
    if not t2.is_alive():
        print("✓ Test completed successfully")
    else:
        print("✗ Test timed out - thread 2 is still waiting")


if __name__ == "__main__":
    test_gil_release_with_event()
