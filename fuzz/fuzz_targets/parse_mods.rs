#![no_main]

use libfuzzer_sys::fuzz_target;
use mrrc::mods::mods_xml_to_record;

// Drive the MODS XML reader over arbitrary bytes. MODS is a third XML
// read surface alongside MARCXML (parse_marcxml), with its own bespoke
// structural mapping on top of the shared XML parser. The reader takes a
// `&str`, so the harness first checks the bytes are valid UTF-8 and only
// reaches the XML parse and the MODS mapping when they are. An `Err` on
// malformed XML — XML-spec parse errors as well as MODS-spec structural
// errors — is correct behavior, so the Result is discarded. libfuzzer
// only flags panics, OOMs, and timeouts.
fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = mods_xml_to_record(text);
    }
});
