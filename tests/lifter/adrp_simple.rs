use crate::common::lib::check_instruction;

// Form PC-relative address to 4KB page
#[test]
fn test_adrp_1() {
    let bytes = [
        0x00, 0x00, 0x00, 0x90, // adrp x0, pc
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
fn test_adrp_2() {
    let bytes = [
        0x00, 0x00, 0x00, 0xB0, // adrp x0, pc+4096
    ];
    let directives = r#"
        #0 check: // entry block
        #1 nextln: v37 = i64.read_reg "pc"
        #2 nextln: v38 = i64.add v37, 0x1000
        #3 nextln: i64.write_reg v38, "x0"
    "#;

    assert!(check_instruction(bytes, directives, None));
}
