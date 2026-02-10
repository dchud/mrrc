# Examples

Runnable example code demonstrating MRRC capabilities.

## Example Index

### Reading and Querying

| Example | Python | Rust | Description |
|---------|--------|------|-------------|
| Reading Records | [reading_and_querying.py] | [reading_and_querying.rs] | Basic file reading and field access |
| Field Queries | [reading_and_querying.py] | [reading_and_querying.rs] | Query DSL for finding specific fields |

### Creating and Writing

| Example | Python | Rust | Description |
|---------|--------|------|-------------|
| Creating Records | [creating_records.py] | [creating_records.rs] | Build records from scratch |
| Authority Records | [authority_records.py] | [authority_records.rs] | Work with authority data |

### Format Conversion

| Example | Python | Rust | Description |
|---------|--------|------|-------------|
| Format Conversion | [format_conversion.py] | [format_conversion.rs] | Convert between formats (JSON, XML, etc.) |
| CSV Export | - | [marc_to_csv.rs] | Export records to CSV |

### Character Encoding

| Example | Python | Rust | Description |
|---------|--------|------|-------------|
| MARC-8 Encoding | [marc8_encoding.py] | [marc8_encoding.rs] | Handle legacy MARC-8 character encoding |
| Multilingual | [multilingual_records.py] | [multilingual_records.rs] | Non-Latin scripts and Unicode |

### Concurrent Processing

| Example | Python | Rust | Description |
|---------|--------|------|-------------|
| Concurrent Reading | [concurrent_reading.py] | [concurrent_reading.rs] | Multi-threaded file reading |
| Concurrent Writing | [concurrent_writing.py] | [concurrent_writing.rs] | Multi-threaded file writing |
| Producer-Consumer | [concurrent_reading_producer_consumer.py] | - | Queue-based parallel processing |
| Rayon Parallelism | - | [rayon_poc.rs] | Rust Rayon parallel iterators |

### BIBFRAME Conversion

| Example | Python | Rust | Description |
|---------|--------|------|-------------|
| MARC to BIBFRAME | [marc_to_bibframe.py] | [marc_to_bibframe.rs] | Convert MARC to BIBFRAME RDF |
| BIBFRAME to MARC | - | [bibframe_to_marc.rs] | Convert BIBFRAME back to MARC |
| BIBFRAME Config | [bibframe_config.py] | - | Configuration options |
| BIBFRAME Batch | - | [bibframe_batch.rs] | Batch conversion |
| Round-trip | [bibframe_roundtrip.py] | - | MARC → BIBFRAME → MARC |

## Running Examples

### Python

```bash
# Run directly
python examples/reading_and_querying.py

# With sample data
python examples/reading_and_querying.py path/to/records.mrc
```

### Rust

```bash
# Run with cargo
cargo run --example reading_and_querying

# Release mode for performance testing
cargo run --release --example concurrent_reading
```

## Difficulty Levels

Examples are organized by complexity:

- **Beginner**: Reading, creating, basic format conversion
- **Intermediate**: Encoding, queries, format workflows
- **Advanced**: Concurrency, BIBFRAME, analytics integration

## Sample Data

Examples use test data from `tests/fixtures/`:

- `test_records.mrc` - Small set of English-language records
- `multilingual.mrc` - Records with non-Latin scripts
- `authority.mrc` - Authority records

## See Also

- [Python Quickstart](../getting-started/quickstart-python.md)
- [Rust Quickstart](../getting-started/quickstart-rust.md)
- [Tutorials](../tutorials/index.md) - Step-by-step guides

<!-- Link definitions -->
[reading_and_querying.py]: https://github.com/dchud/mrrc/blob/main/examples/reading_and_querying.py
[reading_and_querying.rs]: https://github.com/dchud/mrrc/blob/main/examples/reading_and_querying.rs
[creating_records.py]: https://github.com/dchud/mrrc/blob/main/examples/creating_records.py
[creating_records.rs]: https://github.com/dchud/mrrc/blob/main/examples/creating_records.rs
[authority_records.py]: https://github.com/dchud/mrrc/blob/main/examples/authority_records.py
[authority_records.rs]: https://github.com/dchud/mrrc/blob/main/examples/authority_records.rs
[format_conversion.py]: https://github.com/dchud/mrrc/blob/main/examples/format_conversion.py
[format_conversion.rs]: https://github.com/dchud/mrrc/blob/main/examples/format_conversion.rs
[marc_to_csv.rs]: https://github.com/dchud/mrrc/blob/main/examples/marc_to_csv.rs
[marc8_encoding.py]: https://github.com/dchud/mrrc/blob/main/examples/marc8_encoding.py
[marc8_encoding.rs]: https://github.com/dchud/mrrc/blob/main/examples/marc8_encoding.rs
[multilingual_records.py]: https://github.com/dchud/mrrc/blob/main/examples/multilingual_records.py
[multilingual_records.rs]: https://github.com/dchud/mrrc/blob/main/examples/multilingual_records.rs
[concurrent_reading.py]: https://github.com/dchud/mrrc/blob/main/examples/concurrent_reading.py
[concurrent_reading.rs]: https://github.com/dchud/mrrc/blob/main/examples/concurrent_reading.rs
[concurrent_writing.py]: https://github.com/dchud/mrrc/blob/main/examples/concurrent_writing.py
[concurrent_writing.rs]: https://github.com/dchud/mrrc/blob/main/examples/concurrent_writing.rs
[concurrent_reading_producer_consumer.py]: https://github.com/dchud/mrrc/blob/main/examples/concurrent_reading_producer_consumer.py
[rayon_poc.rs]: https://github.com/dchud/mrrc/blob/main/examples/rayon_poc.rs
[marc_to_bibframe.py]: https://github.com/dchud/mrrc/blob/main/examples/python/marc_to_bibframe.py
[marc_to_bibframe.rs]: https://github.com/dchud/mrrc/blob/main/examples/marc_to_bibframe.rs
[bibframe_to_marc.rs]: https://github.com/dchud/mrrc/blob/main/examples/bibframe_to_marc.rs
[bibframe_config.py]: https://github.com/dchud/mrrc/blob/main/examples/python/bibframe_config.py
[bibframe_batch.rs]: https://github.com/dchud/mrrc/blob/main/examples/bibframe_batch.rs
[bibframe_roundtrip.py]: https://github.com/dchud/mrrc/blob/main/examples/python/bibframe_roundtrip.py
