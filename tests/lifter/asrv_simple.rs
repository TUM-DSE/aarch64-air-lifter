use crate::common::lib::check_instruction;

// Arithmetic shift right variable
#[test]
fn test_asrv_1() {
    let bytes = [
        0x20, 0x28, 0xC2, 0x9A, // asr x0, x1, x2
    ];
    let directives = r#"
        #0 check: // entry block    
        #1 nextln: v37 = i64.read_reg "x1"
        #2 nextln: v38 = i64.read_reg "x2"
        #3 nextln: v39 = i64.and v38, 0x3f
        #4 nextln: v40 = i64.ashr v37, v39
        #5 nextln: i64.write_reg v40, "x0"
    "#;

    assert!(check_instruction(bytes, directives, None));
}

#[test]
fn test_asrv_2() {
    let bytes = [
        0x20, 0x28, 0xC2, 0x1A, // asr w0, w1, w2
    ];
    let directives = r#"
        #0 check: // entry block    
        #1 nextln: v37 = i32.read_reg "x1"
        #2 nextln: v38 = i32.read_reg "x2"
        #3 nextln: v39 = i32.and v38, 0x1f
        #4 nextln: v40 = i32.ashr v37, v39
        #5 nextln: i32.write_reg v40, "x0"
    "#;

    assert!(check_instruction(bytes, directives, None));
}
