# Installation Guide

This guide covers installing MRRC for both Python and Rust, including optional features and building from source.

## Quick Install

### Python

```bash
pip install mrrc
```

### Rust

```toml
[dependencies]
mrrc = "0.4"
```

## Python Installation

### Requirements

- Python 3.9, 3.10, 3.11, or 3.12
- pip or your preferred package manager

### Supported Platforms

Pre-built wheels are available for:

| Platform | Architectures |
|----------|---------------|
| Linux | x86_64, aarch64 |
| macOS | x86_64 (Intel), arm64 (Apple Silicon) |
| Windows | x64, arm64 |

### Basic Installation

```bash
pip install mrrc
```

### Optional Dependencies

MRRC provides optional extras for enhanced functionality:

```bash
# Development tools (pytest, mypy, ruff)
pip install mrrc[dev]

# Benchmarking support
pip install mrrc[benchmark]

# All optional dependencies
pip install mrrc[dev,benchmark]
```

### Verifying Installation

```python
import mrrc
print(mrrc.__version__)

# Quick test
with open("test.mrc", "rb") as f:
    reader = mrrc.MARCReader(f)
    for record in reader:
        print(record.title())
```

### Upgrading

```bash
pip install --upgrade mrrc
```

## Rust Installation

### Requirements

- Rust 1.71 or later
- Cargo (included with Rust)

### Basic Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
mrrc = "0.4"
```

### Feature Flags

MRRC uses feature flags for optional format support. Core formats (ISO 2709, Protobuf, JSON, XML, CSV) are always available.

#### Optional Formats

```toml
[dependencies]
# Apache Arrow columnar format (DuckDB, Polars integration)
mrrc = { version = "0.4", features = ["format-arrow"] }

# FlatBuffers zero-copy format
mrrc = { version = "0.4", features = ["format-flatbuffers"] }

# MessagePack compact binary format
mrrc = { version = "0.4", features = ["format-messagepack"] }

# CBOR binary format
mrrc = { version = "0.4", features = ["format-cbor"] }

# Apache Avro format
mrrc = { version = "0.4", features = ["format-avro"] }
```

#### Multiple Features

```toml
[dependencies]
mrrc = { version = "0.4", features = ["format-arrow", "format-messagepack"] }
```

#### All Formats

```toml
[dependencies]
mrrc = { version = "0.4", features = ["all-formats"] }
```

### Feature Reference

| Feature | Description | Dependencies Added |
|---------|-------------|--------------------|
| `format-arrow` | Apache Arrow columnar format | arrow, parquet |
| `format-flatbuffers` | FlatBuffers zero-copy format | flatbuffers |
| `format-messagepack` | MessagePack compact format | rmp-serde |
| `format-cbor` | CBOR binary format | ciborium |
| `format-avro` | Apache Avro format | apache-avro |
| `all-formats` | All optional formats | All above |

### Verifying Installation

```rust
use mrrc::{MarcReader, Record};
use std::fs::File;

fn main() -> mrrc::Result<()> {
    let file = File::open("test.mrc")?;
    let mut reader = MarcReader::new(file);

    while let Some(record) = reader.read_record()? {
        if let Some(title) = record.title() {
            println!("{}", title);
        }
    }
    Ok(())
}
```

## Building from Source

### Python (with maturin)

```bash
# Clone repository
git clone https://github.com/dchud/mrrc.git
cd mrrc

# Create virtual environment
python -m venv venv
source venv/bin/activate  # Linux/macOS
# or: venv\Scripts\activate  # Windows

# Install maturin
pip install maturin

# Build and install in development mode
maturin develop

# Or build a wheel
maturin build --release
pip install target/wheels/*.whl
```

### Rust

```bash
# Clone repository
git clone https://github.com/dchud/mrrc.git
cd mrrc

# Build
cargo build --release

# Run tests
cargo test

# Build with all features
cargo build --release --features all-formats
```

### Running Tests

```bash
# Rust tests
cargo test

# Python tests
pytest tests/python/

# With benchmarks
pytest tests/python/ -m benchmark --benchmark-only
```

## Troubleshooting

### Python: "No matching distribution found"

If pip can't find a wheel for your platform:

1. Ensure you're using a supported Python version (3.9-3.12)
2. Upgrade pip: `pip install --upgrade pip`
3. Try building from source (see above)

### Rust: Feature not found

Ensure your Cargo.toml syntax is correct:

```toml
# Correct
mrrc = { version = "0.4", features = ["format-arrow"] }

# Incorrect (missing quotes)
mrrc = { version = 0.4, features = [format-arrow] }
```

### macOS: Architecture mismatch

If running on Apple Silicon but getting Intel binaries:

```bash
# Force native architecture
pip install --no-binary :all: mrrc

# Or specify platform
pip install mrrc --platform macosx_11_0_arm64
```

### Linux: Missing system libraries

Some features may require system libraries:

```bash
# Debian/Ubuntu
sudo apt-get install build-essential

# Fedora/RHEL
sudo dnf install gcc make
```

## Next Steps

- [Migration Guide](./MIGRATION_GUIDE.md) - Migrate from pymarc
- [Format Selection Guide](./FORMAT_SELECTION_GUIDE.md) - Choose the right format
- [Python Tutorial](./PYTHON_TUTORIAL.md) - Get started with Python
- [Rust Tutorial](./RUST_TUTORIAL.md) - Get started with Rust
