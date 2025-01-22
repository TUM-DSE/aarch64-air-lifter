use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Add with carry
#[test]
fn test_adc_1() {
    let bytes = [
        0x21, 0x00, 0x00, 0x9A, // adc x1, x1, x0
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x1"
        nextln: v38 = i64.read_reg "x0"
        nextln: v39 = i1.read_reg "c"
        nextln: v40 = i64.add v37, v39
        nextln: v41 = i64.add v40, v38
        nextln: i64.write_reg v41, "x1"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}

#[test]
fn test_adc_2() {
    let bytes = [
        0x21, 0x00, 0x00, 0x1A, // adc w1, w1, w0
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i32.read_reg "x1"
        nextln: v38 = i32.read_reg "x0"
        nextln: v39 = i1.read_reg "c"
        nextln: v40 = i32.add v37, v39
        nextln: v41 = i32.add v40, v38
        nextln: i32.write_reg v41, "x1"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}
