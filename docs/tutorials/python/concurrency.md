# Concurrency (Python)

Learn to process MARC records in parallel using Python.

## Quick Reference

| What You're Doing | Approach | Typical Speedup |
|-------------------|----------|-----------------|
| [Reading a single file](#reading-a-single-file) | File path + sequential | 1x (but GIL-friendly) |
| [Processing multiple files](#processing-multiple-files) | ThreadPoolExecutor | 2-3x |
| [Processing one large file](#processing-a-large-file) | ProducerConsumerPipeline | 3-4x |

## Why Concurrency?

MRRC releases Python's GIL during record parsing, enabling true parallel processing:

- 2 threads: ~2x speedup
- 4 threads: ~3x speedup
- Ideal for processing multiple files or large datasets

## Reading a Single File

For single-file processing, pass file paths directly. This uses pure Rust I/O and releases the GIL during parsing, making your code "concurrency-ready" even in sequential use:

```python
from mrrc import MARCReader

# Recommended: file path (GIL released during I/O)
for record in MARCReader("records.mrc"):
    print(record.title())
```

Avoid file objects when possible—they hold the GIL during Python I/O:

```python
# Slower: file object (GIL held for Python I/O)
with open("records.mrc", "rb") as f:
    for record in MARCReader(f):
        print(record.title())
```

Use file objects only when needed (e.g., network streams, custom I/O).

## Processing Multiple Files

When you have many files to process, use `ThreadPoolExecutor` to read them in parallel:

```python
from concurrent.futures import ThreadPoolExecutor
from mrrc import MARCReader

def process_file(path):
    """Process a single MARC file."""
    count = 0
    for record in MARCReader(path):
        if record.title():
            count += 1
    return count

# Process files in parallel (one thread per file)
files = ["file1.mrc", "file2.mrc", "file3.mrc", "file4.mrc"]
with ThreadPoolExecutor(max_workers=4) as executor:
    results = list(executor.map(process_file, files))

print(f"Total records: {sum(results)}")
```

Each thread gets its own reader, and the GIL is released during parsing, so threads run truly in parallel.

## Processing a Large File

When you have a single large file, use `ProducerConsumerPipeline` to parallelize parsing:

```python
from mrrc import ProducerConsumerPipeline

# Create pipeline (auto-scales to CPU cores)
pipeline = ProducerConsumerPipeline.from_file("large_file.mrc")

# Process records (arrives in order)
for record in pipeline:
    print(record.title())
```

The pipeline achieves ~3.7x speedup on 4 cores by splitting work:

1. **Producer thread**: Reads record bytes from disk
2. **Parser threads**: Parse bytes into records in parallel (GIL released)
3. **Consumer**: Receives parsed records in original order

## Thread Safety

**Safe patterns:**

- Create one reader per thread
- Use file paths for maximum parallelism
- Use `ThreadPoolExecutor` for multi-file processing
- Use `ProducerConsumerPipeline` for single large files

**Unsafe patterns:**

- Sharing a `MARCReader` across threads
- Passing file objects between threads
- Modifying the same `Record` from multiple threads

## Complete Example

```python
#!/usr/bin/env python3
"""Process MARC files in parallel."""

from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from mrrc import MARCReader

def extract_titles(path):
    """Extract all titles from a MARC file."""
    titles = []
    for record in MARCReader(path):
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

## Performance Comparison

Typical speedups on a 4-core system:

| Approach | Speedup | Best For |
|----------|---------|----------|
| Sequential (file path) | 1x | Simple scripts, small files |
| ThreadPoolExecutor (2 threads) | 2.0x | A few files |
| ThreadPoolExecutor (4 threads) | 3.2x | Many files |
| ProducerConsumerPipeline | 3.7x | One large file |

## Next Steps

- [Reading Records](reading-records.md) - Basic record access
- [Threading Guide](../../guides/threading-python.md) - GIL behavior and advanced patterns
- [Performance Tuning](../../guides/performance-tuning.md) - Optimization tips
