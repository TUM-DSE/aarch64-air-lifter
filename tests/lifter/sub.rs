use crate::common::lib::check_instruction;

#[test]
fn test_sub_1() {
    let bytes = [
        0x21, 0x00, 0x00, 0xCB, // sub x1, x1, x0
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x1"
        nextln:  v38 = i64.read_reg "x0"
        nextln:  v39 = i64.sub v37, v38
        nextln:  i64.write_reg v39, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_sub_2() {
    let bytes = [
        0x21, 0x00, 0x00, 0x4B, // sub w1, w1, w0
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i32.read_reg "x1"
        nextln:  v38 = i32.read_reg "x0"
        nextln:  v39 = i32.sub v37, v38
        nextln:  i32.write_reg v39, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_sub_3() {
    let bytes = [
        0x02, 0xC0, 0x21, 0x4B, // sub w2, w0, w1, SXTW
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i32.read_reg "x0"
        nextln:  v38 = i32.read_reg "x1"
        nextln:  v39 = i32.trunc_i64 v38
        nextln:  v40 = i32.sext_i32 v39
        nextln:  v41 = i32.sub v37, v40
        nextln:  i32.write_reg v41, "x2"
    "#;

    assert!(check_instruction(bytes, directives, None))
}
