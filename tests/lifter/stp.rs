use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Store pair of registers
#[test]
fn test_stp_1() {
    let bytes = [
        0x41, 0x08, 0x00, 0xA9, // stp x1, x2, [x2]
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x1"
        nextln:  v38 = i64.read_reg "x2"
        nextln:  v39 = i64.read_reg "x2"
        nextln:  v40 = i64.add v39, 0x0
        nextln:  i64.store v37, v40
        nextln:  v41 = i64.add v40, 0x8
        nextln:  i64.store v38, v41
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_stp_2() {
    let bytes = [
        0x41, 0x08, 0x00, 0x29, // stp w1, w2, [x2]
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i32.read_reg "x1"
        nextln:  v38 = i32.read_reg "x2"
        nextln:  v39 = i64.read_reg "x2"
        nextln:  v40 = i64.add v39, 0x0
        nextln:  i32.store v37, v40
        nextln:  v41 = i64.add v40, 0x4
        nextln:  i32.store v38, v41
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_stp_3() {
    let bytes = [
        0xE0, 0x07, 0x00, 0xA9, // stp x0, x1, [sp]
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x0"
        nextln:  v38 = i64.read_reg "x1"
        nextln:  v39 = i64.read_reg "sp"
        nextln:  v40 = i64.add v39, 0x0
        nextln:  i64.store v37, v40
        nextln:  v41 = i64.add v40, 0x8
        nextln:  i64.store v38, v41
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}
