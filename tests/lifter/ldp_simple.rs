use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Load pair of registers
fn test() {
    let bytes = [
        0x40, 0x84, 0xC0, 0xA8, // ldp x0, x1, [x2], #8
        0x81, 0x08, 0xC2, 0x28, // ldp w1, w2, [x4], #16
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
