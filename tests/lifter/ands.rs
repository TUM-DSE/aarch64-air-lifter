use crate::common::lib::{check_instruction, CheckInstructionArgs};

#[test]
// Bitwise ANDS
fn test_ands_1() {
    let bytes = [
        0x21, 0x00, 0x02, 0xEA, // ands x1, x1, x2
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x1"
        nextln: v38 = i64.read_reg "x2"
        nextln: v39 = i64.and v37, v38
        nextln: i64.write_reg v39, "x1"
        nextln: i1.write_reg 0x0, "c"
        nextln: i1.write_reg 0x0, "v"
        nextln: v40 = i64.icmp.eq v39, 0x0
        nextln: i1.write_reg v40, "z"
        nextln: v41 = i64.icmp.slt v39, 0x0
        nextln: i1.write_reg v41, "n"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}
