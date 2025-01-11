use crate::common::lib::check_instruction;

// Bitfield move
#[test]
fn test_bfm_1() {
    let bytes = [
        0x41, 0x78, 0x0C, 0x33, // bfm w1, w2, #12, #30 <=> bfxil w1, w2, #12, #19
    ];
    let directives = r#"
        # check: // entry block
        # nextln:   v37 = i32.read_reg "x2"
        # nextln:   v38 = i64.icmp.uge 0x1e, 0xc
        # nextln:   jumpif v38, bfm_positive_condition, bfm_negative_condition
        # check: bfm_positive_condition:
        # nextln:   v39 = i32.add 0x1, 0x1e
        # nextln:   v40 = i32.sub v39, 0xc
        # nextln:   v41 = i32.lshl 0x1, v40
        # nextln:   v42 = i32.sub v41, 0x1
        # nextln:   v43 = i32.lshl v42, 0xc
        # nextln:   v44 = i32.and v37, v43
        # nextln:   v45 = i32.lshr v44, 0xc
        # nextln:   v46 = i32.lshl 0x1, v40
        # nextln:   v47 = i32.sub v46, 0x1
        # nextln:   v48 = i32.not v47
        # nextln:   v49 = i32.and v37, v48
        # nextln:   v50 = i32.or v45, v49
        # nextln:   i32.write_reg v50, "x1"
        # nextln:   jump block_4
        # check: bfm_negative_condition:
        # nextln:   v51 = i32.add 0x1, 0x1e
        # nextln:   v52 = i32.lshl 0x1, v51
        # nextln:   v53 = i32.sub v52, 0x1
        # nextln:   v54 = i32.and v37, v53
        # nextln:   v55 = i32.sub 0x20, 0xc
        # nextln:   v56 = i32.lshl v54, v55
        # nextln:   v57 = i32.lshl 0x1, v51
        # nextln:   v58 = i32.sub v57, 0x1
        # nextln:   v59 = i32.lshl v58, v55
        # nextln:   v60 = i32.not v59
        # nextln:   v61 = i32.and v37, v60
        # nextln:   v62 = i32.or v56, v61
        # nextln:   i32.write_reg v62, "x1"
        # nextln:   jump block_4
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_bfm_2() {
    let bytes = [
        0x41, 0x04, 0x41, 0xB3, // bfm x1, x2, #1, #1 <=> bfxil x1, x2, #1, #1
    ];
    let directives = r#"
        #0 check: // entry block
        # nextln:   v37 = i64.read_reg "x2"
        # nextln:   v38 = i64.icmp.uge 0x1, 0x1
        # nextln:   jumpif v38, bfm_positive_condition, bfm_negative_condition
        # check: bfm_positive_condition:
        # nextln:   v39 = i64.add 0x1, 0x1
        # nextln:   v40 = i64.sub v39, 0x1
        # nextln:   v41 = i64.lshl 0x1, v40
        # nextln:   v42 = i64.sub v41, 0x1
        # nextln:   v43 = i64.lshl v42, 0x1
        # nextln:   v44 = i64.and v37, v43
        # nextln:   v45 = i64.lshr v44, 0x1
        # nextln:   v46 = i64.lshl 0x1, v40
        # nextln:   v47 = i64.sub v46, 0x1
        # nextln:   v48 = i64.not v47
        # nextln:   v49 = i64.and v37, v48
        # nextln:   v50 = i64.or v45, v49
        # nextln:   i64.write_reg v50, "x1"
        # nextln:   jump block_4
        # check: bfm_negative_condition: // preds: entry
        # nextln:   v51 = i64.add 0x1, 0x1
        # nextln:   v52 = i64.lshl 0x1, v51
        # nextln:   v53 = i64.sub v52, 0x1
        # nextln:   v54 = i64.and v37, v53
        # nextln:   v55 = i64.sub 0x40, 0x1
        # nextln:   v56 = i64.lshl v54, v55
        # nextln:   v57 = i64.lshl 0x1, v51
        # nextln:   v58 = i64.sub v57, 0x1
        # nextln:   v59 = i64.lshl v58, v55
        # nextln:   v60 = i64.not v59
        # nextln:   v61 = i64.and v37, v60
        # nextln:   v62 = i64.or v56, v61
        # nextln:   i64.write_reg v62, "x1"
        # nextln:   jump block_4"#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_bfm_3() {
    let bytes = [
        0x41, 0xC8, 0x42, 0xB3, // bfm x1, x2, #2, #50 <=> bfxil x1, x2, #2, #49
    ];
    let directives = r#"
        check: // entry block
        nextln:   v37 = i64.read_reg "x2"
        nextln:   v38 = i64.icmp.uge 0x32, 0x2
        nextln:   jumpif v38, bfm_positive_condition, bfm_negative_condition
        check: bfm_positive_condition: // preds: entry
        nextln:   v39 = i64.add 0x1, 0x32
        nextln:   v40 = i64.sub v39, 0x2
        nextln:   v41 = i64.lshl 0x1, v40
        nextln:   v42 = i64.sub v41, 0x1
        nextln:   v43 = i64.lshl v42, 0x2
        nextln:   v44 = i64.and v37, v43
        nextln:   v45 = i64.lshr v44, 0x2
        nextln:   v46 = i64.lshl 0x1, v40
        nextln:   v47 = i64.sub v46, 0x1
        nextln:   v48 = i64.not v47
        nextln:   v49 = i64.and v37, v48
        nextln:   v50 = i64.or v45, v49
        nextln:   i64.write_reg v50, "x1"
        nextln:   jump block_4
        check: bfm_negative_condition: // preds: entry
        nextln:   v51 = i64.add 0x1, 0x32
        nextln:   v52 = i64.lshl 0x1, v51
        nextln:   v53 = i64.sub v52, 0x1
        nextln:   v54 = i64.and v37, v53
        nextln:   v55 = i64.sub 0x40, 0x2
        nextln:   v56 = i64.lshl v54, v55
        nextln:   v57 = i64.lshl 0x1, v51
        nextln:   v58 = i64.sub v57, 0x1
        nextln:   v59 = i64.lshl v58, v55
        nextln:   v60 = i64.not v59
        nextln:   v61 = i64.and v37, v60
        nextln:   v62 = i64.or v56, v61
        nextln:   i64.write_reg v62, "x1"
        nextln:   jump block_4"#;

    assert!(check_instruction(bytes, directives, None))
}
