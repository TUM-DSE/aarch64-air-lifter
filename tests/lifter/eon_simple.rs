use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Count leading sign bits
fn test() {
    let bytes = [
        0x41, 0x30, 0x63, 0xCA, // eon x1, x2, x3, lsr #12
        0x41, 0x04, 0xA3, 0xCA, // eon x1, x2, x3, asr #1
        0x41, 0x04, 0xA3, 0x4A, // eon w1, w2, w3, asr #1
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
