use crate::common::lib::check_instruction;

// Count leading sign bits
#[test]
fn test_cls_1() {
    let bytes = [
        0x41, 0x14, 0xC0, 0xDA, // cls x1, x2
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x2"
        nextln: v38 = i64.lshr v37, 0x1
        nextln: v39 = i64.ror 0x1, 0x1
        nextln: v40 = i64.not v39
        nextln: v41 = i64.and v40, v37
        nextln: v42 = i64.xor v38, v41
        nextln: v43 = i64.highest_set_bit v42
        nextln: v44 = i64.sub 0x40, v43
        nextln: v45 = i64.sub v44, 0x1
        nextln: i64.write_reg v45, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_cls_2() {
    let bytes = [
        0x41, 0x14, 0xC0, 0x5A, // cls w1, w2
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i32.read_reg "x2"
        nextln: v38 = i32.lshr v37, 0x1
        nextln: v39 = i32.ror 0x1, 0x1
        nextln: v40 = i32.not v39
        nextln: v41 = i32.and v40, v37
        nextln: v42 = i32.xor v38, v41
        nextln: v43 = i32.highest_set_bit v42
        nextln: v44 = i32.sub 0x20, v43
        nextln: v45 = i32.sub v44, 0x1
        nextln: i32.write_reg v45, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None))
}
