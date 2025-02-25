use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Counting leading zeroes
#[test]
fn test_clz_1() {
    let bytes = [
        0x41, 0x10, 0xC0, 0xDA, // clz x1, x2
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x2"
        nextln: v38 = i64.highest_set_bit v37
        nextln: v39 = i64.sub 0x40, v38
        nextln: v40 = i64.sub v39, 0x1
        nextln: i64.write_reg v40, "x1"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}

#[test]
fn test_clz_2() {
    let bytes = [
        0x41, 0x10, 0xC0, 0x5A, // clz w1, w2
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i32.read_reg "x2"
        nextln: v38 = i32.highest_set_bit v37
        nextln: v39 = i32.sub 0x20, v38
        nextln: v40 = i32.sub v39, 0x1
        nextln: i32.write_reg v40, "x1"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}
