#![no_main]

use libfuzzer_sys::fuzz_target;
use mrrc::bibframe::{RdfFormat, RdfGraph, bibframe_to_marc};

// Drive the BIBFRAME read path over arbitrary bytes: RDF parsing via
// oxrdfio (the largest external-parser dependency surface in the read
// stack) followed by the BIBFRAME-to-MARC reverse conversion. The
// first input byte selects the RDF concrete syntax; the rest is fed to
// the parser as-is, so the syntax decoders see raw (possibly
// non-UTF-8) bytes. When the bytes parse as a graph, the harness also
// runs the reverse converter, whose graph traversal is mrrc's own
// code. An `Err` from either stage on malformed input is correct
// behavior, so the Results are discarded — libfuzzer only flags
// panics, OOMs, and timeouts.
fuzz_target!(|data: &[u8]| {
    let Some((selector, rdf_bytes)) = data.split_first() else {
        return;
    };
    let format = match selector % 4 {
        0 => RdfFormat::RdfXml,
        1 => RdfFormat::JsonLd,
        2 => RdfFormat::Turtle,
        _ => RdfFormat::NTriples,
    };
    if let Ok(graph) = RdfGraph::parse_from_reader(rdf_bytes, format) {
        let _ = bibframe_to_marc(&graph);
    }
});
