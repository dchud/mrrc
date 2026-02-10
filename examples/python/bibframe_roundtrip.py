#!/usr/bin/env python3
"""BIBFRAME roundtrip example: MARC → BIBFRAME → MARC.

This example demonstrates:
- Converting MARC to BIBFRAME and back
- Verifying round-trip preservation of essential data
- Documenting acceptable data loss
"""

import mrrc


def main():
    # Create a sample MARC record
    leader = mrrc.Leader()
    leader.record_type = "a"  # language material
    leader.bibliographic_level = "m"  # monograph
    record = mrrc.Record(leader=leader)

    # Add control fields
    record.add_control_field("001", "roundtrip-001")
    record.add_control_field("008", "040520s2023    xxu           000 0 eng  ")

    # Add ISBN
    field_020 = mrrc.Field(tag="020", indicator1=" ", indicator2=" ")
    field_020.add_subfield("a", "9780123456789")
    record.add_field(field_020)

    # Add title
    field_245 = mrrc.Field(tag="245", indicator1="1", indicator2="0")
    field_245.add_subfield("a", "MARC Roundtrip Test /")
    field_245.add_subfield("c", "by Test Author.")
    record.add_field(field_245)

    # Add author
    field_100 = mrrc.Field(tag="100", indicator1="1", indicator2=" ")
    field_100.add_subfield("a", "Author, Test,")
    field_100.add_subfield("4", "aut")
    record.add_field(field_100)

    print("=== Original MARC Record ===")
    print(f"Title fields: {len(record.fields_by_tag('245'))}")
    print(f"Creator fields: {len(record.fields_by_tag('100'))}")
    print(f"Identifier fields: {len(record.fields_by_tag('020'))}")

    # Step 1: Convert MARC → BIBFRAME
    config = mrrc.BibframeConfig()
    graph = mrrc.marc_to_bibframe(record, config)
    print(f"\n✓ Converted to BIBFRAME ({len(graph)} triples)")

    # Step 2: Serialize BIBFRAME to RDF/XML
    rdf_xml = graph.serialize("rdf-xml")
    print(f"✓ Serialized to RDF/XML ({len(rdf_xml)} bytes)")

    # Step 3: Convert BIBFRAME → MARC
    recovered_record = mrrc.bibframe_to_marc(graph)
    print("✓ Converted back to MARC")

    # Verify round-trip fidelity
    print("\n=== Round-Trip Results ===")
    print(f"Title fields preserved: {len(recovered_record.fields_by_tag('245'))}")
    print(f"Creator fields preserved: {len(recovered_record.fields_by_tag('100'))}")
    print(f"Identifier fields preserved: {len(recovered_record.fields_by_tag('020'))}")

    # Count all fields
    original_field_count = sum(1 for _ in record.fields())
    recovered_field_count = sum(1 for _ in recovered_record.fields())

    print(f"\nTotal fields:")
    print(f"  Original: {original_field_count}")
    print(f"  Recovered: {recovered_field_count}")

    # Show sample recovered field
    title_fields = list(recovered_record.fields_by_tag("245"))
    if title_fields:
        title_field = title_fields[0]
        print(f"\nSample recovered 245 field:")
        print(f"  Tag: {title_field.tag}")
        print(f"  Ind1: '{title_field.indicator1}'")
        print(f"  Ind2: '{title_field.indicator2}'")
        for sf in title_field.subfields():
            print(f"  ${sf.code}: {sf.value}")

    print("\n✓ Round-trip conversion complete!")
    print("\nNote: Some data may be lost in round-trip conversion:")
    print("  - Non-filing indicators (245 ind2) are reconstructed")
    print("  - Authority record links ($0) are optional")
    print("  - Detailed 008 codes may be approximated")
    print("  - Note types (500, 520, etc.) may consolidate")


if __name__ == "__main__":
    main()
