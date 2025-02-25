use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Add with settings flags
#[test]
fn test_add_1() {
    let bytes = [
        0x21, 0x00, 0x00, 0xAB, // add x1, x1, x0
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x1"
        nextln: v38 = i64.read_reg "x0"
        nextln: v39 = i64.add v37, v38
        nextln: i64.write_reg v39, "x1"
        nextln: v40 = i64.add v37, v38
        nextln: v41 = i64.add v40, 0x0
        nextln: v42 = i64.icmp.eq v41, 0x0
        nextln: i1.write_reg v42, "z"
        nextln: v43 = i64.icmp.slt v41, 0x0
        nextln: i1.write_reg v43, "n"
        nextln: v44 = i64.icmp.ugt v37, v41
        nextln: v45 = i64.icmp.ugt v38, v41
        nextln: v46 = i1.or v44, v45
        nextln: i1.write_reg v46, "c"
        nextln: v47 = i64.icmp.slt v37, 0x0
        nextln: v48 = i64.icmp.slt v38, 0x0
        nextln: v49 = i1.icmp.eq v47, v48
        nextln: v50 = i1.icmp.ne v47, v43
        nextln: v51 = i1.and v49, v50
        nextln: i1.write_reg v51, "v"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}

#[test]
fn test_add_2() {
    let bytes = [
        0x21, 0x00, 0x00, 0x2B, // add w1, w1, w0
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i32.read_reg "x1"
        nextln: v38 = i32.read_reg "x0"
        nextln: v39 = i32.add v37, v38
        nextln: i32.write_reg v39, "x1"
        nextln: v40 = i32.add v37, v38
        nextln: v41 = i32.add v40, 0x0
        nextln: v42 = i32.icmp.eq v41, 0x0
        nextln: i1.write_reg v42, "z"
        nextln: v43 = i32.icmp.slt v41, 0x0
        nextln: i1.write_reg v43, "n"
        nextln: v44 = i32.icmp.ugt v37, v41
        nextln: v45 = i32.icmp.ugt v38, v41
        nextln: v46 = i1.or v44, v45
        nextln: i1.write_reg v46, "c"
        nextln: v47 = i32.icmp.slt v37, 0x0
        nextln: v48 = i32.icmp.slt v38, 0x0
        nextln: v49 = i1.icmp.eq v47, v48
        nextln: v50 = i1.icmp.ne v47, v43
        nextln: v51 = i1.and v49, v50
        nextln: i1.write_reg v51, "v"
  "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}
