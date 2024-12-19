use crate::common::lib::check_instruction;

// Negate
#[test]
fn test_neg_1() {
    let bytes = [
        0xE0, 0x03, 0x01, 0xCB, // neg x0, x1
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x1"
        nextln:  v38 = i64.sub 0x0, v37
        nextln:  i64.write_reg v38, "x0"
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_neg_2() {
    let bytes = [
        0xE0, 0x03, 0x01, 0x4B, // neg w0, w1
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i32.read_reg "x1"
        nextln:  v38 = i32.sub 0x0, v37
        nextln:  i32.write_reg v38, "x0"
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_neg_3() {
    let bytes = [
        0xE1, 0x0F, 0x01, 0xCB, // neg x1, x1, lsl #3
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x1"
        nextln:  v38 = i64.lshl v37, 0x3
        nextln:  v39 = i64.sub 0x0, v38
        nextln:  i64.write_reg v39, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None))
}
