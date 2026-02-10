# MRRC

A Rust library for MARC bibliographic records, with Python bindings.

## What MRRC Does

- Reads and writes ISO 2709 (MARC21) binary format
- Provides Python bindings with a pymarc-compatible API
- Supports multiple serialization formats (JSON, XML, CSV, Dublin Core, MODS, BIBFRAME)
- Handles MARC-8 and UTF-8 character encodings

## Performance

In benchmarks (see [methodology](benchmarks/results.md)):

- Python: ~4x throughput compared to pymarc
- Rust: ~1M records/sec

## Quick Example

=== "Python"

    ```python
    from mrrc import MARCReader

    with open("records.mrc", "rb") as f:
        for record in MARCReader(f):
            print(record.title())
    ```

=== "Rust"

    ```rust
    use mrrc::MarcReader;
    use std::fs::File;

    let file = File::open("records.mrc")?;
    let mut reader = MarcReader::new(file);
    while let Some(record) = reader.read_record()? {
        if let Some(title) = record.title() {
            println!("{}", title);
        }
    }
    ```

## Getting Started

- [Installation](getting-started/installation.md)
- [Python Quickstart](getting-started/quickstart-python.md)
- [Rust Quickstart](getting-started/quickstart-rust.md)

## pymarc Users

MRRC's Python API is similar to pymarc but not identical. See the [migration guide](guides/migration-from-pymarc.md) for specific differences.

## Status

This library is experimental. APIs may change between versions.
