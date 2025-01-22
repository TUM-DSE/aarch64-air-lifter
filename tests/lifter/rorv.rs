use crate::common::lib::{check_instruction, CheckInstructionArgs};

#[test]
fn test_rorv_1() {
    let bytes = [
        0x21, 0x2C, 0xC2, 0x9A, // rorv x1, x1, x2
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x1"
        nextln: v38 = i64.read_reg "x2"
        nextln: v39 = i64.and v38, 0x3f
        nextln: v40 = i64.ror v37, v39
        nextln: i64.write_reg v40, "x1"
   "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_rorv_2() {
    let bytes = [
        0x21, 0x2C, 0xC2, 0x1A, // rorv w1, w1, x2
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i32.read_reg "x1"
        nextln: v38 = i32.read_reg "x2"
        nextln: v39 = i32.and v38, 0x1f
        nextln: v40 = i32.ror v37, v39
        nextln: i32.write_reg v40, "x1"
   "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}
