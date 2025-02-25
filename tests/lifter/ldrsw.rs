use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Load register signed word (immediate)
#[test]
fn test_ldrsw_1() {
    let bytes = [
        0x21, 0xC4, 0x80, 0xB8, // ldrsw x1, [x1], #12
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x1"
        nextln: v38 = i64.add v37, 0xc
        nextln: v39 = i32.load v38
        nextln: v40 = i64.sext_i32 v39
        nextln: i64.write_reg v40, "x1"
   "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}
