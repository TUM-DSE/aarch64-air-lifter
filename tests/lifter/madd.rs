use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

use crate::common::lib::check_instruction;

#[test]
// Multiply-add
fn test() {
    let bytes = [];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}

#[test]
fn test_madd_1() {
    let bytes = [
        0x20, 0x24, 0xC2, 0x1A, // lsr w0, w1, w2
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i32.read_reg "x1"
        nextln:  v38 = i32.read_reg "x2"
        nextln:  v39 = i32.and v38, 0x1f
        nextln:  v40 = i32.lshr v37, v39
        nextln:  i32.write_reg v40, "x0"
    "#;

    assert!(check_instruction(bytes, directives, None))
}

#[test]
fn test_madd_2() {
    let bytes = [
        0x20, 0x24, 0xC2, 0x9A, // lsr x0, x1, x2
    ];
    let directives = r#"
        check: // entry block
        nextln:  v37 = i64.read_reg "x1"
        nextln:  v38 = i64.read_reg "x2"
        nextln:  v39 = i64.and v38, 0x3f
        nextln:  v40 = i64.lshr v37, v39
        nextln:  i64.write_reg v40, "x0"
    "#;

    assert!(check_instruction(bytes, directives, None))
}
