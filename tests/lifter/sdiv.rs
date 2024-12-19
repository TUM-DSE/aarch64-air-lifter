use crate::common::lib::check_instruction;

// Signed divide
#[test]
fn test_sdiv_1() {
    let bytes = [
        0x20, 0x0C, 0xC2, 0x9A, // sdiv x0, x1, x2
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x1"
        nextln:  v38 = i64.read_reg "x2"
        nextln:  v39 = i64.idiv v37, v38
        nextln:  i64.write_reg v39, "x0"
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_sdiv_2() {
    let bytes = [
        0x20, 0x0C, 0xC2, 0x1A, // sdiv w0, w1, w2
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i32.read_reg "x1"
        nextln:  v38 = i32.read_reg "x2"
        nextln:  v39 = i32.idiv v37, v38
        nextln:  i32.write_reg v39, "x0"
    "#;

    assert!(check_instruction(bytes, directives, None))
}
