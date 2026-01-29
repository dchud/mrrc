#!/usr/bin/env python3
"""BIBFRAME conversion example: Basic MARC → BIBFRAME conversion.

This example demonstrates:
- Converting a MARC record to BIBFRAME
- Accessing the RDF graph
- Serializing to different RDF formats
"""

import mrrc


def main():
    # Create a sample MARC record using the Python API
    leader = mrrc.Leader()
    leader.record_type = "a"  # language material
    leader.bibliographic_level = "m"  # monograph
    record = mrrc.Record(leader=leader)

    # Add control fields
    record.add_control_field("001", "example-001")
    record.add_control_field("008", "040520s2023    xxu           000 0 eng  ")

    # Add ISBN
    field_020 = mrrc.Field(tag="020", indicator1=" ", indicator2=" ")
    field_020.add_subfield("a", "9780123456789")
    record.add_field(field_020)

    # Add title
    field_245 = mrrc.Field(tag="245", indicator1="1", indicator2="0")
    field_245.add_subfield("a", "Introduction to MARC /")
    field_245.add_subfield("c", "by Jane Smith.")
    record.add_field(field_245)

    # Add author
    field_100 = mrrc.Field(tag="100", indicator1="1", indicator2=" ")
    field_100.add_subfield("a", "Smith, Jane,")
    field_100.add_subfield("d", "1970-")
    field_100.add_subfield("4", "aut")
    record.add_field(field_100)

    # Add publication info
    field_260 = mrrc.Field(tag="260", indicator1=" ", indicator2=" ")
    field_260.add_subfield("a", "New York :")
    field_260.add_subfield("b", "Academic Press,")
    field_260.add_subfield("c", "2023.")
    record.add_field(field_260)

    # Add subject
    field_650 = mrrc.Field(tag="650", indicator1=" ", indicator2="0")
    field_650.add_subfield("a", "MARC (Computer record format)")
    field_650.add_subfield("x", "Cataloging.")
    record.add_field(field_650)

    print("✓ Created MARC record with 6 fields\n")

    # Convert to BIBFRAME
    config = mrrc.BibframeConfig()
    graph = mrrc.marc_to_bibframe(record, config)

    print(f"✓ Converted MARC record to BIBFRAME graph")
    print(f"  Graph contains {len(graph)} triples\n")

    # Serialize to different formats
    print("=== RDF/XML Format ===")
    rdf_xml = graph.serialize("rdf-xml")
    print(rdf_xml[:500])
    print()

    print("=== N-Triples Format (first 3 triples) ===")
    ntriples = graph.serialize("ntriples")
    for i, line in enumerate(ntriples.split("\n")):
        if i >= 3:
            break
        if line:
            print(line)

    print("\n=== JSON-LD Format ===")
    jsonld = graph.serialize("jsonld")
    print(jsonld[:300])
    print()

    print("✓ BIBFRAME conversion complete!")


if __name__ == "__main__":
    main()
