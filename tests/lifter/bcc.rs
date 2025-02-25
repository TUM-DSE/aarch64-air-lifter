use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Branch
#[test]
fn test_bcc_1() {
    let bytes = [
        0x00, 0x00, 0x00, 0x54, // b.eq pc
    ];
    let directives = r#"
        check: // entry block    
        check: v37 = i1.read_reg "z"
        nextln: v38 = i1.icmp.eq v37, 0x1
        nextln: jumpif v38, block_0, block_4
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}

#[test]
fn test_bcc_2() {
    let bytes = [
        0x07, 0x00, 0x00, 0x54, // b.vc pc
    ];
    let directives = r#"
    check: // entry block
    check: v37 = i1.read_reg "v"
    nextln: v38 = i1.icmp.ne v37, 0x1
    nextln: jumpif v38, block_0, block_4
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}
