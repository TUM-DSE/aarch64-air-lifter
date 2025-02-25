use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Signed multiply-add long
#[test]
fn test_smaddl_1() {
    let bytes = [
        0x20, 0x00, 0x22, 0x9B, // smaddl x0, w1, w2, x0
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i32.read_reg "x1"
        nextln: v38 = i32.read_reg "x2"
        nextln: v39 = i64.read_reg "x0"
        nextln: v40 = i32.imul v37, v38
        nextln: v41 = i64.add v40, v39
        nextln: i64.write_reg v41, "x0"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}
