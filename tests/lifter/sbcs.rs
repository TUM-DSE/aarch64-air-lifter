use crate::common::lib::{check_instruction, CheckInstructionArgs};

#[test]
fn test_sbcs_1() {
    let bytes = [
        0x41, 0x00, 0x03, 0x7A, // sbcs w1, w2, w3
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i32.read_reg "x2"
        nextln: v38 = i32.read_reg "x3"
        nextln: v39 = i1.read_reg "c"
        nextln: v40 = i1.bitwise_not v39
        nextln: v41 = i32.sub v37, v38
        nextln: v42 = i32.sub v41, v40
        nextln: i32.write_reg v42, "x1"
        nextln: v43 = i1.read_reg "c"
        nextln: v44 = i32.add v37, v38
        nextln: v45 = i32.add v44, v43
        nextln: v46 = i32.icmp.eq v45, 0x0
        nextln: i1.write_reg v46, "z"
        nextln: v47 = i32.icmp.slt v45, 0x0
        nextln: i1.write_reg v47, "n"
        nextln: v48 = i32.icmp.ugt v37, v45
        nextln: v49 = i32.icmp.ugt v38, v45
        nextln: v50 = i1.or v48, v49
        nextln: i1.write_reg v50, "c"
        nextln: v51 = i32.icmp.slt v37, 0x0
        nextln: v52 = i32.icmp.slt v38, 0x0
        nextln: v53 = i1.icmp.eq v51, v52
        nextln: v54 = i1.icmp.ne v51, v47
        nextln: v55 = i1.and v53, v54
        nextln: i1.write_reg v55, "v"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_sbcs_2() {
    let bytes = [
        0x41, 0x00, 0x03, 0xFA, // sbcs x1, x2, x3
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x2"
        nextln: v38 = i64.read_reg "x3"
        nextln: v39 = i1.read_reg "c"
        nextln: v40 = i1.bitwise_not v39
        nextln: v41 = i64.sub v37, v38
        nextln: v42 = i64.sub v41, v40
        nextln: i64.write_reg v42, "x1"
        nextln: v43 = i1.read_reg "c"
        nextln: v44 = i64.add v37, v38
        nextln: v45 = i64.add v44, v43
        nextln: v46 = i64.icmp.eq v45, 0x0
        nextln: i1.write_reg v46, "z"
        nextln: v47 = i64.icmp.slt v45, 0x0
        nextln: i1.write_reg v47, "n"
        nextln: v48 = i64.icmp.ugt v37, v45
        nextln: v49 = i64.icmp.ugt v38, v45
        nextln: v50 = i1.or v48, v49
        nextln: i1.write_reg v50, "c"
        nextln: v51 = i64.icmp.slt v37, 0x0
        nextln: v52 = i64.icmp.slt v38, 0x0
        nextln: v53 = i1.icmp.eq v51, v52
        nextln: v54 = i1.icmp.ne v51, v47
        nextln: v55 = i1.and v53, v54
        nextln: i1.write_reg v55, "v"
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}
