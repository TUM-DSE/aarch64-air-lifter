use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Form PC-relative address to 4KB page
#[test]
fn test_adrp_1() {
    let bytes = [
        0x00, 0x00, 0x00, 0x90, // adrp x0, pc
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.not 0xfff
        nextln: v38 = i64.read_reg "pc"
        nextln: v39 = i64.and v38, v37
        nextln: v40 = i64.add v39, 0x0
        nextln: i64.write_reg v40, "x0"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_adrp_2() {
    let bytes = [
        0x00, 0x00, 0x00, 0xB0, // adrp x0, pc+4096
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.not 0xfff
        nextln: v38 = i64.read_reg "pc"
        nextln: v39 = i64.and v38, v37
        nextln: v40 = i64.add v39, 0x1000
        nextln: i64.write_reg v40, "x0"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}
