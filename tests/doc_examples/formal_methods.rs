//! Code for `docs/contributing/formal-methods.md`.

mod unit_test {
    // --8<-- [start:unit-test]
    use mrrc::MarcReader;
    use std::io::Cursor;

    #[test]
    fn parses_simple_record() {
        let bytes = std::fs::read("tests/data/simple_book.mrc").unwrap();
        let record = MarcReader::new(Cursor::new(&bytes[..]))
            .read_record()
            .unwrap()
            .unwrap();
        assert_eq!(
            record.get_field("100").unwrap().get_subfield('a'),
            Some("Fitzgerald, F. Scott")
        );
    }
    // --8<-- [end:unit-test]
}

mod property_test {
    use mrrc::{Field, Leader, MarcWriter, Record};

    /// Records with one to five data fields, each with one to three
    /// subfields of printable ASCII.
    fn arb_record() -> impl Strategy<Value = Record> {
        let subfield = ("[a-z]", "[A-Za-z0-9 .,]{1,20}");
        let field = (
            "[1-9][0-9][0-9]",
            "[ 0-9]",
            "[ 0-9]",
            proptest::collection::vec(subfield, 1..=3),
        )
            .prop_map(|(tag, ind1, ind2, subfields)| {
                let mut field = Field::new(
                    tag,
                    ind1.chars().next().unwrap(),
                    ind2.chars().next().unwrap(),
                );
                for (code, value) in subfields {
                    field.add_subfield(code.chars().next().unwrap(), value);
                }
                field
            });
        proptest::collection::vec(field, 1..=5).prop_map(|fields| {
            let mut builder = Record::builder(Leader::default());
            for field in fields {
                builder = builder.field(field);
            }
            builder.build()
        })
    }

    /// The record as ISO 2709 bytes.
    fn serialize(record: &Record) -> Vec<u8> {
        let mut bytes = Vec::new();
        MarcWriter::new(&mut bytes).write_record(record).unwrap();
        bytes
    }

    // --8<-- [start:property-test]
    use mrrc::MarcReader;
    use proptest::prelude::*;
    use std::io::Cursor;

    proptest! {
        #[test]
        fn binary_roundtrip(record in arb_record()) {
            let bytes = serialize(&record);
            let parsed = MarcReader::new(Cursor::new(&bytes[..]))
                .read_record().unwrap().unwrap();
            let original: Vec<_> = record.fields().collect();
            let round_tripped: Vec<_> = parsed.fields().collect();
            prop_assert_eq!(original, round_tripped);
        }
    }
    // --8<-- [end:property-test]
}
