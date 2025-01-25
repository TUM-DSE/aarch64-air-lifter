use crate::common::lib::{check_instruction, CheckInstructionArgs};

#[test]
fn test_movn_1() {
    let bytes = [
        0x81, 0x01, 0xA0, 0x12, // movn w1, #12, LSL#16
    ];
    let directives = r#"
        check: // entry block
        nextln: i32.write_reg 0x0, "x1"
        nextln: v37 = i16.bitwise_not 0xc0000
        nextln: i16.write_reg v37, "x1"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_movn_2() {
    let bytes = [
        0xA1, 0x01, 0x80, 0x92, // movn x1, #13, LSL#0
    ];
    let directives = r#"
        check: // entry block
        nextln:  i64.write_reg 0x0, "x1"
        nextln:  v37 = i16.bitwise_not 0xd
        nextln:  i16.write_reg v37, "x1"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}
