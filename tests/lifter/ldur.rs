use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Load register (unscaled)
fn test() {
    let bytes = [
        0xE1, 0x73, 0x41, 0xB8, // ldur w1, [sp, #23]
        0xE1, 0x73, 0x41, 0xF8, // ldur x1, [sp, #23]
        0x41, 0x40, 0x40, 0xF8, // ldur x1, [x2, #4]
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
