use crate::common::lib::{check_instruction, CheckInstructionArgs};

// bit-wise exclusive OR
#[test]
fn test_extr_1() {
    let bytes = [
        0x20, 0x30, 0xC2, 0x93, // extr x0, x1, x2, #12
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x1"
        nextln:  v38 = i64.read_reg "x2"
        nextln:  v39 = i64.lshr v38, 0xc
        nextln:  v40 = i64.sub 0x40, 0xc
        nextln:  v41 = i64.lshl v37, v40
        nextln:  v42 = i64.or v41, v39
        nextln:  i64.write_reg v42, "x0"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}

#[test]
fn test_extr_2() {
    let bytes = [
        0x00, 0xC8, 0xC0, 0x93, // ror x0, x0, #50 <=> extr x0, x0, x0, #50
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x0"
        nextln:  v38 = i64.read_reg "x0"
        nextln:  v39 = i64.lshr v38, 0x32
        nextln:  v40 = i64.sub 0x40, 0x32
        nextln:  v41 = i64.lshl v37, v40
        nextln:  v42 = i64.or v41, v39
        nextln:  i64.write_reg v42, "x0"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}

#[test]
fn test_extr_3() {
    let bytes = [
        0x41, 0x0C, 0x83, 0x13, // extr w1, w2, w3, #3
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i32.read_reg "x2"
        nextln:  v38 = i32.read_reg "x3"
        nextln:  v39 = i32.lshr v38, 0x3
        nextln:  v40 = i32.sub 0x20, 0x3
        nextln:  v41 = i32.lshl v37, v40
        nextln:  v42 = i32.or v41, v39
        nextln:  i32.write_reg v42, "x1"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}
