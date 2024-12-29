use crate::common::lib::check_instruction;

#[test]
fn test_sbc_1() {
    let bytes = [
        0x41, 0x00, 0x03, 0x5A, // sbc w1, w2, w3
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i32.read_reg "x2"
        nextln: v38 = i32.read_reg "x3"
        nextln: v39 = i1.read_reg "c"
        nextln: v40 = i32.sub v37, v38
        nextln: v41 = i32.sub v40, v39
        nextln: i32.write_reg v41, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_sbc_2() {
    let bytes = [
        0x41, 0x00, 0x03, 0xDA, // sbc x1, x2, x3
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x2"
        nextln: v38 = i64.read_reg "x3"
        nextln: v39 = i1.read_reg "c"
        nextln: v40 = i64.sub v37, v38
        nextln: v41 = i64.sub v40, v39
        nextln: i64.write_reg v41, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None))
}
