"""MessagePack format support (Tier 2).

MessagePack provides compact binary serialization with broad
language support (50+ languages).
"""

from mrrc import (
    MessagePackReader,
    MessagePackWriter,
    record_to_messagepack,
    messagepack_to_record,
)

__all__ = [
    "MessagePackReader",
    "MessagePackWriter",
    "read",
    "write",
    "serialize",
    "deserialize",
]


def read(source):
    """Read MARC records from a MessagePack file or bytes.

    Args:
        source: File path (str) or bytes containing MessagePack data.

    Returns:
        Iterator over Record objects.
    """
    return MessagePackReader(source)


def write(records, dest):
    """Write MARC records to a MessagePack file.

    Args:
        records: Iterable of Record objects.
        dest: File path (str).

    Returns:
        Number of records written.
    """
    writer = MessagePackWriter(dest)
    count = 0
    for record in records:
        inner = record._inner if hasattr(record, "_inner") else record
        writer.write_record(inner)
        count += 1
    writer.close()
    return count


def serialize(record):
    """Serialize a single record to MessagePack bytes.

    Args:
        record: A Record object.

    Returns:
        bytes containing the MessagePack-encoded record.
    """
    inner = record._inner if hasattr(record, "_inner") else record
    return record_to_messagepack(inner)


def deserialize(data):
    """Deserialize a single record from MessagePack bytes.

    Args:
        data: bytes containing a MessagePack-encoded record.

    Returns:
        A Record object.
    """
    return messagepack_to_record(data)
