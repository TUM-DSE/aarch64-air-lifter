use crate::common::lib::check_instruction;

#[test]
fn test_movn_1() {
    let bytes = [
        0x81, 0x01, 0xA0, 0x12, // movn w1, #12, LSL#16
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i16.not 0xc0000
        nextln: i16.write_reg v37, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_movn_2() {
    let bytes = [
        0xA1, 0x01, 0x80, 0x92, // movn x1, #13, LSL#0
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i16.not 0xd
        nextln: i16.write_reg v37, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None))
}
