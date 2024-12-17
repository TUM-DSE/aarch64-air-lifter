use crate::common::lib::check_instruction;

// Add shifted register
#[test]
fn test_add_1() {
    let bytes = [
        0x21, 0x00, 0x00, 0x8B, // add x1, x1, s0
    ];
    let directives = r#"
        #0 check: // entry block
        #1 nextln: v37 = i64.read_reg "x1"
        #2 nextln: v38 = i64.read_reg "x0"
        #3 nextln: v39 = i64.add v37, v38
        #4 nextln: i64.write_reg v39, "x1"
    "#;

    check_instruction(bytes, directives, None);
}

#[test]
fn test_add_2() {
    let bytes = [
        0x21, 0x00, 0x00, 0x0B, // add w1, w1, w0
    ];
    let directives = r#"
        #0 check: // entry block
        #1 nextln: v37 = i32.read_reg "x1"
        #2 nextln: v38 = i32.read_reg "x0"
        #3 nextln: v39 = i32.add v37, v38
        #4 nextln: i32.write_reg v39, "x1"
  "#;

    check_instruction(bytes, directives, None);
}

#[test]
fn test_add_3() {
    let bytes = [
        0x02, 0xc0, 0x21, 0x0B, // add w2, w0, w1, SXTW
    ];
    let directives = r#"
        #0 check: // entry block
        #1 nextln: v37 = i32.read_reg "x0"
        #2 nextln: v38 = i32.read_reg "x1"
        #3 check: $VAR_NAME = i32.add $VAR_NAME, $VAR_NAME
        #4 nextln: i32.write_reg $VAR_NAME, "x2"
    "#;

    check_instruction(bytes, directives, None);
}
