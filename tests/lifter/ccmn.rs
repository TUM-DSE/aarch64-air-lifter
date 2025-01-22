use crate::common::lib::{check_instruction, CheckInstructionArgs};

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
        nextln:   v39 = i64.read_reg "x0"
        nextln:   v40 = i64.read_reg "x1"
        nextln:   v41 = i64.add v39, v40
        nextln:   v42 = i64.add v41, 0x0
        nextln:   v43 = i64.icmp.eq v42, 0x0
        nextln:   i1.write_reg v43, "z"
        nextln:   v44 = i64.icmp.slt v42, 0x0
        nextln:   i1.write_reg v44, "n"
        nextln:   v45 = i64.icmp.ugt v39, v42
        nextln:   v46 = i64.icmp.ugt v40, v42
        nextln:   v47 = i1.or v45, v46
        nextln:   i1.write_reg v47, "c"
        nextln:   v48 = i64.icmp.slt v39, 0x0
        nextln:   v49 = i64.icmp.slt v40, 0x0
        nextln:   v50 = i1.icmp.eq v48, v49
        nextln:   v51 = i1.icmp.ne v48, v44
        nextln:   v52 = i1.and v50, v51
        nextln:   i1.write_reg v52, "v"
        nextln:   jump $LABEL
        check: ccmp_negative_condition:
        nextln:   v53 = i64.and 0x8, 0x3
        nextln:   v54 = i64.icmp.ne 0x0, v53
        nextln:   i1.write_reg v54, "n"
        nextln:   v55 = i64.and 0x4, 0x3
        nextln:   v56 = i64.icmp.ne 0x0, v55
        nextln:   i1.write_reg v56, "z"
        nextln:   v57 = i64.and 0x2, 0x3
        nextln:   v58 = i64.icmp.ne 0x0, v57
        nextln:   i1.write_reg v58, "c"
        nextln:   v59 = i64.and 0x1, 0x3
        nextln:   v60 = i64.icmp.ne 0x0, v59
        nextln:   i1.write_reg v60, "v"
        nextln:   jump $LABEL
   "#;
    check_instruction(bytes, directives, CheckInstructionArgs::default());
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
        check: ccmp_positive_condition:
        nextln:   v38 = i64.read_reg "x6"
        nextln:   v39 = i64.read_reg "x6"
        nextln:   v40 = i64.add v38, v39
        nextln:   v41 = i64.add v40, 0x0
        nextln:   v42 = i64.icmp.eq v41, 0x0
        nextln:   i1.write_reg v42, "z"
        nextln:   v43 = i64.icmp.slt v41, 0x0
        nextln:   i1.write_reg v43, "n"
        nextln:   v44 = i64.icmp.ugt v38, v41
        nextln:   v45 = i64.icmp.ugt v39, v41
        nextln:   v46 = i1.or v44, v45
        nextln:   i1.write_reg v46, "c"
        nextln:   v47 = i64.icmp.slt v38, 0x0
        nextln:   v48 = i64.icmp.slt v39, 0x0
        nextln:   v49 = i1.icmp.eq v47, v48
        nextln:   v50 = i1.icmp.ne v47, v43
        nextln:   v51 = i1.and v49, v50
        nextln:   i1.write_reg v51, "v"
        nextln:   jump $LABEL
        check: ccmp_negative_condition:
        nextln:   v52 = i64.and 0x8, 0x0
        nextln:   v53 = i64.icmp.ne 0x0, v52
        nextln:   i1.write_reg v53, "n"
        nextln:   v54 = i64.and 0x4, 0x0
        nextln:   v55 = i64.icmp.ne 0x0, v54
        nextln:   i1.write_reg v55, "z"
        nextln:   v56 = i64.and 0x2, 0x0
        nextln:   v57 = i64.icmp.ne 0x0, v56
        nextln:   i1.write_reg v57, "c"
        nextln:   v58 = i64.and 0x1, 0x0
        nextln:   v59 = i64.icmp.ne 0x0, v58
        nextln:   i1.write_reg v59, "v"
        nextln:   jump $LABEL
   "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
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
        check: ccmp_positive_condition: // preds: entry
        nextln:   v39 = i32.read_reg "x0"
        nextln:   v40 = i32.read_reg "x2"
        nextln:   v41 = i32.add v39, v40
        nextln:   v42 = i32.add v41, 0x0
        nextln:   v43 = i32.icmp.eq v42, 0x0
        nextln:   i1.write_reg v43, "z"
        nextln:   v44 = i32.icmp.slt v42, 0x0
        nextln:   i1.write_reg v44, "n"
        nextln:   v45 = i32.icmp.ugt v39, v42
        nextln:   v46 = i32.icmp.ugt v40, v42
        nextln:   v47 = i1.or v45, v46
        nextln:   i1.write_reg v47, "c"
        nextln:   v48 = i32.icmp.slt v39, 0x0
        nextln:   v49 = i32.icmp.slt v40, 0x0
        nextln:   v50 = i1.icmp.eq v48, v49
        nextln:   v51 = i1.icmp.ne v48, v44
        nextln:   v52 = i1.and v50, v51
        nextln:   i1.write_reg v52, "v"
        nextln:   jump $LABEL
        check: ccmp_negative_condition:
        nextln:   v53 = i32.and 0x8, 0x4
        nextln:   v54 = i32.icmp.ne 0x0, v53
        nextln:   i1.write_reg v54, "n"
        nextln:   v55 = i32.and 0x4, 0x4
        nextln:   v56 = i32.icmp.ne 0x0, v55
        nextln:   i1.write_reg v56, "z"
        nextln:   v57 = i32.and 0x2, 0x4
        nextln:   v58 = i32.icmp.ne 0x0, v57
        nextln:   i1.write_reg v58, "c"
        nextln:   v59 = i32.and 0x1, 0x4
        nextln:   v60 = i32.icmp.ne 0x0, v59
        nextln:   i1.write_reg v60, "v"
        nextln:   jump $LABEL
   "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}
