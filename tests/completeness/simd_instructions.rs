use crate::common::lib::{check_instruction, CheckInstructionArgs};

// Add
#[test]
fn test_simd_scalar() {
    let bytes = [
        0x41, 0x84, 0xE3, 0x5E, // add d1, d2, d3
    ];
    let directives = r#"
        check: // entry block
        nextln: v37 = i64.opaque
        nextln: v38 = i64.opaque
        nextln: v39 = i64.add v37, v38
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}

#[test]
fn test_simd_vector() {
    let bytes = [
        0x20, 0x84, 0xA2, 0x4E, // add v0.4s, v1.4s, v2.4s
    ];

    let directives = r#"
        check: // entry block
        nextln: v37 = i128.opaque
        nextln: v38 = i128.opaque
        nextln: v39 = i128.add v37, v38
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}

#[test]
fn test_simd_vector_2() {
    let bytes = [
        0x20, 0x84, 0x22, 0x0E, // add v0.8b, v1.8b, v2.8b
    ];

    let directives = r#"
        check: // entry block
        nextln: v37 = i64.opaque
        nextln: v38 = i64.opaque
        nextln: v39 = i64.add v37, v38
    "#;

    assert!(check_instruction(
        bytes,
        directives,
        CheckInstructionArgs::default()
    ));
}
