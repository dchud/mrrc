"""FlatBuffers format support (Tier 2).

FlatBuffers provides zero-copy deserialization for memory-efficient
streaming of large record sets.

Unlike other formats, FlatBuffers data can be accessed directly without
parsing or unpacking. This makes it ideal for memory-constrained environments
and real-time systems.

Use Cases
---------
- Mobile applications with limited memory
- Embedded systems
- Real-time MARC data processing
- Streaming APIs where memory is critical
- Memory-mapped file access

Performance
-----------
- Read: ~1,000,000+ records/second (zero-copy)
- Write: ~700,000 records/second
- Memory: 64% savings vs standard serialization

Zero-Copy Access
----------------
FlatBuffers data is accessed directly from the buffer without creating
intermediate objects. Fields are read lazily, so unused data is never
touched.

Examples
--------
Read records from a FlatBuffers file:

>>> from mrrc.formats import flatbuffers
>>> for record in flatbuffers.read("records.fb"):
...     print(record.title())

Write records to a FlatBuffers file:

>>> from mrrc.formats import flatbuffers
>>> count = flatbuffers.write(records, "output.fb")

Serialize a single record:

>>> from mrrc.formats import flatbuffers
>>> data = flatbuffers.serialize(record)  # bytes
>>> restored = flatbuffers.deserialize(data)

Memory-mapped reading (advanced):

>>> import mmap
>>> with open("records.fb", "rb") as f:
...     mm = mmap.mmap(f.fileno(), 0, access=mmap.ACCESS_READ)
...     # Data can be accessed without loading entire file
...     for record in flatbuffers.read(mm):
...         if needs_processing(record):
...             process(record)

See Also
--------
- FlatbuffersReader: Streaming reader class
- FlatbuffersWriter: Streaming writer class
"""

from mrrc import (
    FlatbuffersReader,
    FlatbuffersWriter,
    record_to_flatbuffers,
    flatbuffers_to_record,
)

__all__ = [
    "FlatbuffersReader",
    "FlatbuffersWriter",
    "read",
    "write",
    "serialize",
    "deserialize",
]


def read(source):
    """Read MARC records from a FlatBuffers file or bytes.

    Args:
        source: File path (str) or bytes containing FlatBuffers data.

    Returns:
        Iterator over Record objects.
    """
    return FlatbuffersReader(source)


def write(records, dest):
    """Write MARC records to a FlatBuffers file.

    Args:
        records: Iterable of Record objects.
        dest: File path (str).

    Returns:
        Number of records written.
    """
    writer = FlatbuffersWriter(dest)
    count = 0
    for record in records:
        inner = record._inner if hasattr(record, "_inner") else record
        writer.write_record(inner)
        count += 1
    writer.close()
    return count


def serialize(record):
    """Serialize a single record to FlatBuffers bytes.

    Args:
        record: A Record object.

    Returns:
        bytes containing the FlatBuffers-encoded record.
    """
    inner = record._inner if hasattr(record, "_inner") else record
    return record_to_flatbuffers(inner)


def deserialize(data):
    """Deserialize a single record from FlatBuffers bytes.

    Args:
        data: bytes containing a FlatBuffers-encoded record.

    Returns:
        A Record object.
    """
    return flatbuffers_to_record(data)
