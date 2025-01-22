use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Logical shift left variable
#[test]
fn test_lslv_1() {
    let bytes = [
        0x20, 0x20, 0xC2, 0x9A, // lslv x0, x1, x2
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x1"
        nextln:  v38 = i64.read_reg "x2"
        nextln:  v39 = i64.and v38, 0x3f
        nextln:  v40 = i64.lshl v37, v39
        nextln:  i64.write_reg v40, "x0"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_lslv_2() {
    let bytes = [
        0x20, 0x20, 0xC2, 0x1A, // lslv w0, w1, w2
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i32.read_reg "x1"
        nextln:  v38 = i32.read_reg "x2"
        nextln:  v39 = i32.and v38, 0x1f
        nextln:  v40 = i32.lshl v37, v39
        nextln:  i32.write_reg v40, "x0"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}
