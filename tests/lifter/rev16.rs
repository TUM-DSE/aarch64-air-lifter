use crate::common::lib::check_instruction;

#[test]
fn test_rev16_1() {
    let bytes = [
        0x21, 0x04, 0xC0, 0xDA, // rev16 x1, x1
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i16.read_reg "x1"
        nextln: v38 = i16.reverse_bytes v37
        nextln: v39 = i16.or 0x0, v38
        nextln: v40 = i32.lshl v39, 0x10
        nextln: v41 = i16.read_reg "x1"
        nextln: v42 = i16.reverse_bytes v41
        nextln: v43 = i16.or v40, v42
        nextln: v44 = i32.lshl v43, 0x10
        nextln: v45 = i16.read_reg "x1"
        nextln: v46 = i16.reverse_bytes v45
        nextln: v47 = i16.or v44, v46
        nextln: v48 = i32.lshl v47, 0x10
        nextln: v49 = i16.read_reg "x1"
        nextln: v50 = i16.reverse_bytes v49
        nextln: v51 = i16.or v48, v50
        nextln: i64.write_reg v51, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_rev16_2() {
    let bytes = [
        0x21, 0x04, 0xC0, 0x5A, // rev16 w1, w1
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i16.read_reg "x1"
        nextln: v38 = i16.reverse_bytes v37
        nextln: v39 = i16.or 0x0, v38
        nextln: v40 = i32.lshl v39, 0x10
        nextln: v41 = i16.read_reg "x1"
        nextln: v42 = i16.reverse_bytes v41
        nextln: v43 = i16.or v40, v42
        nextln: i32.write_reg v43, "x1"
   "#;

    assert!(check_instruction(bytes, directives, None))
}
