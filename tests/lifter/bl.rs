use crate::common::lib::check_instruction;

// Branch with link
#[test]
fn test_bl_1() {
    let bytes = [
        0x02, 0x00, 0x00, 0x94, // bl, pc
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "pc"
        nextln: v38 = i64.add v37, 0x4
        nextln: i64.write_reg v38, "x30"
        nextln: jump $LABEL
    "#;

    assert!(check_instruction(bytes, directives, None))
}
#[test]

fn test_bl_2() {
    let bytes = [
        0x01, 0x00, 0x00, 0x94, // bl, pc+1
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "pc"
        nextln: v38 = i64.add v37, 0x4
        nextln: i64.write_reg v38, "x30"
        nextln: jump $LABEL
    "#;

    assert!(check_instruction(bytes, directives, None))
}
#[test]

fn test_bl_3() {
    let bytes = [
        0xfe, 0xff, 0xff, 0x97, // bl, pc-2
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "pc"
        nextln: v38 = i64.add v37, 0x4
        nextln: i64.write_reg v38, "x30"
        nextln: jump $LABEL
    "#;

    assert!(check_instruction(bytes, directives, None))
}
