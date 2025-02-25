use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Bitwise bit clear (shifted register)
#[test]
fn test_bic_1() {
    let bytes = [
        0x41, 0x08, 0x23, 0x0A, // bic w1, w2, w3, lsl #2
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i32.read_reg "x2"
        nextln: v38 = i32.read_reg "x3"
        nextln: v39 = i32.lshl v38, 0x2
        nextln: v40 = i32.bitwise_not v39
        nextln: v41 = i32.and v37, v40
        nextln: i32.write_reg v41, "x1"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}

#[test]
fn test_bic_2() {
    let bytes = [
        0x41, 0x0C, 0x23, 0x8A, // bic x1, x2, x3, lsl #3
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x2"
        nextln: v38 = i64.read_reg "x3"
        nextln: v39 = i64.lshl v38, 0x3
        nextln: v40 = i64.bitwise_not v39
        nextln: v41 = i64.and v37, v40
        nextln: i64.write_reg v41, "x1"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}

#[test]
fn test_bic_3() {
    let bytes = [
        0x41, 0x04, 0xA3, 0x8A, // bic x1, x2, x3, asr #1
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x2"
        nextln: v38 = i64.read_reg "x3"
        nextln: v39 = i64.ashr v38, 0x1
        nextln: v40 = i64.bitwise_not v39
        nextln: v41 = i64.and v37, v40
        nextln: i64.write_reg v41, "x1"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}
