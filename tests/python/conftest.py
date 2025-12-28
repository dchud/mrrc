"""
Pytest configuration and fixtures for Python wrapper benchmarks.
"""

import pytest
from pathlib import Path
import io


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
    with open(path, 'rb') as f:
        return f.read()


@pytest.fixture(scope="session")
def fixture_10k(fixture_dir):
    """Load 10k record fixture as bytes."""
    path = fixture_dir / "10k_records.mrc"
    if not path.exists():
        pytest.skip(f"Fixture not found: {path}")
    with open(path, 'rb') as f:
        return f.read()


@pytest.fixture(scope="session")
def fixture_100k(fixture_dir):
    """Load 100k record fixture as bytes."""
    path = fixture_dir / "100k_records.mrc"
    if not path.exists():
        pytest.skip(f"Fixture not found: {path}")
    with open(path, 'rb') as f:
        return f.read()


@pytest.fixture
def fixture_1k_io(fixture_1k):
    """Return 1k fixture as file-like object."""
    return io.BytesIO(fixture_1k)


@pytest.fixture
def fixture_10k_io(fixture_10k):
    """Return 10k fixture as file-like object."""
    return io.BytesIO(fixture_10k)


@pytest.fixture
def fixture_100k_io(fixture_100k):
    """Return 100k fixture as file-like object."""
    return io.BytesIO(fixture_100k)


def pytest_configure(config):
    """Register custom markers."""
    config.addinivalue_line(
        "markers",
        "benchmark: mark test as a benchmark (deselect with '-m \"not benchmark\"')"
    )
    config.addinivalue_line(
        "markers",
        "slow: mark test as slow (deselect with '-m \"not slow\"')"
    )
