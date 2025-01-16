use crate::common::lib::check_instruction;

// Branch with link
#[test]
fn test_blr_1() {
    let bytes = [
        0x20, 0x00, 0x3F, 0xD6, // blr x1
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "pc"
        nextln: v38 = i64.add v37, 0x4
        nextln: i64.write_reg v38, "x30"
        nextln: v39 = i64.read_reg "x1"
        nextln: dynamic_jump v39
    "#;

    assert!(check_instruction(bytes, directives, None))
}
