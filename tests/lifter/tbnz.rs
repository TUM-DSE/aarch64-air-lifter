use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Test bit and branch if non zero
#[test]
fn test_tbnz_1() {
    let bytes = [
        0x80, 0x80, 0x40, 0xB7, // tbz x0, #40, pc+4099
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.lshr 0x28, 0x1
        nextln:  v38 = i64.and v37, "x0"
        nextln:  v39 = i64.icmp.ne v38, 0x0
        nextln:  jumpif v39, $LABEL, $LABEL
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_tbnz_2() {
    let bytes = [
        0xE1, 0xFF, 0x67, 0x37, // tbz x1, #12, pc-1
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i32.lshr 0xc, 0x1
        nextln:  v38 = i32.and v37, "x1"
        nextln:  v39 = i32.icmp.ne v38, 0x0
        nextln:  jumpif v39, $LABEL, $LABEL
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_tbnz_3() {
    let bytes = [
        0xC1, 0xFF, 0x0F, 0x37, // tbz w1, #1, pc-2
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i32.lshr 0x1, 0x1
        nextln:  v38 = i32.and v37, "x1"
        nextln:  v39 = i32.icmp.ne v38, 0x0
        nextln:  jumpif v39, $LABEL, $LABEL
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}

#[test]
fn test_tbnz_4() {
    let bytes = [
        0xA2, 0xFF, 0x07, 0x37, // tbz w2, #0, pc-3
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i32.lshr 0x0, 0x1
        nextln:  v38 = i32.and v37, "x2"
        nextln:  v39 = i32.icmp.ne v38, 0x0
        nextln:  jumpif v39, $LABEL, $LABEL
    "#;

    check_instruction(bytes, directives, CheckInstructionArgs::default());
}
