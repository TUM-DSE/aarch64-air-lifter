use crate::common::lib::{check_instruction, CheckInstructionArgs};

#[test]
fn test_udiv_1() {
    let bytes = [
        0x41, 0x08, 0xC3, 0x1A, // udiv x1, x2, x3
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i32.read_reg "x2"
        nextln: v38 = i32.read_reg "x3"
        nextln: v39 = i32.icmp.eq v38, 0x0
        nextln: v40 = i32.trapif v39
        nextln: v41 = i32.udiv v37, v38
        nextln: i32.write_reg v41, "x1"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_udiv_2() {
    let bytes = [
        0x41, 0x08, 0xC3, 0x9A, // udiv x1, x2, x3
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x2"
        nextln: v38 = i64.read_reg "x3"
        nextln: v39 = i64.icmp.eq v38, 0x0
        nextln: v40 = i64.trapif v39
        nextln: v41 = i64.udiv v37, v38
        nextln: i64.write_reg v41, "x1"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}
