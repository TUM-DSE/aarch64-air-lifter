use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Test bit and branch zero
fn test() {
    let bytes = [
        0x80, 0x80, 0x40, 0xB6, // tbz x0, #40, pc+4099
        0xE1, 0xFF, 0x67, 0x36, // tbz x1, #12, pc-1
        0xC1, 0xFF, 0x0F, 0x36, // tbz w1, #1, pc-2
        0xA2, 0xFF, 0x07, 0x36, // tbz w2, #0, pc-3
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
