//! Code for `docs/reference/marc-primer.md`.

use mrrc::Record;

fn leader(record: Record) {
    // --8<-- [start:leader]
    let leader = &record.leader;
    println!("{}", leader.record_status);
    println!("{}", leader.record_type);
    // --8<-- [end:leader]
}

fn control_fields(record: Record) {
    // --8<-- [start:control-fields]
    if let Some(data) = record.get_control_field("001") {
        println!("{data}");
    }
    // --8<-- [end:control-fields]
}

fn subfields(record: Record) {
    // --8<-- [start:subfields]
    if let Some(field) = record.get_field("245") {
        println!("{:?}", field.get_subfield('a'));
        println!("{:?}", field.get_subfield('b'));
        println!("Indicators: {} {}", field.indicator1, field.indicator2);
    }
    // --8<-- [end:subfields]
}

fn helper_methods(record: Record) {
    // --8<-- [start:helper-methods]
    use mrrc::RecordHelpers;

    println!("{:?}", record.title());
    println!("{:?}", record.author());
    for isbn in record.isbns() {
        println!("{isbn}");
    }
    // --8<-- [end:helper-methods]
}

#[test]
fn fragments_run_against_fixtures() {
    for record in crate::fixtures::records() {
        leader(record.clone());
        control_fields(record.clone());
        subfields(record.clone());
        helper_methods(record);
    }
}
