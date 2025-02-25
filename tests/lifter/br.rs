use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Branch with link
#[test]
fn test_blr_1() {
    let bytes = [
        0x20, 0x00, 0x1F, 0xD6, // blr x1
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x1"
        nextln: dynamic_jump v37
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}
