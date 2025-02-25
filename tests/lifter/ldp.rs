use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Load pair of registers
#[test]
fn test_ldp_1() {
    let bytes = [
        0x40, 0x84, 0xC0, 0xA8, // ldp x0, x1, [x2], #8
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x2"
        nextln:  v38 = i64.add v37, 0x8
        nextln:  v39 = i64.load v38
        nextln:  i64.write_reg v39, "x0"
        nextln:  v40 = i64.add v38, 0x8
        nextln:  v41 = i64.load v40
        nextln:  i64.write_reg v41, "x1"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}

#[test]
fn test_ldp_2() {
    let bytes = [
        0x81, 0x08, 0xC2, 0x28, // ldp w1, w2, [x4], #16
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x4"
        nextln:  v38 = i64.add v37, 0x10
        nextln:  v39 = i32.load v38
        nextln:  i32.write_reg v39, "x1"
        nextln:  v40 = i64.add v38, 0x4
        nextln:  v41 = i32.load v40
        nextln:  i32.write_reg v41, "x2"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}
