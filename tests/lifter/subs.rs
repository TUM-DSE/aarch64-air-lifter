use crate::common::lib::check_instruction;

#[test]
fn test_subs_1() {
    let bytes = [
        0x21, 0x00, 0x00, 0xEB, // subs x1, x1, x0
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x1"
        nextln:  v38 = i64.read_reg "x0"
        nextln:  v39 = i64.sub v37, v38
        nextln:  i64.write_reg v39, "x1"
        nextln:  v40 = i64.not v38
        nextln:  v41 = i64.add v37, v40
        nextln:  v42 = i64.add v41, 0x1
        nextln:  v43 = i64.icmp.eq v42, 0x0
        nextln:  i1.write_reg v43, "z"
        nextln:  v44 = i64.icmp.slt v42, 0x0
        nextln:  i1.write_reg v44, "n"
        nextln:  v45 = i64.icmp.ugt v37, v42
        nextln:  v46 = i64.icmp.ugt v40, v42
        nextln:  v47 = i1.or v45, v46
        nextln:  i1.write_reg v47, "c"
        nextln:  v48 = i64.icmp.slt v37, 0x0
        nextln:  v49 = i64.icmp.slt v40, 0x0
        nextln:  v50 = i1.icmp.eq v48, v49
        nextln:  v51 = i1.icmp.ne v48, v44
        nextln:  v52 = i1.and v50, v51
        nextln:  i1.write_reg v52, "v"
"#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_subs_2() {
    let bytes = [
        0x21, 0x00, 0x00, 0x6B, // subs w1, w1, w0
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i32.read_reg "x1"
        nextln:  v38 = i32.read_reg "x0"
        nextln:  v39 = i32.sub v37, v38
        nextln:  i32.write_reg v39, "x1"
        nextln:  v40 = i32.not v38
        nextln:  v41 = i32.add v37, v40
        nextln:  v42 = i32.add v41, 0x1
        nextln:  v43 = i32.icmp.eq v42, 0x0
        nextln:  i1.write_reg v43, "z"
        nextln:  v44 = i32.icmp.slt v42, 0x0
        nextln:  i1.write_reg v44, "n"
        nextln:  v45 = i32.icmp.ugt v37, v42
        nextln:  v46 = i32.icmp.ugt v40, v42
        nextln:  v47 = i1.or v45, v46
        nextln:  i1.write_reg v47, "c"
        nextln:  v48 = i32.icmp.slt v37, 0x0
        nextln:  v49 = i32.icmp.slt v40, 0x0
        nextln:  v50 = i1.icmp.eq v48, v49
        nextln:  v51 = i1.icmp.ne v48, v44
        nextln:  v52 = i1.and v50, v51
        nextln:  i1.write_reg v52, "v"
   "#;

    assert!(check_instruction(bytes, directives, None))
}
