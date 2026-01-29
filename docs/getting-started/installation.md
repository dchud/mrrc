# Installation

## Python

**Requirements**: Python 3.9+

```bash
pip install mrrc
```

Or with uv:

```bash
uv add mrrc
```

### Verify Installation

```python
import mrrc
print("MRRC installed successfully")
```

### Supported Platforms

Pre-built wheels are available for:

| Platform | Architectures |
|----------|---------------|
| Linux | x86_64, aarch64 |
| macOS | x86_64 (Intel), arm64 (Apple Silicon) |
| Windows | x64 |

## Rust

**Requirements**: Rust 1.71+

Add to your `Cargo.toml`:

```toml
[dependencies]
mrrc = "0.6"
```

Or with cargo:

```bash
cargo add mrrc
```

### Feature Flags

Core formats (ISO 2709, JSON, XML) are always available. Optional formats require feature flags:

```toml
[dependencies]
mrrc = { version = "0.6", features = ["format-arrow", "format-protobuf"] }
```

| Feature | Description |
|---------|-------------|
| `format-arrow` | Apache Arrow columnar format |
| `format-protobuf` | Protocol Buffers |
| `format-flatbuffers` | FlatBuffers zero-copy |
| `format-messagepack` | MessagePack compact binary |
| `format-bibframe` | BIBFRAME RDF conversion |

## Building from Source

### Python

```bash
git clone https://github.com/dchud/mrrc.git
cd mrrc
python -m venv venv
source venv/bin/activate
pip install maturin
maturin develop --release
```

### Rust

```bash
git clone https://github.com/dchud/mrrc.git
cd mrrc
cargo build --release
```

## Troubleshooting

### Python: "No matching distribution found"

Ensure you're using a supported Python version (3.9-3.12) and platform.

### Rust: Protobuf compilation errors

Install the Protocol Buffers compiler:

```bash
# macOS
brew install protobuf

# Ubuntu/Debian
apt install protobuf-compiler

# Windows (with chocolatey)
choco install protoc
```

## Next Steps

- [Python Quickstart](quickstart-python.md)
- [Rust Quickstart](quickstart-rust.md)
