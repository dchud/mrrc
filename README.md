# MRRC: MARC Rust Crate

[![Tests](https://github.com/dchud/mrrc/actions/workflows/test.yml/badge.svg)](https://github.com/dchud/mrrc/actions/workflows/test.yml)
[![Lint](https://github.com/dchud/mrrc/actions/workflows/lint.yml/badge.svg)](https://github.com/dchud/mrrc/actions/workflows/lint.yml)
[![Build](https://github.com/dchud/mrrc/actions/workflows/build.yml/badge.svg)](https://github.com/dchud/mrrc/actions/workflows/build.yml)
[![codecov](https://codecov.io/gh/dchud/mrrc/branch/main/graph/badge.svg)](https://codecov.io/gh/dchud/mrrc)

A Rust library for reading, writing, and manipulating MARC bibliographic records, with Python bindings.

## Features

- Reads and writes ISO 2709 (MARC21) binary format
- Python bindings with pymarc-compatible API (minor differences documented)
- Multiple serialization formats: JSON, XML, MARCJSON, CSV, Protobuf, Arrow, and others
- MARC-8 and UTF-8 character encoding support
- Benchmarked at ~4x pymarc throughput in Python, ~1M records/sec in Rust

## Installation

**Python** (3.9+):

```bash
pip install mrrc
# or with uv:
uv add mrrc
```

**Rust**:

```bash
cargo add mrrc
```

For optional formats in Rust:

```bash
cargo add mrrc --features format-arrow,format-protobuf
```

## Example

**Python:**

```python
from mrrc import MARCReader

with open("records.mrc", "rb") as f:
    for record in MARCReader(f):
        print(record.title())
```

**Rust:**

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

## Documentation

- [Getting Started](https://dchud.github.io/mrrc/getting-started/)
- [Python Tutorial](https://dchud.github.io/mrrc/tutorials/python/)
- [Rust Tutorial](https://dchud.github.io/mrrc/tutorials/rust/)
- [API Reference](https://dchud.github.io/mrrc/reference/)
- [Migration from pymarc](https://dchud.github.io/mrrc/guides/migration-from-pymarc/)

## Format Support

| Format | Read | Write | Notes |
|--------|------|-------|-------|
| ISO 2709 | Yes | Yes | Standard MARC binary |
| JSON | Yes | Yes | Generic JSON |
| MARCJSON | Yes | Yes | LOC standard |
| XML | Yes | Yes | MARCXML |
| CSV | - | Yes | Tabular export |
| Protobuf | Yes | Yes | Feature-gated |
| Arrow | Yes | Yes | Feature-gated |
| FlatBuffers | Yes | Yes | Feature-gated |
| MessagePack | Yes | Yes | Feature-gated |
| BIBFRAME | - | Yes | Feature-gated |

[Full format matrix](https://dchud.github.io/mrrc/reference/formats/)

## Platforms

Pre-built Python wheels are available for:

| Platform | Architectures |
|----------|---------------|
| Linux | x86_64, aarch64 |
| macOS | x86_64 (Intel), arm64 (Apple Silicon) |
| Windows | x64 |

## Status

**Experimental.** The Python API aims for pymarc compatibility but has some differences; see the [migration guide](https://dchud.github.io/mrrc/guides/migration-from-pymarc/). Rust APIs may change between minor versions.

## License

MIT

## Links

- [Documentation](https://dchud.github.io/mrrc/)
- [PyPI](https://pypi.org/project/mrrc/)
- [crates.io](https://crates.io/crates/mrrc/)
- [GitHub](https://github.com/dchud/mrrc)
