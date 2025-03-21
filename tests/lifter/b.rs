use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Branch
#[test]
fn test_b_1() {
    let bytes = [
        0x00, 0x00, 0x00, 0x14, // b, pc
    ];
    let directives = r#"
        check: // entry block    
        nextln: jump block_0
        check: block_0: // preds: entry block_0
        nextln: jump block_0
        check: block_4: // no preds!
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}
