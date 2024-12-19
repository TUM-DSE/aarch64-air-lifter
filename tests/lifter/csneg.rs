use crate::common::lib::check_instruction;

// Conditional select negation
#[test]
fn test_csneg_1() {
    let bytes = [
        0x20, 0x04, 0x82, 0xDA, // csneg x0, x1, x2, eq
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i1.read_reg "z"
        nextln:   v38 = i1.icmp.eq v37, 0x1
        nextln:   jumpif v38, csneg_positive_condition, csneg_negative_condition
        check: csneg_positive_condition:
        nextln:   v39 = i64.read_reg "x1"
        nextln:   i64.write_reg v39, "x0"
        nextln:   jump $LABEL
        check: csneg_negative_condition:
        nextln:   v40 = i64.read_reg "x2"
        nextln:   v41 = i64.sub 0x0, v40
        nextln:   i64.write_reg v41, "x0"
        nextln:   jump $LABEL
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_csneg_2() {
    let bytes = [
        0x20, 0x04, 0x82, 0x5A, // csneg w0, w1, w2, eq
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i1.read_reg "z"
        nextln:   v38 = i1.icmp.eq v37, 0x1
        nextln:   jumpif v38, csneg_positive_condition, csneg_negative_condition
        check: csneg_positive_condition:
        nextln:   v39 = i32.read_reg "x1"
        nextln:   i32.write_reg v39, "x0"
        nextln:   jump $LABEL
        check: csneg_negative_condition:
        nextln:   v40 = i32.read_reg "x2"
        nextln:   v41 = i32.sub 0x0, v40
        nextln:   i32.write_reg v41, "x0"
        nextln:   jump $LABEL
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_csneg_3() {
    let bytes = [
        0x62, 0xD4, 0x84, 0xDA, // csneg x2, x3, x4, le
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i1.read_reg "z"
        nextln:   v38 = i1.read_reg "n"
        nextln:   v39 = i1.read_reg "v"
        nextln:   v40 = i1.icmp.eq v37, 0x1
        nextln:   v41 = i1.icmp.ne v38, v39
        nextln:   v42 = i1.or v40, v41
        nextln:   jumpif v42, csneg_positive_condition, csneg_negative_condition
        check: csneg_positive_condition:
        nextln:   v43 = i64.read_reg "x3"
        nextln:   i64.write_reg v43, "x2"
        nextln:   jump $LABEL
        check: csneg_negative_condition:
        nextln:   v44 = i64.read_reg "x4"
        nextln:   v45 = i64.sub 0x0, v44
        nextln:   i64.write_reg v45, "x2"
        nextln:   jump $LABEL
    "#;

    assert!(check_instruction(bytes, directives, None))
}
