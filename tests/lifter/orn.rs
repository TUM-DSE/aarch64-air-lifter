use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Bitwise AND
#[test]
fn test_and_1() {
    let bytes = [
        0x20, 0x0C, 0x22, 0x2A, // orn w0, w1, w2, lsl #3
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i32.read_reg "x1"
        nextln:  v38 = i32.read_reg "x2"
        nextln:  v39 = i32.lshl v38, 0x3
        nextln:  v40 = i32.bitwise_not v39
        nextln:  v41 = i32.or v37, v40
        nextln:  i32.write_reg v41, "x0"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}

#[test]
fn test_and_2() {
    let bytes = [
        0x41, 0x00, 0x21, 0xAA, // orn x1, x2, x1
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x2"
        nextln:  v38 = i64.read_reg "x1"
        nextln:  v39 = i64.bitwise_not v38
        nextln:  v40 = i64.or v37, v39
        nextln:  i64.write_reg v40, "x1"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}

#[test]
fn test_orn_3() {
    let bytes = [
        0x20, 0x10, 0x22, 0xAA, // orn x0, x1, x2, lsl #4
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x1"
        nextln:  v38 = i64.read_reg "x2"
        nextln:  v39 = i64.lshl v38, 0x4
        nextln:  v40 = i64.bitwise_not v39
        nextln:  v41 = i64.or v37, v40
        nextln:  i64.write_reg v41, "x0"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}
