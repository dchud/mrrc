"""FlatBuffers format support (Tier 2).

FlatBuffers provides zero-copy deserialization for memory-efficient
streaming of large record sets.
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
