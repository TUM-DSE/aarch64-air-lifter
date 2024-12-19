use crate::common::lib::check_instruction;

#[test]
fn test_ldarb_1() {
    let bytes = [
        0xE1, 0xFF, 0xDF, 0x08, // ldarb w1, [sp]
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "sp"
        nextln:  v38 = i64.add v37, 0x0
        nextln:  v39 = i8.load v38
        nextln:  v40 = i32.zext_i8 v39
        nextln:  i32.write_reg v40, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_ldarb_2() {
    let bytes = [
        0x21, 0xFC, 0xDF, 0x08, // ldarb w1, [x1]
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x1"
        nextln:  v38 = i64.add v37, 0x0
        nextln:  v39 = i8.load v38
        nextln:  v40 = i32.zext_i8 v39
        nextln:  i32.write_reg v40, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None))
}
