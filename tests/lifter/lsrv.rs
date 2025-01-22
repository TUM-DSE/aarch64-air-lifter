use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Logical shift left variable
#[test]
fn test_lsrv_1() {
    let bytes = [
        0x20, 0x24, 0xC2, 0x1A, // lsr w0, w1, w2
    ];
    let directives = r#"
        check: // entry block
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_lsrv_2() {
    let bytes = [
        0x20, 0x24, 0xC2, 0x9A, // lsr x0, x1, x2
    ];
    let directives = r#"
        check: // entry block
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}
