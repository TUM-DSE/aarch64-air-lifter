use crate::common::lib::{check_instruction, CheckInstructionArgs};

#[ignore]
#[test]
fn test_no_invalidate_regs() {
    let bytes = [
        0x20, 0x00, 0x3F, 0xD6, // blr x1
    ];

    // matches the "invalidate_regs" string
    let directives = r#"
        check: $(=invalidate_regs)
    "#;

    assert!(
        !check_instruction(bytes, directives, CheckInstructionArgs::default()),
        "invalidate_regs instruction found when translating blr"
    );
}
