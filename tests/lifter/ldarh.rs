use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

use crate::common::lib::check_instruction;

#[test]
// Load-acquire register halfword
fn test() {
    let bytes = [];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}

#[test]
fn test_ldarh_1() {
    let bytes = [
        0xE1, 0xFF, 0xDF, 0x48, // ldarh w1, [sp]
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "sp"
        nextln:  v38 = i64.add v37, 0x0
        nextln:  v39 = i16.load v38
        nextln:  v40 = i32.zext_i16 v39
        nextln:  i32.write_reg v40, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_ldarh_2() {
    let bytes = [
        0x21, 0xFC, 0xDF, 0x48, // ldarh w1, [x1]
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x1"
        nextln:  v38 = i64.add v37, 0x0
        nextln:  v39 = i16.load v38
        nextln:  v40 = i32.zext_i16 v39
        nextln:  i32.write_reg v40, "x1"
    "#;

    assert!(check_instruction(bytes, directives, None))
}
