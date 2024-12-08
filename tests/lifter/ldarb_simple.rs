use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Load-acquire register byte
fn test() {
    let bytes = [
        0xE1, 0xFF, 0xDF, 0x08, // ldarb w1, [sp]
        0x21, 0xFC, 0xDF, 0x08, // ldarb w1, [x1]
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
