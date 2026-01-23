"""Format-specific modules for MARC record serialization.

This package provides organized access to all supported MARC formats.
Each format module provides consistent read/write functions.

Format Tiers
------------
- **Tier 1 (Core)**: Always available
  - marc: ISO 2709 binary MARC (standard interchange format)
  - protobuf: Protocol Buffers (APIs, cross-language IPC)

- **Tier 2 (High-Value)**: Available in Python, feature-gated in Rust
  - arrow: Apache Arrow IPC (analytics, DuckDB/Polars)
  - flatbuffers: FlatBuffers (zero-copy, memory-efficient)
  - messagepack: MessagePack (compact binary, 50+ languages)

Quick Start
-----------
Each format module provides `read()` and `write()` functions:

>>> from mrrc.formats import protobuf
>>> # Read records
>>> for record in protobuf.read("records.pb"):
...     print(record.title())
>>> # Write records
>>> count = protobuf.write(records, "output.pb")

Format Conversion
-----------------
Convert between formats using read/write:

>>> from mrrc.formats import marc, arrow
>>> records = list(marc.read("input.mrc"))
>>> arrow.write(records, "output.arrow")

For single-record serialization, use serialize/deserialize:

>>> from mrrc.formats import protobuf
>>> data = protobuf.serialize(record)
>>> restored = protobuf.deserialize(data)

See Also
--------
- FORMAT_SELECTION_GUIDE.md for choosing formats
- STREAMING_GUIDE.md for large file handling
"""

from . import marc
from . import protobuf
from . import arrow
from . import flatbuffers
from . import messagepack

__all__ = [
    "marc",
    "protobuf",
    "arrow",
    "flatbuffers",
    "messagepack",
]
