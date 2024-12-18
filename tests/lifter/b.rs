use crate::common::lib::check_instruction;

// Branch
#[test]
fn test_b_1() {
    let bytes = [
        0x00, 0x00, 0x00, 0x14, // b, pc
    ];
    let directives = r#"
        #0 check: // entry block    
        #1 check: block_0: // preds: block_0
        #2 nextln: jump block_0
        #3 check: block_4: // no preds!
    "#;

    assert!(check_instruction(bytes, directives, None));
}
