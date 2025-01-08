use crate::common::lib::check_instruction;

#[test]
fn test_umsubl_1() {
    let bytes = [
        0x21, 0x88, 0xA2, 0x9B, // umsubl x1, w1, w2, x2
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i32.read_reg "x1"
        nextln: v38 = i32.read_reg "x2"
        nextln: v39 = i64.read_reg "x2"
        nextln: v40 = i32.umul v37, v38
        nextln: v41 = i64.sub v39, v40
        nextln: i64.write_reg v41, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None))
}
