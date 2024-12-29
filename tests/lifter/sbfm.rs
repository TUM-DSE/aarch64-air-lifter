use crate::common::lib::check_instruction;

#[test]
fn test_sbfm_1() {
    let bytes = [
        0x41, 0x58, 0x4C, 0x93, // sbfx x1, x2, #12, #11
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i64.read_reg "x2"
        nextln:   v38 = i64.icmp.ult 0xc, 0x16
        nextln:   jumpif v38, bfm_positive_condition, bfm_negative_condition
        check: bfm_positive_condition:
        nextln:   v39 = i64.add 0x1, 0x16
        nextln:   v40 = i64.sub v39, 0xc
        nextln:   v41 = i64.add 0x16, 0x1
        nextln:   v42 = i64.sub 0x40, v41
        nextln:   v43 = i64.lshl v37, v42
        nextln:   v44 = i64.sub 0x40, v40
        nextln:   v45 = i64.ashr v43, v44
        nextln:   i64.write_reg v45, "x1"
        nextln:   jump $LABEL 
        check: bfm_negative_condition:
        nextln:   v46 = i64.add 0x16, 0x1
        nextln:   v47 = i64.sub 0x40, v46
        nextln:   v48 = i64.lshl v37, v47
        nextln:   v49 = i64.sub 0x40, 0xc
        nextln:   v50 = i64.ashr v48, v49
        nextln:   i64.write_reg v50, "x1"
        nextln:   jump $LABEL
        "#;
    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_sbfm_2() {
    let bytes = [
        0x41, 0x58, 0x4C, 0x93, // sbfx x1, x2, #12, #11
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i64.read_reg "x2"
        nextln:   v38 = i64.icmp.ult 0xc, 0x16
        nextln:   jumpif v38, bfm_positive_condition, bfm_negative_condition
        check: bfm_positive_condition:
        nextln:   v39 = i64.add 0x1, 0x16
        nextln:   v40 = i64.sub v39, 0xc
        nextln:   v41 = i64.add 0x16, 0x1
        nextln:   v42 = i64.sub 0x40, v41
        nextln:   v43 = i64.lshl v37, v42
        nextln:   v44 = i64.sub 0x40, v40
        nextln:   v45 = i64.ashr v43, v44
        nextln:   i64.write_reg v45, "x1"
        nextln:   jump $LABEL
        check: bfm_negative_condition:
        nextln:   v46 = i64.add 0x16, 0x1
        nextln:   v47 = i64.sub 0x40, v46
        nextln:   v48 = i64.lshl v37, v47
        nextln:   v49 = i64.sub 0x40, 0xc
        nextln:   v50 = i64.ashr v48, v49
        nextln:   i64.write_reg v50, "x1"
        nextln:   jump $LABEL
    "#;

    assert!(check_instruction(bytes, directives, None))
}
