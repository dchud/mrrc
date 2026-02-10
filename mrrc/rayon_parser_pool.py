"""
Rayon Parser Pool - Parallel MARC record parsing.

This module exposes the Rust Rayon-based parallel parser pool for batch parsing
of MARC records. It is particularly useful when you have record boundaries
from RecordBoundaryScanner and want to parse them in parallel.

# Example Usage

```python
from mrrc import RecordBoundaryScanner
from mrrc.rayon_parser_pool import parse_batch_parallel
import io

# Read a MARC file
with open('records.mrc', 'rb') as f:
    buffer = f.read()

# Scan for record boundaries
scanner = RecordBoundaryScanner()
boundaries = scanner.scan(buffer)

# Parse all records in parallel (respects RAYON_NUM_THREADS env var)
records = parse_batch_parallel(boundaries, buffer)

print(f"Parsed {len(records)} records in parallel")
for record in records:
    print(f"  {record.title()}")
```

# Thread Configuration

The parser respects the `RAYON_NUM_THREADS` environment variable. Set it
before importing mrrc to control parallelism:

```bash
RAYON_NUM_THREADS=4 python my_script.py
```

By default, Rayon will use all available CPU cores.
"""

from ._mrrc import (
    parse_batch_parallel as _parse_batch_parallel,
    parse_batch_parallel_limited as _parse_batch_parallel_limited,
)
from typing import List, Tuple, Union

__all__ = ["parse_batch_parallel", "parse_batch_parallel_limited"]


def parse_batch_parallel(
    boundaries: List[Tuple[int, int]], buffer: Union[bytes, bytearray]
) -> List:
    """Parse a batch of MARC record boundaries in parallel using Rayon.

    Given a buffer and a list of record boundaries (offset, length pairs),
    this function parses each record in parallel using Rayon's work-stealing
    thread pool. This is the recommended approach when you have many records
    to parse and want to utilize multiple CPU cores.

    # Arguments

    - `boundaries`: List of (offset, length) tuples identifying record boundaries.
                    These are typically obtained from RecordBoundaryScanner.scan().
    - `buffer`: The complete binary buffer containing all records.
                Can be bytes or bytearray.

    # Returns

    A list of Record objects, one for each boundary, in the same order.

    # Raises

    - `ValueError`: If any boundary exceeds the buffer size or if a record fails to parse.

    # Example

    ```python
    from mrrc import RecordBoundaryScanner
    from mrrc.rayon_parser_pool import parse_batch_parallel

    with open('records.mrc', 'rb') as f:
        buffer = f.read()

    scanner = RecordBoundaryScanner()
    boundaries = scanner.scan(buffer)

    # Parse all records in parallel
    records = parse_batch_parallel(boundaries, buffer)
    ```

    # Thread Configuration

    The number of threads used is controlled by the `RAYON_NUM_THREADS` environment
    variable. By default, Rayon uses all available CPU cores:

    ```bash
    RAYON_NUM_THREADS=4 python script.py  # Use 4 threads
    RAYON_NUM_THREADS=1 python script.py  # Single thread (useful for testing)
    ```

    # Performance Notes

    - Ideal for >10 records; overhead may exceed benefit for small batches
    - Memory usage scales with boundary count (each thread has its own stack)
    - GIL is released during parsing, allowing other Python threads to run
    """
    return _parse_batch_parallel(boundaries, buffer)


def parse_batch_parallel_limited(
    boundaries: List[Tuple[int, int]], buffer: Union[bytes, bytearray], limit: int
) -> List:
    """Parse a limited batch of MARC records in parallel.

    Like parse_batch_parallel(), but limits the number of records to parse.
    Useful for pipeline stages that need to control batch size.

    # Arguments

    - `boundaries`: List of (offset, length) tuples
    - `buffer`: The complete binary buffer
    - `limit`: Maximum number of records to parse

    # Returns

    A list of up to `limit` Record objects, in order.

    # Example

    ```python
    from mrrc import RecordBoundaryScanner
    from mrrc.rayon_parser_pool import parse_batch_parallel_limited

    with open('records.mrc', 'rb') as f:
        buffer = f.read()

    scanner = RecordBoundaryScanner()
    boundaries = scanner.scan(buffer)

    # Parse only first 10 records in parallel
    records = parse_batch_parallel_limited(boundaries, buffer, 10)
    ```
    """
    return _parse_batch_parallel_limited(boundaries, buffer, limit)
