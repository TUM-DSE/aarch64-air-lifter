use crate::common::lib::{check_instruction, CheckInstructionArgs};

#[test]
fn test_rev_1() {
    let bytes = [
        0x41, 0x0C, 0xC0, 0xDA, // rev x1, x2
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x2"
        nextln: v38 = i64.reverse_bytes v37
        nextln: i64.write_reg v38, "x1"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}

#[test]
fn test_rev_2() {
    let bytes = [
        0x41, 0x08, 0xC0, 0x5A, // rev w1, w2
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i32.read_reg "x2"
        nextln: v38 = i32.reverse_bytes v37
        nextln: i32.write_reg v38, "x1"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}
