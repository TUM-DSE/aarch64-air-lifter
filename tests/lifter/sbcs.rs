use crate::common::lib::check_instruction;

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
        nextln: v40 = i32.sub v37, v38
        nextln: v41 = i32.sub v40, v39
        nextln: i32.write_reg v41, "x1"
        nextln: v42 = i1.read_reg "c"
        nextln: v43 = i32.add v37, v38
        nextln: v44 = i32.add v43, v42
        nextln: v45 = i32.icmp.eq v44, 0x0
        nextln: i1.write_reg v45, "z"
        nextln: v46 = i32.icmp.slt v44, 0x0
        nextln: i1.write_reg v46, "n"
        nextln: v47 = i32.icmp.ugt v37, v44
        nextln: v48 = i32.icmp.ugt v38, v44
        nextln: v49 = i1.or v47, v48
        nextln: i1.write_reg v49, "c"
        nextln: v50 = i32.icmp.slt v37, 0x0
        nextln: v51 = i32.icmp.slt v38, 0x0
        nextln: v52 = i1.icmp.eq v50, v51
        nextln: v53 = i1.icmp.ne v50, v46
        nextln: v54 = i1.and v52, v53
        nextln: i1.write_reg v54, "v"
    "#;

    assert!(check_instruction(bytes, directives, None))
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
        nextln: v40 = i64.sub v37, v38
        nextln: v41 = i64.sub v40, v39
        nextln: i64.write_reg v41, "x1"
        nextln: v42 = i1.read_reg "c"
        nextln: v43 = i64.add v37, v38
        nextln: v44 = i64.add v43, v42
        nextln: v45 = i64.icmp.eq v44, 0x0
        nextln: i1.write_reg v45, "z"
        nextln: v46 = i64.icmp.slt v44, 0x0
        nextln: i1.write_reg v46, "n"
        nextln: v47 = i64.icmp.ugt v37, v44
        nextln: v48 = i64.icmp.ugt v38, v44
        nextln: v49 = i1.or v47, v48
        nextln: i1.write_reg v49, "c"
        nextln: v50 = i64.icmp.slt v37, 0x0
        nextln: v51 = i64.icmp.slt v38, 0x0
        nextln: v52 = i1.icmp.eq v50, v51
        nextln: v53 = i1.icmp.ne v50, v46
        nextln: v54 = i1.and v52, v53
        nextln: i1.write_reg v54, "v"
    "#;

    assert!(check_instruction(bytes, directives, None))
}
