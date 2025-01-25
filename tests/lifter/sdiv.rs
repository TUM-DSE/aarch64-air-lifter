use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Signed divide
#[test]
fn test_sdiv_1() {
    let bytes = [
        0x20, 0x0C, 0xC2, 0x9A, // sdiv x0, x1, x2
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x1"
        nextln: v38 = i64.read_reg "x2"
        nextln: v39 = i64.icmp.eq v38, 0x0
        nextln: trapif v39
        nextln: v40 = i64.idiv v37, v38
        nextln: i64.write_reg v40, "x0"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_sdiv_2() {
    let bytes = [
        0x20, 0x0C, 0xC2, 0x1A, // sdiv w0, w1, w2
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i32.read_reg "x1"
        nextln: v38 = i32.read_reg "x2"
        nextln: v39 = i32.icmp.eq v38, 0x0
        nextln: trapif v39
        nextln: v40 = i32.idiv v37, v38
        nextln: i32.write_reg v40, "x0"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}
