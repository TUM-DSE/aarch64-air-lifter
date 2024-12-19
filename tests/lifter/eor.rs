use crate::common::lib::check_instruction;

// bit-wise exclusive OR
#[test]
fn test_eor_1() {
    let bytes = [
        0x20, 0x00, 0x7E, 0xD2, // eor x0, x1, #0x4
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x1"
        nextln:  v38 = i64.xor v37, 0x4
        nextln:  i64.write_reg v37, "x0"
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_eor_2() {
    let bytes = [
        0x41, 0x00, 0x1D, 0x52, // eor w1, w2, #0x8
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i32.read_reg "x2"
        nextln:  v38 = i32.xor v37, 0x8
        nextln:  i32.write_reg v37, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None))
}
