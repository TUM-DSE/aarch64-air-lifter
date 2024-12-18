use crate::common::lib::check_instruction;

// Conditional Select
#[test]
fn test_csel_1() {
    let bytes = [
        0x20, 0x40, 0x82, 0x9A, // csel x0, x1, x2, mi = first
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i1.read_reg "n"
        nextln:   v38 = i1.icmp.eq v37, 0x1
        nextln:   jumpif v38, csel_positive_condition, csel_negative_condition
        check: csel_positive_condition:
        nextln:   v39 = i64.read_reg "x1"
        nextln:   i64.write_reg v39, "x0"
        nextln:   jump $LABEL 
        check: csel_negative_condition:
        nextln:   v40 = i64.read_reg "x2"
        nextln:   i64.write_reg v40, "x0"
        nextln:   jump $LABEL
        "#;
    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_csel_2() {
    let bytes = [
        0x20, 0x20, 0x82, 0x9A, // csel x0, x1, x2, cs = hs, nlast
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i1.read_reg "c"
        nextln:   v38 = i1.icmp.eq v37, 0x1
        nextln:   jumpif v38, csel_positive_condition, csel_negative_condition
        check: csel_positive_condition:
        nextln:   v39 = i64.read_reg "x1"
        nextln:   i64.write_reg v39, "x0"
        nextln:   jump $LABEL
        check: csel_negative_condition:
        nextln:   v40 = i64.read_reg "x2"
        nextln:   i64.write_reg v40, "x0"
        nextln:   jump $LABEL
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_csel_3() {
    let bytes = [
        0x20, 0x00, 0x82, 0x1A, // csel w0, w1, w2, eq = none
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i1.read_reg "z"
        nextln:   v38 = i1.icmp.eq v37, 0x1
        nextln:   jumpif v38, csel_positive_condition, csel_negative_condition
        check: csel_positive_condition:
        nextln:   v39 = i32.read_reg "x1"
        nextln:   i32.write_reg v39, "x0"
        nextln:   jump $LABEL
        check: csel_negative_condition:
        nextln:   v40 = i32.read_reg "x2"
        nextln:   i32.write_reg v40, "x0"
        nextln:   jump $LABEL
    "#;

    assert!(check_instruction(bytes, directives, None))
}
