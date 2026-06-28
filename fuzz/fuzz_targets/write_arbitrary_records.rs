//! Writer fuzz harness: arbitrary records through the three ISO 2709 writers.
//!
//! The reader-gated targets (`roundtrip_binary`) can only feed the writer
//! records the reader first produced, so they never reach writer behavior that
//! only manifests on API-constructed records — the class of bug bd-24gf was
//! (a field whose serialized length overflows the 4-digit directory entry).
//! This target builds arbitrary `Record` / `AuthorityRecord` / `HoldingsRecord`
//! values directly from fuzz input and runs each through its writer, asserting:
//!
//! - the writer never panics, and
//! - whatever bytes it emits parse back through the matching reader without an
//!   error (i.e. the writer never produces framing the reader chokes on).
//!
//! A `WriterError` (e.g. an over-9999-byte field) is a clean rejection and is
//! accepted. Each record type gets a leader with the record type its format
//! reader requires (`z` for authority, `{u,v,x,y}` for holdings), mirroring the
//! proptest generators; otherwise the format reader rejects the readback as a
//! category mismatch. The arbitrary generation lives here in the fuzz
//! workspace, so it adds no dependency or public surface to the `mrrc` crate.
//!
//! Write-side analogue of the read-side `parse_record_from_bytes` target. See
//! `docs/contributing/fuzzing.md` for triage.

#![no_main]

use arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use mrrc::{
    AuthorityMarcReader, AuthorityMarcWriter, AuthorityRecord, Field, HoldingsMarcReader,
    HoldingsMarcWriter, HoldingsRecord, Leader, MarcReader, MarcWriter, Record,
};
use std::io::Cursor;

struct FieldSpec {
    tag: String,
    ind1: char,
    ind2: char,
    subfields: Vec<(char, String)>,
}

/// Map fuzz bytes to a delimiter-free printable-ASCII string of
/// fuzzer-controlled length. Avoiding the MARC delimiters (0x1D/1E/1F) keeps
/// the focus on length/framing behavior rather than delimiter injection (the
/// proptest generators exclude them too); the length is fuzzer-driven, so the
/// directory-length boundary is still reachable on large inputs.
fn arb_value(u: &mut Unstructured) -> arbitrary::Result<String> {
    let raw: Vec<u8> = u.arbitrary()?;
    Ok(raw.iter().map(|b| (33 + (b % 94)) as char).collect())
}

