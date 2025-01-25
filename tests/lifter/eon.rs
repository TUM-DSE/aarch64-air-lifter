use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Bitwise exclusive-OR NOT
#[test]
fn test_eon_1() {
    let bytes = [
        0x41, 0x30, 0x63, 0xCA, // eon x1, x2, x3, lsr #12
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x2"
        nextln:  v38 = i64.read_reg "x3"
        nextln:  v39 = i64.lshr v38, 0xc
        nextln:  v40 = i64.bitwise_not v39
        nextln:  v41 = i64.xor v37, v40
        nextln:  i64.write_reg v41, "x1"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_eon_2() {
    let bytes = [
        0x41, 0x04, 0xA3, 0xCA, // eon x1, x2, x3, asr #1
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x2"
        nextln:  v38 = i64.read_reg "x3"
        nextln:  v39 = i64.ashr v38, 0x1
        nextln:  v40 = i64.bitwise_not v39
        nextln:  v41 = i64.xor v37, v40
        nextln:  i64.write_reg v41, "x1"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_eon_3() {
    let bytes = [
        0x41, 0x04, 0xA3, 0x4A, // eon w1, w2, w3, asr #1
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i32.read_reg "x2"
        nextln:  v38 = i32.read_reg "x3"
        nextln:  v39 = i32.ashr v38, 0x1
        nextln:  v40 = i32.bitwise_not v39
        nextln:  v41 = i32.xor v37, v40
        nextln:  i32.write_reg v41, "x1"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}
