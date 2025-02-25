use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Load register signed halfword (immediate)
#[test]
fn test_ldrsh_1() {
    let bytes = [
        0x21, 0xC4, 0xC0, 0x78, // ldrsh w1, [x1], #12
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x1"
        nextln: v38 = i64.add v37, 0xc
        nextln: v39 = i16.load v38
        nextln: v40 = i32.sext_i16 v39
        nextln: i32.write_reg v40, "x1"
   "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}

#[test]
fn test_ldrsh_2() {
    let bytes = [
        0x21, 0xC4, 0x80, 0x78, // ldrsh x1, [x1], #12
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x1"
        nextln: v38 = i64.add v37, 0xc
        nextln: v39 = i16.load v38
        nextln: v40 = i64.sext_i16 v39
        nextln: i64.write_reg v40, "x1"
   "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}
