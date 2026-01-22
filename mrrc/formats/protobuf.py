"""Protocol Buffers format support (Tier 1).

Protocol Buffers provides efficient binary serialization with strong
schema support. This is a Tier 1 format, always available.
"""

from mrrc import (
    ProtobufReader,
    ProtobufWriter,
    record_to_protobuf,
    protobuf_to_record,
)

__all__ = [
    "ProtobufReader",
    "ProtobufWriter",
    "read",
    "write",
    "serialize",
    "deserialize",
]


def read(source):
    """Read MARC records from a Protobuf file or bytes.

    Args:
        source: File path (str) or bytes containing Protobuf data.

    Returns:
        Iterator over Record objects.
    """
    return ProtobufReader(source)


def write(records, dest):
    """Write MARC records to a Protobuf file.

    Args:
        records: Iterable of Record objects.
        dest: File path (str).

    Returns:
        Number of records written.
    """
    writer = ProtobufWriter(dest)
    count = 0
    for record in records:
        inner = record._inner if hasattr(record, "_inner") else record
        writer.write_record(inner)
        count += 1
    writer.close()
    return count


def serialize(record):
    """Serialize a single record to Protobuf bytes.

    Args:
        record: A Record object.

    Returns:
        bytes containing the Protobuf-encoded record.
    """
    inner = record._inner if hasattr(record, "_inner") else record
    return record_to_protobuf(inner)


def deserialize(data):
    """Deserialize a single record from Protobuf bytes.

    Args:
        data: bytes containing a Protobuf-encoded record.

    Returns:
        A Record object.
    """
    return protobuf_to_record(data)
