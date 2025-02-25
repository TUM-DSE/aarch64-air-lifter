use crate::common::lib::{check_instruction, CheckInstructionArgs};

#[test]
fn test_umaddl_1() {
    let bytes = [
        0x21, 0x08, 0xA2, 0x9B, // umaddl x1, w1, w2, x2
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i32.read_reg "x1"
        nextln: v38 = i32.read_reg "x2"
        nextln: v39 = i64.read_reg "x2"
        nextln: v40 = i32.umul v37, v38
        nextln: v41 = i64.add v40, v39
        nextln: i64.write_reg v41, "x1"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}
