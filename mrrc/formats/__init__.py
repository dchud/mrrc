"""Format-specific modules for MARC record serialization.

This package provides organized access to all supported MARC formats:
- marc: ISO 2709 binary MARC (baseline format)
- protobuf: Protocol Buffers (Tier 1)
- arrow: Apache Arrow IPC (Tier 2)
- flatbuffers: FlatBuffers (Tier 2)
- messagepack: MessagePack (Tier 2)
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
