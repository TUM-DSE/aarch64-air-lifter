use crate::common::lib::{check_instruction, CheckInstructionArgs};

#[test]
fn test_strb_1() {
    let bytes = [
        0xE0, 0x33, 0x00, 0x39, // strb w0, [sp, #12]
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i32.read_reg "x0"
        nextln: v38 = i64.read_reg "sp"
        nextln: v39 = i64.add v38, 0xc
        nextln: i8.store v37, v39
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}

#[test]
fn test_strb_2() {
    let bytes = [
        0x20, 0x08, 0x00, 0x39, // strb w0, [x1]
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i32.read_reg "x0"
        nextln: v38 = i64.read_reg "x1"
        nextln: v39 = i64.add v38, 0x2
        nextln: i8.store v37, v39
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}

#[test]
fn test_strb_3() {
    let bytes = [
        0x21, 0xD8, 0x21, 0x38, // strb w1, [x1, w1, sxtw #0]
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i32.read_reg "x1"
        nextln: v38 = i64.read_reg "x1"
        nextln: v39 = i32.read_reg "x1"
        nextln: v40 = i32.trunc_i64 v39
        nextln: v41 = i64.sext_i32 v40
        nextln: v42 = i64.add v38, v41
        nextln: i8.store v37, v42
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}
