"""
Pytest configuration and fixtures for Python wrapper benchmarks.
"""

import io
from pathlib import Path

import pytest


@pytest.fixture(scope="session")
def fixture_dir():
    """Return path to test fixtures directory."""
    return Path(__file__).parent.parent / "data" / "fixtures"


@pytest.fixture(scope="session")
def fixture_1k(fixture_dir):
    """Load 1k record fixture as bytes."""
    path = fixture_dir / "1k_records.mrc"
    if not path.exists():
        pytest.skip(f"Fixture not found: {path}")
    with open(path, "rb") as f:
        return f.read()


@pytest.fixture(scope="session")
def fixture_10k(fixture_dir):
    """Load 10k record fixture as bytes."""
    path = fixture_dir / "10k_records.mrc"
    if not path.exists():
        pytest.skip(f"Fixture not found: {path}")
    with open(path, "rb") as f:
        return f.read()


def _load_fixture(fixture_dir, name):
    path = fixture_dir / name
    if not path.exists():
        pytest.skip(f"Fixture not found: {path}")
    return path.read_bytes()


@pytest.fixture(scope="session")
def fixture_realistic(fixture_dir):
    """2,000 realistic-shaped records of 12-30 fields."""
    return _load_fixture(fixture_dir, "realistic.mrc")


@pytest.fixture(scope="session")
def fixture_large(fixture_dir):
    """100 large records of 150-250 fields."""
    return _load_fixture(fixture_dir, "large_records.mrc")


@pytest.fixture(scope="session")
def fixture_very_large(fixture_dir):
    """10 very large records of 800-1,000 fields."""
    return _load_fixture(fixture_dir, "very_large_records.mrc")


@pytest.fixture(scope="session")
def fixture_marc8(fixture_dir):
    """1,000 realistic-shaped MARC-8 records (leader position 09 blank)."""
    return _load_fixture(fixture_dir, "realistic_marc8.mrc")


@pytest.fixture(scope="session")
def fixture_1k_path(fixture_dir):
    """Path to the 1k record fixture file on disk."""
    path = fixture_dir / "1k_records.mrc"
    if not path.exists():
        pytest.skip(f"Fixture not found: {path}")
    return str(path)


@pytest.fixture(scope="session")
def fixture_10k_path(fixture_dir):
    """Path to the 10k record fixture file on disk."""
    path = fixture_dir / "10k_records.mrc"
    if not path.exists():
        pytest.skip(f"Fixture not found: {path}")
    return str(path)


@pytest.fixture(scope="session")
def fixture_small(fixture_1k):
    """Load small fixture (first 10 records from 1k fixture)."""
    # Create a small subset for quick tests
    from mrrc import MARCReader

    reader = MARCReader(io.BytesIO(fixture_1k))
    small_records = []
    for i, record in enumerate(reader):
        small_records.append(record)
        if i >= 9:  # 10 records
            break

    # Serialize back to bytes
    from mrrc import MARCWriter

    output = io.BytesIO()
    writer = MARCWriter(output)
    for record in small_records:
        writer.write_record(record)
    writer.close()
    return output.getvalue()


@pytest.fixture(scope="session")
def fixture_217(fixture_1k):
    """Create a fixture with exactly 217 records (tests partial batch at EOF)."""
    from mrrc import MARCReader, MARCWriter

    reader = MARCReader(io.BytesIO(fixture_1k))
    records = []
    for i, record in enumerate(reader):
        records.append(record)
        if i >= 216:  # 217 records (0-216 inclusive)
            break

    output = io.BytesIO()
    writer = MARCWriter(output)
    for record in records:
        writer.write_record(record)
    writer.close()
    return output.getvalue()


@pytest.fixture(scope="session")
def fixture_500(fixture_1k):
    """Create a fixture with exactly 500 records."""
    from mrrc import MARCReader, MARCWriter

    reader = MARCReader(io.BytesIO(fixture_1k))
    records = []
    for i, record in enumerate(reader):
        records.append(record)
        if i >= 499:  # 500 records
            break

    output = io.BytesIO()
    writer = MARCWriter(output)
    for record in records:
        writer.write_record(record)
    writer.close()
    return output.getvalue()


@pytest.fixture(scope="session")
def fixture_5k(fixture_1k, fixture_10k):
    """Create a fixture with 5k records (halfway between 1k and 10k)."""
    # Use first 5 copies of 1k fixture
    from mrrc import MARCReader, MARCWriter

    reader = MARCReader(io.BytesIO(fixture_10k))
    records = []
    for i, record in enumerate(reader):
        records.append(record)
        if i >= 4999:  # 5000 records
            break

    output = io.BytesIO()
    writer = MARCWriter(output)
    for record in records:
        writer.write_record(record)
    writer.close()
    return output.getvalue()


@pytest.fixture(scope="session")
def fixture_with_error(fixture_small):
    """Create a fixture that contains a malformed record for error testing."""
    # Append an incomplete record header to trigger error
    return fixture_small + b"00"  # Incomplete record length field


@pytest.fixture
def fixture_1k_io(fixture_1k):
    """Return 1k fixture as file-like object."""
    return io.BytesIO(fixture_1k)


@pytest.fixture
def fixture_10k_io(fixture_10k):
    """Return 10k fixture as file-like object."""
    return io.BytesIO(fixture_10k)


def pytest_configure(config):
    """Register custom markers."""
    config.addinivalue_line(
        "markers",
        "benchmark: mark test as a benchmark (deselect with '-m \"not benchmark\"')",
    )
    config.addinivalue_line(
        "markers", "slow: mark test as slow (deselect with '-m \"not slow\"')"
    )
