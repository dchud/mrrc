#!/usr/bin/env python3
"""BIBFRAME configuration options example.

This example demonstrates:
- Using BibframeConfig to customize conversion behavior
- Setting custom base URIs
- Controlling output formats
- Authority linking options
"""

import mrrc


def main():
    # Create a sample MARC record
    leader = mrrc.Leader()
    leader.record_type = "a"  # language material
    leader.bibliographic_level = "m"  # monograph
    record = mrrc.Record(leader=leader)

    # Add control fields
    record.add_control_field("001", "config-example")
    record.add_control_field("008", "040520s2023    xxu           000 0 eng  ")

    # Add title
    field_245 = mrrc.Field(tag="245", indicator1="1", indicator2="0")
    field_245.add_subfield("a", "Configuration Example /")
    field_245.add_subfield("c", "by Demo User.")
    record.add_field(field_245)

    # Add author with authority link
    field_100 = mrrc.Field(tag="100", indicator1="1", indicator2=" ")
    field_100.add_subfield("a", "User, Demo,")
    field_100.add_subfield("0", "(OCoLC)12345678")
    record.add_field(field_100)

    print("=== BIBFRAME Configuration Options ===\n")

    # Configuration 1: Default settings
    print("Configuration 1: Default Settings")
    config1 = mrrc.BibframeConfig()
    graph1 = mrrc.marc_to_bibframe(record, config1)
    print(f"  Triples: {len(graph1)}")
    rdf1 = graph1.serialize("ntriples")
    print(f"  First triple: {rdf1.split(chr(10))[0][:80]}...\n")

    # Configuration 2: Custom base URI
    print("Configuration 2: Custom Base URI")
    config2 = mrrc.BibframeConfig()
    config2.set_base_uri("http://mylib.org/catalog/")
    graph2 = mrrc.marc_to_bibframe(record, config2)
    print(f"  Triples: {len(graph2)}")
    rdf2 = graph2.serialize("ntriples")
    print(f"  First triple: {rdf2.split(chr(10))[0][:80]}...\n")

    # Configuration 3: Different output formats
    print("Configuration 3: Output Format Options")
    config3 = mrrc.BibframeConfig()
    graph3 = mrrc.marc_to_bibframe(record, config3)

    formats = [
        ("rdf-xml", "RDF/XML (W3C standard)"),
        ("ntriples", "N-Triples (simple, line-based)"),
        ("turtle", "Turtle (readable, prefixed)"),
        ("jsonld", "JSON-LD (JSON representation)"),
    ]

    for fmt, description in formats:
        try:
            output = graph3.serialize(fmt)
            print(
                f"  {fmt:12} ({description}): {len(output):6} bytes"
            )
        except Exception as e:
            print(f"  {fmt:12}: Error - {e}")

    print("\n=== Best Practices ===")
    print("\n1. Base URI Configuration")
    print("   - Use organization-specific base URI for consistency")
    print("   - Example: http://library.example.org/bibframe/")
    print("   - Ensures URIs are stable and resolvable")

    print("\n2. Output Format Selection")
    print("   - RDF/XML: Compatibility with RDF tools")
    print("   - Turtle: Human-readable development/debugging")
    print("   - JSON-LD: Modern web applications")
    print("   - N-Triples: Triple stores and databases")

    print("\n3. Authority Linking")
    print("   - Enable for systems managing authority records")
    print("   - Preserves links to LCSH, LCNAF, VIAF")
    print("   - Important for linked data applications")

    print("\n4. BFLC Extensions")
    print("   - Enable for Library of Congress compatibility")
    print("   - Includes additional descriptive properties")
    print("   - Useful for detailed cataloging workflows")

    print("\n✓ Configuration example complete!")


if __name__ == "__main__":
    main()
