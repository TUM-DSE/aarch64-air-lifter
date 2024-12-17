use crate::common::lib::check_instruction;

// Form pc-relative address
#[test]
fn test_addr_1() {
    let bytes = [
        0x00, 0x00, 0x00, 0x10, // adr x0, pc
    ];
    let directives = r#"
        #0 check: // entry block
        #1 nextln: v37 = i64.read_reg "pc"
        #2 nextln: v38 = i64.add v37, 0x0
        #3 nextln: i64.write_reg v38, "x0"
    "#;

    assert!(check_instruction(bytes, directives, None));
}

#[test]
fn test_addr_2() {
    let bytes = [
        0x21, 0x00, 0x00, 0x10, // adr x1, pc+1
    ];
    let directives = r#"
        #0 check: // entry block
        #1 nextln: v37 = i64.read_reg "pc"
        #2 nextln: v38 = i64.add v37, 0x4
        #3 nextln: i64.write_reg v38, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None));
}

#[test]
fn test_addr_3() {
    let bytes = [
        0xC0, 0xFF, 0xFF, 0x10, // adr x0, pc-2
    ];
    let directives = r#"
        #0 check: // entry block
        #1 nextln: v37 = i64.read_reg "pc"
        #2 nextln: v38 = i64.add v37, 0xfffffffffffffff8
        #3 nextln: i64.write_reg v38, "x0"
    "#;

    assert!(check_instruction(bytes, directives, None));
}
