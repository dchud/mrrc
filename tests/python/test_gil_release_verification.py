"""
Deterministic verification that the GIL is actually released during iteration.

Uses sys.setswitchinterval(100.0) to suppress CPython's automatic GIL switching,
then proves a background counter thread makes progress during iteration. The counter
can ONLY advance if the main thread explicitly releases the GIL via py.detach().

If py.detach() were removed, automatic switching is disabled (100s interval),
so the counter stays at 0 and the test fails.

See: src-python/src/readers.rs lines 232-249 for the py.detach() call.
"""

import io
import sys
import threading
import time

import pytest
from mrrc import MARCReader


@pytest.fixture
def suppress_auto_switching():
    """Disable CPython automatic GIL switching for the duration of the test."""
    original = sys.getswitchinterval()
    sys.setswitchinterval(100.0)
    yield
    sys.setswitchinterval(original)


@pytest.fixture
def mrc_file(fixture_1k, tmp_path):
    """Write fixture to a temp .mrc file for RustFile backend."""
    path = tmp_path / "gil_test.mrc"
    path.write_bytes(fixture_1k)
    return path


def _run_with_counter_thread(iterate_fn):
    """Run iterate_fn on main thread while a background thread increments a counter.

    Returns (counter_value, record_count). The counter can only advance if the
    main thread releases the GIL during iteration.
    """
    counter = [0]
    stop = threading.Event()

    def count_loop():
        while not stop.is_set():
            counter[0] += 1
            # Yield GIL cooperatively so the main thread can re-acquire it
            # after py.detach() returns. Without this, setswitchinterval(100.0)
            # would let the counter hold the GIL for 100s, starving the main thread.
            time.sleep(0)

    t = threading.Thread(target=count_loop, daemon=True)
    t.start()
    record_count = iterate_fn()
    stop.set()
    t.join(timeout=2.0)
    return counter[0], record_count


class TestGILReleaseVerification:
    """Verify that py.detach() actually releases the GIL during record parsing."""

    def test_rustfile_releases_gil(self, suppress_auto_switching, mrc_file):
        reader = MARCReader(str(mrc_file))
        assert reader.backend_type == "rust_file"

        def iterate():
            count = 0
            for _ in reader:
                count += 1
            return count

        counter, record_count = _run_with_counter_thread(iterate)
        assert record_count == 1000
        assert counter > 0, "Counter thread never ran — GIL was not released"

    def test_cursor_releases_gil(self, suppress_auto_switching, fixture_1k):
        reader = MARCReader(fixture_1k)
        assert reader.backend_type == "cursor"

        def iterate():
            count = 0
            for _ in reader:
                count += 1
            return count

        counter, record_count = _run_with_counter_thread(iterate)
        assert record_count == 1000
        assert counter > 0, "Counter thread never ran — GIL was not released"

    def test_python_file_releases_gil(self, suppress_auto_switching, fixture_1k):
        reader = MARCReader(io.BytesIO(fixture_1k))
        assert reader.backend_type == "python_file"

        def iterate():
            count = 0
            for _ in reader:
                count += 1
            return count

        counter, record_count = _run_with_counter_thread(iterate)
        assert record_count == 1000
        assert counter > 0, "Counter thread never ran — GIL was not released"
