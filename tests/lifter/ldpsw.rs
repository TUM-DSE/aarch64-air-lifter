use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Load pair of registers signed word
#[test]
fn test_ldpsw_1() {
    let bytes = [
        0x21, 0x08, 0x40, 0x69, // ldpsw x1, x2, [x1]
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x1"
        nextln: v38 = i64.add v37, 0x0
        nextln: v39 = i32.load v38
        nextln: v40 = i64.sext_i32 v39
        nextln: i64.write_reg v40, "x1"
        nextln: v41 = i64.add v38, 0x4
        nextln: v42 = i32.load v41
        nextln: v43 = i64.sext_i32 v42
        nextln: i64.write_reg v43, "x2"       
   "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}
