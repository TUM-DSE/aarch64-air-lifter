use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Bitwise AND
fn test() {
    let bytes = [
        0x20, 0x0C, 0x22, 0x2A, // orn w0, w1, w2, lsl #3
        0x41, 0x00, 0x21, 0xAA, // orn x1, x2, x1
        0x20, 0x10, 0x22, 0xAA, // orn x0, x1, x2, lsl #4
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
