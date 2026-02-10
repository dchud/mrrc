# Tutorials

Step-by-step guides for learning MRRC. Each tutorial builds on previous concepts.

## Choose Your Path

MRRC provides both Python and Rust APIs. Choose based on your needs:

| If you... | Start with |
|-----------|------------|
| Are a Python developer | [Python Tutorials](#python-tutorials) |
| Are a Rust developer | [Rust Tutorials](#rust-tutorials) |
| Want maximum performance | Rust (native) or Python with threading |
| Are migrating from pymarc | [Python Tutorials](#python-tutorials) + [Migration Guide](../guides/migration-from-pymarc.md) |

## Python Tutorials

For Python developers. Covers the PyO3 bindings with pymarc-compatible API.

1. [Reading Records](python/reading-records.md) - Load MARC files and iterate over records
2. [Writing Records](python/writing-records.md) - Create and save MARC records
3. [Format Conversion](python/format-conversion.md) - Convert between JSON, XML, and other formats
4. [Querying Fields](python/querying-fields.md) - Use the Query DSL for complex field matching
5. [Concurrency](python/concurrency.md) - Parallel processing with threading

## Rust Tutorials

For Rust developers. Covers the native API with builder patterns and traits.

1. [Reading Records](rust/reading-records.md) - Parse MARC files with MarcReader
2. [Writing Records](rust/writing-records.md) - Build records with the builder pattern
3. [Format Conversion](rust/format-conversion.md) - Serialize to JSON, XML, BIBFRAME, and more
4. [Querying Fields](rust/querying-fields.md) - Use query types for field filtering
5. [Concurrency](rust/concurrency.md) - Parallel processing with Rayon

## Prerequisites

Before starting the tutorials:

- **Python**: Install with `pip install mrrc` or `uv add mrrc`
- **Rust**: Add to Cargo.toml with `cargo add mrrc`

See [Installation](../getting-started/installation.md) for detailed setup instructions.

## Next Steps

After completing the tutorials:

- [Guides](../guides/index.md) - In-depth coverage of specific topics
- [Reference](../reference/index.md) - API documentation
- [Examples](../examples/index.md) - Annotated code examples
