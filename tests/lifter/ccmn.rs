use crate::common::lib::check_instruction;

// Conditional compare negative (immediate)
#[test]
fn test_ccmn_1() {
    let bytes = [
        0x03, 0x00, 0x41, 0xBA, // ccmn x0, x1, #0x3, eq
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i1.read_reg "z"
        nextln:   v38 = i1.icmp.eq v37, 0x1
        nextln:   jumpif v38, ccmp_positive_condition, ccmp_negative_condition
        check: ccmp_positive_condition:
        nextln:   v39 = i64.and 0x8, 0x3
        nextln:   v40 = i64.icmp.ne 0x0, v39
        nextln:   i1.write_reg v40, "n"
        nextln:   v41 = i64.and 0x4, 0x3
        nextln:   v42 = i64.icmp.ne 0x0, v41
        nextln:   i1.write_reg v42, "z"
        nextln:   v43 = i64.and 0x2, 0x3
        nextln:   v44 = i64.icmp.ne 0x0, v43
        nextln:   i1.write_reg v44, "c"
        nextln:   v45 = i64.and 0x1, 0x3
        nextln:   v46 = i64.icmp.ne 0x0, v45
        nextln:   i1.write_reg v46, "v"
        nextln:   jump $LABEL 
        check: ccmp_negative_condition:
        nextln:   v47 = i64.read_reg "x0"
        nextln:   v48 = i64.read_reg "x1"
        nextln:   v49 = i64.sub 0x0, v48
        nextln:   v50 = i64.not v49
        nextln:   v51 = i64.add v47, v50
        nextln:   v52 = i64.add v51, 0x1
        nextln:   v53 = i64.icmp.eq v52, 0x0
        nextln:   i1.write_reg v53, "z"
        nextln:   v54 = i64.icmp.slt v52, 0x0
        nextln:   i1.write_reg v54, "n"
        nextln:   v55 = i64.icmp.ugt v47, v52
        nextln:   v56 = i64.icmp.ugt v50, v52
        nextln:   v57 = i1.or v55, v56
        nextln:   i1.write_reg v57, "c"
        nextln:   v58 = i64.icmp.slt v47, 0x0
        nextln:   v59 = i64.icmp.slt v50, 0x0
        nextln:   v60 = i1.icmp.eq v58, v59
        nextln:   v61 = i1.icmp.ne v58, v54
        nextln:   v62 = i1.and v60, v61
        nextln:   i1.write_reg v62, "v"
        nextln:   jump $LABEL
    "#;
    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_ccmn_2() {
    let bytes = [
        0xC0, 0xE0, 0x46, 0xBA, // ccmn x6, x6, #0x0, al
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i1.and 0x1, 0x1
        nextln:   jumpif v37, ccmp_positive_condition, ccmp_negative_condition
        check: block_4:
        check: ccmp_positive_condition:
        nextln:   v38 = i64.and 0x8, 0x0
        nextln:   v39 = i64.icmp.ne 0x0, v38
        nextln:   i1.write_reg v39, "n"
        nextln:   v40 = i64.and 0x4, 0x0
        nextln:   v41 = i64.icmp.ne 0x0, v40
        nextln:   i1.write_reg v41, "z"
        nextln:   v42 = i64.and 0x2, 0x0
        nextln:   v43 = i64.icmp.ne 0x0, v42
        nextln:   i1.write_reg v43, "c"
        nextln:   v44 = i64.and 0x1, 0x0
        nextln:   v45 = i64.icmp.ne 0x0, v44
        nextln:   i1.write_reg v45, "v"
        nextln:   jump $LABEL
        check: ccmp_negative_condition: // preds: entry
        nextln:   v46 = i64.read_reg "x6"
        nextln:   v47 = i64.read_reg "x6"
        nextln:   v48 = i64.sub 0x0, v47
        nextln:   v49 = i64.not v48
        nextln:   v50 = i64.add v46, v49
        nextln:   v51 = i64.add v50, 0x1
        nextln:   v52 = i64.icmp.eq v51, 0x0
        nextln:   i1.write_reg v52, "z"
        nextln:   v53 = i64.icmp.slt v51, 0x0
        nextln:   i1.write_reg v53, "n"
        nextln:   v54 = i64.icmp.ugt v46, v51
        nextln:   v55 = i64.icmp.ugt v49, v51
        nextln:   v56 = i1.or v54, v55
        nextln:   i1.write_reg v56, "c"
        nextln:   v57 = i64.icmp.slt v46, 0x0
        nextln:   v58 = i64.icmp.slt v49, 0x0
        nextln:   v59 = i1.icmp.eq v57, v58
        nextln:   v60 = i1.icmp.ne v57, v53
        nextln:   v61 = i1.and v59, v60
        nextln:   i1.write_reg v61, "v"
        nextln:   jump $LABEL
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_ccmn_3() {
    let bytes = [
        0x04, 0x60, 0x42, 0x3A, // ccmn w0, w2, #0x4, vs
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i1.read_reg "v"
        nextln:   v38 = i1.icmp.eq v37, 0x1
        nextln:   jumpif v38, ccmp_positive_condition, ccmp_negative_condition
        check: ccmp_positive_condition:
        nextln:   v39 = i32.and 0x8, 0x4
        nextln:   v40 = i32.icmp.ne 0x0, v39
        nextln:   i1.write_reg v40, "n"
        nextln:   v41 = i32.and 0x4, 0x4
        nextln:   v42 = i32.icmp.ne 0x0, v41
        nextln:   i1.write_reg v42, "z"
        nextln:   v43 = i32.and 0x2, 0x4
        nextln:   v44 = i32.icmp.ne 0x0, v43
        nextln:   i1.write_reg v44, "c"
        nextln:   v45 = i32.and 0x1, 0x4
        nextln:   v46 = i32.icmp.ne 0x0, v45
        nextln:   i1.write_reg v46, "v"
        nextln:   jump $LABEL
        check: ccmp_negative_condition:
        nextln:   v47 = i32.read_reg "x0"
        nextln:   v48 = i32.read_reg "x2"
        nextln:   v49 = i32.sub 0x0, v48
        nextln:   v50 = i32.not v49
        nextln:   v51 = i32.add v47, v50
        nextln:   v52 = i32.add v51, 0x1
        nextln:   v53 = i32.icmp.eq v52, 0x0
        nextln:   i1.write_reg v53, "z"
        nextln:   v54 = i32.icmp.slt v52, 0x0
        nextln:   i1.write_reg v54, "n"
        nextln:   v55 = i32.icmp.ugt v47, v52
        nextln:   v56 = i32.icmp.ugt v50, v52
        nextln:   v57 = i1.or v55, v56
        nextln:   i1.write_reg v57, "c"
        nextln:   v58 = i32.icmp.slt v47, 0x0
        nextln:   v59 = i32.icmp.slt v50, 0x0
        nextln:   v60 = i1.icmp.eq v58, v59
        nextln:   v61 = i1.icmp.ne v58, v54
        nextln:   v62 = i1.and v60, v61
        nextln:   i1.write_reg v62, "v"
        nextln:   jump $LABEL
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_ccmn_4() {
    let bytes = [
        0x49, 0x50, 0x43, 0xBA, // ccmn x2, x3, #0x9, pl
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i1.read_reg "n"
        nextln:   v38 = i1.icmp.ne v37, 0x1
        nextln:   jumpif v38, ccmp_positive_condition, ccmp_negative_condition
        check: ccmp_positive_condition:
        nextln:   v39 = i64.and 0x8, 0x9
        nextln:   v40 = i64.icmp.ne 0x0, v39
        nextln:   i1.write_reg v40, "n"
        nextln:   v41 = i64.and 0x4, 0x9
        nextln:   v42 = i64.icmp.ne 0x0, v41
        nextln:   i1.write_reg v42, "z"
        nextln:   v43 = i64.and 0x2, 0x9
        nextln:   v44 = i64.icmp.ne 0x0, v43
        nextln:   i1.write_reg v44, "c"
        nextln:   v45 = i64.and 0x1, 0x9
        nextln:   v46 = i64.icmp.ne 0x0, v45
        nextln:   i1.write_reg v46, "v"
        nextln:   jump $LABEL
        check: ccmp_negative_condition:
        nextln:   v47 = i64.read_reg "x2"
        nextln:   v48 = i64.read_reg "x3"
        nextln:   v49 = i64.sub 0x0, v48
        nextln:   v50 = i64.not v49
        nextln:   v51 = i64.add v47, v50
        nextln:   v52 = i64.add v51, 0x1
        nextln:   v53 = i64.icmp.eq v52, 0x0
        nextln:   i1.write_reg v53, "z"
        nextln:   v54 = i64.icmp.slt v52, 0x0
        nextln:   i1.write_reg v54, "n"
        nextln:   v55 = i64.icmp.ugt v47, v52
        nextln:   v56 = i64.icmp.ugt v50, v52
        nextln:   v57 = i1.or v55, v56
        nextln:   i1.write_reg v57, "c"
        nextln:   v58 = i64.icmp.slt v47, 0x0
        nextln:   v59 = i64.icmp.slt v50, 0x0
        nextln:   v60 = i1.icmp.eq v58, v59
        nextln:   v61 = i1.icmp.ne v58, v54
        nextln:   v62 = i1.and v60, v61
        nextln:   i1.write_reg v62, "v"
        nextln:   jump $LABEL
    "#;

    assert!(check_instruction(bytes, directives, None))
}
