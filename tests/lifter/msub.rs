use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Multiply-sub
#[test]
fn test_msub_1() {
    let bytes = [
        0x20, 0x8C, 0x02, 0x9B, // msub x0, x1, x2, x3
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x1"
        nextln:  v38 = i64.read_reg "x2"
        nextln:  v39 = i64.read_reg "x3"
        nextln:  v40 = i64.imul v37, v38
        nextln:  v41 = i64.sub v39, v40
        nextln:  i64.write_reg v41, "x0"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_msub_2() {
    let bytes = [
        0x20, 0x8C, 0x02, 0x1B, // msub w0, w1, w2, w3
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i32.read_reg "x1"
        nextln:  v38 = i32.read_reg "x2"
        nextln:  v39 = i32.read_reg "x3"
        nextln:  v40 = i32.imul v37, v38
        nextln:  v41 = i32.sub v39, v40
        nextln:  i32.write_reg v41, "x0"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}
