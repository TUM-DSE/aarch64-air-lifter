use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Conditional select increment
#[test]
fn test_csinc_1() {
    let bytes = [
        0x20, 0x04, 0x82, 0x9A, // csinc x0, x1, x2, EQ
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i1.read_reg "z"
        nextln:   v38 = i1.icmp.eq v37, 0x1
        nextln:   jumpif v38, csinc_positive_condition, csinc_negative_condition
        check:  csinc_positive_condition:
        nextln:   v39 = i64.read_reg "x1"
        nextln:   i64.write_reg v39, "x0"
        nextln:   jump $LABEL
        check:  csinc_negative_condition:
        nextln:   v40 = i64.read_reg "x2"
        nextln:   v41 = i64.add v40, 0x1
        nextln:   i64.write_reg v41, "x0"
        nextln:   jump $LABEL
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_csinc_2() {
    let bytes = [
        0x20, 0x04, 0x84, 0x1A, // csinc w0, w1, w2, EQ
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i1.read_reg "z"
        nextln:   v38 = i1.icmp.eq v37, 0x1
        nextln:   jumpif v38, csinc_positive_condition, csinc_negative_condition
        check: csinc_positive_condition:
        nextln:   v39 = i32.read_reg "x1"
        nextln:   i32.write_reg v39, "x0"
        nextln:   jump $LABEL
        check: csinc_negative_condition:
        nextln:   v40 = i32.read_reg "x4"
        nextln:   v41 = i32.add v40, 0x1
        nextln:   i32.write_reg v41, "x0"
        nextln:   jump $LABEL
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_csinc_3() {
    let bytes = [
        0x62, 0xD4, 0x84, 0x9A, // csinc x2, x3, x4, LE
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i1.read_reg "z"
        nextln:   v38 = i1.read_reg "n"
        nextln:   v39 = i1.read_reg "v"
        nextln:   v40 = i1.icmp.eq v37, 0x1
        nextln:   v41 = i1.icmp.ne v38, v39
        nextln:   v42 = i1.or v40, v41
        nextln:   jumpif v42, csinc_positive_condition, csinc_negative_condition
        check: csinc_positive_condition:
        nextln:   v43 = i64.read_reg "x3"
        nextln:   i64.write_reg v43, "x2"
        nextln:   jump $LABEL
        check: csinc_negative_condition:
        nextln:   v44 = i64.read_reg "x4"
        nextln:   v45 = i64.add v44, 0x1
        nextln:   i64.write_reg v45, "x2"
        nextln:   jump $LABEL
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}
