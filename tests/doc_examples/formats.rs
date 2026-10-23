//! Code for `docs/reference/formats.md`.

use mrrc::Record;

fn iso_2709(record: Record) -> mrrc::Result<()> {
    // --8<-- [start:iso-2709]
    use mrrc::{MarcReader, MarcWriter, RecordHelpers};
    use std::fs::File;

    // Reading
    let mut reader = MarcReader::new(File::open("records.mrc")?);
    while let Some(record) = reader.read_record()? {
        println!("{:?}", record.title());
    }

    // Writing
    let mut writer = MarcWriter::new(File::create("output.mrc")?);
    writer.write_record(&record)?;
    // --8<-- [end:iso-2709]
    Ok(())
}
