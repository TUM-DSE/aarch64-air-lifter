use crate::common::lib::{check_instruction, CheckInstructionArgs};

#[test]
fn test_smulh_1() {
    let bytes = [
        0x41, 0x7C, 0x43, 0x9B, // smulh x1, x2, x3
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x2"
        nextln: v38 = i64.read_reg "x3"
        nextln: v39 = i64.imul v37, v38
        nextln: v40 = i128.ashr v39, 0x40
        nextln: i64.write_reg v40, "x1"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}
