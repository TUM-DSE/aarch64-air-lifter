use crate::common::lib::check_instruction;

// Store register
#[test]
fn test_str_1() {
    let bytes = [
        0x00, 0x24, 0x00, 0xF8, // str x0, [x0], #2
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x0"
        nextln:  v38 = i64.read_reg "x0"
        nextln:  v39 = i64.add v38, 0x2
        nextln:  i64.store v37, v39
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_str_2() {
    let bytes = [
        0x00, 0x14, 0x00, 0xB8, // str w0, [x0], #1
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i32.read_reg "x0"
        nextln:  v38 = i64.read_reg "x0"
        nextln:  v39 = i64.add v38, 0x1
        nextln:  i32.store v37, v39
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_str_3() {
    let bytes = [
        0x41, 0x68, 0x22, 0xF8, // str x1, [x2, x2, lsl #0]
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x1"
        nextln:  v38 = i64.read_reg "x2"
        nextln:  v39 = i64.read_reg "x2"
        nextln:  v40 = i64.lshl v39, 0x0
        nextln:  v41 = i64.add v38, v40
        nextln:  i64.store v37, v41
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_str_4() {
    let bytes = [
        0x21, 0x78, 0x23, 0xB8, // str w1, [x1, x3, lsl #2]
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i32.read_reg "x1"
        nextln:  v38 = i64.read_reg "x1"
        nextln:  v39 = i64.read_reg "x3"
        nextln:  v40 = i64.lshl v39, 0x0
        nextln:  v41 = i64.add v38, v40
        nextln:  i32.store v37, v41
    "#;

    assert!(check_instruction(bytes, directives, None))
}
