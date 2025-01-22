use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Load registers
#[test]
fn test_ldr_1() {
    let bytes = [
        0x40, 0x44, 0x40, 0xF8, // ldr x0, [x2], #4
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x2"
        nextln: v38 = i64.add v37, 0x4
        nextln: v39 = i64.load v38
        nextln: i64.write_reg v39, "x0"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_ldr_2() {
    let bytes = [
        0x41, 0xC4, 0x40, 0xB8, // ldr w1, [x2], #12
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x2"
        nextln: v38 = i64.add v37, 0xc
        nextln: v39 = i32.load v38
        nextln: i32.write_reg v39, "x1"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_ldr_3() {
    let bytes = [
        0xA2, 0xFF, 0xFF, 0x58, // ldr x2, -0xc
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.load 0xfffffffffffffff4
        nextln: i64.write_reg v37, "x2"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}
