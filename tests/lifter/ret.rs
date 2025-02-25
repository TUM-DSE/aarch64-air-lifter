use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Ret
#[test]
fn test_ret_1() {
    let bytes = [
        0xc0, 0x03, 0x5f, 0xd6, // ret
    ];
    let directives = r#"
        check: // entry block
        ret
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}

#[test]
fn test_ret_2() {
    let bytes = [
        0x20, 0x00, 0x5f, 0xd6, // ret	x1
    ];
    let directives = r#"
        check: // entry block
        ret
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}
