use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Store pair of registers
fn test() {
    let bytes = [
        0x41, 0x08, 0x00, 0xA9, // stp x1, x2, [x2]
        0x41, 0x08, 0x00, 0x29, // stp w1, w2, [x2]
        0xE0, 0x07, 0x00, 0xA9, // stp x0, x1, [sp]
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
