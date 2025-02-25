use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Add with carry with settings flags
#[test]
fn test_adcs_1() {
    let bytes = [
        0x21, 0x00, 0x00, 0xBA, // adcs x1, x1, x0
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x1"
        nextln:  v38 = i64.read_reg "x0"
        nextln:  v39 = i1.read_reg "c"
        nextln:  v40 = i64.add v37, v39
        nextln:  v41 = i64.add v40, v38
        nextln:  i64.write_reg v41, "x1"
        nextln:  v42 = i64.add v37, v38
        nextln:  v43 = i64.add v42, v39
        nextln:  v44 = i64.icmp.eq v43, 0x0
        nextln:  i1.write_reg v44, "z"
        nextln:  v45 = i64.icmp.slt v43, 0x0
        nextln:  i1.write_reg v45, "n"
        nextln:  v46 = i64.icmp.ugt v37, v43
        nextln:  v47 = i64.icmp.ugt v38, v43
        nextln:  v48 = i1.or v46, v47
        nextln:  i1.write_reg v48, "c"
        nextln:  v49 = i64.icmp.slt v37, 0x0
        nextln:  v50 = i64.icmp.slt v38, 0x0
        nextln:  v51 = i1.icmp.eq v49, v50
        nextln:  v52 = i1.icmp.ne v49, v45
        nextln:  v53 = i1.and v51, v52
        nextln:  i1.write_reg v53, "v"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}

#[test]
fn test_adcs_2() {
    let bytes = [
        0x21, 0x00, 0x00, 0x3A, // adcs w1, w1, w0
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i32.read_reg "x1"
        nextln: v38 = i32.read_reg "x0"
        nextln: v39 = i1.read_reg "c"
        nextln: v40 = i32.add v37, v39
        nextln: v41 = i32.add v40, v38
        nextln: i32.write_reg v41, "x1"
        nextln: v42 = i32.add v37, v38
        nextln: v43 = i32.add v42, v39
        nextln: v44 = i32.icmp.eq v43, 0x0
        nextln: i1.write_reg v44, "z"
        nextln: v45 = i32.icmp.slt v43, 0x0
        nextln: i1.write_reg v45, "n"
        nextln: v46 = i32.icmp.ugt v37, v43
        nextln: v47 = i32.icmp.ugt v38, v43
        nextln: v48 = i1.or v46, v47
        nextln: i1.write_reg v48, "c"
        nextln: v49 = i32.icmp.slt v37, 0x0
        nextln: v50 = i32.icmp.slt v38, 0x0
        nextln: v51 = i1.icmp.eq v49, v50
        nextln: v52 = i1.icmp.ne v49, v45
        nextln: v53 = i1.and v51, v52
        nextln: i1.write_reg v53, "v"
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}
