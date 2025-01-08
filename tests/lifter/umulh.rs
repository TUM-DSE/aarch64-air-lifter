use crate::common::lib::check_instruction;

#[test]
fn test_umulh_1() {
    let bytes = [
        0x41, 0x7C, 0xC3, 0x9B, // umulh x1, x2, x3
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x2"
        nextln: v38 = i64.read_reg "x3"
        nextln: v39 = i128.umul v37, v38
        nextln: v40 = i128.ashr v39, 0x40
        nextln: i64.write_reg v40, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None))
}
