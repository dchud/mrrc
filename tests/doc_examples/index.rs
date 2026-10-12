//! Code for `docs/index.md`.

fn quick_example() -> mrrc::Result<()> {
    // --8<-- [start:quick-example]
    use mrrc::{MarcReader, RecordHelpers};
    use std::fs::File;

    let file = File::open("records.mrc")?;
    let mut reader = MarcReader::new(file);
    while let Some(record) = reader.read_record()? {
        if let Some(title) = record.title() {
            println!("{title}");
        }
    }
    // --8<-- [end:quick-example]
    Ok(())
}
