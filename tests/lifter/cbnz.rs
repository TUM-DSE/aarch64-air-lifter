use crate::common::lib::check_instruction;

// Compare and branch on nonzero
#[test]
fn test_cbnz_1() {
    let bytes = [
        0x00, 0x00, 0x00, 0xB5, // cbnz x0, pc
    ];
    let directives = r#"
        check: // entry block
        check: block_0:
        nextln:   v37 = i64.read_reg "x0"
        nextln:   v38 = i64.icmp.ne v37, 0x0
        nextln:   jumpif v38, $LABEL, $LABEL 
    "#;

    assert!(check_instruction(bytes, directives, None))
}
#[test]

fn test_cbnz_2() {
    let bytes = [
        0xE0, 0xFF, 0xFF, 0x35, // cbnz w0, pc-1
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i32.read_reg "x0"
        nextln: v38 = i32.icmp.ne v37, 0x0
        nextln: jumpif v38, $LABEL, $LABEL
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_cbnz_3() {
    let bytes = [
        0x20, 0x80, 0x00, 0xB5, // cbnz x0, pc+1
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.read_reg "x0"
        nextln: v38 = i64.icmp.ne v37, 0x0
        nextln: jumpif v38, $LABEL, $LABEL
    "#;

    assert!(check_instruction(bytes, directives, None))
}