fn arb_indicator(u: &mut Unstructured) -> arbitrary::Result<char> {
    Ok(*u.choose(&[' ', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'])?)
}

fn arb_subfield_code(u: &mut Unstructured) -> arbitrary::Result<char> {
    let n = u.int_in_range(0u8..=35)?;
    Ok(if n < 26 {
        (b'a' + n) as char
    } else {
        (b'0' + (n - 26)) as char
    })
}

/// The control and data fields shared across the three record types.
fn arb_fields(u: &mut Unstructured) -> arbitrary::Result<(Vec<(String, String)>, Vec<FieldSpec>)> {
    let mut control = Vec::new();
    for _ in 0..u.int_in_range(0u32..=3)? {
        control.push((format!("00{}", u.int_in_range(1u8..=9)?), arb_value(u)?));
    }
    let mut data = Vec::new();
    for _ in 0..u.int_in_range(0u32..=8)? {
        let tag = format!("{:03}", u.int_in_range(10u16..=999)?);
        let ind1 = arb_indicator(u)?;
        let ind2 = arb_indicator(u)?;
        let mut subfields = Vec::new();
        for _ in 0..u.int_in_range(1u32..=6)? {
            subfields.push((arb_subfield_code(u)?, arb_value(u)?));
        }
        data.push(FieldSpec {
            tag,
            ind1,
            ind2,
            subfields,
        });
    }
    Ok((control, data))
}

fn build_fields(data: &[FieldSpec]) -> Vec<Field> {
    data.iter()
        .map(|f| {
            let mut field = Field::new(f.tag.clone(), f.ind1, f.ind2);
            for (code, value) in &f.subfields {
                field.add_subfield(*code, value.clone());
            }
            field
        })
        .collect()
}

// Per-type leaders, mirroring the proptest generators so the format readers
// accept the readback at the default (Structural) validation level.

fn bib_leader(u: &mut Unstructured) -> arbitrary::Result<Leader> {
    Ok(Leader {
        record_length: 0,
        record_status: *u.choose(&['a', 'c', 'd', 'n', 'p'])?,
        record_type: *u.choose(&['a', 'c', 'd', 'e', 'f', 'g', 'i', 'j', 'k', 'm', 'o', 'p', 'r', 't'])?,
        bibliographic_level: *u.choose(&['a', 'c', 'd', 'i', 'm', 's'])?,
        control_record_type: *u.choose(&[' ', 'a'])?,
        character_coding: *u.choose(&[' ', 'a'])?,
        indicator_count: 2,
        subfield_code_count: 2,
        data_base_address: 0,
        encoding_level: *u.choose(&[' ', '1', '3', '7'])?,
        cataloging_form: *u.choose(&[' ', 'a', 'c', 'i'])?,
        multipart_level: *u.choose(&[' ', 'a', 'b', 'c'])?,
        reserved: "4500".to_string(),
    })
}

fn authority_leader(u: &mut Unstructured) -> arbitrary::Result<Leader> {
    Ok(Leader {
        record_length: 0,
        record_status: *u.choose(&['a', 'c', 'd', 'n', 'o', 's', 'x'])?,
        record_type: 'z',
        bibliographic_level: *u.choose(&[' ', '|'])?,
        control_record_type: *u.choose(&[' ', '|'])?,
        character_coding: *u.choose(&[' ', 'a'])?,
        indicator_count: 2,
        subfield_code_count: 2,
        data_base_address: 0,
        encoding_level: *u.choose(&['n', 'o'])?,
        cataloging_form: ' ',
        multipart_level: ' ',
        reserved: "4500".to_string(),
    })
}

fn holdings_leader(u: &mut Unstructured) -> arbitrary::Result<Leader> {
    Ok(Leader {
        record_length: 0,
        record_status: *u.choose(&['c', 'd', 'n'])?,
        record_type: *u.choose(&['x', 'y', 'v', 'u'])?,
        bibliographic_level: *u.choose(&[' ', '|'])?,
        control_record_type: *u.choose(&[' ', '|'])?,
        character_coding: *u.choose(&[' ', 'a'])?,
        indicator_count: 2,
        subfield_code_count: 2,
        data_base_address: 0,
        encoding_level: *u.choose(&['1', '2', '3', '4', '5', 'm', 'u', 'z'])?,
        cataloging_form: ' ',
        multipart_level: ' ',
        reserved: "4500".to_string(),
    })
}

fn add_fields<R>(record: &mut R, control: &[(String, String)], data: &[FieldSpec])
where
    R: RecordSink,
{
    for (tag, value) in control {
        record.sink_control_field(tag.clone(), value.clone());
    }
    for field in build_fields(data) {
        record.sink_field(field);
    }
}

/// Minimal sink trait so the three record types share the field-population code
/// without one generic write path (their writers/readers are distinct types).
trait RecordSink {
    fn sink_control_field(&mut self, tag: String, value: String);
    fn sink_field(&mut self, field: Field);
}

macro_rules! impl_record_sink {
    ($Rec:ty) => {
        impl RecordSink for $Rec {
            fn sink_control_field(&mut self, tag: String, value: String) {
                self.add_control_field(tag, value);
            }
            fn sink_field(&mut self, field: Field) {
                self.add_field(field);
            }
        }
    };
}

impl_record_sink!(Record);
impl_record_sink!(AuthorityRecord);
impl_record_sink!(HoldingsRecord);

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let Ok((control, data)) = arb_fields(&mut u) else {
        return;
    };

    if let Ok(leader) = bib_leader(&mut u) {
        let mut record = Record::new(leader);
        add_fields(&mut record, &control, &data);
        let mut buf = Vec::new();
        if MarcWriter::new(&mut buf).write_record(&record).is_ok() {
            let mut reader = MarcReader::new(Cursor::new(&buf));
            reader
                .read_record()
                .expect("bibliographic writer output must be readable");
        }
    }

    if let Ok(leader) = authority_leader(&mut u) {
        let mut record = AuthorityRecord::new(leader);
        add_fields(&mut record, &control, &data);
        let mut buf = Vec::new();
        if AuthorityMarcWriter::new(&mut buf).write_record(&record).is_ok() {
            let mut reader = AuthorityMarcReader::new(Cursor::new(&buf));
            reader
                .read_record()
                .expect("authority writer output must be readable");
        }
    }

    if let Ok(leader) = holdings_leader(&mut u) {
        let mut record = HoldingsRecord::new(leader);
        add_fields(&mut record, &control, &data);
        let mut buf = Vec::new();
        if HoldingsMarcWriter::new(&mut buf).write_record(&record).is_ok() {
            let mut reader = HoldingsMarcReader::new(Cursor::new(&buf));
            reader
                .read_record()
                .expect("holdings writer output must be readable");
        }
    }
});
