use crate::common::lib::check_instruction;

// Conditional select increment
#[test]
fn test_csinv_1() {
    let bytes = [
        0x41, 0xA0, 0x83, 0x5A, // csinv w1, w2, w3, ge
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i1.read_reg "n"
        nextln:   v38 = i1.read_reg "v"
        nextln:   v39 = i1.icmp.eq v37, v38
        nextln:   jumpif v39, csinv_positive_condition, csinv_negative_condition
        check: csinv_positive_condition:
        nextln:   $VAR_NAME = i32.read_reg "x2"
        nextln:   i32.write_reg $VAR_NAME, "x1"
        nextln:   jump $LABEL
        check: csinv_negative_condition:
        nextln:   $VAR_NAME = i32.read_reg "x3"
        nextln:   $VAR_NAME = i32.not $VAR_NAME 
        nextln:   i32.write_reg $VAR_NAME, "x1"
        nextln:   jump $LABEL
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_csinv_2() {
    let bytes = [
        0x41, 0xA0, 0x83, 0xDA, // csinv x1, x2, x3, ge
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i1.read_reg "n"
        nextln:   v38 = i1.read_reg "v"
        nextln:   v39 = i1.icmp.eq v37, v38
        nextln:   jumpif v39, csinv_positive_condition, csinv_negative_condition
        check: csinv_positive_condition:
        nextln:   $VAR_NAME = i64.read_reg "x2"
        nextln:   i64.write_reg $VAR_NAME, "x1"
        nextln:   jump $LABEL
        check: csinv_negative_condition:
        nextln:   $VAR_NAME = i64.read_reg "x3"
        nextln:   $VAR_NAME = i64.not $VAR_NAME
        nextln:   i64.write_reg $VAR_NAME, "x1"
        nextln:   jump $LABEL
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_csinv_3() {
    let bytes = [
        0x21, 0x30, 0x82, 0xDA, // csinv x1, x1, x2, cc
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i1.read_reg "c"
        nextln:   v38 = i1.icmp.ne v37, 0x1
        nextln:   jumpif v38, csinv_positive_condition, csinv_negative_condition
        check: csinv_positive_condition:
        nextln:   v39 = i64.read_reg "x1"
        nextln:   i64.write_reg v39, "x1"
        nextln:   jump $LABEL
        check: csinv_negative_condition:
        nextln:   v40 = i64.read_reg "x2"
        nextln:   v41 = i64.not v40
        nextln:   i64.write_reg v41, "x1"
        nextln:   jump $LABEL
    "#;

    assert!(check_instruction(bytes, directives, None))
}
