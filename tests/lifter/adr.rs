use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Form pc-relative address
#[test]
fn test_addr_1() {
    let bytes = [
        0x00, 0x00, 0x00, 0x10, // adr x0, pc
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "pc"
        nextln: v38 = i64.add v37, 0x0
        nextln: i64.write_reg v38, "x0"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_addr_2() {
    let bytes = [
        0x21, 0x00, 0x00, 0x10, // adr x1, pc+1
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "pc"
        nextln: v38 = i64.add v37, 0x4
        nextln: i64.write_reg v38, "x1"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_addr_3() {
    let bytes = [
        0xC0, 0xFF, 0xFF, 0x10, // adr x0, pc-2
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "pc"
        nextln: v38 = i64.add v37, 0xfffffffffffffff8
        nextln: i64.write_reg v38, "x0"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}
