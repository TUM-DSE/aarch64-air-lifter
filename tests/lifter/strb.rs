use crate::common::lib::check_instruction;

#[test]
fn test_strb_1() {
    let bytes = [
        0xE0, 0x33, 0x00, 0x39, // strb w0, [sp, #12]
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i32.read_reg "x0"
        nextln:  v38 = i8.trunc_i64 v37
        nextln:  v39 = i64.read_reg "sp"
        nextln:  v40 = i64.add v39, 0xc
        nextln:  i8.store v38, v40
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_strb_2() {
    let bytes = [
        0x20, 0x08, 0x00, 0x39, // strb w0, [x1]
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i32.read_reg "x0"
        nextln:  v38 = i8.trunc_i64 v37
        nextln:  v39 = i64.read_reg "x1"
        nextln:  v40 = i64.add v39, 0x2
        nextln:  i8.store v38, v40
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_strb_3() {
    let bytes = [
        0x21, 0xD8, 0x21, 0x38, // strb w1, [x1, w1, sxtw #0]
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i32.read_reg "x1"
        nextln:  v38 = i8.trunc_i64 v37
        nextln:  v39 = i64.read_reg "x1"
        nextln:  v40 = i32.read_reg "x1"
        nextln:  v41 = i32.trunc_i64 v40
        nextln:  v42 = i64.sext_i32 v41
        nextln:  v43 = i64.add v39, v42
        nextln:  i8.store v38, v43
    "#;

    assert!(check_instruction(bytes, directives, None))
}
