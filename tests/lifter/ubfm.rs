use crate::common::lib::{check_instruction, CheckInstructionArgs};

#[test]
fn test_ubfm_1() {
    let bytes = [
        0x41, 0x2C, 0x4C, 0xD3, // ubfm x1, x2, #12, #11
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i64.read_reg "x2"
        nextln:   v38 = i64.icmp.ult 0xc, 0xb
        nextln:   jumpif v38, ubfm_positive_condition, ubfm_negative_condition
        check: ubfm_positive_condition:
        nextln:   v39 = i64.add 0x1, 0xb
        nextln:   v40 = i64.sub v39, 0xc
        nextln:   v41 = i64.add 0xb, 0x1
        nextln:   v42 = i64.sub 0x40, v41
        nextln:   v43 = i64.lshl v37, v42
        nextln:   v44 = i64.sub 0x40, v40
        nextln:   v45 = i64.lshr v43, v44
        nextln:   i64.write_reg v45, "x1"
        nextln:   jump $LABEL
        check: ubfm_negative_condition:
        nextln:   v46 = i64.add 0xb, 0x1
        nextln:   v47 = i64.sub 0x40, v46
        nextln:   v48 = i64.lshl v37, v47
        nextln:   v49 = i64.sub 0x40, 0xc
        nextln:   v50 = i64.lshr v48, v49
        nextln:   i64.write_reg v50, "x1"
        nextln:   jump $LABEL
        "#;
    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_ubfm_2() {
    let bytes = [
        0x41, 0x2C, 0x4C, 0xD3, // ubfm x1, x2, #12, #11
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i64.read_reg "x2"
        nextln:   v38 = i64.icmp.ult 0xc, 0xb
        nextln:   jumpif v38, ubfm_positive_condition, ubfm_negative_condition
        check: ubfm_positive_condition:
        nextln:   v39 = i64.add 0x1, 0xb
        nextln:   v40 = i64.sub v39, 0xc
        nextln:   v41 = i64.add 0xb, 0x1
        nextln:   v42 = i64.sub 0x40, v41
        nextln:   v43 = i64.lshl v37, v42
        nextln:   v44 = i64.sub 0x40, v40
        nextln:   v45 = i64.lshr v43, v44
        nextln:   i64.write_reg v45, "x1"
        nextln:   jump $LABEL
        check: ubfm_negative_condition:
        nextln:   v46 = i64.add 0xb, 0x1
        nextln:   v47 = i64.sub 0x40, v46
        nextln:   v48 = i64.lshl v37, v47
        nextln:   v49 = i64.sub 0x40, 0xc
        nextln:   v50 = i64.lshr v48, v49
        nextln:   i64.write_reg v50, "x1"
        nextln:   jump block_4
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}
