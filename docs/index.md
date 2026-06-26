# MRRC

A Rust library for MARC bibliographic records, with Python bindings.

## What MRRC Does

- Reads and writes ISO 2709 (MARC21) binary format
- Provides Python bindings with a pymarc-compatible API
- Supports multiple serialization formats (JSON, XML, CSV, Dublin Core, MODS, BIBFRAME)
- Handles MARC-8 and UTF-8 character encodings

## Performance

Record parsing runs in Rust, and the Python bindings release the GIL during
parsing so multi-threaded workloads parse in parallel. On a realistic corpus
the Python wrapper reads roughly 7× faster than pymarc per record and ~30×
through the parallel batch path, and the native Rust crate is faster still.
See [benchmarks](benchmarks/results.md) for the full three-way comparison and
how to benchmark mrrc on your own hardware and data.

## Quick Example

=== "Python"

    ```python
    from mrrc import MARCReader

    for record in MARCReader("records.mrc"):
        print(record.title)
    ```

=== "Rust"

    ```rust
    use mrrc::{MarcReader, RecordHelpers};
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
