use crate::common::lib::{check_instruction, CheckInstructionArgs};

#[test]
fn test_ldar_1() {
    let bytes = [
        0x21, 0xFC, 0xDF, 0x88, // ldar w1, [x1]
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x1"
        nextln:  v38 = i64.add v37, 0x0
        nextln:  v39 = i32.load v38
        nextln:  i32.write_reg v39, "x1"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_ldar_2() {
    let bytes = [
        0xE1, 0xFF, 0xDF, 0xC8, // ldar x1, [sp]
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "sp"
        nextln:  v38 = i64.add v37, 0x0
        nextln:  v39 = i64.load v38
        nextln:  i64.write_reg v39, "x1"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_ldar_3() {
    let bytes = [
        0x21, 0xFC, 0xDF, 0xC8, // ldar x1, [x1]
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x1"
        nextln:  v38 = i64.add v37, 0x0
        nextln:  v39 = i64.load v38
        nextln:  i64.write_reg v39, "x1"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}
