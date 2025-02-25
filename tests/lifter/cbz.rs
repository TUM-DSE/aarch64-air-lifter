use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Compare and branch on zero
#[test]
fn test_cbz_1() {
    let bytes = [
        0x00, 0x00, 0x00, 0x34, // cbz w0, pc
    ];
    let directives = r#"
        check: // entry block
        check: block_0:
        nextln:   v37 = i32.read_reg "x0"
        nextln:   v38 = i32.icmp.eq v37, 0x0
        nextln:   jumpif v38, $LABEL, $LABEL
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}

#[test]
fn test_cbz_2() {
    let bytes = [
        0x20, 0x00, 0x00, 0xB4, // cbz x0, pc+1
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x0"
        nextln: v38 = i64.icmp.eq v37, 0x0
        nextln: jumpif v38, $LABEL, $LABEL
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}

#[test]
fn test_cbz_3() {
    let bytes = [
        0xC1, 0xFF, 0xFF, 0xB4, // cbz x1, pc-2
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x1"
        nextln: v38 = i64.icmp.eq v37, 0x0
        nextln: jumpif v38, $LABEL, $LABEL
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}
