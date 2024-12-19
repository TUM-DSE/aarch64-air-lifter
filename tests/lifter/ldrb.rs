use crate::common::lib::check_instruction;

// Load register byte
#[test]
fn test_ldrb_1() {
    let bytes = [
        0xE0, 0xDB, 0x62, 0x38, // ldrb w0, [sp, w2, sxtw #0]
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "sp"
        nextln:  v38 = i32.read_reg "x2"
        nextln:  v39 = i32.trunc_i64 v38
        nextln:  v40 = i64.sext_i32 v39
        nextln:  v41 = i64.add v37, v40
        nextln:  v42 = i8.load v41
        nextln:  v43 = i32.zext_i8 v42
        nextln:  i32.write_reg v43, "x0"
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_ldrb_2() {
    let bytes = [
        0x20, 0xD8, 0x62, 0x38, // ldrb w0, [x1, w2, sxtw #0]
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x1"
        nextln:  v38 = i32.read_reg "x2"
        nextln:  v39 = i32.trunc_i64 v38
        nextln:  v40 = i64.sext_i32 v39
        nextln:  v41 = i64.add v37, v40
        nextln:  v42 = i8.load v41
        nextln:  v43 = i32.zext_i8 v42
        nextln:  i32.write_reg v43, "x0"
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_ldrb_3() {
    let bytes = [
        0x20, 0x58, 0x62, 0x38, // ldrb w0, [x1, w2, uxtw #0]
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x1"
        nextln:  v38 = i32.read_reg "x2"
        nextln:  v39 = i64.add v37, v38
        nextln:  v40 = i8.load v39
        nextln:  v41 = i32.zext_i8 v40
        nextln:  i32.write_reg v41, "x0"
    "#;

    assert!(check_instruction(bytes, directives, None))
}
