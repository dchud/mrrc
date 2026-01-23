"""MessagePack format support (Tier 2).

MessagePack provides compact binary serialization with broad
language support (50+ programming languages).

MessagePack is like JSON but faster and smaller. It's self-describing
and doesn't require a schema, making it ideal for REST APIs and IPC.

Use Cases
---------
- REST API responses (smaller than JSON)
- Inter-process communication
- Language-agnostic data exchange
- Systems where JSON is too verbose

Performance
-----------
- Read: ~750,000 records/second
- Write: ~700,000 records/second
- Size: ~25% smaller than equivalent JSON

Language Support
----------------
MessagePack has official libraries for: Python, Ruby, Java, Go, C, C++,
C#, JavaScript, PHP, Perl, Rust, Scala, Swift, and many more.

Examples
--------
Read records from a MessagePack file:

>>> from mrrc.formats import messagepack
>>> for record in messagepack.read("records.msgpack"):
...     print(record.title())

Write records to a MessagePack file:

>>> from mrrc.formats import messagepack
>>> count = messagepack.write(records, "output.msgpack")

Serialize a single record:

>>> from mrrc.formats import messagepack
>>> data = messagepack.serialize(record)  # bytes
>>> restored = messagepack.deserialize(data)

Use in a REST API:

>>> from flask import Flask, Response
>>> from mrrc.formats import messagepack
>>>
>>> @app.route('/record/<id>')
>>> def get_record(id):
...     record = fetch_record(id)
...     return Response(
...         messagepack.serialize(record),
...         mimetype='application/msgpack'
...     )

See Also
--------
- MessagePackReader: Streaming reader class
- MessagePackWriter: Streaming writer class
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
