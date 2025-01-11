use crate::common::lib::check_instruction;

#[test]
fn test_rev32_1() {
    let bytes = [
        0x21, 0x08, 0xC0, 0xDA, // rev32 x1, x1
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x1"
        nextln: v38 = i32.reverse_bytes v37
        nextln: v39 = i32.or 0x0, v38
        nextln: v40 = i64.ror v39, 0x20
        nextln: v41 = i64.ror v37, 0x20
        nextln: v42 = i32.reverse_bytes v41
        nextln: v43 = i32.or v40, v42
        nextln: v44 = i64.ror v43, 0x20
        nextln: i64.write_reg v44, "x1"
   "#;

    assert!(check_instruction(bytes, directives, None))
}
