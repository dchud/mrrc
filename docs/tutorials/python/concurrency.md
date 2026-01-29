# Concurrency (Python)

Learn to process MARC records in parallel using Python.

## Why Concurrency?

MRRC releases Python's GIL during record parsing, enabling true parallel processing:

- 2 threads: ~2x speedup
- 4 threads: ~3x speedup
- Ideal for processing multiple files or large datasets

## File Path vs File Object

For best performance, pass file paths directly:

```python
import mrrc

# Faster: file path (GIL released during I/O)
for record in mrrc.MARCReader("records.mrc"):
    print(record.title())

# Slower: file object (GIL held for Python I/O)
with open("records.mrc", "rb") as f:
    for record in mrrc.MARCReader(f):
        print(record.title())
```

## Processing Multiple Files

Use ThreadPoolExecutor to process files in parallel:

```python
from concurrent.futures import ThreadPoolExecutor
import mrrc

def process_file(path):
    """Process a single MARC file."""
    count = 0
    for record in mrrc.MARCReader(path):
        if record.title():
            count += 1
    return count

# Process 4 files in parallel
files = ["file1.mrc", "file2.mrc", "file3.mrc", "file4.mrc"]
with ThreadPoolExecutor(max_workers=4) as executor:
    results = list(executor.map(process_file, files))

print(f"Total records: {sum(results)}")
```

## Producer-Consumer Pipeline

For high-throughput processing of a single large file:

```python
from mrrc import ProducerConsumerPipeline

# Create pipeline (auto-scales to CPU cores)
pipeline = ProducerConsumerPipeline.from_file("large_file.mrc")

# Process records
for record in pipeline:
    # Your processing logic here
    print(record.title())
```

The pipeline achieves ~3.7x speedup on 4 cores by:

1. **Producer thread**: Reads record bytes from disk
2. **Parser threads**: Parse bytes into records (GIL released)
3. **Consumer**: Receives parsed records in order

## Thread Safety Rules

**Do:**

- Create one reader per thread
- Use file paths for maximum parallelism
- Use ThreadPoolExecutor for multi-file processing

**Don't:**

- Share a MARCReader across threads
- Pass file objects between threads

## Complete Example

```python
#!/usr/bin/env python3
"""Process MARC files in parallel."""

from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
import mrrc

def extract_titles(path):
    """Extract all titles from a MARC file."""
    titles = []
    for record in mrrc.MARCReader(str(path)):
        if title := record.title():
            titles.append(title)
    return path.name, titles

def main():
    # Find all .mrc files
    marc_files = list(Path("data").glob("*.mrc"))
    print(f"Found {len(marc_files)} MARC files")

    all_titles = {}

    # Process in parallel
    with ThreadPoolExecutor(max_workers=4) as executor:
        futures = {executor.submit(extract_titles, f): f for f in marc_files}

        for future in as_completed(futures):
            filename, titles = future.result()
            all_titles[filename] = titles
            print(f"{filename}: {len(titles)} titles")

    total = sum(len(t) for t in all_titles.values())
    print(f"Total: {total} titles from {len(marc_files)} files")

if __name__ == "__main__":
    main()
```

## Benchmarks

Typical speedups on a 4-core system:

| Approach | Speedup | Use Case |
|----------|---------|----------|
| Sequential | 1x | Baseline |
| 2 threads | 2.0x | Light parallelism |
| 4 threads | 3.2x | Multiple files |
| ProducerConsumer | 3.7x | Single large file |

## When to Use What

| Scenario | Approach |
|----------|----------|
| Single small file | Sequential reading |
| Multiple files | ThreadPoolExecutor |
| One large file | ProducerConsumerPipeline |
| Memory constrained | Sequential with streaming |

## Next Steps

- [Reading Records](reading-records.md) - Basic record access
- [Threading Guide](../../guides/threading-python.md) - Advanced patterns
- [Performance Tuning](../../guides/performance-tuning.md) - Optimization tips
