"""Protocol Buffers format support (Tier 1).

Protocol Buffers provides efficient binary serialization with strong
schema support and excellent forward/backward compatibility. This is a
Tier 1 format, always available.

Use Cases
---------
- Building REST APIs or microservices
- Cross-language communication (Java, Go, C++, etc.)
- Systems requiring schema evolution
- Smaller payloads than JSON

Performance
-----------
- Read: ~750,000 records/second
- Write: ~700,000 records/second
- Size: ~20% smaller than ISO 2709

Schema Evolution
----------------
The Protobuf schema supports adding optional fields without breaking
existing readers. Old clients can read new data, and new clients
can read old data.

Examples
--------
Read records from a Protobuf file:

>>> from mrrc.formats import protobuf
>>> for record in protobuf.read("records.pb"):
...     print(record.title())

Write records to a Protobuf file:

>>> from mrrc.formats import protobuf
>>> count = protobuf.write(records, "output.pb")

Serialize a single record:

>>> from mrrc.formats import protobuf
>>> data = protobuf.serialize(record)  # bytes
>>> restored = protobuf.deserialize(data)

See Also
--------
- ProtobufReader: Streaming reader class
- ProtobufWriter: Streaming writer class
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
