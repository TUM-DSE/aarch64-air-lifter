use crate::common::lib::check_instruction;

#[test]
// Bitwise AND
fn test_and_1() {
    let bytes = [
        0x20, 0x04, 0x00, 0x12, // and w0, w1, #3
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i32.read_reg "x1"
        nextln: v38 = i32.and v37, 0x3
        nextln: i32.write_reg v38, "x0"
    "#;

    assert!(check_instruction(bytes, directives, None));
}

#[test]
fn test_and_2() {
    let bytes = [
        0x41, 0x00, 0x40, 0x92, // and x1, x2, #1
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x2"
        nextln: v38 = i64.and v37, 0x1
        nextln: i64.write_reg v38, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None));
}

#[test]
fn test_and_3() {
    let bytes = [
        0x20, 0x10, 0x02, 0x8A, // and x0, x1, x2, LSL#4
    ];
    let directives = r#"
        check: // entry block    
        nextln: v37 = i64.read_reg "x1"
        nextln: v38 = i64.read_reg "x2"
        nextln: v39 = i64.lshl v38, 0x4
        nextln: v40 = i64.and v37, v39
        nextln: i64.write_reg v40, "x0"
    "#;

    assert!(check_instruction(bytes, directives, None));
}
