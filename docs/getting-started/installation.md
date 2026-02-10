# Installation

## Python

**Requirements**: Python 3.10+

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

All formats (ISO 2709, JSON, XML, CSV, Dublin Core, MODS, BIBFRAME) are included by default. No feature flags are needed.

## Building from Source

### Python

With pip:

```bash
git clone https://github.com/dchud/mrrc.git
cd mrrc
python -m venv venv
source venv/bin/activate
pip install maturin
maturin develop --release
```

Or with uv:

```bash
git clone https://github.com/dchud/mrrc.git
cd mrrc
uv venv
uv pip install maturin
uv run maturin develop --release
```

### Rust

```bash
git clone https://github.com/dchud/mrrc.git
cd mrrc
cargo build --release
```

## Troubleshooting

### Python: "No matching distribution found"

Ensure you're using a supported Python version (3.10-3.14) and platform.

## Next Steps

- [Python Quickstart](quickstart-python.md)
- [Rust Quickstart](quickstart-rust.md)
