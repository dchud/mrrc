# Parallel Processing with MRRC

**This documentation has been consolidated. See [threading.md](threading.md) for comprehensive coverage of:**

- **Pattern 1: ProducerConsumerPipeline** - Single-file high-throughput multi-threading (3.74x speedup on 4 cores)
- **Pattern 2: ThreadPoolExecutor** - Multi-file concurrent processing
- **Pattern 3: Multiprocessing** - CPU-intensive workloads
- **GIL behavior** - Automatic release during I/O and parsing
- **Thread safety** - Best practices and gotchas
- **Memory usage** - Efficient patterns with threading
- **Debugging** - Logging and detecting deadlocks
- **Performance tuning** - Thread pool sizing, batch optimization

## Quick Summary

**For a single large file:** Use `ProducerConsumerPipeline` (recommended, 3.74x speedup)

```python
from mrrc import ProducerConsumerPipeline, PipelineConfig

pipeline = ProducerConsumerPipeline.from_file('large_file.mrc', PipelineConfig())
for record in pipeline.into_iter():
    # Process record
    ...
```

**For multiple files:** Use `ThreadPoolExecutor` (standard Python pattern, 3-4x speedup)

```python
from concurrent.futures import ThreadPoolExecutor
from mrrc import MARCReader

def process_file(filename):
    reader = MARCReader(filename)
    for record in reader:
        # Process record
        ...

with ThreadPoolExecutor(max_workers=4) as executor:
    executor.map(process_file, files)
```

See [threading.md](threading.md) for complete examples, configuration options, and troubleshooting.
