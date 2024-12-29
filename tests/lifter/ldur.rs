use crate::common::lib::check_instruction;

// Load register (unscaled)
#[test]
fn test_ldur_1() {
    let bytes = [
        0xE1, 0x73, 0x41, 0xB8, // ldur w1, [sp, #23]
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "sp"
        nextln:  v38 = i64.add v37, 0x17
        nextln:  v39 = i32.load v38
        nextln:  i32.write_reg v39, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_ldur_2() {
    let bytes = [
        0xE1, 0x73, 0x41, 0xF8, // ldur x1, [sp, #23]
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "sp"
        nextln:  v38 = i64.add v37, 0x17
        nextln:  v39 = i64.load v38
        nextln:  i64.write_reg v39, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_ldur_3() {
    let bytes = [
        0x41, 0x40, 0x40, 0xF8, // ldur x1, [x2, #4]
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x2"
        nextln:  v38 = i64.add v37, 0x4
        nextln:  v39 = i64.load v38
        nextln:  i64.write_reg v39, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None))
}
