use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Bitwise bit clear (shifted register)
fn test() {
    let bytes = [
        0x41, 0x08, 0x23, 0x0A, // bic w1, w2, w3, lsl #2
        0x41, 0x0C, 0x23, 0x8A, // bic x1, x2, x3, lsl #3
        0x41, 0x04, 0xA3, 0x8A, // bic x1, x2, x3, asr #1
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
