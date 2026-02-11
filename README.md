# MRRC: MARC Rust Crate

[![Tests](https://github.com/dchud/mrrc/actions/workflows/test.yml/badge.svg)](https://github.com/dchud/mrrc/actions/workflows/test.yml)
[![Lint](https://github.com/dchud/mrrc/actions/workflows/lint.yml/badge.svg)](https://github.com/dchud/mrrc/actions/workflows/lint.yml)
[![Build](https://github.com/dchud/mrrc/actions/workflows/build.yml/badge.svg)](https://github.com/dchud/mrrc/actions/workflows/build.yml)
[![codecov](https://codecov.io/gh/dchud/mrrc/branch/main/graph/badge.svg)](https://codecov.io/gh/dchud/mrrc)
[![CodSpeed](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json?org=dchud&repo=mrrc)](https://codspeed.io/dchud/mrrc)

A Rust library for reading, writing, and manipulating MARC bibliographic records, with Python bindings.

**Note:** This project was developed using agentic coding tools ([amp](https://ampcode.com/) and [Claude](https://claude.ai/)) and uses [beads](https://github.com/steveyegge/beads) for agentic issue tracking. The package has not yet had extensive practical testing by humans and should be considered experimental.

## Features

- Reads and writes ISO 2709 (MARC21) binary format
- Python bindings with pymarc-compatible API (minor differences documented)
- Multiple serialization formats: JSON, XML, MARCJSON, CSV, Dublin Core, MODS, BIBFRAME
- MARC-8 and UTF-8 character encoding support
- Benchmarked at ~4x pymarc throughput in Python, ~1M records/sec in Rust

## Installation

**Python** (3.10+):

```bash
pip install mrrc
# or with uv:
uv add mrrc
```

**Rust**:

```bash
cargo add mrrc
```

## Example

**Python:**

```python
from mrrc import MARCReader

# Pass filename directly for best performance (releases GIL)
for record in MARCReader("records.mrc"):
    print(record.title())
```

> File paths use pure Rust I/O, releasing Python's GIL for multi-threaded workloads. See the [threading guide](https://dchud.github.io/mrrc/guides/threading-python/) for details.

**Rust:**

```rust,ignore
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
- [Python Tutorials](https://dchud.github.io/mrrc/tutorials/python/reading-records/)
- [Rust Tutorials](https://dchud.github.io/mrrc/tutorials/rust/reading-records/)
- [API Reference](https://dchud.github.io/mrrc/reference/)
- [Migration from pymarc](https://dchud.github.io/mrrc/guides/migration-from-pymarc/)

## Format Support

| Format | Read | Write |
|--------|------|-------|
| ISO 2709 | Yes | Yes |
| JSON | Yes | Yes |
| MARCJSON | Yes | Yes |
| XML | Yes | Yes |
| CSV | - | Yes |
| Dublin Core | - | Yes |
| MODS | Yes | Yes |
| BIBFRAME | Yes | Yes |

[Full format reference](https://dchud.github.io/mrrc/reference/formats/)

## Platforms

Pre-built Python wheels are available for:

| Platform | Architectures |
|----------|---------------|
| Linux | `x86_64`, `aarch64`, `i686` |
| macOS | `x86_64` (Intel), `arm64` (Apple Silicon) |
| Windows | x64 |

## Status

**Experimental.** The Python API aims for pymarc compatibility but has some differences; see the [migration guide](https://dchud.github.io/mrrc/guides/migration-from-pymarc/). Rust APIs may change between minor versions.

## Roadmap

Version 0.7.0 is suitable for testing but remains experimental. Before a 1.0 release, we plan to complete:

1. **Real-world data testing** — Validate against large-scale MARC datasets from LOC, Internet Archive, and other sources to discover edge cases
2. **Code review** — Thorough review of the codebase, particularly the Rust core and `PyO3` bindings
3. **Performance analysis** — Profile with production workloads, optimize bottlenecks, and update benchmark documentation

## License

MIT

## Links

- [Documentation](https://dchud.github.io/mrrc/)
- [PyPI](https://pypi.org/project/mrrc/)
- [crates.io](https://crates.io/crates/mrrc/)
- [GitHub](https://github.com/dchud/mrrc)
