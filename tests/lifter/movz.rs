use crate::common::lib::check_instruction;

#[test]
fn test_movz_1() {
    let bytes = [
        0x81, 0x01, 0xA0, 0x52, // movz w1, #12, LSL#16
    ];
    let directives = r#"
        check: // entry block
        nextln: i32.write_reg 0x0, "x1"
        nextln: i16.write_reg 0xc0000, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_movz_2() {
    let bytes = [
        0xA1, 0x01, 0x80, 0xd2, // movz x1, #13, LSL#0
    ];
    let directives = r#"
        check: // entry block
        nextln: i64.write_reg 0x0, "x1"
        nextln: i16.write_reg 0xd, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None))
}
