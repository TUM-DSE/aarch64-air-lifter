use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Move with keep
#[test]
fn test_movk_1() {
    let bytes = [
        0x81, 0x01, 0xA0, 0x72, // movk w1, #0xc, lsl #16
    ];
    let directives = r#"
        check: // entry block
        nextln: i16.write_reg 0xc0000, "x1"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_movk_2() {
    let bytes = [
        0xA1, 0x01, 0x80, 0xF2, // movk x1, #0xd
    ];
    let directives = r#"
        check: // entry block
        nextln: i16.write_reg 0xd, "x1"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}
