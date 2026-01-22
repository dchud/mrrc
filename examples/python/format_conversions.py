#!/usr/bin/env python3
"""Example: Converting between MARC formats.

This example demonstrates how to convert MARC records between different
formats using mrrc.formats modules and the format-agnostic helpers.

Supported formats:
- ISO 2709 MARC (.mrc)
- Protocol Buffers (.pb)
- Apache Arrow IPC (.arrow)
- FlatBuffers (.fb)
- MessagePack (.msgpack)
- Parquet (.parquet) - export only
"""

import os
import tempfile
import mrrc
import mrrc.formats.marc as marc
import mrrc.formats.protobuf as protobuf
import mrrc.formats.arrow as arrow
import mrrc.formats.flatbuffers as flatbuffers
import mrrc.formats.messagepack as messagepack


def main():
    # Load sample records
    print("Loading MARC records...")
    records = list(mrrc.read("tests/data/fixtures/1k_records.mrc"))
    print(f"Loaded {len(records)} records\n")

    with tempfile.TemporaryDirectory() as tmpdir:
        # Example 1: Using format-agnostic read/write
        print("=" * 60)
        print("Example 1: Format-agnostic read/write")
        print("=" * 60)

        formats = [
            ("marc", ".mrc"),
            ("protobuf", ".pb"),
            ("arrow", ".arrow"),
            ("flatbuffers", ".fb"),
            ("messagepack", ".msgpack"),
        ]

        for fmt, ext in formats:
            path = os.path.join(tmpdir, f"output{ext}")
            count = mrrc.write(records, path)
            size = os.path.getsize(path)
            read_back = list(mrrc.read(path))
            print(f"  {fmt:12} -> {size:>10,} bytes, {count} written, {len(read_back)} read")
        print()

        # Example 2: Using format-specific modules
        print("=" * 60)
        print("Example 2: Format-specific module functions")
        print("=" * 60)

        # Protobuf
        pb_path = os.path.join(tmpdir, "records.pb")
        protobuf.write(records, pb_path)
        pb_records = list(protobuf.read(pb_path))
        print(f"  Protobuf: wrote {len(records)}, read {len(pb_records)}")

        # FlatBuffers
        fb_path = os.path.join(tmpdir, "records.fb")
        flatbuffers.write(records, fb_path)
        fb_records = list(flatbuffers.read(fb_path))
        print(f"  FlatBuffers: wrote {len(records)}, read {len(fb_records)}")

        # MessagePack
        mp_path = os.path.join(tmpdir, "records.msgpack")
        messagepack.write(records, mp_path)
        mp_records = list(messagepack.read(mp_path))
        print(f"  MessagePack: wrote {len(records)}, read {len(mp_records)}")

        # Arrow
        arrow_path = os.path.join(tmpdir, "records.arrow")
        arrow.write(records, arrow_path)
        arrow_records = list(arrow.read(arrow_path))
        print(f"  Arrow: wrote {len(records)}, read {len(arrow_records)}")
        print()

        # Example 3: Single record serialization
        print("=" * 60)
        print("Example 3: Single record serialization")
        print("=" * 60)

        record = records[0]
        title = record.title() or "Unknown"
        print(f"  Original record title: {title[:50]}...")

        # Protobuf
        pb_bytes = protobuf.serialize(record)
        pb_restored = protobuf.deserialize(pb_bytes)
        print(f"  Protobuf: {len(pb_bytes)} bytes")

        # FlatBuffers
        fb_bytes = flatbuffers.serialize(record)
        fb_restored = flatbuffers.deserialize(fb_bytes)
        print(f"  FlatBuffers: {len(fb_bytes)} bytes")

        # MessagePack
        mp_bytes = messagepack.serialize(record)
        mp_restored = messagepack.deserialize(mp_bytes)
        print(f"  MessagePack: {len(mp_bytes)} bytes")
        print()

        # Example 4: Arrow to Parquet export
        print("=" * 60)
        print("Example 4: Export to Parquet")
        print("=" * 60)

        arrow_path = os.path.join(tmpdir, "records.arrow")
        parquet_path = os.path.join(tmpdir, "records.parquet")

        arrow.write(records, arrow_path)
        row_count = arrow.export_to_parquet(arrow_path, parquet_path)

        arrow_size = os.path.getsize(arrow_path)
        parquet_size = os.path.getsize(parquet_path)
        print(f"  Arrow IPC: {arrow_size:,} bytes")
        print(f"  Parquet: {parquet_size:,} bytes ({row_count} rows)")
        print(f"  Compression ratio: {parquet_size/arrow_size:.1%}")
        print()

        # Example 5: Format conversion pipeline
        print("=" * 60)
        print("Example 5: Conversion pipeline")
        print("=" * 60)
        print("  MARC -> Protobuf -> FlatBuffers -> MessagePack -> MARC")

        # Step 1: MARC -> Protobuf
        pb_path = os.path.join(tmpdir, "step1.pb")
        protobuf.write(records, pb_path)

        # Step 2: Protobuf -> FlatBuffers
        pb_records = list(protobuf.read(pb_path))
        fb_path = os.path.join(tmpdir, "step2.fb")
        flatbuffers.write(pb_records, fb_path)

        # Step 3: FlatBuffers -> MessagePack
        fb_records = list(flatbuffers.read(fb_path))
        mp_path = os.path.join(tmpdir, "step3.msgpack")
        messagepack.write(fb_records, mp_path)

        # Step 4: MessagePack -> MARC
        mp_records = list(messagepack.read(mp_path))
        final_path = os.path.join(tmpdir, "final.mrc")
        marc.write(mp_records, final_path)

        # Verify
        final_records = list(marc.read(final_path))
        print(f"  Original: {len(records)} records")
        print(f"  Final: {len(final_records)} records")
        print(f"  Pipeline successful: {len(records) == len(final_records)}")


if __name__ == "__main__":
    main()
