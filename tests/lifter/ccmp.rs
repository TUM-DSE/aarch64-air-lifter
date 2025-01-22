use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Conditional Compare (register) sets the value of the condition flags to the result of the comparison of two registers if the condition is TRUE, and an immediate value otherwise.
#[test]
fn test_ccmp_1() {
    let bytes = [
        0x03, 0x00, 0x41, 0xFA, // ccmp x0, x1, #0x3, eq
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i1.read_reg "z"
        nextln:   v38 = i1.icmp.eq v37, 0x1
        nextln:   jumpif v38, ccmp_positive_condition, ccmp_negative_condition
        check: ccmp_positive_condition:
        nextln:   v39 = i64.read_reg "x0"
        nextln:   v40 = i64.read_reg "x1"
        nextln:   v41 = i64.not v40
        nextln:   v42 = i64.add v39, v41
        nextln:   v43 = i64.add v42, 0x0
        nextln:   v44 = i64.icmp.eq v43, 0x0
        nextln:   i1.write_reg v44, "z"
        nextln:   v45 = i64.icmp.slt v43, 0x0
        nextln:   i1.write_reg v45, "n"
        nextln:   v46 = i64.icmp.ugt v39, v43
        nextln:   v47 = i64.icmp.ugt v41, v43
        nextln:   v48 = i1.or v46, v47
        nextln:   i1.write_reg v48, "c"
        nextln:   v49 = i64.icmp.slt v39, 0x0
        nextln:   v50 = i64.icmp.slt v41, 0x0
        nextln:   v51 = i1.icmp.eq v49, v50
        nextln:   v52 = i1.icmp.ne v49, v45
        nextln:   v53 = i1.and v51, v52
        nextln:   i1.write_reg v53, "v"
        nextln:   jump $LABEL
        check: ccmp_negative_condition:
        nextln:   v54 = i64.and 0x8, 0x3
        nextln:   v55 = i64.icmp.ne 0x0, v54
        nextln:   i1.write_reg v55, "n"
        nextln:   v56 = i64.and 0x4, 0x3
        nextln:   v57 = i64.icmp.ne 0x0, v56
        nextln:   i1.write_reg v57, "z"
        nextln:   v58 = i64.and 0x2, 0x3
        nextln:   v59 = i64.icmp.ne 0x0, v58
        nextln:   i1.write_reg v59, "c"
        nextln:   v60 = i64.and 0x1, 0x3
        nextln:   v61 = i64.icmp.ne 0x0, v60
        nextln:   i1.write_reg v61, "v"
        nextln:   jump $LABEL
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_ccmp_2() {
    let bytes = [
        0x04, 0x60, 0x42, 0x7A, // ccmp w0, w2, #0x4, vs
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i1.read_reg "v"
        nextln:   v38 = i1.icmp.eq v37, 0x1
        nextln:   jumpif v38, ccmp_positive_condition, ccmp_negative_condition
        check: ccmp_positive_condition:
        nextln:   v39 = i32.read_reg "x0"
        nextln:   v40 = i32.read_reg "x2"
        nextln:   v41 = i32.not v40
        nextln:   v42 = i32.add v39, v41
        nextln:   v43 = i32.add v42, 0x0
        nextln:   v44 = i32.icmp.eq v43, 0x0
        nextln:   i1.write_reg v44, "z"
        nextln:   v45 = i32.icmp.slt v43, 0x0
        nextln:   i1.write_reg v45, "n"
        nextln:   v46 = i32.icmp.ugt v39, v43
        nextln:   v47 = i32.icmp.ugt v41, v43
        nextln:   v48 = i1.or v46, v47
        nextln:   i1.write_reg v48, "c"
        nextln:   v49 = i32.icmp.slt v39, 0x0
        nextln:   v50 = i32.icmp.slt v41, 0x0
        nextln:   v51 = i1.icmp.eq v49, v50
        nextln:   v52 = i1.icmp.ne v49, v45
        nextln:   v53 = i1.and v51, v52
        nextln:   i1.write_reg v53, "v"
        nextln:   jump $LABEL
        check: ccmp_negative_condition:
        nextln:   v54 = i32.and 0x8, 0x4
        nextln:   v55 = i32.icmp.ne 0x0, v54
        nextln:   i1.write_reg v55, "n"
        nextln:   v56 = i32.and 0x4, 0x4
        nextln:   v57 = i32.icmp.ne 0x0, v56
        nextln:   i1.write_reg v57, "z"
        nextln:   v58 = i32.and 0x2, 0x4
        nextln:   v59 = i32.icmp.ne 0x0, v58
        nextln:   i1.write_reg v59, "c"
        nextln:   v60 = i32.and 0x1, 0x4
        nextln:   v61 = i32.icmp.ne 0x0, v60
        nextln:   i1.write_reg v61, "v"
        nextln:   jump $LABEL
  "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_ccmp_3() {
    let bytes = [
        0xC0, 0xE0, 0x46, 0xFA, // ccmp x6, x6, #0x0, al
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i1.and 0x1, 0x1
        nextln:   jumpif v37, ccmp_positive_condition, ccmp_negative_condition
        check: ccmp_positive_condition:
        nextln:   v38 = i64.read_reg "x6"
        nextln:   v39 = i64.read_reg "x6"
        nextln:   v40 = i64.not v39
        nextln:   v41 = i64.add v38, v40
        nextln:   v42 = i64.add v41, 0x0
        nextln:   v43 = i64.icmp.eq v42, 0x0
        nextln:   i1.write_reg v43, "z"
        nextln:   v44 = i64.icmp.slt v42, 0x0
        nextln:   i1.write_reg v44, "n"
        nextln:   v45 = i64.icmp.ugt v38, v42
        nextln:   v46 = i64.icmp.ugt v40, v42
        nextln:   v47 = i1.or v45, v46
        nextln:   i1.write_reg v47, "c"
        nextln:   v48 = i64.icmp.slt v38, 0x0
        nextln:   v49 = i64.icmp.slt v40, 0x0
        nextln:   v50 = i1.icmp.eq v48, v49
        nextln:   v51 = i1.icmp.ne v48, v44
        nextln:   v52 = i1.and v50, v51
        nextln:   i1.write_reg v52, "v"
        nextln:   jump block_4
        check: ccmp_negative_condition:
        nextln:   v53 = i64.and 0x8, 0x0
        nextln:   v54 = i64.icmp.ne 0x0, v53
        nextln:   i1.write_reg v54, "n"
        nextln:   v55 = i64.and 0x4, 0x0
        nextln:   v56 = i64.icmp.ne 0x0, v55
        nextln:   i1.write_reg v56, "z"
        nextln:   v57 = i64.and 0x2, 0x0
        nextln:   v58 = i64.icmp.ne 0x0, v57
        nextln:   i1.write_reg v58, "c"
        nextln:   v59 = i64.and 0x1, 0x0
        nextln:   v60 = i64.icmp.ne 0x0, v59
        nextln:   i1.write_reg v60, "v"
        nextln:   jump $LABEL
  "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}
